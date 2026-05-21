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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpaqueLoadTrace {
    pub load_idx: usize,
    pub next: OpaqueImageOp,
}

impl OpaqueLoadTrace {
    pub fn last_result_id(&self) -> usize {
        match self.next {
            OpaqueImageOp::RawImage(raw_image_op) => raw_image_op.result_idx(),
            OpaqueImageOp::RawStorage(storage_texture_op) => storage_texture_op.result_idx(),
            OpaqueImageOp::Sampled(sampled_image_op) => sampled_image_op.next.result_idx(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpaqueImageOp {
    RawImage(RawImageOp),
    RawStorage(StorageTextureOp),
    Sampled(SampledImageOp),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawImageOp {
    Fetch(usize),
    Gather(usize),
    DrefGather(usize),
    // TODO: Image Query Capability
}

impl RawImageOp {
    pub fn result_idx(&self) -> usize {
        match self {
            RawImageOp::Fetch(i) | RawImageOp::Gather(i) | RawImageOp::DrefGather(i) => *i,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SampledImageOp {
    pub idx: usize,
    pub next: SampledImageVariant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampledImageVariant {
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

impl SampledImageVariant {
    pub fn result_idx(&self) -> usize {
        match self {
            SampledImageVariant::SampleImplicitLod(i)
            | SampledImageVariant::SampleExplicitLod(i)
            | SampledImageVariant::SampleDrefImplicitLod(i)
            | SampledImageVariant::SampleDrefExplicitLod(i)
            | SampledImageVariant::SampleProjImplicitLod(i)
            | SampledImageVariant::SampleProjExplicitLod(i)
            | SampledImageVariant::SampleProjDrefImplicitLod(i)
            | SampledImageVariant::SampleProjDrefExplicitLod(i)
            | SampledImageVariant::Gather(i)
            | SampledImageVariant::DrefGather(i) => *i,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageTextureOp {
    Read(usize),
    Write(usize),
    SparseRead(usize),
    TexelPointer(usize),
    // TODO: Image Query Capability
}

impl StorageTextureOp {
    // OpImageWrite has no result; all other storage ops do.
    pub fn result_idx(&self) -> usize {
        match self {
            StorageTextureOp::Read(i)
            | StorageTextureOp::SparseRead(i)
            | StorageTextureOp::TexelPointer(i)
            | StorageTextureOp::Write(i) => *i,
        }
    }
}

// Generally, spv[idx + 1] => result type, spv[idx + 2] => result, spv[idx + 3] => image / sampled image
pub fn trace_loaded_opaques(spv: &[u32], load_idxs: &[usize]) -> Vec<OpaqueLoadTrace> {
    // TODO: Memoize, we can do better than this.
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

    let load_result_ids = load_idxs
        .iter()
        .map(|&idx| (spv[idx + 2], idx))
        .collect::<HashMap<_, _>>();

    let mut results = vec![];

    for &(instruction, idx) in &raw_image_op_idxs {
        let loaded_image_id = spv[idx + 3];
        if let Some(&load_idx) = load_result_ids.get(&loaded_image_id) {
            let op = match instruction {
                SPV_INSTRUCTION_OP_IMAGE_FETCH => RawImageOp::Fetch(idx),
                SPV_INSTRUCTION_OP_IMAGE_GATHER => RawImageOp::Gather(idx),
                SPV_INSTRUCTION_OP_IMAGE_DREF_GATHER => RawImageOp::DrefGather(idx),
                _ => unreachable!(),
            };
            results.push(OpaqueLoadTrace {
                load_idx,
                next: OpaqueImageOp::RawImage(op),
            });
        }
    }

    for &(instruction, idx) in &storage_op_idxs {
        let image_id = if instruction == SPV_INSTRUCTION_OP_IMAGE_WRITE {
            spv[idx + 1]
        } else {
            spv[idx + 3]
        };
        if let Some(&load_idx) = load_result_ids.get(&image_id) {
            let op = match instruction {
                SPV_INSTRUCTION_OP_IMAGE_READ => StorageTextureOp::Read(idx),
                SPV_INSTRUCTION_OP_IMAGE_WRITE => StorageTextureOp::Write(idx),
                SPV_INSTRUCTION_OP_IMAGE_SPARSE_READ => StorageTextureOp::SparseRead(idx),
                SPV_INSTRUCTION_OP_IMAGE_TEXEL_POINTER => StorageTextureOp::TexelPointer(idx),
                _ => unreachable!(),
            };
            results.push(OpaqueLoadTrace {
                load_idx,
                next: OpaqueImageOp::RawStorage(op),
            });
        }
    }

    // (result_id, sampled_image_idx) for OpSampledImage nodes rooted at our loads
    let sampled_image_entries = op_sampled_image_idxs
        .iter()
        .filter_map(|&idx| {
            load_result_ids
                .get(&spv[idx + 3])
                .copied()
                .map(|load_idx| (idx, load_idx))
        })
        .map(|(idx, load_idx)| (spv[idx + 2], idx, load_idx))
        .collect::<Vec<_>>();

    for &(instruction, idx) in sampled_image_op_idxs.iter() {
        let Some(&(_, si_idx, load_idx)) =
            sampled_image_entries.iter().find(|(result_id, _, _)| {
                let loaded_image_id = spv[idx + 3];
                *result_id == loaded_image_id
            })
        else {
            continue;
        };
        let variant = match instruction {
            SPV_INSTRUCTION_OP_IMAGE_SAMPLE_IMPLICIT_LOD => {
                SampledImageVariant::SampleImplicitLod(idx)
            }
            SPV_INSTRUCTION_OP_IMAGE_SAMPLE_EXPLICIT_LOD => {
                SampledImageVariant::SampleExplicitLod(idx)
            }
            SPV_INSTRUCTION_OP_IMAGE_SAMPLE_DREF_IMPLICIT_LOD => {
                SampledImageVariant::SampleDrefImplicitLod(idx)
            }
            SPV_INSTRUCTION_OP_IMAGE_SAMPLE_DREF_EXPLICIT_LOD => {
                SampledImageVariant::SampleDrefExplicitLod(idx)
            }
            SPV_INSTRUCTION_OP_IMAGE_SAMPLE_PROJ_IMPLICIT_LOD => {
                SampledImageVariant::SampleProjImplicitLod(idx)
            }
            SPV_INSTRUCTION_OP_IMAGE_SAMPLE_PROJ_EXPLICIT_LOD => {
                SampledImageVariant::SampleProjExplicitLod(idx)
            }
            SPV_INSTRUCTION_OP_IMAGE_SAMPLE_PROJ_DREF_IMPLICIT_LOD => {
                SampledImageVariant::SampleProjDrefImplicitLod(idx)
            }
            SPV_INSTRUCTION_OP_IMAGE_SAMPLE_PROJ_DREF_EXPLICIT_LOD => {
                SampledImageVariant::SampleProjDrefExplicitLod(idx)
            }
            _ => unreachable!(),
        };
        results.push(OpaqueLoadTrace {
            load_idx,
            next: OpaqueImageOp::Sampled(SampledImageOp {
                idx: si_idx,
                next: variant,
            }),
        });
    }

    for &(instruction, idx) in &raw_image_op_idxs {
        let Some(&(_, si_idx, load_idx)) =
            sampled_image_entries.iter().find(|(result_id, _, _)| {
                let loaded_image_id = spv[idx + 3];
                *result_id == loaded_image_id
            })
        else {
            continue;
        };
        let variant = match instruction {
            SPV_INSTRUCTION_OP_IMAGE_GATHER => SampledImageVariant::Gather(idx),
            SPV_INSTRUCTION_OP_IMAGE_DREF_GATHER => SampledImageVariant::DrefGather(idx),
            _ => continue,
        };
        results.push(OpaqueLoadTrace {
            load_idx,
            next: OpaqueImageOp::Sampled(SampledImageOp {
                idx: si_idx,
                next: variant,
            }),
        });
    }

    results
}

pub fn reconstruct_opaque_trace_and_overwrite(
    spv: &[u32],
    new_spv: &mut [u32],
    trace: &OpaqueLoadTrace,
) -> Vec<u32> {
    fn take_instruction(spv: &[u32], idx: usize) -> &[u32] {
        let word_count = hiword(spv[idx]) as usize;
        &spv[idx..idx + word_count]
    }

    fn write_nop_instruction(new_spv: &mut [u32], idx: usize) {
        let word_count = hiword(new_spv[idx]) as usize;
        new_spv[idx..idx + word_count].fill(encode_word(1, SPV_INSTRUCTION_OP_NOP));
    }

    let mut out = take_instruction(spv, trace.load_idx).to_vec();
    write_nop_instruction(new_spv, trace.load_idx);

    match &trace.next {
        OpaqueImageOp::RawImage(op) => {
            let op_idx = match op {
                RawImageOp::Fetch(i) | RawImageOp::Gather(i) | RawImageOp::DrefGather(i) => *i,
            };
            out.extend_from_slice(take_instruction(spv, op_idx));
            write_nop_instruction(new_spv, op_idx);
        }
        OpaqueImageOp::RawStorage(op) => {
            let op_idx = match op {
                StorageTextureOp::Read(i)
                | StorageTextureOp::Write(i)
                | StorageTextureOp::SparseRead(i)
                | StorageTextureOp::TexelPointer(i) => *i,
            };
            out.extend_from_slice(take_instruction(spv, op_idx));
            write_nop_instruction(new_spv, op_idx);
        }
        OpaqueImageOp::Sampled(SampledImageOp { idx: si_idx, next }) => {
            out.extend_from_slice(take_instruction(spv, *si_idx));
            write_nop_instruction(new_spv, *si_idx);

            let op_idx = match next {
                SampledImageVariant::SampleImplicitLod(i)
                | SampledImageVariant::SampleExplicitLod(i)
                | SampledImageVariant::SampleDrefImplicitLod(i)
                | SampledImageVariant::SampleDrefExplicitLod(i)
                | SampledImageVariant::SampleProjImplicitLod(i)
                | SampledImageVariant::SampleProjExplicitLod(i)
                | SampledImageVariant::SampleProjDrefImplicitLod(i)
                | SampledImageVariant::SampleProjDrefExplicitLod(i)
                | SampledImageVariant::Gather(i)
                | SampledImageVariant::DrefGather(i) => *i,
            };
            out.extend_from_slice(take_instruction(spv, op_idx));
            write_nop_instruction(new_spv, op_idx);
        }
    }

    out
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
        OpaqueLoadTrace {
            load_idx: 0,
            next: OpaqueImageOp::RawImage(RawImageOp::Fetch(4))
        }
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
        OpaqueLoadTrace {
            load_idx: 0,
            next: OpaqueImageOp::Sampled(SampledImageOp {
                idx: 8,
                next: SampledImageVariant::SampleImplicitLod(13),
            })
        }
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
        OpaqueLoadTrace {
            load_idx: 0,
            next: OpaqueImageOp::RawStorage(StorageTextureOp::Write(4))
        }
    ));
}
