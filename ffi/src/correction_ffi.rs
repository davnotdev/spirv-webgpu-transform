use super::*;

pub type SpvTransformCorrectionMap = *mut ffi::c_void;
pub const C_FALSE: u8 = 0;
pub const C_TRUE: u8 = 1;

#[repr(C)]
#[derive(Debug, Default)]
pub struct SpvTransformOptionalU32 {
    pub some: u8,
    pub value: u32,
}

#[repr(C)]
pub enum TransformCorrectionType {
    SpirvWebgpuTransformCorrectionTypeSplitCombined = 0,
    SpirvWebgpuTransformCorrectionTypeSplitDrefRegular = 1,
    SpirvWebgpuTransformCorrectionTypeSplitDrefComparison = 2,
    SpirvWebgpuTransformCorrectionTypeConvertStorageCube = 3,
    SpirvWebgpuTransformCorrectionTypeSplitBindingArray = 4,
}

pub unsafe fn cast_correction_map(map: SpvTransformCorrectionMap) -> &'static mut CorrectionMap {
    unsafe { &mut *(map as *mut CorrectionMap) }
}

pub unsafe fn cast_correction_map_or_default_alloc(
    map: *mut SpvTransformCorrectionMap,
) -> &'static mut CorrectionMap {
    unsafe {
        let map = &mut *map;
        if map.is_null() {
            let owned = Box::new(CorrectionMap::default());
            let r = Box::leak(owned);
            *map = r as *mut CorrectionMap as SpvTransformCorrectionMap;
            r
        } else {
            let ptr = *map;
            cast_correction_map(ptr)
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn spirv_webgpu_transform_correction_map_free(
    correction_map: SpvTransformCorrectionMap,
) {
    if !correction_map.is_null() {
        let _ = unsafe { Box::from_raw(correction_map as *mut CorrectionMap) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn spirv_webgpu_transform_correction_sets_index(
    correction_map: SpvTransformCorrectionMap,
    set: u32,
    binding: u32,
    corrections_ptr: *mut *mut u16,
    corrections_count: *mut u32,
) -> u8 {
    unsafe {
        *corrections_ptr = ptr::null_mut();
        *corrections_count = 0;

        if correction_map.is_null() {
            C_FALSE
        } else {
            let correction_map = cast_correction_map(correction_map);
            if let Some(sets) = correction_map.sets.as_mut()
                && let Some(set) = sets.get(&set)
                && let Some(binding) = set.bindings.get(&binding)
                && !binding.corrections.is_empty()
            {
                *corrections_ptr =
                    binding.corrections.as_ptr() as *mut TransformCorrectionType as *mut u16;
                *corrections_count = binding.corrections.len() as u32;

                C_TRUE
            } else {
                C_FALSE
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn spirv_webgpu_transform_correction_immediates_set(
    correction_map: SpvTransformCorrectionMap,
) -> SpvTransformOptionalU32 {
    if !correction_map.is_null() {
        let correction_map = unsafe { cast_correction_map(correction_map) };
        if let Some(value) = correction_map.immediates_set {
            return SpvTransformOptionalU32 { some: 1, value };
        }
    }
    SpvTransformOptionalU32 {
        some: C_FALSE,
        ..Default::default()
    }
}
