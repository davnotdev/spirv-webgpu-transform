#![allow(clippy::missing_safety_doc)]

use core::{ffi, ptr, slice};
use spirv_webgpu_transform::{
    CorrectionMap, combimgsampsplitter, drefsplitter, immediatespatch, isnanisinfpatch,
    mirrorpatch, pruneunuseddref, splitbindingarray, storagecubepatch,
};

mod correction_ffi;

pub use correction_ffi::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn spirv_webgpu_transform_combimgsampsplitter_alloc(
    in_spv: *const u32,
    in_count: u32,
    out_spv: *mut *const u32,
    out_count: *mut u32,
    correction_map: *mut SpvTransformCorrectionMap,
) {
    let map = correction_map;
    let correction_map = unsafe { cast_correction_map_or_default_alloc(map) };

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
    correction_map: *mut SpvTransformCorrectionMap,
) {
    let map = correction_map;
    let correction_map = unsafe { cast_correction_map_or_default_alloc(map) };

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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn spirv_webgpu_transform_immediatespatch_alloc(
    in_spv: *const u32,
    in_count: u32,
    out_spv: *mut *const u32,
    out_count: *mut u32,
    correction_map: *mut SpvTransformCorrectionMap,
) {
    let map = correction_map;
    let correction_map = unsafe { cast_correction_map_or_default_alloc(map) };

    let in_spv = unsafe { slice::from_raw_parts(in_spv, in_count as usize) };
    match immediatespatch(in_spv, correction_map) {
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
pub unsafe extern "C" fn spirv_webgpu_transform_immediatespatch_free(out_spv: *mut u32) {
    unsafe { drop(Box::from_raw(out_spv)) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn spirv_webgpu_transform_isnanisinfpatch_alloc(
    in_spv: *const u32,
    in_count: u32,
    out_spv: *mut *const u32,
    out_count: *mut u32,
) {
    let in_spv = unsafe { slice::from_raw_parts(in_spv, in_count as usize) };
    match isnanisinfpatch(in_spv) {
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
pub unsafe extern "C" fn spirv_webgpu_transform_isnanisinfpatch_free(out_spv: *mut u32) {
    unsafe { drop(Box::from_raw(out_spv)) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn spirv_webgpu_transform_storagecubepatch_alloc(
    in_spv: *const u32,
    in_count: u32,
    out_spv: *mut *const u32,
    out_count: *mut u32,
    correction_map: *mut SpvTransformCorrectionMap,
) {
    let map = correction_map;
    let correction_map = unsafe { cast_correction_map_or_default_alloc(map) };

    let in_spv = unsafe { slice::from_raw_parts(in_spv, in_count as usize) };
    match storagecubepatch(in_spv, correction_map) {
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
pub unsafe extern "C" fn spirv_webgpu_transform_storagecubepatch_free(out_spv: *mut u32) {
    unsafe { drop(Box::from_raw(out_spv)) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn spirv_webgpu_transform_pruneunuseddref_alloc(
    in_spv: *const u32,
    in_count: u32,
    out_spv: *mut *const u32,
    out_count: *mut u32,
) {
    let in_spv = unsafe { slice::from_raw_parts(in_spv, in_count as usize) };
    match pruneunuseddref(in_spv) {
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
pub unsafe extern "C" fn spirv_webgpu_transform_pruneunuseddref_free(out_spv: *mut u32) {
    unsafe { drop(Box::from_raw(out_spv)) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn spirv_webgpu_transform_splitbindingarray_alloc(
    in_spv: *const u32,
    in_count: u32,
    out_spv: *mut *const u32,
    out_count: *mut u32,
    correction_map: *mut SpvTransformCorrectionMap,
) {
    let map = correction_map;
    let correction_map = unsafe { cast_correction_map_or_default_alloc(map) };

    let in_spv = unsafe { slice::from_raw_parts(in_spv, in_count as usize) };
    match splitbindingarray(in_spv, correction_map) {
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
pub unsafe extern "C" fn spirv_webgpu_transform_splitbindingarray_free(out_spv: *mut u32) {
    unsafe { drop(Box::from_raw(out_spv)) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn spirv_webgpu_transform_mirrorpatch_alloc(
    in_left_spv: *const u32,
    in_left_count: u32,
    left_corrections: *mut SpvTransformCorrectionMap,
    in_right_spv: *const u32,
    in_right_count: u32,
    right_corrections: *mut SpvTransformCorrectionMap,
    out_left_spv: *mut *const u32,
    out_left_count: *mut u32,
    out_right_spv: *mut *const u32,
    out_right_count: *mut u32,
) {
    let left_correction_map = unsafe { cast_correction_map_or_default_alloc(left_corrections) };
    let right_correction_map = unsafe { cast_correction_map_or_default_alloc(right_corrections) };

    let in_left_spv = unsafe { slice::from_raw_parts(in_left_spv, in_left_count as usize) };
    let in_right_spv = unsafe { slice::from_raw_parts(in_right_spv, in_right_count as usize) };

    match mirrorpatch(
        in_left_spv,
        left_correction_map,
        in_right_spv,
        right_correction_map,
    ) {
        Ok((left_spv, right_spv)) => unsafe {
            // We will return an copied output if output is null just so that no one blows their
            // foot off (no null outputs).
            let left_spv = left_spv.unwrap_or_else(|| in_left_spv.to_vec());
            let right_spv = right_spv.unwrap_or_else(|| in_right_spv.to_vec());

            *out_left_count = left_spv.len() as u32;
            let leaked = Box::leak(left_spv.into_boxed_slice());
            *out_left_spv = leaked.as_ptr();

            *out_right_count = right_spv.len() as u32;
            let leaked = Box::leak(right_spv.into_boxed_slice());
            *out_right_spv = leaked.as_ptr();
        },
        Err(_) => unsafe {
            *out_left_spv = ptr::null();
            *out_left_count = 0;
        },
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn spirv_webgpu_transform_mirrorpatch_free(
    out_left_spv: *mut u32,
    out_right_spv: *mut u32,
) {
    unsafe {
        drop(Box::from_raw(out_left_spv));
        drop(Box::from_raw(out_right_spv));
    }
}
