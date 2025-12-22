use super::*;

fn inc(ib: &mut u32) -> u32 {
    *ib += 1;
    *ib - 1
}

// Someone should make a rust-spirv dsl macro
mod isnan_isinf;
mod shared;

use isnan_isinf::*;
use shared::*;

pub fn isnanisinfpatch(in_spv: &[u32]) -> Result<Vec<u32>, ()> {
    let spv = in_spv.to_owned();

    let mut instruction_bound = spv[SPV_HEADER_INSTRUCTION_BOUND_OFFSET];
    let magic_number = spv[SPV_HEADER_MAGIC_NUM_OFFSET];

    let spv_header = spv[0..SPV_HEADER_LENGTH].to_owned();

    assert_eq!(magic_number, SPV_HEADER_MAGIC);

    // let mut instruction_inserts = vec![];
    // let mut word_inserts = vec![];

    // let spv = spv.into_iter().skip(SPV_HEADER_LENGTH).collect::<Vec<_>>();
    // let mut new_spv = spv.clone();

    // let mut op_function_idxs = vec![];
    // let mut op_is_nan_idxs = vec![];
    // let mut op_is_inf_idxs = vec![];

    // let mut spv_idx = 0;
    // while spv_idx < spv.len() {
    //     let op = spv[spv_idx];
    //     let word_count = hiword(op);
    //     let instruction = loword(op);
    // }

    todo!()
}
