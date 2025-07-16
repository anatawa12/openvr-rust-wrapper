extern crate core;
macro_rules! c_like_enum {
    ($name: ident as $ty: ty; $($value: ident = $expr: expr,)*) => {
        #[derive(Copy, Clone, PartialEq, Eq, Debug)]
        pub struct $name($ty);
        #[allow(non_upper_case_globals)]
        impl $name {
            $(pub const $value: $name = $name($expr);)*
        }

        impl $name {
            pub fn as_raw(self) -> $ty {
                self.0
            }

            pub fn from_raw(raw: $ty) -> Self {
                Self(raw)
            }
        }
    };
}

macro_rules! bits_enum_display {
    ($name: ident = $all: expr; $($value: ident = $expr: expr,)*) => {
        impl ::core::fmt::Display for $name {
            #[allow(unused_assignments)]
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut inner = self.0;
                // shorthand for all.
                if inner == $all {
                    return f.write_str(concat!(stringify!($name), "(All)"));
                }
                // shorthand for 0
                if inner == 0 {
                    return $( if $expr == 0 {
                        f.write_str(concat!(stringify!($name), "(", stringify!($value), ")"))
                    } else )* {
                        f.write_str(concat!(stringify!($name), "(0)"))
                    }
                }

                f.write_str(concat!(stringify!($name), "("))?;

                let mut written = false;

                // write bitflags (names that is not zero)
                $( if $expr != 0 && (inner & $expr) == $expr {
                    if written {
                        f.write_str(" | ")?;
                    }
                    f.write_str(stringify!($value))?;
                    written = true;
                    inner = inner & !$expr;
                })*

                if inner != 0 {
                    // there's rest elements
                    write!(f, " | {:#8x}", inner)?;
                }
                f.write_str(")")
            }
        }
    };
}

macro_rules! simple_enum_display {
    ($name: ident; $($value: ident = $expr: expr,)*) => {
        impl ::core::fmt::Display for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                $( if self.0 == $expr {
                    f.write_str(concat!(stringify!($name), "(", stringify!($name), ")"))
                } else )* {
                    write!(f, "{}({})", stringify!($name), self.0)
                }
            }
        }
    };
}

macro_rules! unsigned_bits_enum {
    ($name: ident; $($value: ident = $expr: expr,)*) => {
        c_like_enum!{$name as u32; $($value = $expr,)*}
        bits_enum_display!{$name = u32::MAX; $($value = $expr,)*}
    };
}

macro_rules! signed_bits_enum {
    ($name: ident; $($value: ident = $expr: expr,)*) => {
        c_like_enum!{$name as i32; $($value = $expr,)*}
        bits_enum_display!{$name = -1; $($value = $expr,)*}
    };
}

macro_rules! unsigned_enum {
    ($name: ident; $($value: ident = $expr: expr,)*) => {
        c_like_enum!{$name as u32; $($value = $expr,)*}
        simple_enum_display!{$name; $($value = $expr,)*}
    };
}

macro_rules! signed_enum {
    ($name: ident; $($value: ident = $expr: expr,)*) => {
        c_like_enum!{$name as i32; $($value = $expr,)*}
        simple_enum_display!{$name; $($value = $expr,)*}
    };
}

macro_rules! return_err {
    ($err_value: expr, $($ty: ident)::+) => {
        return_err!($err_value, $($ty)::+, None)
    };
    ($err_value: expr, $($ty: ident)::+, $success: ident) => {
        {
            let err = $($ty)::+::from_raw($err_value);
            if !matches!(err, $($ty)::+::$success) {
                Err(err)
            } else {
                Ok(())
            }
        }
    };
}

macro_rules! some_if {
    ($value: expr ;if $success: expr) => {
        if $success {
            Some($value)
        } else {
            None
        }
    };
}

macro_rules! wrapper_layout_test {
    ($fn_name: ident for $ty: ty as $c: ty) => {
        #[test]
        fn vr_system_mem_layout() {
            std::assert_eq!(
                ::std::mem::size_of::<$ty>(),
                ::std::mem::size_of::<$c>(),
                concat!("Size of: ", stringify!($ty))
            );
            std::assert_eq!(
                ::std::mem::align_of::<$ty>(),
                ::std::mem::align_of::<$c>(),
                concat!("Alignment of: ", stringify!($ty))
            );
            //std::assert_eq!(
            //    unsafe { &(*(::std::ptr::null::<$ty>())).table as *const _ as usize },
            //    0,
            //    concat!("Offset of: ", stringify!($ty), "::table")
            //);
        }
    };
}

#[macro_export]
macro_rules! cstr {
    ($str: literal) => {{
        unsafe {
            ::std::ffi::CStr::from_bytes_with_nul_unchecked(::std::concat!($str, "\0").as_bytes())
        }
    }};
}

mod internal {
    pub trait Sealed {}
}

#[inline(always)]
unsafe fn as_mut_ptr<T>(value: &T) -> *mut T {
    value as *const T as *mut T
}

pub(crate) use internal::Sealed;
use once_cell::unsync::OnceCell;
use std::marker::PhantomData;
use std::ptr::NonNull;

pub mod interlop;
pub mod system;

pub use system::VRSystem;

pub mod overlay;
pub use overlay::VROverlay;

pub mod input;
pub use input::VRInput;

pub mod applications;
pub use applications::VRApplications;

pub mod structs;
pub use structs::*;

pub mod enums {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}
pub use enums::*;

pub fn init(app_type: ApplicationType) -> Result<VRContext, InitError> {
    let mut err: openvr_sys::EVRInitError = 0;
    let token = unsafe { openvr_sys::VR_InitInternal(&mut err, app_type.as_raw()) };
    return_err!(err, InitError)?;

    let system = VRContext::new(token);

    if !unsafe {
        openvr_sys::VR_IsInterfaceVersionValid(openvr_sys::IVRSystem_Version.as_ptr() as *const i8)
    } {
        // version mismatch
        system.shutdown();
        return Err(InitError::InitInterfaceNotFound);
    }

    Ok(system)
}

pub struct VRContext {
    _token: isize,
    system: OnceCell<NonNull<openvr_sys::VR_IVRSystem_FnTable>>,
    overlay: OnceCell<NonNull<openvr_sys::VR_IVROverlay_FnTable>>,
    input: OnceCell<NonNull<openvr_sys::VR_IVRInput_FnTable>>,
    application: OnceCell<NonNull<openvr_sys::VR_IVRApplications_FnTable>>,
    _markers: PhantomData<(*const (),)>, // !Send & !Sync
}

pub unsafe fn get_function_table<T>(pch_interface_version: &[u8]) -> Result<NonNull<T>, InitError> {
    unsafe {
        let mut err = 0;
        let mut table_len =
            Vec::<u8>::with_capacity(b"FnTable:".len() + pch_interface_version.len());
        table_len.extend_from_slice(b"FnTable:");
        table_len.extend_from_slice(pch_interface_version);
        let ptr = openvr_sys::VR_GetGenericInterface(table_len.as_ptr().cast(), &mut err);
        NonNull::new(ptr as *mut T).ok_or(InitError::from_raw(err))
    }
}

macro_rules! interface_writer {
    (fn $fn_name: ident -> $wrapper: ident from $name_ref: ident) => {
        pub fn $fn_name(&self) -> Result<$wrapper, InitError> {
            unsafe {
                let ptr = self
                    .$fn_name
                    .get_or_try_init(|| get_function_table(openvr_sys::$name_ref))?;
                Ok($wrapper::new(&*ptr.as_ptr().cast()))
            }
        }
    };
}

impl VRContext {
    pub(crate) fn new(token: isize) -> VRContext {
        VRContext {
            _token: token,
            system: OnceCell::new(),
            overlay: OnceCell::new(),
            input: OnceCell::new(),
            application: OnceCell::new(),
            _markers: PhantomData,
        }
    }

    interface_writer!(fn system -> VRSystem from IVRSystem_Version);
    interface_writer!(fn overlay -> VROverlay from IVROverlay_Version);
    interface_writer!(fn input -> VRInput from IVRInput_Version);
    interface_writer!(fn application -> VRApplications from IVRApplications_Version);

    pub fn shutdown(self) {
        // drop does
    }
}

impl Drop for VRContext {
    fn drop(&mut self) {
        unsafe {
            openvr_sys::VR_ShutdownInternal();
        }
    }
}
