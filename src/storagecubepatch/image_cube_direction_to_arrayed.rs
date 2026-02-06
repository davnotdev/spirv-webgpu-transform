use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct CubeDirectionTypeInputs {
    pub int_id: u32,
    pub v3int_id: u32,
    pub v2int_id: u32,
    pub bool_id: u32,
    pub ptr_v3int_id: u32,
    pub ptr_int_id: u32,
    pub ptr_bool_id: u32,
    pub ptr_v2int_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CubeDirectionFunctionType(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CubeDirectionConstants {
    pub int_0: u32,
    pub int_1: u32,
    pub int_2: u32,
    pub int_3: u32,
    pub int_4: u32,
    pub int_5: u32,
}

pub(super) fn image_cube_direction_to_arrayed_fn_type(
    ib: &mut u32,
    ty_inputs: CubeDirectionTypeInputs,
) -> (CubeDirectionFunctionType, Vec<u32>) {
    // %_function_type = OpTypeFunction %v3int %_ptr_Function_v3int
    let function_type = inc(ib);
    #[rustfmt::skip]
    let spv = vec![
        encode_word(4, SPV_INSTRUCTION_OP_TYPE_FUNCTION),
            function_type, ty_inputs.v3int_id, ty_inputs.ptr_v3int_id,
    ];
    (CubeDirectionFunctionType(function_type), spv)
}

pub(super) fn image_cube_direction_to_arrayed_constants_spv(
    ib: &mut u32,
    int_id: u32,
) -> (CubeDirectionConstants, Vec<u32>) {
    // %int_0 = OpConstant %int 0
    // %int_1 = OpConstant %int 1
    // %int_2 = OpConstant %int 2
    // %int_3 = OpConstant %int 3
    // %int_4 = OpConstant %int 4
    // %int_5 = OpConstant %int 5

    let int_0 = inc(ib);
    let int_1 = inc(ib);
    let int_2 = inc(ib);
    let int_3 = inc(ib);
    let int_4 = inc(ib);
    let int_5 = inc(ib);

    #[rustfmt::skip]
    let spv = vec![
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

pub(super) fn image_cube_direction_to_arrayed_spv(
    ib: &mut u32,
    ty_inputs: CubeDirectionTypeInputs,
    function_type: CubeDirectionFunctionType,
    constants: CubeDirectionConstants,
    glsl_std_id: u32,
) -> (u32, Vec<u32>) {
    // %_imageCubeDirectionToArrayed_vi3_ = OpFunction %v3int None %_function_type
    //       %r = OpFunctionParameter %_ptr_Function_v3int
    //      %12 = OpLabel
    //       %a = OpVariable %_ptr_Function_v3int Function
    //       %x = OpVariable %_ptr_Function_bool Function
    //       %y = OpVariable %_ptr_Function_bool Function
    //    %face = OpVariable %_ptr_Function_int Function
    //      %53 = OpVariable %_ptr_Function_int Function
    //      %64 = OpVariable %_ptr_Function_int Function
    //      %st = OpVariable %_ptr_Function_v2int Function
    //      %87 = OpVariable %_ptr_Function_v2int Function
    //     %100 = OpVariable %_ptr_Function_v2int Function
    //     %112 = OpVariable %_ptr_Function_v2int Function
    //     %123 = OpVariable %_ptr_Function_v2int Function
    //     %135 = OpVariable %_ptr_Function_v2int Function
    //      %14 = OpLoad %v3int %r
    //      %15 = OpExtInst %v3int %1 SAbs %14
    //            OpStore %a %15
    //      %22 = OpAccessChain %_ptr_Function_int %a %int_0
    //      %23 = OpLoad %int %22
    //      %25 = OpAccessChain %_ptr_Function_int %a %int_1
    //      %26 = OpLoad %int %25
    //      %27 = OpSGreaterThanEqual %bool %23 %26
    //            OpSelectionMerge %29 None
    //            OpBranchConditional %27 %28 %29
    //      %28 = OpLabel
    //      %30 = OpAccessChain %_ptr_Function_int %a %int_0
    //      %31 = OpLoad %int %30
    //      %33 = OpAccessChain %_ptr_Function_int %a %int_2
    //      %34 = OpLoad %int %33
    //      %35 = OpSGreaterThanEqual %bool %31 %34
    //            OpBranch %29
    //      %29 = OpLabel
    //      %36 = OpPhi %bool %27 %12 %35 %28
    //            OpStore %x %36
    //      %38 = OpAccessChain %_ptr_Function_int %a %int_1
    //      %39 = OpLoad %int %38
    //      %40 = OpAccessChain %_ptr_Function_int %a %int_0
    //      %41 = OpLoad %int %40
    //      %42 = OpSGreaterThan %bool %39 %41
    //            OpSelectionMerge %44 None
    //            OpBranchConditional %42 %43 %44
    //      %43 = OpLabel
    //      %45 = OpAccessChain %_ptr_Function_int %a %int_1
    //      %46 = OpLoad %int %45
    //      %47 = OpAccessChain %_ptr_Function_int %a %int_2
    //      %48 = OpLoad %int %47
    //      %49 = OpSGreaterThanEqual %bool %46 %48
    //            OpBranch %44
    //      %44 = OpLabel
    //      %50 = OpPhi %bool %42 %29 %49 %43
    //            OpStore %y %50
    //      %52 = OpLoad %bool %x
    //            OpSelectionMerge %55 None
    //            OpBranchConditional %52 %54 %62
    //      %54 = OpLabel
    //      %56 = OpAccessChain %_ptr_Function_int %r %int_0
    //      %57 = OpLoad %int %56
    //      %59 = OpSGreaterThan %bool %57 %int_0
    //      %61 = OpSelect %int %59 %int_0 %int_1
    //            OpStore %53 %61
    //            OpBranch %55
    //      %62 = OpLabel
    //      %63 = OpLoad %bool %y
    //            OpSelectionMerge %66 None
    //            OpBranchConditional %63 %65 %73
    //      %65 = OpLabel
    //      %67 = OpAccessChain %_ptr_Function_int %r %int_1
    //      %68 = OpLoad %int %67
    //      %69 = OpSGreaterThan %bool %68 %int_0
    //      %72 = OpSelect %int %69 %int_2 %int_3
    //            OpStore %64 %72
    //            OpBranch %66
    //      %73 = OpLabel
    //      %74 = OpAccessChain %_ptr_Function_int %r %int_2
    //      %75 = OpLoad %int %74
    //      %76 = OpSGreaterThan %bool %75 %int_0
    //      %79 = OpSelect %int %76 %int_4 %int_5
    //            OpStore %64 %79
    //            OpBranch %66
    //      %66 = OpLabel
    //      %80 = OpLoad %int %64
    //            OpStore %53 %80
    //            OpBranch %55
    //      %55 = OpLabel
    //      %81 = OpLoad %int %53
    //            OpStore %face %81
    //      %85 = OpLoad %int %face
    //      %86 = OpIEqual %bool %85 %int_0
    //            OpSelectionMerge %89 None
    //            OpBranchConditional %86 %88 %97
    //      %88 = OpLabel
    //      %90 = OpAccessChain %_ptr_Function_int %r %int_2
    //      %91 = OpLoad %int %90
    //      %92 = OpSNegate %int %91
    //      %93 = OpAccessChain %_ptr_Function_int %r %int_1
    //      %94 = OpLoad %int %93
    //      %95 = OpSNegate %int %94
    //      %96 = OpCompositeConstruct %v2int %92 %95
    //            OpStore %87 %96
    //            OpBranch %89
    //      %97 = OpLabel
    //      %98 = OpLoad %int %face
    //      %99 = OpIEqual %bool %98 %int_1
    //            OpSelectionMerge %102 None
    //            OpBranchConditional %99 %101 %109
    //     %101 = OpLabel
    //     %103 = OpAccessChain %_ptr_Function_int %r %int_2
    //     %104 = OpLoad %int %103
    //     %105 = OpAccessChain %_ptr_Function_int %r %int_1
    //     %106 = OpLoad %int %105
    //     %107 = OpSNegate %int %106
    //     %108 = OpCompositeConstruct %v2int %104 %107
    //            OpStore %100 %108
    //            OpBranch %102
    //     %109 = OpLabel
    //     %110 = OpLoad %int %face
    //     %111 = OpIEqual %bool %110 %int_2
    //            OpSelectionMerge %114 None
    //            OpBranchConditional %111 %113 %120
    //     %113 = OpLabel
    //     %115 = OpAccessChain %_ptr_Function_int %r %int_0
    //     %116 = OpLoad %int %115
    //     %117 = OpAccessChain %_ptr_Function_int %r %int_2
    //     %118 = OpLoad %int %117
    //     %119 = OpCompositeConstruct %v2int %116 %118
    //            OpStore %112 %119
    //            OpBranch %114
    //     %120 = OpLabel
    //     %121 = OpLoad %int %face
    //     %122 = OpIEqual %bool %121 %int_3
    //            OpSelectionMerge %125 None
    //            OpBranchConditional %122 %124 %132
    //     %124 = OpLabel
    //     %126 = OpAccessChain %_ptr_Function_int %r %int_0
    //     %127 = OpLoad %int %126
    //     %128 = OpAccessChain %_ptr_Function_int %r %int_2
    //     %129 = OpLoad %int %128
    //     %130 = OpSNegate %int %129
    //     %131 = OpCompositeConstruct %v2int %127 %130
    //            OpStore %123 %131
    //            OpBranch %125
    //     %132 = OpLabel
    //     %133 = OpLoad %int %face
    //     %134 = OpIEqual %bool %133 %int_4
    //            OpSelectionMerge %137 None
    //            OpBranchConditional %134 %136 %144
    //     %136 = OpLabel
    //     %138 = OpAccessChain %_ptr_Function_int %r %int_0
    //     %139 = OpLoad %int %138
    //     %140 = OpAccessChain %_ptr_Function_int %r %int_1
    //     %141 = OpLoad %int %140
    //     %142 = OpSNegate %int %141
    //     %143 = OpCompositeConstruct %v2int %139 %142
    //            OpStore %135 %143
    //            OpBranch %137
    //     %144 = OpLabel
    //     %145 = OpAccessChain %_ptr_Function_int %r %int_0
    //     %146 = OpLoad %int %145
    //     %147 = OpSNegate %int %146
    //     %148 = OpAccessChain %_ptr_Function_int %r %int_1
    //     %149 = OpLoad %int %148
    //     %150 = OpSNegate %int %149
    //     %151 = OpCompositeConstruct %v2int %147 %150
    //            OpStore %135 %151
    //            OpBranch %137
    //     %137 = OpLabel
    //     %152 = OpLoad %v2int %135
    //            OpStore %123 %152
    //            OpBranch %125
    //     %125 = OpLabel
    //     %153 = OpLoad %v2int %123
    //            OpStore %112 %153
    //            OpBranch %114
    //     %114 = OpLabel
    //     %154 = OpLoad %v2int %112
    //            OpStore %100 %154
    //            OpBranch %102
    //     %102 = OpLabel
    //     %155 = OpLoad %v2int %100
    //            OpStore %87 %155
    //            OpBranch %89
    //      %89 = OpLabel
    //     %156 = OpLoad %v2int %87
    //            OpStore %st %156
    //     %157 = OpLoad %v2int %st
    //     %158 = OpLoad %int %face
    //     %159 = OpCompositeExtract %int %157 0
    //     %160 = OpCompositeExtract %int %157 1
    //     %161 = OpCompositeConstruct %v3int %159 %160 %158
    //            OpReturnValue %161
    //            OpFunctionEnd

    let function_type = function_type.0;

    let CubeDirectionConstants {
        int_0,
        int_1,
        int_2,
        int_3,
        int_4,
        int_5,
    } = constants;

    let CubeDirectionTypeInputs {
        int_id,
        v3int_id,
        v2int_id,
        bool_id,
        ptr_v3int_id,
        ptr_int_id,
        ptr_bool_id,
        ptr_v2int_id,
    } = ty_inputs;

    // Function and parameter IDs
    let func_id = inc(ib);
    let r = inc(ib);

    let label_12 = inc(ib);

    let a = inc(ib);
    let x = inc(ib);
    let y = inc(ib);
    let face = inc(ib);
    let var_53 = inc(ib);
    let var_64 = inc(ib);
    let st = inc(ib);
    let var_87 = inc(ib);
    let var_100 = inc(ib);
    let var_112 = inc(ib);
    let var_123 = inc(ib);
    let var_135 = inc(ib);

    let res_14 = inc(ib);
    let res_15 = inc(ib);
    let res_22 = inc(ib);
    let res_23 = inc(ib);
    let res_25 = inc(ib);
    let res_26 = inc(ib);
    let res_27 = inc(ib);

    let label_29 = inc(ib);
    let label_28 = inc(ib);
    let res_30 = inc(ib);
    let res_31 = inc(ib);
    let res_33 = inc(ib);
    let res_34 = inc(ib);
    let res_35 = inc(ib);
    let res_36 = inc(ib);

    let res_38 = inc(ib);
    let res_39 = inc(ib);
    let res_40 = inc(ib);
    let res_41 = inc(ib);
    let res_42 = inc(ib);

    let label_44 = inc(ib);
    let label_43 = inc(ib);
    let res_45 = inc(ib);
    let res_46 = inc(ib);
    let res_47 = inc(ib);
    let res_48 = inc(ib);
    let res_49 = inc(ib);
    let res_50 = inc(ib);

    let res_52 = inc(ib);
    let label_55 = inc(ib);
    let label_54 = inc(ib);
    let label_62 = inc(ib);

    let res_56 = inc(ib);
    let res_57 = inc(ib);
    let res_59 = inc(ib);
    let res_61 = inc(ib);

    let res_63 = inc(ib);
    let label_66 = inc(ib);
    let label_65 = inc(ib);
    let label_73 = inc(ib);

    let res_67 = inc(ib);
    let res_68 = inc(ib);
    let res_69 = inc(ib);
    let res_72 = inc(ib);

    let res_74 = inc(ib);
    let res_75 = inc(ib);
    let res_76 = inc(ib);
    let res_79 = inc(ib);

    let res_80 = inc(ib);
    let res_81 = inc(ib);

    let res_85 = inc(ib);
    let res_86 = inc(ib);
    let label_89 = inc(ib);
    let label_88 = inc(ib);
    let label_97 = inc(ib);

    let res_90 = inc(ib);
    let res_91 = inc(ib);
    let res_92 = inc(ib);
    let res_93 = inc(ib);
    let res_94 = inc(ib);
    let res_95 = inc(ib);
    let res_96 = inc(ib);

    let res_98 = inc(ib);
    let res_99 = inc(ib);
    let label_102 = inc(ib);
    let label_101 = inc(ib);
    let label_109 = inc(ib);

    let res_103 = inc(ib);
    let res_104 = inc(ib);
    let res_105 = inc(ib);
    let res_106 = inc(ib);
    let res_107 = inc(ib);
    let res_108 = inc(ib);

    let res_110 = inc(ib);
    let res_111 = inc(ib);
    let label_114 = inc(ib);
    let label_113 = inc(ib);
    let label_120 = inc(ib);

    let res_115 = inc(ib);
    let res_116 = inc(ib);
    let res_117 = inc(ib);
    let res_118 = inc(ib);
    let res_119 = inc(ib);

    let res_121 = inc(ib);
    let res_122 = inc(ib);
    let label_125 = inc(ib);
    let label_124 = inc(ib);
    let label_132 = inc(ib);

    let res_126 = inc(ib);
    let res_127 = inc(ib);
    let res_128 = inc(ib);
    let res_129 = inc(ib);
    let res_130 = inc(ib);
    let res_131 = inc(ib);

    let res_133 = inc(ib);
    let res_134 = inc(ib);
    let label_137 = inc(ib);
    let label_136 = inc(ib);
    let label_144 = inc(ib);

    let res_138 = inc(ib);
    let res_139 = inc(ib);
    let res_140 = inc(ib);
    let res_141 = inc(ib);
    let res_142 = inc(ib);
    let res_143 = inc(ib);

    let res_145 = inc(ib);
    let res_146 = inc(ib);
    let res_147 = inc(ib);
    let res_148 = inc(ib);
    let res_149 = inc(ib);
    let res_150 = inc(ib);
    let res_151 = inc(ib);

    let res_152 = inc(ib);
    let res_153 = inc(ib);
    let res_154 = inc(ib);
    let res_155 = inc(ib);

    let res_156 = inc(ib);
    let res_157 = inc(ib);
    let res_158 = inc(ib);
    let res_159 = inc(ib);
    let res_160 = inc(ib);
    let res_161 = inc(ib);

    #[rustfmt::skip]
    let spv = vec![
        encode_word(5, SPV_INSTRUCTION_OP_FUNCTION),
            v3int_id, func_id, SPV_FUNCTION_CONTROL_INLINE, function_type,
        encode_word(3, SPV_INSTRUCTION_OP_FUNCTION_PARAMETER),
            ptr_v3int_id, r,
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_12,
            
        encode_word(4, SPV_INSTRUCTION_OP_VARIABLE),
            ptr_v3int_id, a, SPV_STORAGE_CLASS_FUNCTION,
        encode_word(4, SPV_INSTRUCTION_OP_VARIABLE),
            ptr_bool_id, x, SPV_STORAGE_CLASS_FUNCTION,
        encode_word(4, SPV_INSTRUCTION_OP_VARIABLE),
            ptr_bool_id, y, SPV_STORAGE_CLASS_FUNCTION,
        encode_word(4, SPV_INSTRUCTION_OP_VARIABLE),
            ptr_int_id, face, SPV_STORAGE_CLASS_FUNCTION,
        encode_word(4, SPV_INSTRUCTION_OP_VARIABLE),
            ptr_int_id, var_53, SPV_STORAGE_CLASS_FUNCTION,
        encode_word(4, SPV_INSTRUCTION_OP_VARIABLE),
            ptr_int_id, var_64, SPV_STORAGE_CLASS_FUNCTION,
        encode_word(4, SPV_INSTRUCTION_OP_VARIABLE),
            ptr_v2int_id, st, SPV_STORAGE_CLASS_FUNCTION,
        encode_word(4, SPV_INSTRUCTION_OP_VARIABLE),
            ptr_v2int_id, var_87, SPV_STORAGE_CLASS_FUNCTION,
        encode_word(4, SPV_INSTRUCTION_OP_VARIABLE),
            ptr_v2int_id, var_100, SPV_STORAGE_CLASS_FUNCTION,
        encode_word(4, SPV_INSTRUCTION_OP_VARIABLE),
            ptr_v2int_id, var_112, SPV_STORAGE_CLASS_FUNCTION,
        encode_word(4, SPV_INSTRUCTION_OP_VARIABLE),
            ptr_v2int_id, var_123, SPV_STORAGE_CLASS_FUNCTION,
        encode_word(4, SPV_INSTRUCTION_OP_VARIABLE),
            ptr_v2int_id, var_135, SPV_STORAGE_CLASS_FUNCTION,
            
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            v3int_id, res_14, r,
        encode_word(6, SPV_INSTRUCTION_OP_EXT_INST),
            v3int_id, res_15, glsl_std_id, SPV_GLSL_STD_INSTRUCTION_SABS, res_14,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            a, res_15,
            
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_22, a, int_0,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_23, res_22,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_25, a, int_1,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_26, res_25,
        encode_word(5, SPV_INSTRUCTION_OP_S_GREATER_THAN_EQUAL),
            bool_id, res_27, res_23, res_26,
        encode_word(3, SPV_INSTRUCTION_OP_SELECTION_MERGE),
            label_29, 0, 
        encode_word(4, SPV_INSTRUCTION_OP_BRANCH_CONDITIONAL),
            res_27, label_28, label_29,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_28,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_30, a, int_0,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_31, res_30,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_33, a, int_2,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_34, res_33,
        encode_word(5, SPV_INSTRUCTION_OP_S_GREATER_THAN_EQUAL),
            bool_id, res_35, res_31, res_34,
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
            label_29,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_29,
        encode_word(7, SPV_INSTRUCTION_OP_PHI),
            bool_id, res_36, res_27, label_12, res_35, label_28,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            x, res_36,
            
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_38, a, int_1,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_39, res_38,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_40, a, int_0,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_41, res_40,
        encode_word(5, SPV_INSTRUCTION_OP_S_GREATER_THAN),
            bool_id, res_42, res_39, res_41,
        encode_word(3, SPV_INSTRUCTION_OP_SELECTION_MERGE),
            label_44, 0,
        encode_word(4, SPV_INSTRUCTION_OP_BRANCH_CONDITIONAL),
            res_42, label_43, label_44,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_43,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_45, a, int_1,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_46, res_45,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_47, a, int_2,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_48, res_47,
        encode_word(5, SPV_INSTRUCTION_OP_S_GREATER_THAN_EQUAL),
            bool_id, res_49, res_46, res_48,
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
            label_44,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_44,
        encode_word(7, SPV_INSTRUCTION_OP_PHI),
            bool_id, res_50, res_42, label_29, res_49, label_43,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            y, res_50,
            
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            bool_id, res_52, x,
        encode_word(3, SPV_INSTRUCTION_OP_SELECTION_MERGE),
            label_55, 0,
        encode_word(4, SPV_INSTRUCTION_OP_BRANCH_CONDITIONAL),
            res_52, label_54, label_62,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_54,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_56, r, int_0,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_57, res_56,
        encode_word(5, SPV_INSTRUCTION_OP_S_GREATER_THAN),
            bool_id, res_59, res_57, int_0,
        encode_word(6, SPV_INSTRUCTION_OP_SELECT),
            int_id, res_61, res_59, int_0, int_1,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            var_53, res_61,
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
            label_55,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_62,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            bool_id, res_63, y,
        encode_word(3, SPV_INSTRUCTION_OP_SELECTION_MERGE),
            label_66, 0,
        encode_word(4, SPV_INSTRUCTION_OP_BRANCH_CONDITIONAL),
            res_63, label_65, label_73,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_65,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_67, r, int_1,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_68, res_67,
        encode_word(5, SPV_INSTRUCTION_OP_S_GREATER_THAN),
            bool_id, res_69, res_68, int_0,
        encode_word(6, SPV_INSTRUCTION_OP_SELECT),
            int_id, res_72, res_69, int_2, int_3,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            var_64, res_72,
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
            label_66,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_73,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_74, r, int_2,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_75, res_74,
        encode_word(5, SPV_INSTRUCTION_OP_S_GREATER_THAN),
            bool_id, res_76, res_75, int_0,
        encode_word(6, SPV_INSTRUCTION_OP_SELECT),
            int_id, res_79, res_76, int_4, int_5,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            var_64, res_79,
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
            label_66,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_66,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_80, var_64,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            var_53, res_80,
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
            label_55,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_55,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_81, var_53,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            face, res_81,
            
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_85, face,
        encode_word(5, SPV_INSTRUCTION_OP_I_EQUAL),
            bool_id, res_86, res_85, int_0,
        encode_word(3, SPV_INSTRUCTION_OP_SELECTION_MERGE),
            label_89, 0,
        encode_word(4, SPV_INSTRUCTION_OP_BRANCH_CONDITIONAL),
            res_86, label_88, label_97,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_88,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_90, r, int_2,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_91, res_90,
        encode_word(4, SPV_INSTRUCTION_OP_S_NEGATE),
            int_id, res_92, res_91,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_93, r, int_1,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_94, res_93,
        encode_word(4, SPV_INSTRUCTION_OP_S_NEGATE),
            int_id, res_95, res_94,
        encode_word(5, SPV_INSTRUCTION_OP_COMPOSITE_CONSTRUCT),
            v2int_id, res_96, res_92, res_95,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            var_87, res_96,
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
            label_89,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_97,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_98, face,
        encode_word(5, SPV_INSTRUCTION_OP_I_EQUAL),
            bool_id, res_99, res_98, int_1,
        encode_word(3, SPV_INSTRUCTION_OP_SELECTION_MERGE),
            label_102, 0,
        encode_word(4, SPV_INSTRUCTION_OP_BRANCH_CONDITIONAL),
            res_99, label_101, label_109,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_101,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_103, r, int_2,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_104, res_103,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_105, r, int_1,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_106, res_105,
        encode_word(4, SPV_INSTRUCTION_OP_S_NEGATE),
            int_id, res_107, res_106,
        encode_word(5, SPV_INSTRUCTION_OP_COMPOSITE_CONSTRUCT),
            v2int_id, res_108, res_104, res_107,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            var_100, res_108,
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
            label_102,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_109,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_110, face,
        encode_word(5, SPV_INSTRUCTION_OP_I_EQUAL),
            bool_id, res_111, res_110, int_2,
        encode_word(3, SPV_INSTRUCTION_OP_SELECTION_MERGE),
            label_114, 0,
        encode_word(4, SPV_INSTRUCTION_OP_BRANCH_CONDITIONAL),
            res_111, label_113, label_120,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_113,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_115, r, int_0,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_116, res_115,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_117, r, int_2,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_118, res_117,
        encode_word(5, SPV_INSTRUCTION_OP_COMPOSITE_CONSTRUCT),
            v2int_id, res_119, res_116, res_118,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            var_112, res_119,
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
            label_114,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_120,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_121, face,
        encode_word(5, SPV_INSTRUCTION_OP_I_EQUAL),
            bool_id, res_122, res_121, int_3,
        encode_word(3, SPV_INSTRUCTION_OP_SELECTION_MERGE),
            label_125, 0,
        encode_word(4, SPV_INSTRUCTION_OP_BRANCH_CONDITIONAL),
            res_122, label_124, label_132,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_124,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_126, r, int_0,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_127, res_126,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_128, r, int_2,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_129, res_128,
        encode_word(4, SPV_INSTRUCTION_OP_S_NEGATE),
            int_id, res_130, res_129,
        encode_word(5, SPV_INSTRUCTION_OP_COMPOSITE_CONSTRUCT),
            v2int_id, res_131, res_127, res_130,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            var_123, res_131,
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
            label_125,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_132,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_133, face,
        encode_word(5, SPV_INSTRUCTION_OP_I_EQUAL),
            bool_id, res_134, res_133, int_4,
        encode_word(3, SPV_INSTRUCTION_OP_SELECTION_MERGE),
            label_137, 0,
        encode_word(4, SPV_INSTRUCTION_OP_BRANCH_CONDITIONAL),
            res_134, label_136, label_144,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_136,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_138, r, int_0,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_139, res_138,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_140, r, int_1,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_141, res_140,
        encode_word(4, SPV_INSTRUCTION_OP_S_NEGATE),
            int_id, res_142, res_141,
        encode_word(5, SPV_INSTRUCTION_OP_COMPOSITE_CONSTRUCT),
            v2int_id, res_143, res_139, res_142,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            var_135, res_143,
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
            label_137,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_144,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_145, r, int_0,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_146, res_145,
        encode_word(4, SPV_INSTRUCTION_OP_S_NEGATE),
            int_id, res_147, res_146,
        encode_word(5, SPV_INSTRUCTION_OP_ACCESS_CHAIN),
            ptr_int_id, res_148, r, int_1,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_149, res_148,
        encode_word(4, SPV_INSTRUCTION_OP_S_NEGATE),
            int_id, res_150, res_149,
        encode_word(5, SPV_INSTRUCTION_OP_COMPOSITE_CONSTRUCT),
            v2int_id, res_151, res_147, res_150,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            var_135, res_151,
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
            label_137,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_137,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            v2int_id, res_152, var_135,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            var_123, res_152,
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
            label_125,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_125,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            v2int_id, res_153, var_123,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            var_112, res_153,
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
            label_114,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_114,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            v2int_id, res_154, var_112,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            var_100, res_154,
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
            label_102,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_102,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            v2int_id, res_155, var_100,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            var_87, res_155,
        encode_word(2, SPV_INSTRUCTION_OP_BRANCH),
            label_89,
            
        encode_word(2, SPV_INSTRUCTION_OP_LABEL),
            label_89,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            v2int_id, res_156, var_87,
        encode_word(3, SPV_INSTRUCTION_OP_STORE),
            st, res_156,
            
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            v2int_id, res_157, st,
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
            int_id, res_158, face,
        encode_word(5, SPV_INSTRUCTION_OP_COMPOSITE_EXTRACT),
            int_id, res_159, res_157, 0,
        encode_word(5, SPV_INSTRUCTION_OP_COMPOSITE_EXTRACT),
            int_id, res_160, res_157, 1,
        encode_word(6, SPV_INSTRUCTION_OP_COMPOSITE_CONSTRUCT),
            v3int_id, res_161, res_159, res_160, res_158,
        encode_word(2, SPV_INSTRUCTION_OP_RETURN_VALUE),
            res_161,
        encode_word(1, SPV_INSTRUCTION_OP_FUNCTION_END),
    ];

    (func_id, spv)
}
