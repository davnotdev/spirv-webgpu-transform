use super::*;

// Opaque types cannot be operated on in the same way as non-opaque types.
// We need tools to trace the instruction chain up to the point an opaque type becomes a non-opaque.
//
// We care about OpTypeSampler and OpTypeImage, or more specifically, textures, storage textures,
// and samplers.
//
// My notes on the instruction structure:
//
// ```
// Textures:
// - OpLoad
//     - OpImageFetch
//     - OpImageGather
//     - OpImageDrefGather
//     - OpSampledImage (DAG NODE)
//         - OpImageSampleImplicitLod
//         - OpImageSampleExplicitLod
//         - OpImageSampleDrefImplicitLod
//         - OpImageSampleDrefExplicitLod
//         - OpImageSampleProjImplicitLod
//         - OpImageSampleProjExplicitLod
//         - OpImageSampleProjDrefImplicitLod
//         - OpImageSampleProjDrefExplicitLod
//         - (SparseResidency Capability)
//             - OpImageSparseSample*
//     OpImageGather
//     OpImageDrefGather
//     - (ImageQuery Capability)
//         - OpImageQuerySizeLod
//         - OpImageQuerySize
//         - OpImageQueryLevels
//         - OpImageQuerySamples
//         - OpImageQueryLod
//         - OpImageQueryFormat
//         - OpImageQueryOrder
//     - (SparseResidency Capability)
//         - OpImageSparseFetch
//         - OpImageSparseGather
//         - OpImageSparseDrefGather
//
// Storage Textures:
// - OpLoad
//     - OpImageRead
//     - OpImageWrite
//     - OpImageSparseRead
//     - OpImageTexelPointer
//     - (ImageQuery Capability)
//         - OpImageQuerySizeLod
//         - OpImageQuerySize
//         - OpImageQueryLevels
//         - OpImageQuerySamples
//         - OpImageQueryLod
//         - OpImageQueryFormat
//         - OpImageQueryOrder
//
// Samplers:
// - OpLoad
//     - OpSampledImage (DAG NODE)
// ```
//
// We can build a DAG for the instruction chains, but if we handle sampler's `OpSampledImage`
// separately, we can get away with a tree, or just a `struct`
//

pub enum OpaqueLoadTrace {
    RawImage(RawImageOp),
    RawStorage(StorageTextureOp),
    Sampled(SampledImageOp),
}

pub enum RawImageOp {
    Fetch(usize),
    Gather(usize),
    DrefGather(usize),
    // TODO: Image Query Capability
}

pub enum SampledImageOp {
    SampleImplicitLod(usize),
    SampleExplicitLod(usize),
    SampleDrefImplicitLod(usize),
    SampleDrefExplicitLod(usize),
    SampleProjImplicitLod(usize),
    SampleProjExplicitLod(usize),
    SampleProjDrefImplicitLod(usize),
    SampleProjDrefExplicitLod(usize),
    Gather(usize),
    DrefGather(usize),
    // TODO: Image Query Capability
    // TODO: Sparse Residency Capability
}

pub enum StorageTextureOp {
    Read(usize),
    Write(usize),
    SparseRead(usize),
    TexelPointer(usize),
    // TODO: Image Query Capability
}

// Generally, spv[idx + 1] => result type, spv[idx + 2] => result, spv[idx + 3] => image / sampled image
pub fn trace_loaded_opaques(spv: &[u32], load_idxs: &[usize]) -> Vec<OpaqueLoadTrace> {
    let mut op_sampled_image_idxs = vec![];
    let mut raw_image_op_idxs: Vec<(u16, usize)> = vec![];
    let mut sampled_image_op_idxs: Vec<(u16, usize)> = vec![];
    let mut storage_op_idxs: Vec<(u16, usize)> = vec![];

    let mut spv_idx = 0;
    while spv_idx < spv.len() {
        let op = spv[spv_idx];
        let word_count = hiword(op) as usize;
        let instruction = loword(op);

        match instruction {
            SPV_INSTRUCTION_OP_SAMPLED_IMAGE => op_sampled_image_idxs.push(spv_idx),
            SPV_INSTRUCTION_OP_IMAGE_FETCH
            | SPV_INSTRUCTION_OP_IMAGE_GATHER
            | SPV_INSTRUCTION_OP_IMAGE_DREF_GATHER => {
                raw_image_op_idxs.push((instruction, spv_idx))
            }
            SPV_INSTRUCTION_OP_IMAGE_SAMPLE_IMPLICIT_LOD
            | SPV_INSTRUCTION_OP_IMAGE_SAMPLE_EXPLICIT_LOD
            | SPV_INSTRUCTION_OP_IMAGE_SAMPLE_DREF_IMPLICIT_LOD
            | SPV_INSTRUCTION_OP_IMAGE_SAMPLE_DREF_EXPLICIT_LOD
            | SPV_INSTRUCTION_OP_IMAGE_SAMPLE_PROJ_IMPLICIT_LOD
            | SPV_INSTRUCTION_OP_IMAGE_SAMPLE_PROJ_EXPLICIT_LOD
            | SPV_INSTRUCTION_OP_IMAGE_SAMPLE_PROJ_DREF_IMPLICIT_LOD
            | SPV_INSTRUCTION_OP_IMAGE_SAMPLE_PROJ_DREF_EXPLICIT_LOD => {
                sampled_image_op_idxs.push((instruction, spv_idx))
            }
            SPV_INSTRUCTION_OP_IMAGE_READ
            | SPV_INSTRUCTION_OP_IMAGE_WRITE
            | SPV_INSTRUCTION_OP_IMAGE_SPARSE_READ
            | SPV_INSTRUCTION_OP_IMAGE_TEXEL_POINTER => {
                storage_op_idxs.push((instruction, spv_idx))
            }
            _ => {}
        }

        spv_idx += word_count;
    }

    let load_result_ids: Vec<u32> = load_idxs.iter().map(|&idx| spv[idx + 2]).collect();

    let mut results = vec![];

    for &(instruction, idx) in &raw_image_op_idxs {
        if load_result_ids.contains(&spv[idx + 3]) {
            let op = match instruction {
                SPV_INSTRUCTION_OP_IMAGE_FETCH => RawImageOp::Fetch(idx),
                SPV_INSTRUCTION_OP_IMAGE_GATHER => RawImageOp::Gather(idx),
                SPV_INSTRUCTION_OP_IMAGE_DREF_GATHER => RawImageOp::DrefGather(idx),
                _ => unreachable!(),
            };
            results.push(OpaqueLoadTrace::RawImage(op));
        }
    }

    for &(instruction, idx) in &storage_op_idxs {
        let image_id = if instruction == SPV_INSTRUCTION_OP_IMAGE_WRITE {
            spv[idx + 1]
        } else {
            spv[idx + 3]
        };
        if load_result_ids.contains(&image_id) {
            let op = match instruction {
                SPV_INSTRUCTION_OP_IMAGE_READ => StorageTextureOp::Read(idx),
                SPV_INSTRUCTION_OP_IMAGE_WRITE => StorageTextureOp::Write(idx),
                SPV_INSTRUCTION_OP_IMAGE_SPARSE_READ => StorageTextureOp::SparseRead(idx),
                SPV_INSTRUCTION_OP_IMAGE_TEXEL_POINTER => StorageTextureOp::TexelPointer(idx),
                _ => unreachable!(),
            };
            results.push(OpaqueLoadTrace::RawStorage(op));
        }
    }

    let sampled_image_result_ids: Vec<u32> = op_sampled_image_idxs
        .iter()
        .filter(|&&idx| load_result_ids.contains(&spv[idx + 3]))
        .map(|&idx| spv[idx + 2])
        .collect();

    for &(instruction, idx) in &sampled_image_op_idxs {
        if sampled_image_result_ids.contains(&spv[idx + 3]) {
            let op = match instruction {
                SPV_INSTRUCTION_OP_IMAGE_SAMPLE_IMPLICIT_LOD => {
                    SampledImageOp::SampleImplicitLod(idx)
                }
                SPV_INSTRUCTION_OP_IMAGE_SAMPLE_EXPLICIT_LOD => {
                    SampledImageOp::SampleExplicitLod(idx)
                }
                SPV_INSTRUCTION_OP_IMAGE_SAMPLE_DREF_IMPLICIT_LOD => {
                    SampledImageOp::SampleDrefImplicitLod(idx)
                }
                SPV_INSTRUCTION_OP_IMAGE_SAMPLE_DREF_EXPLICIT_LOD => {
                    SampledImageOp::SampleDrefExplicitLod(idx)
                }
                SPV_INSTRUCTION_OP_IMAGE_SAMPLE_PROJ_IMPLICIT_LOD => {
                    SampledImageOp::SampleProjImplicitLod(idx)
                }
                SPV_INSTRUCTION_OP_IMAGE_SAMPLE_PROJ_EXPLICIT_LOD => {
                    SampledImageOp::SampleProjExplicitLod(idx)
                }
                SPV_INSTRUCTION_OP_IMAGE_SAMPLE_PROJ_DREF_IMPLICIT_LOD => {
                    SampledImageOp::SampleProjDrefImplicitLod(idx)
                }
                SPV_INSTRUCTION_OP_IMAGE_SAMPLE_PROJ_DREF_EXPLICIT_LOD => {
                    SampledImageOp::SampleProjDrefExplicitLod(idx)
                }
                _ => unreachable!(),
            };
            results.push(OpaqueLoadTrace::Sampled(op));
        }
    }

    for &(instruction, idx) in &raw_image_op_idxs {
        if sampled_image_result_ids.contains(&spv[idx + 3]) {
            let op = match instruction {
                SPV_INSTRUCTION_OP_IMAGE_GATHER => SampledImageOp::Gather(idx),
                SPV_INSTRUCTION_OP_IMAGE_DREF_GATHER => SampledImageOp::DrefGather(idx),
                _ => continue,
            };
            results.push(OpaqueLoadTrace::Sampled(op));
        }
    }

    results
}

#[test]
fn raw_image_fetch() {
    let spv: &[u32] = &[
        // %20 = OpLoad %10 %30
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
        10,
        20,
        30,
        // %21 = OpImageFetch %11 %20 %40
        encode_word(5, SPV_INSTRUCTION_OP_IMAGE_FETCH),
        11,
        21,
        20,
        40,
    ];
    let traces = trace_loaded_opaques(spv, &[0]);
    assert_eq!(traces.len(), 1);
    assert!(matches!(
        traces[0],
        OpaqueLoadTrace::RawImage(RawImageOp::Fetch(4))
    ));
}

#[test]
fn sampled_image_implicit_lod() {
    let spv: &[u32] = &[
        // %20 = OpLoad %10 %30
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
        10,
        20,
        30,
        // %21 = OpLoad %11 %31
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
        11,
        21,
        31,
        // %22 = OpSampledImage %12 %20 %21
        encode_word(5, SPV_INSTRUCTION_OP_SAMPLED_IMAGE),
        12,
        22,
        20,
        21,
        // %23 = OpImageSampleImplicitLod %13 %22 %40
        encode_word(5, SPV_INSTRUCTION_OP_IMAGE_SAMPLE_IMPLICIT_LOD),
        13,
        23,
        22,
        40,
    ];
    let traces = trace_loaded_opaques(spv, &[0]);
    assert_eq!(traces.len(), 1);
    assert!(matches!(
        traces[0],
        OpaqueLoadTrace::Sampled(SampledImageOp::SampleImplicitLod(13))
    ));
}

#[test]
fn storage_image_write() {
    let spv: &[u32] = &[
        // %20 = OpLoad %10 %30
        encode_word(4, SPV_INSTRUCTION_OP_LOAD),
        10,
        20,
        30,
        // OpImageWrite %20 %40 %50
        encode_word(4, SPV_INSTRUCTION_OP_IMAGE_WRITE),
        20,
        40,
        50,
    ];
    let traces = trace_loaded_opaques(spv, &[0]);
    assert_eq!(traces.len(), 1);
    assert!(matches!(
        traces[0],
        OpaqueLoadTrace::RawStorage(StorageTextureOp::Write(4))
    ));
}
