use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum IsNanOrIsInf {
    IsNan,
    IsInf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct NanInfSharedTypeInputs {
    pub uint_id: u32,
    pub ptr_uint_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct NanInfSharedFunctionInputs {
    pub bool_id: u32,
    pub float_id: u32,
    pub ptr_float_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NanInfFunctionType(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NanInfSharedConstants {
    pub uint_23: u32,
    pub uint_255: u32,
    pub uint_8388607: u32,
    pub uint_0: u32,
}

pub(super) fn nan_inf_fn_type(
    ib: &mut u32,
    inputs: NanInfSharedFunctionInputs,
) -> (NanInfFunctionType, Vec<u32>) {
    //
    //    %_function_type = OpTypeFunction %bool %_ptr_Function_float

    let function_type = inc(ib);
    #[rustfmt::skip]
    let spv = vec![
        encode_word(4, SPV_INSTRUCTION_OP_TYPE_FUNCTION), 
            function_type, inputs.bool_id, inputs.ptr_float_id,
    ];

    (NanInfFunctionType(function_type), spv)
}

pub(super) fn nan_inf_shared_constants(
    ib: &mut u32,
    inputs: NanInfSharedTypeInputs,
) -> (NanInfSharedConstants, Vec<u32>) {
    //
    //           %uint_23 = OpConstant %uint 23
    //          %uint_255 = OpConstant %uint 255
    //      %uint_8388607 = OpConstant %uint 8388607
    //            %uint_0 = OpConstant %uint 0

    let uint_23 = inc(ib);
    let uint_255 = inc(ib);
    let uint_8388607 = inc(ib);
    let uint_0 = inc(ib);

    #[rustfmt::skip]
    let spv = vec![
        encode_word(4, SPV_INSTRUCTION_OP_CONSTANT), 
            uint_23, inputs.uint_id, 23,
        encode_word(4, SPV_INSTRUCTION_OP_CONSTANT), 
            uint_255, inputs.uint_id, 255,
        encode_word(4, SPV_INSTRUCTION_OP_CONSTANT), 
            uint_8388607, inputs.uint_id, 8388607,
        encode_word(4, SPV_INSTRUCTION_OP_CONSTANT), 
            uint_0, inputs.uint_id, 0,
    ];

    (
        NanInfSharedConstants {
            uint_23,
            uint_255,
            uint_8388607,
            uint_0,
        },
        spv,
    )
}
