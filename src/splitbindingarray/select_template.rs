use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct SelectTemplateFunctionInputs {
    pub uint32_id: u32,
    pub item_type_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SelectFunctionType(pub u32);

pub(super) fn select_template_type_spv(
    ib: &mut u32,
    inputs: SelectTemplateFunctionInputs,
    length: usize,
) -> (SelectFunctionType, Vec<u32>) {
    // %_select_type_fn = OpTypeFunction %item_type %uint_32 %item_type [%item_type ..]
    let SelectTemplateFunctionInputs {
        uint32_id,
        item_type_id,
    } = inputs;
    let fn_type_id = inc(ib);
    let mut spv = vec![];

    spv.extend_from_slice(&[
        encode_word(length as u16 + 4, SPV_INSTRUCTION_OP_TYPE_FUNCTION),
        fn_type_id,
        item_type_id,
        uint32_id,
    ]);
    for _ in 0..length {
        spv.push(item_type_id);
    }

    (SelectFunctionType(fn_type_id), spv)
}

pub(super) fn select_template_spv(
    ib: &mut u32,
    inputs: SelectTemplateFunctionInputs,
    function_type: SelectFunctionType,
    length: usize,
) -> (u32, Vec<u32>) {
    //      %_select_fn = OpFunction %item_type Inline %_select_type_fn
    //             %idx = OpFunctionParameter %uint_32
    //          %item_0 = OpFunctionParameter %item_type
    //          ...
    //          %item_N = OpFunctionParameter %item_type
    //               %1 = OpLabel
    //                    OpSelectionMerge %merge None
    //                    OpSwitch %idx %merge 0 %case_0 1 %case_1 ... N %case_N
    //          %case_0 = OpLabel
    //                    OpReturnValue %item_0
    //          %case_1 = OpLabel
    //                    OpReturnValue %item_1
    //          ...
    //          %case_N = OpLabel
    //                    OpReturnValue %item_N
    //           %merge = OpLabel
    //                    OpReturnValue %item_0
    //                    OpFunctionEnd
    //

    let SelectTemplateFunctionInputs {
        uint32_id,
        item_type_id,
    } = inputs;

    let SelectFunctionType(fn_type_id) = function_type;
    let fn_id = inc(ib);
    let idx_param = inc(ib);
    let item_params = (0..length).map(|_| inc(ib)).collect::<Vec<u32>>();
    let entry_label = inc(ib);
    let case_labels = (0..length).map(|_| inc(ib)).collect::<Vec<u32>>();
    let merge_label = inc(ib);

    let mut spv = vec![];

    spv.extend_from_slice(&[
        encode_word(5, SPV_INSTRUCTION_OP_FUNCTION),
        item_type_id,
        fn_id,
        SPV_FUNCTION_CONTROL_INLINE,
        fn_type_id,
        encode_word(3, SPV_INSTRUCTION_OP_FUNCTION_PARAMETER),
        uint32_id,
        idx_param,
    ]);
    for &item_param in &item_params {
        spv.extend_from_slice(&[
            encode_word(3, SPV_INSTRUCTION_OP_FUNCTION_PARAMETER),
            item_type_id,
            item_param,
        ]);
    }

    spv.extend_from_slice(&[
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
        entry_label,
        encode_word(3, SPV_INSTRUCTION_OP_SELECTION_MERGE),
        merge_label,
        SPV_SELECTION_CONTROL_NONE,
        encode_word(3 + 2 * length as u16, SPV_INSTRUCTION_OP_SWITCH),
        idx_param,
        merge_label,
    ]);
    for (i, &case_label) in case_labels.iter().enumerate() {
        spv.push(i as u32);
        spv.push(case_label);
    }

    for (case_label, item_param) in case_labels.iter().zip(item_params.iter()) {
        spv.extend_from_slice(&[
            encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            *case_label,
            encode_word(2, SPV_INSTRUCTION_OP_RETURN_VALUE),
            *item_param,
        ]);
    }

    spv.extend_from_slice(&[
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
        merge_label,
        encode_word(2, SPV_INSTRUCTION_OP_RETURN_VALUE),
        item_params[0],
        encode_word(1, SPV_INSTRUCTION_OP_FUNCTION_END),
    ]);

    (fn_id, spv)
}
