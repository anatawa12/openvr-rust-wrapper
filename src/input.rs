use crate::as_mut_ptr;
use memchr::memchr;
use std::ffi::{CStr, CString};
use std::mem::{size_of, size_of_val, zeroed};
use std::ptr::{null, null_mut};

/// The reference to VRInput. this is same size as pointer
#[derive(Copy, Clone)]
pub struct VRInput<'a> {
    table: &'a openvr_sys::VR_IVRInput_FnTable,
}
wrapper_layout_test!(vrsystem_layout_test for VRInput as * const openvr_sys::VR_IVRInput_FnTable);

type Result<T = ()> = std::result::Result<T, crate::InputError>;

fn mk_err(err: openvr_sys::EVRInputError) -> Result {
    return_err!(err, crate::InputError)
}

impl<'a> VRInput<'a> {
    pub(crate) fn new(table: &'a openvr_sys::VR_IVRInput_FnTable) -> Self {
        Self { table }
    }
}

impl<'a> VRInput<'a> {
    pub fn set_action_manifest_path(self, action_manifest_path: &CStr) -> Result {
        unsafe {
            mk_err(self.table.SetActionManifestPath.unwrap()(
                action_manifest_path.as_ptr() as _,
            ))
        }
    }

    pub fn get_action_set_handle(
        self,
        action_set_name: &CStr,
    ) -> Result<crate::VRActionSetHandle_t> {
        unsafe {
            let mut result = 0;
            mk_err(self.table.GetActionSetHandle.unwrap()(
                action_set_name.as_ptr() as _,
                &mut result,
            ))?;
            Ok(result)
        }
    }

    pub fn get_action_handle(self, action_name: &CStr) -> Result<crate::VRActionHandle_t> {
        unsafe {
            let mut result = 0;
            mk_err(self.table.GetActionHandle.unwrap()(
                action_name.as_ptr() as _,
                &mut result,
            ))?;
            Ok(result)
        }
    }

    pub fn get_input_source_handle(
        self,
        input_source_path: &CStr,
    ) -> Result<crate::VRInputValueHandle_t> {
        unsafe {
            let mut result = 0;
            mk_err(self.table.GetInputSourceHandle.unwrap()(
                input_source_path.as_ptr() as _,
                &mut result,
            ))?;
            Ok(result)
        }
    }

    pub fn update_action_state(self, action_set: &[crate::VRActiveActionSet_t]) -> Result {
        unsafe {
            mk_err(self.table.UpdateActionState.unwrap()(
                action_set.as_ptr() as _,
                size_of::<crate::VRActiveActionSet_t>() as u32,
                action_set.len() as u32,
            ))
        }
    }
}

macro_rules! get_action_data {
    ($fn_name: ident from $vtable_name: ident -> $result_ty: ident) => {
        pub fn $fn_name(
            self,
            action: crate::VRActionHandle_t,
            restrict_to_device: crate::VRInputValueHandle_t,
        ) -> Result<crate::$result_ty> {
            unsafe {
                let mut result = zeroed();
                mk_err(self.table.$vtable_name.unwrap()(
                    action,
                    &mut result,
                    size_of::<crate::$result_ty>() as u32,
                    restrict_to_device,
                ))?;
                Ok(result)
            }
        }
    };
}

impl<'a> VRInput<'a> {
    get_action_data!(get_digital_action_data from GetDigitalActionData -> InputDigitalActionData_t);
    get_action_data!(get_analog_action_data from GetAnalogActionData -> InputAnalogActionData_t);

    pub fn get_pose_action_data_relative_to_now(
        self,
        action: crate::VRActionHandle_t,
        origin: crate::TrackingUniverseOrigin,
        predicted_seconds_from_now: f32,
        restrict_to_device: crate::VRInputValueHandle_t,
    ) -> Result<crate::InputPoseActionData_t> {
        unsafe {
            let mut result = zeroed();
            mk_err(self.table.GetPoseActionDataRelativeToNow.unwrap()(
                action,
                origin.as_raw(),
                predicted_seconds_from_now,
                &mut result,
                size_of::<crate::InputPoseActionData_t>() as u32,
                restrict_to_device,
            ))?;
            Ok(result)
        }
    }

    pub fn get_pose_action_data_for_next_frame(
        self,
        action: crate::VRActionHandle_t,
        origin: crate::TrackingUniverseOrigin,
        restrict_to_device: crate::VRInputValueHandle_t,
    ) -> Result<crate::InputPoseActionData_t> {
        unsafe {
            let mut result = zeroed();
            mk_err(self.table.GetPoseActionDataForNextFrame.unwrap()(
                action,
                origin.as_raw(),
                &mut result,
                size_of::<crate::InputPoseActionData_t>() as u32,
                restrict_to_device,
            ))?;
            Ok(result)
        }
    }

    pub fn get_skeletal_action_data(
        self,
        action: crate::VRActionHandle_t,
    ) -> Result<crate::InputSkeletalActionData_t> {
        unsafe {
            let mut result = zeroed();
            mk_err(self.table.GetSkeletalActionData.unwrap()(
                action,
                &mut result,
                size_of::<crate::InputSkeletalActionData_t>() as u32,
            ))?;
            Ok(result)
        }
    }

    pub fn get_bone_count(self, action: crate::VRActionHandle_t) -> Result<u32> {
        unsafe {
            let mut result = 0;
            mk_err(self.table.GetBoneCount.unwrap()(action, &mut result))?;
            Ok(result)
        }
    }

    pub fn get_bone_hierarchy(
        self,
        action: crate::VRActionHandle_t,
        parent_id: &mut [crate::BoneIndex_t],
    ) -> Result {
        unsafe {
            mk_err(self.table.GetBoneHierarchy.unwrap()(
                action,
                parent_id.as_mut_ptr(),
                parent_id.len() as u32,
            ))
        }
    }

    pub fn get_bone_name(
        self,
        action: crate::VRActionHandle_t,
        bone_index: crate::BoneIndex_t,
    ) -> Result<CString> {
        unsafe {
            let mut buffer = Vec::<u8>::with_capacity(openvr_sys::k_unMaxBoneNameLength as usize);
            buffer.set_len(buffer.capacity());
            mk_err(self.table.GetBoneName.unwrap()(
                action,
                bone_index,
                buffer.as_mut_ptr() as _,
                buffer.capacity() as _,
            ))?;
            let strlen = memchr(0, &buffer).expect("incorrect text response");
            buffer.set_len(strlen);
            Ok(CString::from_vec_with_nul_unchecked(buffer))
        }
    }

    /// transforms.len() should be number of bones.
    pub fn get_skeletal_reference_transforms(
        self,
        action: crate::VRActionHandle_t,
        transform_space: crate::SkeletalTransformSpace,
        reference_pose: crate::SkeletalReferencePose,
        transforms: &mut [crate::VRBoneTransform_t],
    ) -> Result {
        unsafe {
            mk_err(self.table.GetSkeletalReferenceTransforms.unwrap()(
                action,
                transform_space.as_raw(),
                reference_pose.as_raw(),
                transforms.as_mut_ptr(),
                transforms.len() as _,
            ))
        }
    }

    pub fn get_skeletal_tracking_level(
        self,
        action: crate::VRActionHandle_t,
    ) -> Result<crate::SkeletalTrackingLevel> {
        unsafe {
            let mut result = 0;
            mk_err(self.table.GetSkeletalTrackingLevel.unwrap()(
                action,
                &mut result,
            ))?;
            Ok(crate::SkeletalTrackingLevel::from_raw(result))
        }
    }

    pub fn get_skeletal_bone_data(
        self,
        action: crate::VRActionHandle_t,
        transform_space: crate::SkeletalTransformSpace,
        motion_range: crate::SkeletalMotionRange,
        transform_array: &mut [crate::VRBoneTransform_t],
    ) -> Result {
        unsafe {
            mk_err(self.table.GetSkeletalBoneData.unwrap()(
                action,
                transform_space.as_raw(),
                motion_range.as_raw(),
                transform_array.as_mut_ptr(),
                transform_array.len() as _,
            ))
        }
    }

    pub fn get_skeletal_summary_data(
        self,
        action: crate::VRActionHandle_t,
        summary_type: crate::SummaryType,
    ) -> Result<crate::VRSkeletalSummaryData_t> {
        unsafe {
            let mut result = zeroed();
            mk_err(self.table.GetSkeletalSummaryData.unwrap()(
                action,
                summary_type.as_raw(),
                &mut result,
            ))?;
            Ok(result)
        }
    }

    pub fn get_skeletal_bone_data_compressed(
        self,
        action: crate::VRActionHandle_t,
        motion_range: crate::SkeletalMotionRange,
    ) -> Result<Vec<u8>> {
        let mut buffer = Vec::<u8>::new();
        loop {
            unsafe {
                let mut required_len: u32 = 0;
                let err = self.table.GetSkeletalBoneDataCompressed.unwrap()(
                    action,
                    motion_range.as_raw(),
                    buffer.as_mut_ptr() as _,
                    buffer.capacity() as _,
                    &mut required_len as _,
                );
                if err == openvr_sys::EVRInputError_VRInputError_BufferTooSmall {
                    buffer.reserve(required_len as _);
                    continue;
                }
                mk_err(err)?;
                buffer.set_len(required_len as _);
                return Ok(buffer);
            }
        }
    }

    pub fn decompress_skeletal_bone_data(
        self,
        compressed_buffer: &[u8],
        transform_space: crate::SkeletalTransformSpace,
        transform_array: &mut [crate::VRBoneTransform_t],
    ) -> Result {
        unsafe {
            mk_err(self.table.DecompressSkeletalBoneData.unwrap()(
                compressed_buffer.as_ptr() as _,
                compressed_buffer.len() as _,
                transform_space.as_raw(),
                transform_array.as_mut_ptr(),
                transform_array.len() as _,
            ))
        }
    }

    pub fn trigger_haptic_vibration_action(
        self,
        action: crate::VRActionHandle_t,
        start_seconds_from_now: f32,
        duration_seconds: f32,
        frequency: f32,
        amplitude: f32,
        restrict_to_device: crate::VRInputValueHandle_t,
    ) -> Result {
        unsafe {
            mk_err(self.table.TriggerHapticVibrationAction.unwrap()(
                action,
                start_seconds_from_now,
                duration_seconds,
                frequency,
                amplitude,
                restrict_to_device,
            ))
        }
    }

    pub fn get_action_origins(
        self,
        action_set_handle: crate::VRActionSetHandle_t,
        digital_action_handle: crate::VRActionHandle_t,
        origins: &mut [crate::VRInputValueHandle_t],
    ) -> Result {
        unsafe {
            mk_err(self.table.GetActionOrigins.unwrap()(
                action_set_handle,
                digital_action_handle,
                origins.as_mut_ptr(),
                origins.len() as _,
            ))
        }
    }

    // buffer size unknown
    //pub fn get_origin_localized_name(self, origin: crate::VRInputValueHandle_t)

    pub fn get_origin_tracked_device_info(
        self,
        origin: crate::VRInputValueHandle_t,
    ) -> Result<crate::InputOriginInfo_t> {
        unsafe {
            let mut result = zeroed();
            mk_err(self.table.GetOriginTrackedDeviceInfo.unwrap()(
                origin,
                &mut result,
                size_of_val(&result) as _,
            ))?;
            Ok(result)
        }
    }

    pub fn show_action_origins(
        self,
        action_set_handle: crate::VRActionSetHandle_t,
        action_handle: crate::VRActionHandle_t,
    ) -> Result {
        unsafe {
            mk_err(self.table.ShowActionOrigins.unwrap()(
                action_set_handle,
                action_handle,
            ))
        }
    }

    pub fn show_bindings_for_action_set(
        self,
        sets: &[crate::VRActiveActionSet_t],
        origin_to_highlight: crate::VRInputValueHandle_t,
    ) -> Result {
        unsafe {
            mk_err(self.table.ShowBindingsForActionSet.unwrap()(
                sets.as_ptr() as _,
                size_of::<crate::VRActiveActionSet_t>() as _,
                sets.len() as _,
                origin_to_highlight,
            ))
        }
    }

    pub fn is_using_legacy_input(self) -> bool {
        unsafe { self.table.IsUsingLegacyInput.unwrap()() }
    }
}
