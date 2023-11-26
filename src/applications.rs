use std::ffi::CStr;

/// The reference to VRInput. this is same size as pointer
#[derive(Copy, Clone)]
pub struct VRApplications<'a> {
    table: &'a openvr_sys::VR_IVRApplications_FnTable,
}
wrapper_layout_test!(vrsystem_layout_test for VRApplications as * const openvr_sys::VR_IVRApplications_FnTable);

type Result<T = ()> = std::result::Result<T, crate::ApplicationError>;

fn mk_err(err: openvr_sys::EVRApplicationError) -> Result {
    return_err!(err, crate::ApplicationError)
}

impl<'a> VRApplications<'a> {
    pub(crate) fn new(table: &'a openvr_sys::VR_IVRApplications_FnTable) -> Self {
        Self { table }
    }
}

impl<'a> VRApplications<'a> {
    pub fn is_application_installed(self, action_manifest_path: &CStr) -> bool {
        unsafe { self.table.IsApplicationInstalled.unwrap()(action_manifest_path.as_ptr() as _) }
    }

    pub fn add_application_manifest(self, action_manifest_path: &CStr, temporary: bool) -> Result {
        // temporary: false by default for cpp
        unsafe {
            mk_err(self.table.AddApplicationManifest.unwrap()(
                action_manifest_path.as_ptr() as _,
                temporary,
            ))
        }
    }

    pub fn remove_application_manifest(self, action_manifest_path: &CStr) -> Result {
        unsafe {
            mk_err(self.table.RemoveApplicationManifest.unwrap()(
                action_manifest_path.as_ptr() as _,
            ))
        }
    }
}
