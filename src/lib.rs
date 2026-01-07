//! ##
//!
//! ## Features
//!
//! At the moment, the following transformations are supported:
//!
//! | Feature                   | `spirv-val` | `naga` | `tint` |
//! | ------------------------- | ----------- | ------ | ------ |
//! | Combined Image Samplers   | ✅          | ✅     | ✅     |
//! | Mixed Depth / Comparison  | ✅          | ⚠️\*   | ❌     |
//! | isnan / isinf Patching    | ✅          | ✅     | ✅     |
//!
//! > \* Simple cases are OK.
//! > With some [special patches](https://github.com/davnotdev/wgpu/tree/trunk-naga-patches), `naga` can process these.
//!
//! ## Using the result
//!
//! After running an individual shader through one or multiple transformations, you will want to:
//!
//! 1. Know which set bindings were affected, use the output [`CorrectionMap`] for this purpose.
//! 2. Ensure that your vertex and fragment shaders shader the same binding layout, use [`mirrorpatch`] for this purpose
//!
//! ## For more details
//!
//! See [https://github.com/davnotdev/spirv-webgpu-transform](https://github.com/davnotdev/spirv-webgpu-transform) for more details.
//!

use std::collections::{HashMap, HashSet};

mod correction;
mod isnanisinfpatch;
mod mirrorpatch;
mod splitcombined;
mod splitdref;
mod spv;
mod util;

#[cfg(test)]
mod test;

use spv::*;
use util::*;

pub use correction::*;
pub use isnanisinfpatch::*;
pub use mirrorpatch::*;
pub use splitcombined::*;
pub use splitdref::*;

#[derive(Debug, Clone)]
struct InstructionInsert {
    previous_spv_idx: usize,
    instruction: Vec<u32>,
}

#[derive(Debug, Clone)]
struct WordInsert {
    idx: usize,
    word: u32,
    head_idx: usize,
}

/// Helper to convert a `&[u8]` into a `Vec<u32>`.
pub fn u8_slice_to_u32_vec(vec: &[u8]) -> Vec<u32> {
    assert_eq!(
        vec.len() % 4,
        0,
        "Input slice length must be a multiple of 4."
    );

    vec.chunks_exact(4)
        .map(|chunk| {
            (chunk[0] as u32)
                | ((chunk[1] as u32) << 8)
                | ((chunk[2] as u32) << 16)
                | ((chunk[3] as u32) << 24)
        })
        .collect::<Vec<_>>()
}

/// Helper to convert a `&[u32]` into a `Vec<u8>`.
pub fn u32_slice_to_u8_vec(vec: &[u32]) -> Vec<u8> {
    vec.iter()
        .flat_map(|&num| {
            vec![
                (num & 0xFF) as u8,
                ((num >> 8) & 0xFF) as u8,
                ((num >> 16) & 0xFF) as u8,
                ((num >> 24) & 0xFF) as u8,
            ]
        })
        .collect::<Vec<u8>>()
}
