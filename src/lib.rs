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

mod overlay;
pub use overlay::VROverlay;

pub mod structs;
pub use structs::*;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

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
    _markers: PhantomData<(*const (),)>, // !Send & !Sync
}

pub unsafe fn get_generic_interface<T>(
    pch_interface_version: &[u8],
) -> Result<NonNull<T>, InitError> {
    let mut err = 0;
    let ptr = openvr_sys::VR_GetGenericInterface(pch_interface_version.as_ptr().cast(), &mut err);
    NonNull::new(ptr as *mut T).ok_or(InitError::from_raw(err))
}

macro_rules! interface_writer {
    (fn $fn_name: ident -> $wrapper: ident from $name_ref: ident) => {
        pub fn $fn_name(&self) -> Result<$wrapper, InitError> {
            unsafe {
                let ptr = self
                    .$fn_name
                    .get_or_try_init(|| get_generic_interface(openvr_sys::$name_ref))?;
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
            _markers: PhantomData,
        }
    }

    interface_writer!(fn system -> VRSystem from IVRSystem_Version);
    interface_writer!(fn overlay -> VROverlay from IVROverlay_Version);

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
