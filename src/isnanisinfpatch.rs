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

/// Perform the operation on a `Vec<u32>`.
/// Use [u8_slice_to_u32_vec] to convert a `&[u8]` into a `Vec<u32>`.
/// Does not produce any side effects or corrections.
pub fn isnanisinfpatch(in_spv: &[u32]) -> Result<Vec<u32>, ()> {
    let spv = in_spv.to_owned();

    let mut instruction_bound = spv[SPV_HEADER_INSTRUCTION_BOUND_OFFSET];
    let magic_number = spv[SPV_HEADER_MAGIC_NUM_OFFSET];

    let spv_header = spv[0..SPV_HEADER_LENGTH].to_owned();

    assert_eq!(magic_number, SPV_HEADER_MAGIC);

    let mut instruction_inserts = vec![];
    let mut word_inserts = vec![];

    let spv = spv.into_iter().skip(SPV_HEADER_LENGTH).collect::<Vec<_>>();
    let mut new_spv = spv.clone();

    // 1. Find locations instructions we need
    let mut op_function_idxs = vec![];
    let mut op_is_nan_idxs = vec![];
    let mut op_is_inf_idxs = vec![];

    let mut op_type_bool_idxs = vec![];
    let mut op_type_int_idxs = vec![];
    let mut op_type_float_idxs = vec![];

    let mut spv_idx = 0;
    while spv_idx < spv.len() {
        let op = spv[spv_idx];
        let word_count = hiword(op);
        let instruction = loword(op);

        match instruction {
            SPV_INSTRUCTION_OP_FUNCTION => op_function_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_IS_NAN => op_is_nan_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_IS_INF => op_is_inf_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_BOOL => op_type_bool_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_INT => op_type_int_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_TYPE_FLOAT => op_type_float_idxs.push(spv_idx),
            _ => {}
        }

        spv_idx += word_count as usize;
    }

    if op_is_nan_idxs.is_empty() && op_is_inf_idxs.is_empty() {
        return Ok(in_spv.to_vec());
    }

    // 2. Find or insert prereq types and definitions

    // 3. Insert shared isnan / isinf definitions
    // TODO: can we have duplicate OpTypeFunctions / OpConstants?

    // We need to consider the existance of isnan used with bvecN

    // 4. Insert and patch isnan

    // 5. Insert and patch isinf

    // 6. Insert New Instructions
    insert_new_instructions(&spv, &mut new_spv, &word_inserts, &instruction_inserts);

    // 7. Remove Instructions that have been Whited Out.
    prune_noops(&mut new_spv);

    // 8. Write New Header and New Code
    Ok(fuse_final(spv_header, new_spv, instruction_bound))
}
