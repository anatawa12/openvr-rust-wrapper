extern crate core;
macro_rules! c_like_enum {
    ($name: ident as $ty: ty; $($value: ident = $expr: expr,)*) => {
        #[derive(Copy, Clone, PartialEq, Eq)]
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

macro_rules! bits_enum {
    ($name: ident; $($value: ident = $expr: expr,)*) => {
        c_like_enum!{$name as u32; $($value = $expr,)*}
    };
}

macro_rules! unsigned_enum {
    ($name: ident; $($value: ident = $expr: expr,)*) => {
        c_like_enum!{$name as u32; $($value = $expr,)*}
    };
}

macro_rules! signed_enum {
    ($name: ident; $($value: ident = $expr: expr,)*) => {
        c_like_enum!{$name as i32; $($value = $expr,)*}
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
                return Err(err)
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

mod util {
    pub(crate) struct UnConstructable {
        _internal: (),
    }
}

pub(crate) use util::UnConstructable;

pub mod interlop;
pub mod system;

pub use system::VRSystem;

pub mod structs;
pub use structs::*;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

pub fn init(app_type: ApplicationType) -> Result<VRContext, InitError> {
    let mut err: openvr_sys::EVRInitError = 0;
    let token = unsafe { openvr_sys::VR_InitInternal(&mut err, app_type.as_raw()) };
    return_err!(err, InitError);

    let system = VRContext { _token: token };

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
}

impl VRContext {
    pub fn shutdown(self) {
        unsafe {
            openvr_sys::VR_ShutdownInternal();
        }
    }
}
