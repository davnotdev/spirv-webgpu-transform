#![allow(clippy::missing_safety_doc)]

use core::{ffi, ptr, slice};
use spirv_webgpu_transform::{CorrectionMap, combimgsampsplitter, drefsplitter};

type TransformCorrectionMap = *mut ffi::c_void;

unsafe fn cast_correction_map(map: TransformCorrectionMap) -> *mut Option<CorrectionMap> {
    map.cast::<Option<CorrectionMap>>()
}

unsafe fn alloc_or_pass_correction_map(
    map: *mut TransformCorrectionMap,
) -> &'static mut Option<CorrectionMap> {
    unsafe {
        if map.is_null() {
            panic!(
                "Got null correction map pointer, pointer to existing correction map or SPIRV_WEBGPU_TRANFORM_CORRECTION_MAP_NULL"
            )
        }

        if (*map).is_null() {
            let owned = Box::new(None);
            let r = Box::leak(owned);
            let ptr = r as *mut Option<CorrectionMap> as TransformCorrectionMap;
            *map = ptr;
            r
        } else {
            Box::leak(Box::from_raw(cast_correction_map(*map)))
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn spirv_webgpu_transform_combimgsampsplitter_alloc(
    in_spv: *const u32,
    in_count: u32,
    out_spv: *mut *const u32,
    out_count: *mut u32,
    correction_map: *mut TransformCorrectionMap,
) {
    let correction_map = unsafe { alloc_or_pass_correction_map(correction_map) };

    let in_spv = unsafe { slice::from_raw_parts(in_spv, in_count as usize) };
    match combimgsampsplitter(in_spv, correction_map) {
        Ok(spv) => unsafe {
            *out_count = spv.len() as u32;
            let leaked = Box::leak(spv.into_boxed_slice());
            *out_spv = leaked.as_ptr();
        },
        Err(_) => unsafe {
            *out_spv = ptr::null();
            *out_count = 0;
        },
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn spirv_webgpu_transform_combimgsampsplitter_free(out_spv: *mut u32) {
    unsafe { drop(Box::from_raw(out_spv)) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn spirv_webgpu_transform_drefsplitter_alloc(
    in_spv: *const u32,
    in_count: u32,
    out_spv: *mut *const u32,
    out_count: *mut u32,
    correction_map: *mut TransformCorrectionMap,
) {
    let correction_map = unsafe { alloc_or_pass_correction_map(correction_map) };

    let in_spv = unsafe { slice::from_raw_parts(in_spv, in_count as usize) };
    match drefsplitter(in_spv, correction_map) {
        Ok(spv) => unsafe {
            *out_count = spv.len() as u32;
            let leaked = Box::leak(spv.into_boxed_slice());
            *out_spv = leaked.as_ptr();
        },
        Err(_) => unsafe {
            *out_spv = ptr::null();
            *out_count = 0;
        },
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn spirv_webgpu_transform_drefsplitter_free(out_spv: *mut u32) {
    unsafe { drop(Box::from_raw(out_spv)) }
}

#[repr(C)]
pub enum TransformCorrectionStatus {
    SpirvWebgpuTransformCorrectionStatusNone = 0,
    SpirvWebgpuTransformCorrectionStatusSome = 1,
}

#[repr(u16)]
pub enum TransformCorrectionType {
    SpirvWebgpuTransformCorrectionTypeSplitCombined = 0,
    SpirvWebgpuTransformCorrectionTypeSplitDrefRegular = 1,
    SpirvWebgpuTransformCorrectionTypeSplitDrefComparison = 2,
}

// TransformCorrectionStatus spirv_webgpu_transform_correction_map_index(uint32_t set, uint32_t binding, TransformCorrectionType** corrections_ptr, uint32_t* correction_count);
#[unsafe(no_mangle)]
pub unsafe extern "C" fn spirv_transform_correction_map_index(
    correction_map: TransformCorrectionMap,
    set: u32,
    binding: u32,
    corrections_ptr: *mut *mut u16,
    corrections_count: *mut u32,
) -> TransformCorrectionStatus {
    unsafe {
        *corrections_ptr = ptr::null_mut();
        *corrections_count = 0;

        if correction_map.is_null() {
            TransformCorrectionStatus::SpirvWebgpuTransformCorrectionStatusNone
        } else {
            let correction_map = &mut *cast_correction_map(correction_map);
            if let Some(correction_map) = correction_map
                && let Some(set) = correction_map.sets.get(&set)
                && let Some(binding) = set.bindings.get(&binding)
                && !binding.corrections.is_empty()
            {
                *corrections_ptr =
                    binding.corrections.as_ptr() as *mut TransformCorrectionType as *mut u16;
                *corrections_count = binding.corrections.len() as u32;
                TransformCorrectionStatus::SpirvWebgpuTransformCorrectionStatusSome
            } else {
                TransformCorrectionStatus::SpirvWebgpuTransformCorrectionStatusNone
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn spirv_webgpu_transform_correction_map_free(
    correction_map: TransformCorrectionMap,
) {
    let _ = unsafe { Box::from_raw(cast_correction_map(correction_map)) };
}
