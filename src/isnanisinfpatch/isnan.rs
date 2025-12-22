use super::*;

pub(super) struct NanSpvInputs {
    bool_id: u32,
    uint_id: u32,
    double_id: u32,
    ptr_function_float_id: u32,
    ptr_function_uint_id: u32,
}

fn is_nan_spv(ib: &mut u32, inputs: NanSpvInputs) -> (u32, Box<[u32]>, Box<[u32]>) {
    //  Required:
    //      %bool
    //      %uint
    //      %double
    //      %_ptr_Function_double
    //      %_ptr_Function_uint
    //
    //    %_function_type = OpTypeFunction %bool %_ptr_Function_float
    //           %uint_23 = OpConstant %uint 23
    //          %uint_255 = OpConstant %uint 255
    //      %uint_8388607 = OpConstant %uint 8388607
    //            %uint_0 = OpConstant %uint 0
    //          -----
    // %isnan_d1_ = OpFunction %bool None %_function_type
    //         %x = OpFunctionParameter %_ptr_Function_double
    //         %1 = OpLabel
    //      %bits = OpVariable %_ptr_Function_uint Function
    //       %exp = OpVariable %_ptr_Function_uint Function
    //      %frac = OpVariable %_ptr_Function_uint Function
    //         %2 = OpLoad %double %x
    //         %3 = OpBitcast %uint %2
    //              OpStore %bits %3
    //         %4 = OpLoad %uint %bits
    //         %5 = OpShiftRightLogical %uint %4 %uint_23
    //         %6 = OpBitwiseAnd %uint %5 %uint_255
    //              OpStore %exp %6
    //         %7 = OpLoad %uint %bits
    //         %8 = OpBitwiseAnd %uint %7 %uint_8388607
    //              OpStore %frac %8
    //         %9 = OpLoad %uint %exp
    //        %10 = OpIEqual %bool %9 %uint_255
    //        %11 = OpLoad %uint %frac
    //        %12 = OpIEqual %bool %11 %uint_0
    //        %13 = OpLogicalAnd %bool %10 %12
    //              OpReturnValue %13
    //              OpFunctionEnd

    let function_type = inc(ib);
    let uint_23 = inc(ib);
    let uint_255 = inc(ib);
    let uint_8388607 = inc(ib);
    let uint_0 = inc(ib);

    let is_nan = inc(ib);
    let x = inc(ib);
    let res_1 = inc(ib);
    let bits = inc(ib);
    let exp = inc(ib);
    let frac = inc(ib);
    let res_2 = inc(ib);
    let res_3 = inc(ib);
    let res_4 = inc(ib);
    let res_5 = inc(ib);
    let res_6 = inc(ib);
    let res_7 = inc(ib);
    let res_8 = inc(ib);
    let res_9 = inc(ib);
    let res_10 = inc(ib);
    let res_11 = inc(ib);
    let res_12 = inc(ib);
    let res_13 = inc(ib);

    #[rustfmt::skip]
    let upper_spv = Box::new([
        encode_word(4, SPV_INSTRUCTION_OP_TYPE_FUNCTION), 
            function_type, inputs.bool_id, inputs.ptr_function_float_id,
        encode_word(4, SPV_INSTRUCTION_OP_CONSTANT), 
            uint_23, inputs.uint_id, 23,
        encode_word(4, SPV_INSTRUCTION_OP_CONSTANT), 
            uint_255, inputs.uint_id, 255,
        encode_word(4, SPV_INSTRUCTION_OP_CONSTANT), 
            uint_8388607, inputs.uint_id, 8388607,
        encode_word(4, SPV_INSTRUCTION_OP_CONSTANT), 
            uint_0, inputs.uint_id, 0,
    ]);
    #[rustfmt::skip]
    let lower_spv = Box::new([
        encode_word(3, SPV_INSTRUCTION_OP_FUNCTION), 
            is_nan, function_type,
        encode_word(3, SPV_INSTRUCTION_OP_FUNCTION_PARAMETER),
            x, inputs.ptr_function_float_id,
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            res_1,
        encode_word(3, SPV_INSTRUCTION_OP_VARIABLE),
            bits, inputs.ptr_function_uint_id,
        encode_word(3, SPV_INSTRUCTION_OP_VARIABLE),
            exp, inputs.ptr_function_uint_id,
        encode_word(3, SPV_INSTRUCTION_OP_VARIABLE),
            frac, inputs.ptr_function_uint_id,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            res_2, x, inputs.double_id,
        encode_word(4, SPV_INSTRUCTION_OP_BITCAST),
            res_3, res_2, inputs.uint_id,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            bits, res_3,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            res_4, bits, inputs.uint_id,
        encode_word(4, SPV_INSTRUCTION_OP_SHIFT_RIGHT_LOGICAL),
            res_5, res_4, uint_23,
        encode_word(4, SPV_INSTRUCTION_OP_BITWISE_AND),
            res_6, res_5, uint_255,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            exp, res_6,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            res_7, bits, inputs.uint_id,
        encode_word(4, SPV_INSTRUCTION_OP_BITWISE_AND),
            res_8, res_7, uint_8388607,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            frac, res_8,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            res_9, exp, inputs.uint_id,
        encode_word(4, SPV_INSTRUCTION_OP_I_EQUAL),
            res_10, res_9, uint_255,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            res_11, frac, inputs.uint_id,
        encode_word(4, SPV_INSTRUCTION_OP_I_EQUAL),
            res_12, res_11, uint_0,
        encode_word(4, SPV_INSTRUCTION_OP_LOGICAL_AND),
            res_13, res_10, res_12,
        encode_word(2, SPV_INSTRUCTION_OP_RETURN_VALUE),
            res_13,
        encode_word(1, SPV_INSTRUCTION_OP_FUNCTION_END),
    ]);

    (is_nan, upper_spv, lower_spv)
}
