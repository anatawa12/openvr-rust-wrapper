use crate::as_mut_ptr;
use std::ffi::{CStr, CString};
use std::mem::{forget, size_of, size_of_val, zeroed};
use std::ptr::{null, null_mut};
use std::time::Duration;

/// The reference to VROverlay. this is same size as pointer
#[derive(Copy, Clone)]
pub struct VROverlay<'a> {
    table: &'a openvr_sys::VR_IVROverlay_FnTable,
}
wrapper_layout_test!(vrsystem_layout_test for VROverlay as * const openvr_sys::VR_IVROverlay_FnTable);

type Result<T = ()> = std::result::Result<T, crate::OverlayError>;

fn mk_err(err: openvr_sys::EVROverlayError) -> Result {
    return_err!(err, crate::OverlayError)
}

impl<'a> VROverlay<'a> {
    pub(crate) fn new(table: &'a openvr_sys::VR_IVROverlay_FnTable) -> Self {
        Self { table }
    }
}

impl<'a> VROverlay<'a> {
    pub fn find_overlay(self, overlay_key: &CStr) -> Result<crate::VROverlayHandle_t> {
        unsafe {
            let mut handle = 0;
            let err = self.table.FindOverlay.unwrap()(overlay_key.as_ptr() as _, &mut handle);
            mk_err(err)?;
            Ok(handle)
        }
    }

    pub fn create_overlay(
        self,
        overlay_key: &CStr,
        overlay_name: &CStr,
    ) -> Result<crate::VROverlayHandle_t> {
        unsafe {
            let mut handle = 0;
            let err = self.table.CreateOverlay.unwrap()(
                overlay_key.as_ptr() as _,
                overlay_name.as_ptr() as _,
                &mut handle,
            );
            mk_err(err)?;
            Ok(handle)
        }
    }

    pub fn destroy_overlay(self, handle: crate::VROverlayHandle_t) -> Result {
        unsafe { mk_err(self.table.DestroyOverlay.unwrap()(handle)) }
    }

    pub fn get_overlay_key(self, handle: crate::VROverlayHandle_t) -> Result<CString> {
        let mut buffer: Vec<u8> = Vec::with_capacity(openvr_sys::k_unVROverlayMaxKeyLength as _);
        unsafe {
            let ptr = buffer.as_mut_ptr() as _;
            let cap = buffer.capacity() as u32;
            let mut err = 0;
            let len = self.table.GetOverlayKey.unwrap()(handle, ptr, cap, &mut err);
            mk_err(err)?;

            buffer.reserve(len as usize);
            buffer.set_len(len as usize);
            return Ok(CString::from_vec_with_nul_unchecked(buffer));
        }
    }

    pub fn get_overlay_name(self, handle: crate::VROverlayHandle_t) -> Result<CString> {
        let mut buffer: Vec<u8> = Vec::with_capacity(openvr_sys::k_unVROverlayMaxNameLength as _);
        unsafe {
            let ptr = buffer.as_mut_ptr() as _;
            let cap = buffer.capacity() as u32;
            let mut err = 0;
            let len = self.table.GetOverlayName.unwrap()(handle, ptr, cap, &mut err);
            mk_err(err)?;

            buffer.reserve(len as usize);
            buffer.set_len(len as usize);
            return Ok(CString::from_vec_with_nul_unchecked(buffer));
        }
    }

    pub fn set_overlay_name(self, handle: crate::VROverlayHandle_t, name: &CStr) -> Result {
        unsafe {
            mk_err(self.table.SetOverlayName.unwrap()(
                handle,
                name.as_ptr() as _,
            ))
        }
    }

    pub fn get_overlay_image_data(
        self,
        handle: crate::VROverlayHandle_t,
    ) -> Result<OverlayImageData> {
        unsafe {
            let mut width = 0;
            let mut height = 0;
            let err = self.table.GetOverlayImageData.unwrap()(
                handle,
                null_mut(),
                0,
                &mut width,
                &mut height,
            );
            if err != openvr_sys::EVROverlayError_VROverlayError_ArrayTooSmall {
                return Err(crate::OverlayError::from_raw(err));
            }
            let mut data = Vec::<u8>::with_capacity((width * height * 4) as usize);
            let err = self.table.GetOverlayImageData.unwrap()(
                handle,
                data.as_mut_ptr().cast(),
                data.capacity() as u32,
                &mut width,
                &mut height,
            );
            mk_err(err)?;
            Ok(OverlayImageData {
                width,
                height,
                data,
            })
        }
    }

    pub fn get_overlay_error_name_from_enum(self, error: crate::OverlayError) -> &'a CStr {
        unsafe {
            CStr::from_ptr(self.table.GetOverlayErrorNameFromEnum.unwrap()(
                error.as_raw(),
            ))
        }
    }

    pub fn set_overlay_rendering_pid(self, handle: crate::VROverlayHandle_t, pid: u32) -> Result {
        unsafe { mk_err(self.table.SetOverlayRenderingPid.unwrap()(handle, pid)) }
    }

    pub fn get_overlay_rendering_pid(self, handle: crate::VROverlayHandle_t) -> u32 {
        unsafe { self.table.GetOverlayRenderingPid.unwrap()(handle) }
    }

    pub fn set_overlay_flag(
        self,
        handle: crate::VROverlayHandle_t,
        flag: crate::OverlayFlags,
        enabled: bool,
    ) -> Result {
        unsafe {
            mk_err(self.table.SetOverlayFlag.unwrap()(
                handle,
                flag.as_raw(),
                enabled,
            ))
        }
    }

    pub fn get_overlay_flag(
        self,
        handle: crate::VROverlayHandle_t,
        flag: crate::OverlayFlags,
    ) -> Result<bool> {
        unsafe {
            let mut enabled: bool = false;
            let err = self.table.GetOverlayFlag.unwrap()(handle, flag.as_raw(), &mut enabled);
            mk_err(err)?;
            Ok(enabled)
        }
    }

    pub fn get_overlay_flags(
        self,
        handle: crate::VROverlayHandle_t,
    ) -> Result<crate::OverlayFlags> {
        unsafe {
            let mut result = 0;
            let err = self.table.GetOverlayFlags.unwrap()(handle, &mut result);
            mk_err(err)?;
            Ok(crate::OverlayFlags::from_raw(result))
        }
    }

    pub fn set_overlay_color(
        self,
        handle: crate::VROverlayHandle_t,
        red: f32,
        green: f32,
        blue: f32,
    ) -> Result {
        unsafe {
            mk_err(self.table.SetOverlayColor.unwrap()(
                handle, red, green, blue,
            ))
        }
    }

    pub fn get_overlay_color(self, handle: crate::VROverlayHandle_t) -> Result<(f32, f32, f32)> {
        unsafe {
            let mut color: (f32, f32, f32) = (0.0, 0.0, 0.0);
            let err = self.table.GetOverlayColor.unwrap()(
                handle,
                &mut color.0,
                &mut color.1,
                &mut color.2,
            );
            mk_err(err)?;
            Ok(color)
        }
    }

    pub fn set_overlay_alpha(self, handle: crate::VROverlayHandle_t, alpha: f32) -> Result {
        unsafe { mk_err(self.table.SetOverlayAlpha.unwrap()(handle, alpha)) }
    }

    pub fn get_overlay_alpha(self, handle: crate::VROverlayHandle_t) -> Result<f32> {
        unsafe {
            let mut alpha: f32 = 0.0;
            let err = self.table.GetOverlayAlpha.unwrap()(handle, &mut alpha);
            mk_err(err)?;
            Ok(alpha)
        }
    }

    pub fn set_overlay_texel_aspect(self, handle: crate::VROverlayHandle_t, aspect: f32) -> Result {
        unsafe { mk_err(self.table.SetOverlayTexelAspect.unwrap()(handle, aspect)) }
    }

    pub fn get_overlay_texel_aspect(self, handle: crate::VROverlayHandle_t) -> Result<f32> {
        unsafe {
            let mut aspect: f32 = 0.0;
            let err = self.table.GetOverlayTexelAspect.unwrap()(handle, &mut aspect);
            mk_err(err)?;
            Ok(aspect)
        }
    }

    pub fn set_overlay_sort_order(self, handle: crate::VROverlayHandle_t, order: u32) -> Result {
        unsafe { mk_err(self.table.SetOverlaySortOrder.unwrap()(handle, order)) }
    }

    pub fn get_overlay_sort_order(self, handle: crate::VROverlayHandle_t) -> Result<u32> {
        unsafe {
            let mut order: u32 = 0;
            let err = self.table.GetOverlaySortOrder.unwrap()(handle, &mut order);
            mk_err(err)?;
            Ok(order)
        }
    }

    pub fn set_overlay_width_in_meters(
        self,
        handle: crate::VROverlayHandle_t,
        width: f32,
    ) -> Result {
        unsafe { mk_err(self.table.SetOverlayWidthInMeters.unwrap()(handle, width)) }
    }

    pub fn get_overlay_width_in_meters(self, handle: crate::VROverlayHandle_t) -> Result<f32> {
        unsafe {
            let mut width: f32 = 0.0;
            let err = self.table.GetOverlayWidthInMeters.unwrap()(handle, &mut width);
            mk_err(err)?;
            Ok(width)
        }
    }

    pub fn set_overlay_curvature(self, handle: crate::VROverlayHandle_t, curvature: f32) -> Result {
        unsafe {
            let err = self.table.SetOverlayCurvature.unwrap()(handle, curvature);
            mk_err(err)
        }
    }

    pub fn get_overlay_curvature(self, handle: crate::VROverlayHandle_t) -> Result<f32> {
        unsafe {
            let mut curvature = 0f32;
            let err = self.table.GetOverlayCurvature.unwrap()(handle, &mut curvature);
            mk_err(err)?;
            Ok(curvature)
        }
    }

    pub fn set_overlay_pre_curve_pitch(
        self,
        handle: crate::VROverlayHandle_t,
        radians: f32,
    ) -> Result {
        unsafe { mk_err(self.table.SetOverlayPreCurvePitch.unwrap()(handle, radians)) }
    }

    pub fn get_overlay_pre_curve_pitch(self, handle: crate::VROverlayHandle_t) -> Result<f32> {
        unsafe {
            let mut curvature = 0f32;
            mk_err(self.table.GetOverlayPreCurvePitch.unwrap()(
                handle,
                &mut curvature,
            ))?;
            Ok(curvature)
        }
    }

    pub fn set_overlay_texture_color_space(
        self,
        handle: crate::VROverlayHandle_t,
        color_space: crate::ColorSpace,
    ) -> Result {
        unsafe {
            mk_err(self.table.SetOverlayTextureColorSpace.unwrap()(
                handle,
                color_space.as_raw(),
            ))
        }
    }

    pub fn get_overlay_texture_color_space(
        self,
        handle: crate::VROverlayHandle_t,
    ) -> Result<crate::ColorSpace> {
        unsafe {
            let mut color_space = 0;
            let err = self.table.GetOverlayTextureColorSpace.unwrap()(handle, &mut color_space);
            mk_err(err)?;
            Ok(crate::ColorSpace::from_raw(color_space))
        }
    }

    pub fn set_overlay_texture_bounds(
        self,
        handle: crate::VROverlayHandle_t,
        bounds: &crate::VRTextureBounds_t,
    ) -> Result {
        unsafe {
            mk_err(self.table.SetOverlayTextureBounds.unwrap()(
                handle,
                as_mut_ptr(bounds),
            ))
        }
    }

    pub fn get_overlay_texture_bounds(
        self,
        handle: crate::VROverlayHandle_t,
    ) -> Result<crate::VRTextureBounds_t> {
        unsafe {
            let mut bounds = zeroed();
            let err = self.table.GetOverlayTextureBounds.unwrap()(handle, &mut bounds);
            mk_err(err)?;
            Ok(bounds)
        }
    }

    pub fn get_overlay_transform_type(
        self,
        handle: crate::VROverlayHandle_t,
    ) -> Result<crate::OverlayTransformType> {
        unsafe {
            let mut result = 0;
            let err = self.table.GetOverlayTransformType.unwrap()(handle, &mut result);
            mk_err(err)?;
            Ok(crate::OverlayTransformType::from_raw(result))
        }
    }

    pub fn set_overlay_transform_absolute(
        self,
        handle: crate::VROverlayHandle_t,
        origin: crate::TrackingUniverseOrigin,
        transform: &crate::HmdMatrix34_t,
    ) -> Result {
        unsafe {
            let err = self.table.SetOverlayTransformAbsolute.unwrap()(
                handle,
                origin.as_raw(),
                as_mut_ptr(transform),
            );
            mk_err(err)
        }
    }

    pub fn get_overlay_transform_absolute(
        self,
        handle: crate::VROverlayHandle_t,
    ) -> Result<(crate::TrackingUniverseOrigin, crate::HmdMatrix34_t)> {
        unsafe {
            let mut result: (_, crate::HmdMatrix34_t) = zeroed();
            let err = self.table.GetOverlayTransformAbsolute.unwrap()(
                handle,
                &mut result.0,
                &mut result.1,
            );
            mk_err(err)?;
            Ok((crate::TrackingUniverseOrigin::from_raw(result.0), result.1))
        }
    }

    pub fn set_overlay_transform_tracked_device_relative(
        self,
        handle: crate::VROverlayHandle_t,
        device: crate::TrackedDeviceIndex_t,
        transform: &crate::HmdMatrix34_t,
    ) -> Result {
        unsafe {
            let err = self.table.SetOverlayTransformTrackedDeviceRelative.unwrap()(
                handle,
                device,
                as_mut_ptr(transform),
            );
            mk_err(err)
        }
    }

    pub fn get_overlay_transform_tracked_device_relative(
        self,
        handle: crate::VROverlayHandle_t,
    ) -> Result<(crate::TrackedDeviceIndex_t, crate::HmdMatrix34_t)> {
        unsafe {
            let mut result: (_, _) = zeroed();
            let err = self.table.GetOverlayTransformTrackedDeviceRelative.unwrap()(
                handle,
                &mut result.0,
                &mut result.1,
            );
            mk_err(err)?;
            Ok(result)
        }
    }

    pub fn set_overlay_transform_tracked_device_component(
        self,
        handle: crate::VROverlayHandle_t,
        device: crate::TrackedDeviceIndex_t,
        name: &CStr,
    ) -> Result {
        unsafe {
            let err = self
                .table
                .SetOverlayTransformTrackedDeviceComponent
                .unwrap()(handle, device, name.as_ptr() as *mut _);
            mk_err(err)
        }
    }

    // We can't know how long
    //pub fn get_overlay_transform_tracked_device_component(self, handle: crate::VROverlayHandle_t) -> Result<(crate::TrackedDeviceIndex_t, crate::HmdMatrix34_t), crate::OverlayError> {
    //    unsafe {
    //        let mut result: (_, _) = zeroed();
    //        let err = self.table.GetOverlayTransformTrackedDeviceComponent.unwrap()(handle, &mut result.0, &mut result.1);
    //        mk_err(err)?;
    //        Ok(result)
    //    }
    //}

    pub fn set_overlay_transform_cursor(
        self,
        handle: crate::VROverlayHandle_t,
        hotspot: &crate::HmdVector2_t,
    ) -> Result {
        unsafe {
            mk_err(self.table.SetOverlayTransformCursor.unwrap()(
                handle,
                as_mut_ptr(hotspot),
            ))
        }
    }

    pub fn get_overlay_transform_cursor(
        self,
        handle: crate::VROverlayHandle_t,
    ) -> Result<crate::HmdVector2_t> {
        unsafe {
            let mut result = zeroed();
            mk_err(self.table.GetOverlayTransformCursor.unwrap()(
                handle,
                &mut result,
            ))?;
            Ok(result)
        }
    }

    pub fn set_overlay_transform_projection(
        self,
        handle: crate::VROverlayHandle_t,
        tracking_origin: crate::TrackingUniverseOrigin,
        tracking_origin_to_overlay_transform: &crate::HmdMatrix34_t,
        projection: &crate::VROverlayProjection_t,
        eye: crate::Eye,
    ) -> Result {
        unsafe {
            mk_err(self.table.SetOverlayTransformProjection.unwrap()(
                handle,
                tracking_origin.as_raw(),
                as_mut_ptr(tracking_origin_to_overlay_transform),
                as_mut_ptr(projection),
                eye.as_raw(),
            ))
        }
    }

    pub fn show_overlay(self, handle: crate::VROverlayHandle_t) -> Result {
        unsafe { mk_err(self.table.ShowOverlay.unwrap()(handle)) }
    }

    pub fn hide_overlay(self, handle: crate::VROverlayHandle_t) -> Result {
        unsafe { mk_err(self.table.HideOverlay.unwrap()(handle)) }
    }

    pub fn is_overlay_visible(self, handle: crate::VROverlayHandle_t) -> bool {
        unsafe { self.table.IsOverlayVisible.unwrap()(handle) }
    }

    pub fn get_transform_for_overlay_coordinates(
        self,
        handle: crate::VROverlayHandle_t,
        origin: crate::TrackingUniverseOrigin,
        coordinates_in_overlay: crate::HmdVector2_t,
    ) -> Result<crate::HmdMatrix34_t> {
        unsafe {
            let mut result = zeroed();
            let err = self.table.GetTransformForOverlayCoordinates.unwrap()(
                handle,
                origin.as_raw(),
                coordinates_in_overlay,
                &mut result,
            );
            mk_err(err)?;
            Ok(result)
        }
    }

    pub fn wait_frame_sync(self, timeout: Duration) -> Result {
        unsafe {
            mk_err(self.table.WaitFrameSync.unwrap()(
                timeout.as_millis().try_into().unwrap_or(u32::MAX),
            ))
        }
    }

    pub fn poll_next_overlay_event(
        self,
        handle: crate::VROverlayHandle_t,
    ) -> Option<crate::VREvent_t> {
        unsafe {
            let mut result = zeroed();
            let found = self.table.PollNextOverlayEvent.unwrap()(
                handle,
                &mut result,
                size_of_val(&result) as u32,
            );
            some_if!(result; if found)
        }
    }

    pub fn get_overlay_input_method(
        self,
        handle: crate::VROverlayHandle_t,
    ) -> Result<crate::OverlayInputMethod> {
        unsafe {
            let mut result = zeroed();
            let err = self.table.GetOverlayInputMethod.unwrap()(handle, &mut result);
            mk_err(err)?;
            Ok(crate::OverlayInputMethod::from_raw(result))
        }
    }

    pub fn set_overlay_input_method(
        self,
        handle: crate::VROverlayHandle_t,
        method: crate::OverlayInputMethod,
    ) -> Result {
        unsafe {
            mk_err(self.table.SetOverlayInputMethod.unwrap()(
                handle,
                method.as_raw(),
            ))
        }
    }

    pub fn get_overlay_mouse_scale(
        self,
        handle: crate::VROverlayHandle_t,
    ) -> Result<crate::HmdVector2_t> {
        unsafe {
            let mut result = zeroed();
            let err = self.table.GetOverlayMouseScale.unwrap()(handle, &mut result);
            mk_err(err)?;
            Ok(result)
        }
    }

    pub fn set_overlay_mouse_scale(
        self,
        handle: crate::VROverlayHandle_t,
        scale: &crate::HmdVector2_t,
    ) -> Result {
        unsafe {
            mk_err(self.table.GetOverlayMouseScale.unwrap()(
                handle,
                as_mut_ptr(scale),
            ))
        }
    }

    pub fn compute_overlay_intersection(
        self,
        handle: crate::VROverlayHandle_t,
        params: &crate::VROverlayIntersectionParams_t,
    ) -> Option<crate::VROverlayIntersectionResults_t> {
        unsafe {
            let mut result = zeroed();
            let success = self.table.ComputeOverlayIntersection.unwrap()(
                handle,
                as_mut_ptr(params),
                &mut result,
            );
            some_if!(result; if success)
        }
    }

    pub fn is_hover_target_overlay(self, handle: crate::VROverlayHandle_t) -> bool {
        unsafe { self.table.IsHoverTargetOverlay.unwrap()(handle) }
    }

    pub fn set_overlay_intersection_mask(
        self,
        handle: crate::VROverlayHandle_t,
        mask_primitives: &[crate::VROverlayIntersectionMaskPrimitive_t],
    ) -> Result {
        unsafe {
            mk_err(self.table.SetOverlayIntersectionMask.unwrap()(
                handle,
                mask_primitives.as_ptr() as _,
                mask_primitives.len() as u32,
                size_of::<crate::VROverlayIntersectionMaskPrimitive_t>() as u32,
            ))
        }
    }

    pub fn trigger_laser_mouse_haptic_vibration(
        self,
        handle: crate::VROverlayHandle_t,
        duration: Duration,
        frequency: f32,
        amplitude: f32,
    ) -> Result {
        unsafe {
            mk_err(self.table.TriggerLaserMouseHapticVibration.unwrap()(
                handle,
                duration.as_secs_f32(),
                frequency,
                amplitude,
            ))
        }
    }

    pub fn set_overlay_cursor(
        self,
        handle: crate::VROverlayHandle_t,
        cursor_handle: crate::VROverlayHandle_t,
    ) -> Result {
        unsafe { mk_err(self.table.SetOverlayCursor.unwrap()(handle, cursor_handle)) }
    }

    pub fn set_overlay_cursor_position_override(
        self,
        handle: crate::VROverlayHandle_t,
        cursor_position: &crate::HmdVector2_t,
    ) -> Result {
        unsafe {
            mk_err(self.table.SetOverlayCursorPositionOverride.unwrap()(
                handle,
                as_mut_ptr(cursor_position),
            ))
        }
    }

    pub fn clear_overlay_cursor_position_override(
        self,
        handle: crate::VROverlayHandle_t,
    ) -> Result {
        unsafe {
            mk_err(self.table.ClearOverlayCursorPositionOverride.unwrap()(
                handle,
            ))
        }
    }

    // consider add new type instead of Texture_t?
    pub fn set_overlay_texture(
        self,
        handle: crate::VROverlayHandle_t,
        texture: impl Into<openvr_sys::Texture_t>,
    ) -> Result {
        unsafe {
            let texture = texture.into();
            mk_err(self.table.SetOverlayTexture.unwrap()(
                handle,
                as_mut_ptr(&texture),
            ))
        }
    }

    pub fn clear_overlay_texture(self, handle: crate::VROverlayHandle_t) -> Result {
        unsafe { mk_err(self.table.ClearOverlayTexture.unwrap()(handle)) }
    }

    pub fn set_overlay_raw(
        self,
        handle: crate::VROverlayHandle_t,
        buffer: &[u8],
        width: u32,
        height: u32,
        depth_in_bytes: u32,
    ) -> Result {
        let buffer_len = (width as usize)
            .checked_mul(height as _)
            .and_then(|x| x.checked_mul(depth_in_bytes as _));
        assert_eq!(buffer_len, Some(buffer.len()), "buffer size mismatch");
        unsafe {
            let err = self.table.SetOverlayRaw.unwrap()(
                handle,
                buffer.as_ptr() as *mut _,
                width,
                height,
                depth_in_bytes,
            );
            mk_err(err)
        }
    }

    pub fn set_overlay_from_file(
        self,
        handle: crate::VROverlayHandle_t,
        file_path: &CStr,
    ) -> Result {
        unsafe {
            mk_err(self.table.SetOverlayFromFile.unwrap()(
                handle,
                file_path.as_ptr() as *mut _,
            ))
        }
    }

    // get_overlay_texture and release_native_overlay_handle

    pub fn get_overlay_texture_size(self, handle: crate::VROverlayHandle_t) -> Result<(u32, u32)> {
        unsafe {
            let mut result: (_, _) = zeroed();
            let err =
                self.table.GetOverlayTextureSize.unwrap()(handle, &mut result.0, &mut result.1);
            mk_err(err)?;
            Ok(result)
        }
    }

    pub fn create_dashboard_overlay(
        self,
        overlay_key: &CStr,
        overlay_friendly_name: &CStr,
    ) -> Result<(crate::VROverlayHandle_t, crate::VROverlayHandle_t)> {
        unsafe {
            let mut result: (_, _) = zeroed();
            let err = self.table.CreateDashboardOverlay.unwrap()(
                overlay_key.as_ptr() as *mut _,
                overlay_friendly_name.as_ptr() as *mut _,
                &mut result.0,
                &mut result.1,
            );
            mk_err(err)?;
            Ok(result)
        }
    }

    pub fn is_dashboard_visible(self) -> bool {
        unsafe { self.table.IsDashboardVisible.unwrap()() }
    }

    pub fn is_active_dashboard_overlay(self, handle: crate::VROverlayHandle_t) -> bool {
        unsafe { self.table.IsActiveDashboardOverlay.unwrap()(handle) }
    }

    pub fn set_dashboard_overlay_scene_process(
        self,
        handle: crate::VROverlayHandle_t,
        process_id: u32,
    ) -> Result {
        unsafe {
            mk_err(self.table.SetDashboardOverlaySceneProcess.unwrap()(
                handle, process_id,
            ))
        }
    }

    pub fn get_dashboard_overlay_scene_process(
        self,
        handle: crate::VROverlayHandle_t,
    ) -> Result<u32> {
        unsafe {
            let mut process_id: _ = 0;
            let err = self.table.GetDashboardOverlaySceneProcess.unwrap()(handle, &mut process_id);
            mk_err(err)?;
            Ok(process_id)
        }
    }

    pub fn show_dashboard(self, overlay_to_show: &CStr) {
        unsafe { self.table.ShowDashboard.unwrap()(overlay_to_show.as_ptr() as *mut _) }
    }

    pub fn get_primary_dashboard_device(self) -> crate::TrackedDeviceIndex_t {
        unsafe { self.table.GetPrimaryDashboardDevice.unwrap()() }
    }

    pub fn show_keyboard(
        self,
        input_mode: crate::GamepadTextInputMode,
        line_input_mode: crate::GamepadTextInputLineMode,
        flags: crate::KeyboardFlags,
        description: &CStr,
        char_max: u32,
        existing_text: &CStr,
        user_value: u64,
    ) -> Result {
        unsafe {
            #[allow(clippy::unnecessary_cast)] // KeyboardFlags can be signed
            mk_err(self.table.ShowKeyboard.unwrap()(
                input_mode.as_raw(),
                line_input_mode.as_raw(),
                flags.as_raw() as u32,
                description.as_ptr() as *mut _,
                char_max,
                existing_text.as_ptr() as *mut _,
                user_value,
            ))
        }
    }

    pub fn show_keyboard_for_overlay(
        self,
        handle: crate::VROverlayHandle_t,
        input_mode: crate::GamepadTextInputMode,
        line_input_mode: crate::GamepadTextInputLineMode,
        flags: crate::KeyboardFlags,
        description: &CStr,
        char_max: u32,
        existing_text: &CStr,
        user_value: u64,
    ) -> Result {
        unsafe {
            #[allow(clippy::unnecessary_cast)] // KeyboardFlags can be signed
            mk_err(self.table.ShowKeyboardForOverlay.unwrap()(
                handle,
                input_mode.as_raw(),
                line_input_mode.as_raw(),
                flags.as_raw() as u32,
                description.as_ptr() as *mut _,
                char_max,
                existing_text.as_ptr() as *mut _,
                user_value,
            ))
        }
    }

    pub fn get_keyboard_text(self) -> Result<CString> {
        unsafe {
            let mut str = Vec::<u8>::new();
            loop {
                let len = self.table.GetKeyboardText.unwrap()(
                    str.as_mut_ptr() as _,
                    str.capacity() as u32,
                );
                if str.capacity() < len as usize {
                    str.reserve(len as usize);
                    continue;
                }
                str.set_len(len as usize);
                return Ok(CString::from_vec_with_nul_unchecked(str));
            }
        }
    }

    pub fn hide_keyboard(self) {
        unsafe { self.table.HideKeyboard.unwrap()() }
    }

    pub fn set_keyboard_transform_absolute(
        self,
        tracking_origin: crate::TrackingUniverseOrigin,
        transform: &crate::HmdMatrix34_t,
    ) {
        unsafe {
            self.table.SetKeyboardTransformAbsolute.unwrap()(
                tracking_origin.as_raw(),
                as_mut_ptr(transform),
            )
        }
    }

    pub fn set_keyboard_position_for_overlay(
        self,
        handle: crate::VROverlayHandle_t,
        avoid_rect: crate::HmdRect2_t,
    ) {
        unsafe { self.table.SetKeyboardPositionForOverlay.unwrap()(handle, avoid_rect) }
    }

    pub fn show_message_overlay(
        self,
        text: &CStr,
        caption: &CStr,
        button0_text: &CStr,
        button1_text: Option<&CStr>,
        button2_text: Option<&CStr>,
        button3_text: Option<&CStr>,
    ) -> Result<crate::MessageOverlayResponse> {
        unsafe {
            let response = self.table.ShowMessageOverlay.unwrap()(
                text.as_ptr() as _,
                caption.as_ptr() as _,
                button0_text.as_ptr() as _,
                button1_text.map_or(null(), CStr::as_ptr) as _,
                button2_text.map_or(null(), CStr::as_ptr) as _,
                button3_text.map_or(null(), CStr::as_ptr) as _,
            );
            Ok(crate::MessageOverlayResponse::from_raw(response))
        }
    }

    pub fn close_message_overlay(self) {
        unsafe { self.table.CloseMessageOverlay.unwrap()() }
    }
}

pub struct OverlayImageData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

pub struct OwnedInVROverlay<'a> {
    overlay: VROverlay<'a>,
    handle: crate::VROverlayHandle_t,
}

macro_rules! overlay_wrapper {
    (
        $name: ident $(<$($generics:tt)+>)? (
            $($param: ident: $param_ty: ty),* $(,)?
        ) $(-> $result: ty)?
    ) => {
        pub fn $name $(<$($generics)+>)? (&self $(, $param: $param_ty)*) $(-> $result)? {
            self.overlay.$name(self.handle$(, $param)*)
        }
    };
}

impl<'a> OwnedInVROverlay<'a> {
    pub fn new(overlay: VROverlay<'a>, overlay_key: &CStr, overlay_name: &CStr) -> Result<Self> {
        Ok(Self {
            overlay,
            handle: overlay.create_overlay(overlay_key, overlay_name)?,
        })
    }

    //overlay_wrapper!(set_high_quality_overlay() -> Result);

    overlay_wrapper!(get_overlay_key() -> Result<CString>);
    overlay_wrapper!(get_overlay_name() -> Result<CString>);
    overlay_wrapper!(set_overlay_name(name: &CStr) -> Result);
    overlay_wrapper!(get_overlay_image_data() -> Result<OverlayImageData>);
    //overlay_wrapper!(set_overlay_rendering_pid(pid: u32) -> Result); // throw away ownership
    //overlay_wrapper!(get_overlay_rendering_pid() -> u32); always this process
    overlay_wrapper!(set_overlay_flag(flag: crate::OverlayFlags, enabled: bool) -> Result);
    overlay_wrapper!(get_overlay_flag(flag: crate::OverlayFlags) -> Result<bool>);
    overlay_wrapper!(get_overlay_flags() -> Result<crate::OverlayFlags>);
    overlay_wrapper!(set_overlay_color(red: f32, green: f32, blue: f32) -> Result);
    overlay_wrapper!(get_overlay_color() -> Result<(f32, f32, f32)>);
    overlay_wrapper!(set_overlay_alpha(alpha: f32) -> Result);
    overlay_wrapper!(get_overlay_alpha() -> Result<f32>);
    overlay_wrapper!(set_overlay_texel_aspect(aspect: f32) -> Result);
    overlay_wrapper!(get_overlay_texel_aspect() -> Result<f32>);
    overlay_wrapper!(set_overlay_sort_order(aspect: u32) -> Result);
    overlay_wrapper!(get_overlay_sort_order() -> Result<u32>);
    overlay_wrapper!(set_overlay_width_in_meters(aspect: f32) -> Result);
    overlay_wrapper!(get_overlay_width_in_meters() -> Result<f32>);
    overlay_wrapper!(set_overlay_curvature(curvature: f32) -> Result);
    overlay_wrapper!(get_overlay_curvature() -> Result<f32>);
    overlay_wrapper!(set_overlay_pre_curve_pitch(radians: f32) -> Result);
    overlay_wrapper!(get_overlay_pre_curve_pitch() -> Result<f32>);

    overlay_wrapper!(set_overlay_texture_color_space(aspect: crate::ColorSpace) -> Result);
    overlay_wrapper!(get_overlay_texture_color_space() -> Result<crate::ColorSpace>);
    overlay_wrapper!(set_overlay_texture_bounds(bounds: &crate::VRTextureBounds_t) -> Result);
    overlay_wrapper!(get_overlay_texture_bounds() -> Result<crate::VRTextureBounds_t>);
    overlay_wrapper!(get_overlay_transform_type() -> Result<crate::OverlayTransformType>);
    overlay_wrapper!(set_overlay_transform_absolute(origin: crate::TrackingUniverseOrigin, transform: &crate::HmdMatrix34_t) -> Result);
    overlay_wrapper!(get_overlay_transform_absolute() -> Result<(crate::TrackingUniverseOrigin, crate::HmdMatrix34_t)>);
    overlay_wrapper!(set_overlay_transform_tracked_device_relative(device: crate::TrackedDeviceIndex_t, transform: &crate::HmdMatrix34_t) -> Result);
    overlay_wrapper!(get_overlay_transform_tracked_device_relative() -> Result<(crate::TrackedDeviceIndex_t, crate::HmdMatrix34_t)>);

    overlay_wrapper!(set_overlay_transform_tracked_device_component(device: crate::TrackedDeviceIndex_t, name: &CStr) -> Result);

    overlay_wrapper!(set_overlay_transform_cursor(hotspot: &crate::HmdVector2_t) -> Result);
    overlay_wrapper!(get_overlay_transform_cursor() -> Result<crate::HmdVector2_t>);

    overlay_wrapper!(set_overlay_transform_projection(tracking_origin: crate::TrackingUniverseOrigin,tracking_origin_to_overlay_transform: &crate::HmdMatrix34_t,projection: &crate::VROverlayProjection_t,eye: crate::Eye) -> Result);

    overlay_wrapper!(show_overlay() -> Result);
    overlay_wrapper!(hide_overlay() -> Result);
    overlay_wrapper!(is_overlay_visible() -> bool);

    overlay_wrapper!(get_transform_for_overlay_coordinates(origin: crate::TrackingUniverseOrigin, coordinates_in_overlay: crate::HmdVector2_t) -> Result<crate::HmdMatrix34_t>);

    overlay_wrapper!(poll_next_overlay_event() -> Option<crate::VREvent_t>); // TODO: replace VREvent_t

    overlay_wrapper!(get_overlay_input_method() -> Result<crate::OverlayInputMethod>);
    overlay_wrapper!(set_overlay_input_method(method: crate::OverlayInputMethod) -> Result);
    overlay_wrapper!(get_overlay_mouse_scale() -> Result<crate::HmdVector2_t>);
    overlay_wrapper!(set_overlay_mouse_scale(scale: &crate::HmdVector2_t) -> Result);
    overlay_wrapper!(compute_overlay_intersection(params: &crate::VROverlayIntersectionParams_t) -> Option<crate::VROverlayIntersectionResults_t>);
    overlay_wrapper!(is_hover_target_overlay() -> bool);
    overlay_wrapper!(set_overlay_intersection_mask(mask_primitives: &[crate::VROverlayIntersectionMaskPrimitive_t]) -> Result);

    // cursor related
    overlay_wrapper!(trigger_laser_mouse_haptic_vibration(duration: Duration, frequency: f32, amplitude: f32) -> Result);
    overlay_wrapper!(set_overlay_cursor(cursor_handle: crate::VROverlayHandle_t) -> Result);
    overlay_wrapper!(set_overlay_cursor_position_override(cursor_position: &crate::HmdVector2_t) -> Result);
    overlay_wrapper!(clear_overlay_cursor_position_override() -> Result);

    // overlay textures
    overlay_wrapper!(set_overlay_texture(texture: impl Into<openvr_sys::Texture_t>) -> Result);
    overlay_wrapper!(clear_overlay_texture() -> Result);
    overlay_wrapper!(set_overlay_raw(buffer: &[u8], width: u32, height: u32, depth_in_bytes: u32) -> Result);
    overlay_wrapper!(set_overlay_from_file(file_path: &CStr) -> Result);

    overlay_wrapper!(get_overlay_texture_size() -> Result<(u32, u32)>);

    // this is for in-vr overlay so dashboard relative functions are not exists

    pub fn destroy(self) -> Result {
        self.overlay.destroy_overlay(self.handle)?;
        forget(self); // already destroyed
        Ok(())
    }
}

impl<'a> Drop for OwnedInVROverlay<'a> {
    fn drop(&mut self) {
        // ignores result
        self.overlay.destroy_overlay(self.handle).ok();
    }
}
