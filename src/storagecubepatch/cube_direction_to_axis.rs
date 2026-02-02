use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct CubeDirectionTypeInputs {
    pub int_id: u32,
    pub v3float_id: u32,
    pub float_id: u32,
    pub bool_id: u32,
    pub ptr_v3float_id: u32,
    pub ptr_float_id: u32,
    pub glsl_std: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CubeDirectionFunctionType(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CubeDirectionConstants {
    pub uint_0: u32,
    pub uint_1: u32,
    pub uint_2: u32,
    pub float_0: u32,
    pub int_0: u32,
    pub int_1: u32,
    pub int_2: u32,
    pub int_3: u32,
    pub int_4: u32,
    pub int_5: u32,
}

pub(super) fn cube_direction_fn_type(
    ib: &mut u32,
    ty_inputs: CubeDirectionTypeInputs,
) -> (CubeDirectionFunctionType, Vec<u32>) {
    //  %_function_type = OpTypeFunction %int %_ptr_Function_v3float
    let function_type = inc(ib);
    #[rustfmt::skip]
    let spv = vec![
        encode_word(4, SPV_INSTRUCTION_OP_TYPE_FUNCTION),
            function_type, ty_inputs.int_id, ty_inputs.ptr_v3float_id,
    ];
    (CubeDirectionFunctionType(function_type), spv)
}

pub(super) fn cube_direction_constants(
    ib: &mut u32,
    uint_id: u32,
    float_id: u32,
    int_id: u32,
) -> (CubeDirectionConstants, Vec<u32>) {
    //      %uint_0 = OpConstant %uint 0
    //      %uint_1 = OpConstant %uint 1
    //      %uint_2 = OpConstant %uint 2
    //     %float_0 = OpConstant %float 0
    //       %int_0 = OpConstant %int 0
    //       %int_1 = OpConstant %int 1
    //       %int_2 = OpConstant %int 2
    //       %int_3 = OpConstant %int 3
    //       %int_4 = OpConstant %int 4
    //       %int_5 = OpConstant %int 5

    let uint_0 = inc(ib);
    let uint_1 = inc(ib);
    let uint_2 = inc(ib);
    let float_0 = inc(ib);
    let int_0 = inc(ib);
    let int_1 = inc(ib);
    let int_2 = inc(ib);
    let int_3 = inc(ib);
    let int_4 = inc(ib);
    let int_5 = inc(ib);

    #[rustfmt::skip]
    let spv = vec![
        encode_word(4, SPV_INSTRUCTION_OP_CONSTANT),
            uint_id, uint_0, 0,
        encode_word(4, SPV_INSTRUCTION_OP_CONSTANT),
            uint_id, uint_1, 1,
        encode_word(4, SPV_INSTRUCTION_OP_CONSTANT),
            uint_id, uint_2, 2,
        encode_word(4, SPV_INSTRUCTION_OP_CONSTANT),
            float_id, float_0, 0,
        encode_word(4, SPV_INSTRUCTION_OP_CONSTANT),
            int_id, int_0, 0,
        encode_word(4, SPV_INSTRUCTION_OP_CONSTANT),
            int_id, int_1, 1,
        encode_word(4, SPV_INSTRUCTION_OP_CONSTANT),
            int_id, int_2, 2,
        encode_word(4, SPV_INSTRUCTION_OP_CONSTANT),
            int_id, int_3, 3,
        encode_word(4, SPV_INSTRUCTION_OP_CONSTANT),
            int_id, int_4, 4,
        encode_word(4, SPV_INSTRUCTION_OP_CONSTANT),
            int_id, int_5, 5,
    ];

    (
        CubeDirectionConstants {
            uint_0,
            uint_1,
            uint_2,
            float_0,
            int_0,
            int_1,
            int_2,
            int_3,
            int_4,
            int_5,
        },
        spv,
    )
}

pub(super) fn cube_direction_to_axis_spv(
    ib: &mut u32,
    ty_inputs: CubeDirectionTypeInputs,
    constants: CubeDirectionConstants,
    function_type: CubeDirectionFunctionType,
) -> (u32, Vec<u32>) {
    // %_cubemapDirectionToAxis_vf3_ = OpFunction %int None %_function_type
    //           %r = OpFunctionParameter %_ptr_Function_v3float
    //          %13 = OpLabel
    //           %a = OpVariable %_ptr_Function_v3float Function
    //          %15 = OpLoad %v3float %r
    //          %16 = OpExtInst %v3float %glslstd FAbs %15
    //                OpStore %a %16
    //          %21 = OpAccessChain %_ptr_Function_float %a %uint_0
    //          %22 = OpLoad %float %21
    //          %24 = OpAccessChain %_ptr_Function_float %a %uint_1
    //          %25 = OpLoad %float %24
    //          %26 = OpFOrdGreaterThanEqual %bool %22 %25
    //                OpSelectionMerge %28 None
    //                OpBranchConditional %26 %27 %28
    //          %27 = OpLabel
    //          %29 = OpAccessChain %_ptr_Function_float %a %uint_0
    //          %30 = OpLoad %float %29
    //          %32 = OpAccessChain %_ptr_Function_float %a %uint_2
    //          %33 = OpLoad %float %32
    //          %34 = OpFOrdGreaterThanEqual %bool %30 %33
    //                OpBranch %28
    //          %28 = OpLabel
    //          %35 = OpPhi %bool %26 %13 %34 %27
    //                OpSelectionMerge %37 None
    //                OpBranchConditional %35 %36 %46
    //          %36 = OpLabel
    //          %38 = OpAccessChain %_ptr_Function_float %r %uint_0
    //          %39 = OpLoad %float %38
    //          %41 = OpFOrdGreaterThan %bool %39 %float_0
    //          %44 = OpSelect %int %41 %int_0 %int_1
    //                OpReturnValue %44
    //          %46 = OpLabel
    //          %47 = OpAccessChain %_ptr_Function_float %a %uint_1
    //          %48 = OpLoad %float %47
    //          %49 = OpAccessChain %_ptr_Function_float %a %uint_0
    //          %50 = OpLoad %float %49
    //          %51 = OpFOrdGreaterThanEqual %bool %48 %50
    //                OpSelectionMerge %53 None
    //                OpBranchConditional %51 %52 %53
    //          %52 = OpLabel
    //          %54 = OpAccessChain %_ptr_Function_float %a %uint_1
    //          %55 = OpLoad %float %54
    //          %56 = OpAccessChain %_ptr_Function_float %a %uint_2
    //          %57 = OpLoad %float %56
    //          %58 = OpFOrdGreaterThanEqual %bool %55 %57
    //                OpBranch %53
    //          %53 = OpLabel
    //          %59 = OpPhi %bool %51 %46 %58 %52
    //                OpSelectionMerge %61 None
    //                OpBranchConditional %59 %60 %69
    //          %60 = OpLabel
    //          %62 = OpAccessChain %_ptr_Function_float %r %uint_1
    //          %63 = OpLoad %float %62
    //          %64 = OpFOrdGreaterThan %bool %63 %float_0
    //          %67 = OpSelect %int %64 %int_2 %int_3
    //                OpReturnValue %67
    //          %69 = OpLabel
    //          %70 = OpAccessChain %_ptr_Function_float %r %uint_2
    //          %71 = OpLoad %float %70
    //          %72 = OpFOrdGreaterThan %bool %71 %float_0
    //          %75 = OpSelect %int %72 %int_4 %int_5
    //                OpReturnValue %75
    //          %61 = OpLabel
    //                OpUnreachable
    //          %37 = OpLabel
    //                OpUnreachable
    //                OpFunctionEnd
    //
    let function_type = function_type.0;
    let CubeDirectionConstants {
        uint_0,
        uint_1,
        uint_2,
        float_0,
        int_0,
        int_1,
        int_2,
        int_3,
        int_4,
        int_5,
    } = constants;

    let CubeDirectionTypeInputs {
        int_id,
        v3float_id,
        float_id,
        bool_id,
        ptr_v3float_id,
        ptr_float_id,
        glsl_std,
    } = ty_inputs;

    let cubemap_fn = inc(ib);
    let r = inc(ib);
    let label_13 = inc(ib);
    let a = inc(ib);
    let res_15 = inc(ib);
    let res_16 = inc(ib);
    let res_21 = inc(ib);
    let res_22 = inc(ib);
    let res_24 = inc(ib);
    let res_25 = inc(ib);
    let res_26 = inc(ib);
    let label_27 = inc(ib);
    let label_28 = inc(ib);
    let res_29 = inc(ib);
    let res_30 = inc(ib);
    let res_32 = inc(ib);
    let res_33 = inc(ib);
    let res_34 = inc(ib);
    let res_35 = inc(ib);
    let label_36 = inc(ib);
    let label_37 = inc(ib);
    let res_38 = inc(ib);
    let res_39 = inc(ib);
    let res_41 = inc(ib);
    let res_44 = inc(ib);
    let label_46 = inc(ib);
    let res_47 = inc(ib);
    let res_48 = inc(ib);
    let res_49 = inc(ib);
    let res_50 = inc(ib);
    let res_51 = inc(ib);
    let label_52 = inc(ib);
    let label_53 = inc(ib);
    let res_54 = inc(ib);
    let res_55 = inc(ib);
    let res_56 = inc(ib);
    let res_57 = inc(ib);
    let res_58 = inc(ib);
    let res_59 = inc(ib);
    let label_60 = inc(ib);
    let label_61 = inc(ib);
    let res_62 = inc(ib);
    let res_63 = inc(ib);
    let res_64 = inc(ib);
    let res_67 = inc(ib);
    let label_69 = inc(ib);
    let res_70 = inc(ib);
    let res_71 = inc(ib);
    let res_72 = inc(ib);
    let res_75 = inc(ib);

    #[rustfmt::skip]
    let spv = vec![
        encode_word(5, SPV_INSTRUCTION_OP_FUNCTION),
            int_id, cubemap_fn, SPV_FUNCTION_CONTROL_INLINE, function_type,
        encode_word(3, SPV_INSTRUCTION_OP_FUNCTION_PARAMETER),
            ptr_v3float_id, r,
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_13,
        encode_word(4, SPV_INSTRUCTION_OP_VARIABLE),
            ptr_v3float_id, a, SPV_STORAGE_CLASS_FUNCTION,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            v3float_id, res_15, r,
        encode_word(7, SPV_INSTRUCTION_OP_EXT_INST),
            v3float_id, res_16, glsl_std, 4, res_15, // 4 = FAbs
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            a, res_16,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_float_id, res_21, a, uint_0,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            float_id, res_22, res_21,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_float_id, res_24, a, uint_1,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            float_id, res_25, res_24,
        encode_word(5, SPV_INSTRUCTION_OP_F_ORD_GREATER_THAN_EQUAL),
            bool_id, res_26, res_22, res_25,
        encode_word(3, SPV_INSTRUCTION_OP_SELECTION_MERGE),
            label_28, SPV_SELECTION_CONTROL_NONE,
        encode_word(4, SPV_INSTRUCTION_OP_BRANCH_CONDITIONAL),
            res_26, label_27, label_28,
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_27,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_float_id, res_29, a, uint_0,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            float_id, res_30, res_29,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_float_id, res_32, a, uint_2,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            float_id, res_33, res_32,
        encode_word(5, SPV_INSTRUCTION_OP_F_ORD_GREATER_THAN_EQUAL),
            bool_id, res_34, res_30, res_33,
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
            label_28,
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_28,
        encode_word(7, SPV_INSTRUCTION_OP_PHI),
            bool_id, res_35, res_26, label_13, res_34, label_27,
        encode_word(3, SPV_INSTRUCTION_OP_SELECTION_MERGE),
            label_37, SPV_SELECTION_CONTROL_NONE,
        encode_word(4, SPV_INSTRUCTION_OP_BRANCH_CONDITIONAL),
            res_35, label_36, label_46,
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_36,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_float_id, res_38, r, uint_0,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            float_id, res_39, res_38,
        encode_word(5, SPV_INSTRUCTION_OP_F_ORD_GREATER_THAN),
            bool_id, res_41, res_39, float_0,
        encode_word(6, SPV_INSTRUCTION_OP_SELECT),
            int_id, res_44, res_41, int_0, int_1,
        encode_word(2, SPV_INSTRUCTION_OP_RETURN_VALUE),
            res_44,
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_46,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_float_id, res_47, a, uint_1,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            float_id, res_48, res_47,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_float_id, res_49, a, uint_0,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            float_id, res_50, res_49,
        encode_word(5, SPV_INSTRUCTION_OP_F_ORD_GREATER_THAN_EQUAL),
            bool_id, res_51, res_48, res_50,
        encode_word(3, SPV_INSTRUCTION_OP_SELECTION_MERGE),
            label_53, SPV_SELECTION_CONTROL_NONE,
        encode_word(4, SPV_INSTRUCTION_OP_BRANCH_CONDITIONAL),
            res_51, label_52, label_53,
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_52,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_float_id, res_54, a, uint_1,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            float_id, res_55, res_54,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_float_id, res_56, a, uint_2,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            float_id, res_57, res_56,
        encode_word(5, SPV_INSTRUCTION_OP_F_ORD_GREATER_THAN_EQUAL),
            bool_id, res_58, res_55, res_57,
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
            label_53,
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_53,
        encode_word(7, SPV_INSTRUCTION_OP_PHI),
            bool_id, res_59, res_51, label_46, res_58, label_52,
        encode_word(3, SPV_INSTRUCTION_OP_SELECTION_MERGE),
            label_61, SPV_SELECTION_CONTROL_NONE,
        encode_word(4, SPV_INSTRUCTION_OP_BRANCH_CONDITIONAL),
            res_59, label_60, label_69,
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_60,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_float_id, res_62, r, uint_1,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            float_id, res_63, res_62,
        encode_word(5, SPV_INSTRUCTION_OP_F_ORD_GREATER_THAN),
            bool_id, res_64, res_63, float_0,
        encode_word(6, SPV_INSTRUCTION_OP_SELECT),
            int_id, res_67, res_64, int_2, int_3,
        encode_word(2, SPV_INSTRUCTION_OP_RETURN_VALUE),
            res_67,
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_69,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_float_id, res_70, r, uint_2,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            float_id, res_71, res_70,
        encode_word(5, SPV_INSTRUCTION_OP_F_ORD_GREATER_THAN),
            bool_id, res_72, res_71, float_0,
        encode_word(6, SPV_INSTRUCTION_OP_SELECT),
            int_id, res_75, res_72, int_4, int_5,
        encode_word(2, SPV_INSTRUCTION_OP_RETURN_VALUE),
            res_75,
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_61,
        encode_word(1, SPV_INSTRUCTION_OP_UNREACHABLE),
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_37,
        encode_word(1, SPV_INSTRUCTION_OP_UNREACHABLE),
        encode_word(1, SPV_INSTRUCTION_OP_FUNCTION_END),
    ];

    (cubemap_fn, spv)
}
