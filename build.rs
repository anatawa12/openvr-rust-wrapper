use serde::Deserialize;
use std::io::Write;
use std::path::Path;
use std::{env, fs, io};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=openvr/headers/openvr_api.json");

    let json: Json =
        serde_json::from_reader(fs::File::open("openvr/headers/openvr_api.json").unwrap()).unwrap();

    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR not found");
    let dest_path = Path::new(&out_dir).join("generated.rs");
    let mut out = fs::File::create(dest_path).unwrap();

    for enum_info in &json.enums {
        generate_enum(enum_info, &mut out).unwrap();
    }

    out.flush().unwrap();
}

fn generate_enum(enum_info: &EnumInfo, out: &mut impl Write) -> io::Result<()> {
    let enum_kind = enum_kind(enum_info);
    match enum_kind {
        EnumKind::Unsigned => writeln!(out, "unsigned_enum! {{")?,
        EnumKind::Signed => writeln!(out, "signed_enum! {{")?,
        EnumKind::UnsignedBitflags => writeln!(out, "unsigned_bits_enum! {{")?,
        EnumKind::SignedBitflags => writeln!(out, "signed_bits_enum! {{")?,
    }

    let enum_name = enum_info
        .name
        .split_once("::")
        .unwrap_or(("", &enum_info.name))
        .1;
    let common_len = find_common_prefix_len(enum_info);

    writeln!(
        out,
        "    {};",
        enum_name
            .trim_start_matches("E")
            .trim_start_matches("VR")
            .split('_')
            .collect::<String>()
    )?;

    fn value_name_rs(common_len: usize, mut value_cpp_name: &str) -> String {
        value_cpp_name = value_cpp_name.split_at(common_len).1;
        let mut value_name = value_cpp_name.split('_').collect::<String>();
        if value_name.as_bytes()[0] == b'e' {
            unsafe { value_name.as_bytes_mut()[0] = b'E' }
        }
        value_name
    }

    fn write_value(
        enum_name: &str,
        value_rs_name: &str,
        value_cpp_name: &str,
        out: &mut impl Write,
    ) -> io::Result<()> {
        if value_cpp_name.starts_with(enum_name) {
            writeln!(
                out,
                "    {} = ::openvr_sys::{},",
                value_rs_name, value_cpp_name
            )
        } else {
            writeln!(
                out,
                "    {} = ::openvr_sys::{}_{},",
                value_rs_name, enum_name, value_cpp_name
            )
        }
    }

    for value_info in &enum_info.values {
        if is_max_entry(&value_info.name) {
            continue;
        }

        let value_name = value_name_rs(common_len, &value_info.name);

        write_value(enum_name, &value_name, &value_info.name, out)?;
    }

    writeln!(out, "}}")
}

enum EnumKind {
    Unsigned,
    Signed,
    UnsignedBitflags,
    SignedBitflags,
}

fn enum_kind(enum_info: &EnumInfo) -> EnumKind {
    let mut existing = 0;
    let mut bits = true;
    let mut signed = false;
    let mut non_zero_cnt = 0;

    for value in &enum_info.values {
        let value: i64 = value.value.parse().unwrap();
        if value < 0 {
            signed = true
        }
        if value != 0 {
            non_zero_cnt += 1;
        }
        if value != -1 {
            if (existing & value) != 0 {
                bits = false;
            }
            existing |= value;
        }
    }

    let prefer_signed = env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows";
    signed |= prefer_signed;

    if non_zero_cnt > 2 && bits {
        if signed {
            EnumKind::SignedBitflags
        } else {
            EnumKind::UnsignedBitflags
        }
    } else {
        if signed {
            EnumKind::Signed
        } else {
            EnumKind::Unsigned
        }
    }
}

fn find_common_prefix_len(enum_info: &EnumInfo) -> usize {
    let mut values = enum_info.values.iter();
    let mut common_prefix: &[u8] = &values.next().unwrap().name.as_bytes();

    for value in values {
        if is_max_entry(&value.name) {
            continue;
        }
        let prefix_count = common_prefix
            .iter()
            .zip(value.name.bytes())
            .take_while(|&(&a, b)| a == b)
            .count();
        common_prefix = &common_prefix[..prefix_count];
    }

    if common_prefix.len() == 0 {
        panic!("err enum: {}", enum_info.name)
    }

    common_prefix
        .iter()
        .rposition(|&x| x == b'_')
        .unwrap_or(common_prefix.len() - 1)
        + 1
}

fn is_max_entry(name: &str) -> bool {
    name.starts_with("MAX_")
        && name
            .as_bytes()
            .iter()
            .all(|&x| x.is_ascii_uppercase() || x == b'_')
}

#[derive(Deserialize)]
struct Json {
    enums: Vec<EnumInfo>,
}

#[derive(Deserialize)]
struct EnumInfo {
    #[serde(rename = "enumname")]
    name: String,
    values: Vec<EnumValueInfo>,
}

#[derive(Deserialize)]
struct EnumValueInfo {
    name: String,
    value: String,
}
