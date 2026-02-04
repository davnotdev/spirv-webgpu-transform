use super::*;

fn inc(ib: &mut u32) -> u32 {
    *ib += 1;
    *ib - 1
}

mod cube_direction_to_axis;

use cube_direction_to_axis::*;

/// Perform the operation on a `Vec<u32>`.
/// Use [u8_slice_to_u32_vec] to convert a `&[u8]` into a `Vec<u32>`.
/// Does not produce any side effects or corrections.
pub fn storagecubepatch(in_spv: &[u32]) -> Result<Vec<u32>, ()> {
    // - Find OpTypeImage, change Cube -> 2D
    // - Find OpTypePointer referencing this ^
    // - Find OpVariable referencing this ^
    // - Find OpImage{Fetch, Read, Write} using this ^
    // - Replace coordinate
    todo!()
}
