use super::*;

#[test]
fn test_mirrorpatch() {
    let vert_spv = include_bytes!("./mirrorpatch/test1.vert.spv");
    let frag_spv = include_bytes!("./mirrorpatch/test1.frag.spv");

    let vert_spv = u8_slice_to_u32_vec(vert_spv);
    let frag_spv = u8_slice_to_u32_vec(frag_spv);

    let mut left_map = None;
    let mut right_map = None;

    let vert_spv = combimgsampsplitter(&vert_spv, &mut left_map).unwrap();
    let frag_spv = combimgsampsplitter(&frag_spv, &mut right_map).unwrap();

    let vert_spv = drefsplitter(&vert_spv, &mut left_map).unwrap();
    let frag_spv = drefsplitter(&frag_spv, &mut right_map).unwrap();

    try_spv_to_wgsl(&vert_spv, DO_ALL);
    try_spv_to_wgsl(&frag_spv, DO_ALL);

    let (new_l_spv, new_r_spv) =
        mirrorpatch(&vert_spv, &mut left_map, &frag_spv, &mut right_map).unwrap();

    assert_eq!(left_map, right_map);
    assert!(new_r_spv.is_none());
    let l_spv = new_l_spv.unwrap();
    let r_spv = frag_spv;

    try_spv_to_wgsl(&l_spv, DO_ALL);
    try_spv_to_wgsl(&r_spv, DO_ALL);
}
