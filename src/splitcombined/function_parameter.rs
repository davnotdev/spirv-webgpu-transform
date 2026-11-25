use super::*;

pub struct FunctionParameterIn<'a> {
    pub spv: &'a [u32],
    pub instruction_bound: &'a mut u32,
    pub instruction_inserts: &'a mut Vec<InstructionInsert>,

    pub op_type_pointer_sampler_res_id: u32,
    pub op_function_parameter_idxs: &'a [usize],

    pub tp_res: &'a HashMap<TypePointerId, TypePointerOut>,
}

pub struct FunctionParameterOut {
    pub sampler_parameter_res_id: u32,
    pub underlying_image_id: u32,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct FunctionParameterId(pub u32);

pub fn function_parameter(
    fp_in: FunctionParameterIn,
) -> HashMap<FunctionParameterId, FunctionParameterOut> {
    let FunctionParameterIn {
        spv,
        instruction_bound,
        instruction_inserts,
        op_type_pointer_sampler_res_id,
        op_function_parameter_idxs,
        tp_res,
    } = fp_in;

    let mut parameter_res_ids = HashMap::new();

    op_function_parameter_idxs
        .iter()
        .filter_map(|&fp_idx| {
            // - Find all OpFunctionParameters that use a combimg OpTypePointer
            tp_res.get(&TypePointerId(spv[fp_idx + 1])).map(
                |&TypePointerOut {
                     underlying_image_id,
                 }| { (fp_idx, spv[fp_idx + 2], underlying_image_id) },
            )
        })
        .for_each(|(fp_idx, image_parameter_res_id, underlying_image_id)| {
            // - Append a new sampler OpFunctionParameter
            let sampler_parameter_res_id = *instruction_bound;
            *instruction_bound += 1;
            instruction_inserts.push(InstructionInsert {
                previous_spv_idx: fp_idx,
                instruction: vec![
                    encode_word(3, SPV_INSTRUCTION_OP_FUNCTION_PARAMETER),
                    op_type_pointer_sampler_res_id,
                    sampler_parameter_res_id,
                ],
            });
            parameter_res_ids.insert(
                FunctionParameterId(image_parameter_res_id),
                FunctionParameterOut {
                    sampler_parameter_res_id,
                    underlying_image_id,
                },
            );
        });

    parameter_res_ids
}
