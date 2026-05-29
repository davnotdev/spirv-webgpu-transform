use super::*;

pub(super) fn select_template_spv<F: FnMut(&mut u32, u32) -> (Vec<u32>, Option<u32>)>(
    ib: &mut u32,
    base_id: u32,
    index_id: u32,
    length: usize,
    mut instruction_builder: F,
    result_type_and_id: Option<(u32, u32)>,
) -> Vec<u32> {
    //
    //  TODO: You can probably decrease the instruction count with OpPhi or OpSelect.
    //
    //              OpSelectionMerge %merge None
    //              OpSwitch %index_id %default %merge 0 %case_0 1 %case_1 ... N %case_N
    //    %case_0 = OpLabel
    //    %temp_0 = {instruction_builder(%base_id+0)}
    //              OpBranch %merge
    //    %case_1 = OpLabel
    //    %temp_1 = {instruction_builder(%base_id+1)}
    //              OpBranch %merge
    //
    //              ...
    //
    //    %case_N = OpLabel
    //    %temp_N = {instruction_builder(%base_id+N)}
    //              OpBranch %merge
    //    %default = OpLabel
    //    %temp_def = {instruction_builder(%base_id+0)}
    //    %merge = OpLabel
    //
    //    ; Only if there will be a result value.
    //    %target_id = OpPhi %underlying_type_id %temp_0 %case_0 %temp_1 %case_1 ... %temp_N %case_N %temp_def %default
    //

    let case_labels = (0..length).map(|_| inc(ib)).collect::<Vec<u32>>();
    let default_label = inc(ib);
    let merge_label = inc(ib);

    let mut spv = vec![];
    spv.extend_from_slice(&[
        encode_word(3, SPV_INSTRUCTION_OP_SELECTION_MERGE),
        merge_label,
        SPV_SELECTION_CONTROL_NONE,
        encode_word(3 + 2 * length as u16, SPV_INSTRUCTION_OP_SWITCH),
        index_id,
        default_label,
    ]);
    for (i, &case_label) in case_labels.iter().enumerate() {
        spv.push(i as u32);
        spv.push(case_label);
    }

    let mut output_ids = vec![];
    for (i, &case_label) in case_labels.iter().enumerate() {
        spv.extend_from_slice(&[encode_word(2, SPV_INSTRUCTION_OP_LABEL), case_label]);
        let (instructions, maybe_output_id) = instruction_builder(ib, base_id + i as u32);
        spv.extend_from_slice(&instructions);
        if let Some(output_id) = maybe_output_id {
            output_ids.push(output_id);
        }
        spv.extend_from_slice(&[encode_word(2, SPV_INSTRUCTION_OP_BRANCH), merge_label]);
    }
    spv.extend_from_slice(&[encode_word(2, SPV_INSTRUCTION_OP_LABEL), default_label]);
    let (instructions, default_output_id) = instruction_builder(ib, base_id);
    spv.extend_from_slice(&instructions);
    spv.extend_from_slice(&[
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
        merge_label,
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
        merge_label,
    ]);
    if let Some((result_type_id, target_id)) = result_type_and_id {
        assert!(output_ids.len() == length);
        spv.push(encode_word(
            3 + 2 * (length as u16 + 1),
            SPV_INSTRUCTION_OP_PHI,
        ));
        spv.push(result_type_id);
        spv.push(target_id);
        for (i, &case_label) in case_labels.iter().enumerate() {
            spv.push(output_ids[i]);
            spv.push(case_label);
        }
        spv.push(
            default_output_id.expect("default block must produce output when result is expected"),
        );
        spv.push(default_label);
    }

    spv
}
