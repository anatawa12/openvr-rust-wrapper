pub use openvr_sys::BoneIndex_t;
pub use openvr_sys::DistortionCoordinates_t;
pub use openvr_sys::HiddenAreaMesh_t;
pub use openvr_sys::HmdMatrix33_t;
pub use openvr_sys::HmdMatrix34_t;
pub use openvr_sys::HmdMatrix44_t;
pub use openvr_sys::HmdQuad_t;
pub use openvr_sys::HmdRect2_t;
pub use openvr_sys::HmdVector2_t;
pub use openvr_sys::HmdVector3_t;
pub use openvr_sys::HmdVector4_t;
pub use openvr_sys::InputAnalogActionData_t;
pub use openvr_sys::InputDigitalActionData_t;
pub use openvr_sys::InputOriginInfo_t;
pub use openvr_sys::InputPoseActionData_t;
pub use openvr_sys::InputSkeletalActionData_t;
pub use openvr_sys::SpatialAnchorPose_t;
//pub use openvr_sys::Texture_t; use OverlayTexture below
pub use openvr_sys::InputBindingInfo_t;
pub use openvr_sys::TrackedDeviceIndex_t;
pub use openvr_sys::TrackedDevicePose_t;
pub use openvr_sys::VRActionHandle_t;
pub use openvr_sys::VRActionSetHandle_t;
pub use openvr_sys::VRActiveActionSet_t;
pub use openvr_sys::VRBoneTransform_t;
pub use openvr_sys::VRControllerState_t;
pub use openvr_sys::VREvent_t;
pub use openvr_sys::VRInputValueHandle_t;
pub use openvr_sys::VROverlayHandle_t;
pub use openvr_sys::VROverlayIntersectionMaskPrimitive_t;
pub use openvr_sys::VROverlayIntersectionParams_t;
pub use openvr_sys::VROverlayIntersectionResults_t;
pub use openvr_sys::VROverlayProjection_t;
pub use openvr_sys::VRSkeletalSummaryData_t;
pub use openvr_sys::VRTextureBounds_t;

pub struct OverlayTexture {
    pub handle: *mut std::os::raw::c_void,
    pub tex_type: crate::TextureType,
    pub color_space: crate::ColorSpace,
}

impl From<OverlayTexture> for openvr_sys::Texture_t {
    fn from(t: OverlayTexture) -> Self {
        Self {
            handle: t.handle,
            eType: t.tex_type.as_raw(),
            eColorSpace: t.color_space.as_raw(),
        }
    }
}
