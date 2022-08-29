use std::ffi::{CStr, CString};
use std::mem::{size_of, zeroed};
use std::os::raw::{c_char, c_ushort};

// this struct is not allowed to construct in Rust.
// returned pointer of VR_IVRSystem_FnTable from C api
// will be converted to &VRSystem
pub struct VRSystem {
    table: openvr_sys::VR_IVRSystem_FnTable,
    _mark: crate::UnConstructable,
}

impl VRSystem {
    pub fn get_recommended_render_target_size(&self) -> (u32, u32) {
        let mut width = 0;
        let mut height = 0;
        unsafe { self.table.GetRecommendedRenderTargetSize.unwrap()(&mut width, &mut height) }
        (width, height)
    }

    pub fn get_projection_matrix(
        &self,
        eye: crate::Eye,
        near_z: f32,
        far_z: f32,
    ) -> crate::HmdMatrix44_t {
        unsafe { self.table.GetProjectionMatrix.unwrap()(eye.as_raw(), near_z, far_z) }
    }

    pub fn get_projection_raw(&self, eye: crate::Eye) -> RawProjection {
        let mut result: RawProjection = unsafe { zeroed() };
        unsafe {
            self.table.GetProjectionRaw.unwrap()(
                eye.as_raw(),
                &mut result.left,
                &mut result.right,
                &mut result.top,
                &mut result.bottom,
            )
        }
        result
    }

    pub fn compute_distortion(
        &self,
        eye: crate::Eye,
        u: f32,
        v: f32,
    ) -> Option<crate::DistortionCoordinates_t> {
        let mut result: crate::DistortionCoordinates_t = unsafe { zeroed() };
        let success =
            unsafe { self.table.ComputeDistortion.unwrap()(eye.as_raw(), u, v, &mut result) };
        some_if!(result; if success)
    }

    pub fn get_eye_to_head_transform(&self, eye: crate::Eye) -> crate::HmdMatrix34_t {
        unsafe { self.table.GetEyeToHeadTransform.unwrap()(eye.as_raw()) }
    }

    pub fn get_time_since_last_vsync(&self) -> Option<TimeSinceLastVsync> {
        let mut result: TimeSinceLastVsync = unsafe { zeroed() };
        let success = unsafe {
            self.table.GetTimeSinceLastVsync.unwrap()(
                &mut result.second_since_last_vsync,
                &mut result.frame_counter,
            )
        };
        some_if!(result; if success)
    }

    pub fn get_d3d9_adapter_index(&self) -> Option<u32> {
        let result = unsafe { self.table.GetD3D9AdapterIndex.unwrap()() };
        some_if!(result as u32; if result != -1)
    }

    pub fn get_dxgi_output_info(&self) -> Option<u32> {
        let mut result: i32 = -1;
        unsafe { self.table.GetDXGIOutputInfo.unwrap()(&mut result) };
        some_if!(result as u32; if result != -1)
    }

    pub fn get_output_device(
        &self,
        texture_type: crate::TextureType,
        instance: impl crate::interlop::VkInstance,
    ) -> u64 {
        let mut result = 0;
        unsafe {
            self.table.GetOutputDevice.unwrap()(
                &mut result,
                texture_type.as_raw(),
                instance.as_pointer() as *mut openvr_sys::VkInstance_T,
            )
        };
        result
    }

    pub fn is_display_on_desktop(&self) -> bool {
        unsafe { self.table.IsDisplayOnDesktop.unwrap()() }
    }

    /// returns true if the change was successful
    pub fn set_display_visibility(&self, value: bool) -> bool {
        unsafe { self.table.SetDisplayVisibility.unwrap()(value) }
    }

    pub fn get_device_to_absolute_tracking_pose(
        &self,
        origin: crate::TrackingUniverseOrigin,
        predicted_seconds_to_phantoms_from_now: f32,
        tracked_device_poses: &mut [crate::TrackedDevicePose_t],
    ) {
        unsafe {
            self.table.GetDeviceToAbsoluteTrackingPose.unwrap()(
                origin.as_raw(),
                predicted_seconds_to_phantoms_from_now,
                tracked_device_poses.as_mut_ptr(),
                tracked_device_poses
                    .len()
                    .try_into()
                    .expect("too big buffer"),
            )
        }
    }

    pub fn reset_seated_zero_pose(&self) {
        unsafe { self.table.ResetSeatedZeroPose.unwrap()() }
    }

    pub fn get_seated_zero_pose_to_standing_absolute_tracking_pose(&self) -> crate::HmdMatrix34_t {
        unsafe {
            self.table
                .GetSeatedZeroPoseToStandingAbsoluteTrackingPose
                .unwrap()()
        }
    }

    pub fn get_raw_zero_pose_to_standing_absolute_tracking_pose(&self) -> crate::HmdMatrix34_t {
        unsafe {
            self.table
                .GetRawZeroPoseToStandingAbsoluteTrackingPose
                .unwrap()()
        }
    }
    /*
    /** Get a sorted array of device indices of a given class of tracked devices (e.g. controllers).  Devices are sorted right to left
	* relative to the specified tracked device (default: hmd -- pass in -1 for absolute tracking space).  Returns the number of devices
	* in the list, or the size of the array needed if not large enough. */
        pub fn get_sorted_tracked_device_indices_of_class(
            &self,
            tracked_device_class: crate::TrackedDeviceClass,
            tracked_device_indices: &mut [crate::TrackedDeviceIndex_t],
            relative_to_tracked_device_index: crate::TrackedDeviceIndex_t
        ) -> u32 {
            // TODO: returns count? IDK
            todo!("no documentation so we cannot provide safe")
        }
    */
    pub fn get_tracked_device_activity_level(
        &self,
        device_id: crate::TrackedDeviceIndex_t,
    ) -> crate::DeviceActivityLevel {
        crate::DeviceActivityLevel::from_raw(unsafe {
            self.table.GetTrackedDeviceActivityLevel.unwrap()(device_id)
        })
    }

    pub fn apply_transform(
        &self,
        tracked_device_pose: &crate::TrackedDevicePose_t,
        transform: &crate::HmdMatrix34_t,
    ) -> crate::TrackedDevicePose_t {
        let mut result: crate::TrackedDevicePose_t = unsafe { zeroed() };

        unsafe {
            self.table.ApplyTransform.unwrap()(
                &mut result,
                as_mut_ptr(tracked_device_pose),
                as_mut_ptr(transform),
            )
        }

        result
    }

    pub fn get_tracked_device_index_for_controller_role(
        &self,
        device_type: crate::TrackedControllerRole,
    ) -> crate::TrackedDeviceIndex_t {
        unsafe { self.table.GetTrackedDeviceIndexForControllerRole.unwrap()(device_type.as_raw()) }
    }

    pub fn get_controller_role_for_tracked_device_index(
        &self,
        device_index: crate::TrackedDeviceIndex_t,
    ) -> crate::TrackedControllerRole {
        crate::TrackedControllerRole::from_raw(unsafe {
            self.table.GetControllerRoleForTrackedDeviceIndex.unwrap()(device_index)
        })
    }

    pub fn get_tracked_device_class(
        &self,
        device_index: crate::TrackedDeviceIndex_t,
    ) -> crate::TrackedDeviceClass {
        crate::TrackedDeviceClass::from_raw(unsafe {
            self.table.GetTrackedDeviceClass.unwrap()(device_index)
        })
    }

    pub fn is_tracked_device_connected(&self, device_index: crate::TrackedDeviceIndex_t) -> bool {
        unsafe { self.table.IsTrackedDeviceConnected.unwrap()(device_index) }
    }
}

macro_rules! device_property {
    ($fn_name: ident, $cfn_name: ident, $result: ty) => {
        pub fn $fn_name(
            &self,
            device_index: crate::TrackedDeviceIndex_t,
            prop: crate::TrackedDeviceProperty,
        ) -> Result<$result, crate::TrackedPropertyError> {
            let mut err = unsafe { zeroed() };
            let result =
                unsafe { self.table.$cfn_name.unwrap()(device_index, prop.as_raw(), &mut err) };
            return_err!(err, crate::TrackedPropertyError, Success);
            Ok(result)
        }
    };
}

impl VRSystem {
    device_property!(
        get_bool_tracked_device_property,
        GetBoolTrackedDeviceProperty,
        bool
    );
    device_property!(
        get_float_tracked_device_property,
        GetFloatTrackedDeviceProperty,
        f32
    );
    device_property!(
        get_int32_tracked_device_property,
        GetInt32TrackedDeviceProperty,
        i32
    );
    device_property!(
        get_uint64_tracked_device_property,
        GetUint64TrackedDeviceProperty,
        u64
    );
    device_property!(
        get_matrix34_tracked_device_property,
        GetMatrix34TrackedDeviceProperty,
        crate::HmdMatrix34_t
    );
    /* Returns an array of one type of property. If the device index is not valid or the property is not a single value or an array of the specified type,
    * this function will return 0. Otherwise it returns the number of bytes necessary to hold the array of properties. If unBufferSize is
    * greater than the returned size and pBuffer is non-NULL, pBuffer is filled with the contents of array of properties. */
    // TODO: GetArrayTrackedDeviceProperty: no docs

    /// returns string without last '\0' char
    pub fn get_string_tracked_device_property(
        &self,
        device_index: crate::TrackedDeviceIndex_t,
        prop: crate::TrackedDeviceProperty,
    ) -> Result<CString, crate::TrackedPropertyError> {
        let mut len: u32 = 1;
        loop {
            let mut buffer = vec![0 as u8; len as usize];

            let mut err = 0;
            len = unsafe {
                self.table.GetStringTrackedDeviceProperty.unwrap()(
                    device_index,
                    prop.as_raw(),
                    buffer.as_mut_ptr() as *mut c_char,
                    buffer.len() as u32,
                    &mut err,
                )
            };
            if err == openvr_sys::ETrackedPropertyError_TrackedProp_Success {
                unsafe {
                    return Ok(CString::from_vec_with_nul_unchecked(buffer));
                }
            } else if err == openvr_sys::ETrackedPropertyError_TrackedProp_BufferTooSmall {
                continue;
            }

            return Err(crate::TrackedPropertyError::from_raw(err));
        }
    }
}

impl VRSystem {
    pub fn get_prop_error_name_from_enum(&self, error: crate::TrackedPropertyError) -> &'_ CStr {
        unsafe { CStr::from_ptr(self.table.GetPropErrorNameFromEnum.unwrap()(error.as_raw())) }
    }

    pub fn poll_next_event(&self) -> Option<crate::VREvent_t> {
        let result: crate::VREvent_t = unsafe { zeroed() };
        let succeed = unsafe {
            self.table.PollNextEvent.unwrap()(
                as_mut_ptr(&result),
                size_of::<crate::VREvent_t>() as u32,
            )
        };
        some_if!(result; if succeed)
    }

    pub fn poll_next_event_with_pose(
        &self,
        origin: crate::TrackingUniverseOrigin,
    ) -> Option<(crate::VREvent_t, crate::TrackedDevicePose_t)> {
        let mut result_event: crate::VREvent_t = unsafe { zeroed() };
        let mut result_pose: crate::TrackedDevicePose_t = unsafe { zeroed() };
        let succeed = unsafe {
            self.table.PollNextEventWithPose.unwrap()(
                origin.as_raw(),
                &mut result_event,
                size_of::<crate::VREvent_t>() as u32,
                as_mut_ptr(&mut result_pose),
            )
        };
        some_if!((result_event, result_pose); if succeed)
    }

    pub fn get_event_type_name_from_enum(&self, event: crate::EventType) -> Option<&'_ CStr> {
        let ptr = unsafe { self.table.GetEventTypeNameFromEnum.unwrap()(event.as_raw()) };
        if ptr.is_null() {
            None
        } else {
            unsafe { Some(CStr::from_ptr(ptr)) }
        }
    }

    pub fn get_hidden_area_mesh(
        &self,
        eye: crate::Eye,
        type_: crate::HiddenAreaMeshType,
    ) -> crate::HiddenAreaMesh_t {
        unsafe { self.table.GetHiddenAreaMesh.unwrap()(eye.as_raw(), type_.as_raw()) }
    }

    pub fn get_controller_state(
        &self,
        controller_device_index: crate::TrackedDeviceIndex_t,
    ) -> Option<crate::VRControllerState_t> {
        let mut result: crate::VRControllerState_t = unsafe { zeroed() };
        let success = unsafe { self.table.GetControllerState.unwrap()(controller_device_index, &mut result, size_of::<crate::VRControllerState_t>() as u32) };
        some_if!(result; if success)
    }

    pub fn get_controller_state_with_pose(
        &self,
        origin: crate::TrackingUniverseOrigin,
        controller_device_index: crate::TrackedDeviceIndex_t,
    ) -> Option<(crate::VRControllerState_t, crate::TrackedDevicePose_t)> {
        let mut result_state: crate::VRControllerState_t = unsafe { zeroed() };
        let mut result_pose: crate::TrackedDevicePose_t = unsafe { zeroed() };
        let success = unsafe { self.table.GetControllerStateWithPose.unwrap()(origin.as_raw(), controller_device_index, &mut result_state, size_of::<crate::VRControllerState_t>() as u32, &mut result_pose) };
        some_if!((result_state, result_pose); if success)
    }

    pub fn trigger_haptic_pulse(
        &self,
        controller_device_index: crate::TrackedDeviceIndex_t,
        axis_id: u32, // TODO
        duration_micro_sec: c_ushort
    ) {
        unsafe { self.table.TriggerHapticPulse.unwrap()(controller_device_index, axis_id, duration_micro_sec) };
    }

    pub fn get_button_id_name_from_enum(
        &self,
        button_id: crate::ButtonId
    ) -> &CStr {
        unsafe { CStr::from_ptr(self.table.GetButtonIdNameFromEnum.unwrap()(button_id.as_raw())) }
    }

    pub fn get_controller_axis_type_name_from_enum(
        &self,
        axis_type: crate::ControllerAxisType
    ) -> &CStr {
        unsafe { CStr::from_ptr(self.table.GetControllerAxisTypeNameFromEnum.unwrap()(axis_type.as_raw())) }
    }

    pub fn is_input_available(&self) -> bool {
        unsafe { self.table.IsInputAvailable.unwrap()() }
    }

    pub fn is_steam_vr_drawing_controllers(&self) -> bool {
        unsafe { self.table.IsSteamVRDrawingControllers.unwrap()() }
    }

    pub fn should_application_pause(&self) -> bool {
        unsafe { self.table.ShouldApplicationPause.unwrap()() }
    }

    pub fn should_application_reduce_rendering_work(&self) -> bool {
        unsafe { self.table.ShouldApplicationReduceRenderingWork.unwrap()() }
    }

    pub fn driver_debug_request(
        &self,
        device_index: crate::TrackedDeviceIndex_t,
        request: &CStr
    ) -> CString {
        unsafe {
            // The maximum response size is 32k
            let mut buffer = Vec::<u8>::with_capacity(32 * 1024);
            let len = self.table.DriverDebugRequest.unwrap()(device_index, as_mut_ptr(&*request.as_ptr()), buffer.as_mut_ptr() as *mut c_char, buffer.capacity() as u32);
            buffer.set_len(len as usize);
            CString::from_vec_with_nul_unchecked(buffer)
        }
    }

    pub fn perform_firmware_update(&self, device_index: crate::TrackedDeviceIndex_t) -> Result<(), crate::FirmwareError> {
        let err = unsafe { self.table.PerformFirmwareUpdate.unwrap()(device_index) };
        return_err!(err, crate::FirmwareError);
        Ok(())
    }

    pub fn acknowledge_quit_exiting(&self) {
        unsafe { self.table.AcknowledgeQuit_Exiting.unwrap()() }
    }

    pub fn acknowledge_quit_user_prompt(&self) {
        unsafe { self.table.AcknowledgeQuit_UserPrompt.unwrap()() }
    }
}

pub struct RawProjection {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

pub struct TimeSinceLastVsync {
    second_since_last_vsync: f32,
    frame_counter: u64,
}

#[inline(always)]
unsafe fn as_mut_ptr<T>(value: &T) -> *mut T {
    value as *const T as *mut T
}
