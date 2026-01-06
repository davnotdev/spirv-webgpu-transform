use super::*;

pub(super) fn is_nan_is_inf_spv(
    ib: &mut u32,
    ty: IsNanOrIsInf,
    ty_inputs: NanInfSharedTypeInputs,
    inputs: NanInfSharedFunctionInputs,
    function_type: NanInfFunctionType,
    shared_constants: NanInfSharedConstants,
) -> (u32, Vec<u32>) {
    // The only difference between the two is one OpIEqual vs OpINotEqual
    //
    // %isnan_f1_ = OpFunction %bool None %_function_type
    //         %x = OpFunctionParameter %_ptr_Function_float
    //         %1 = OpLabel
    //      %bits = OpVariable %_ptr_Function_uint Function
    //       %exp = OpVariable %_ptr_Function_uint Function
    //      %frac = OpVariable %_ptr_Function_uint Function
    //         %2 = OpLoad %float %x
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
    //
    // %_isinf_f1_ = OpFunction %bool None %function_type
    //          %x = OpFunctionParameter %_ptr_Function_float
    //          %1 = OpLabel
    //       %bits = OpVariable %_ptr_Function_uint Function
    //        %exp = OpVariable %_ptr_Function_uint Function
    //       %frac = OpVariable %_ptr_Function_uint Function
    //          %2 = OpLoad %float %x
    //          %3 = OpBitcast %uint %2
    //               OpStore %bits %3
    //          %4 = OpLoad %uint %bits
    //          %5 = OpShiftRightLogical %uint %4 %int_23
    //          %6 = OpBitwiseAnd %uint %5 %uint_255
    //               OpStore %exp %6
    //          %7 = OpLoad %uint %bits
    //          %8 = OpBitwiseAnd %uint %7 %uint_8388607
    //               OpStore %frac %8
    //          %9 = OpLoad %uint %exp
    //         %10 = OpIEqual %bool %9 %uint_255
    //         %11 = OpLoad %uint %frac
    //         %12 = OpINotEqual %bool %11 %uint_0
    //         %13 = OpLogicalAnd %bool %10 %12
    //               OpReturnValue %13
    //               OpFunctionEnd
    //

    let function_type = function_type.0;
    let NanInfSharedConstants {
        uint_23,
        uint_255,
        uint_8388607,
        uint_0,
    } = shared_constants;

    let NanInfSharedTypeInputs {
        uint_id,
        ptr_uint_id,
    } = ty_inputs;

    let NanInfSharedFunctionInputs {
        bool_id,
        float_id,
        ptr_float_id,
    } = inputs;

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
    let spv = vec![
        encode_word(5, SPV_INSTRUCTION_OP_FUNCTION), 
            bool_id, is_nan, SPV_FUNCTION_CONTROL_INLINE, function_type, 
        encode_word(3, SPV_INSTRUCTION_OP_FUNCTION_PARAMETER),
            x, ptr_float_id,
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            res_1,
        encode_word(3, SPV_INSTRUCTION_OP_VARIABLE),
            bits, ptr_uint_id,
        encode_word(3, SPV_INSTRUCTION_OP_VARIABLE),
            exp, ptr_uint_id,
        encode_word(3, SPV_INSTRUCTION_OP_VARIABLE),
            frac, ptr_uint_id,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            res_2, x, float_id,
        encode_word(4, SPV_INSTRUCTION_OP_BITCAST),
            res_3, res_2, uint_id,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            bits, res_3,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            res_4, bits, uint_id,
        encode_word(4, SPV_INSTRUCTION_OP_SHIFT_RIGHT_LOGICAL),
            res_5, res_4, uint_23,
        encode_word(4, SPV_INSTRUCTION_OP_BITWISE_AND),
            res_6, res_5, uint_255,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            exp, res_6,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            res_7, bits, uint_id,
        encode_word(4, SPV_INSTRUCTION_OP_BITWISE_AND),
            res_8, res_7, uint_8388607,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            frac, res_8,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            res_9, exp, uint_id,
        encode_word(4, SPV_INSTRUCTION_OP_I_EQUAL),
            res_10, res_9, uint_255,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            res_11, frac, uint_id,
        encode_word(4, 
            match ty {
                IsNanOrIsInf::IsNan => SPV_INSTRUCTION_OP_I_EQUAL,
                IsNanOrIsInf::IsInf => SPV_INSTRUCTION_OP_I_NOT_EQUAL,
            }
            ),
            res_12, res_11, uint_0,
        encode_word(4, SPV_INSTRUCTION_OP_LOGICAL_AND),
            res_13, res_10, res_12,
        encode_word(2, SPV_INSTRUCTION_OP_RETURN_VALUE),
            res_13,
        encode_word(1, SPV_INSTRUCTION_OP_FUNCTION_END),
    ];

    (is_nan, spv)
}
