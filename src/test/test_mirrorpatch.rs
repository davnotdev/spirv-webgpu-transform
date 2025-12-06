use super::*;
use crate::{CorrectionBinding, CorrectionMap};

macro_rules! test_mirrorpatch_with_spv_and_fn {
    ($NAME:ident, $VERT:expr, $FRAG:expr, $ASSERT:expr) => {
        #[test]
        fn $NAME() {
            let vert_spv = include_bytes!($VERT);
            let frag_spv = include_bytes!($FRAG);

            test_basic_mirrorpatch_inner(vert_spv, frag_spv, $ASSERT);
        }
    };
}

type AssertFunction = fn(Option<Vec<u32>>, &CorrectionMap, Option<Vec<u32>>, &CorrectionMap) -> ();

fn test_basic_mirrorpatch_inner(vert_spv: &[u8], frag_spv: &[u8], assert_fn: AssertFunction) {
    let vert_spv = u8_slice_to_u32_vec(vert_spv);
    let frag_spv = u8_slice_to_u32_vec(frag_spv);

    let mut left_map = None;
    let mut right_map = None;

    // let vert_spv = combimgsampsplitter(&vert_spv, &mut left_map).unwrap();
    // let frag_spv = combimgsampsplitter(&frag_spv, &mut right_map).unwrap();

    let vert_spv = drefsplitter(&vert_spv, &mut left_map).unwrap();
    let frag_spv = drefsplitter(&frag_spv, &mut right_map).unwrap();

    try_spv_to_wgsl(&vert_spv, DO_ALL);
    try_spv_to_wgsl(&frag_spv, DO_ALL);

    let (new_l_spv, new_r_spv) =
        mirrorpatch(&vert_spv, &mut left_map, &frag_spv, &mut right_map).unwrap();

    assert_fn(
        new_l_spv,
        left_map.as_ref().unwrap(),
        new_r_spv,
        right_map.as_ref().unwrap(),
    );
}

fn test_mirrorpatch1_assert(
    new_l_spv: Option<Vec<u32>>,
    l: &CorrectionMap,
    new_r_spv: Option<Vec<u32>>,
    r: &CorrectionMap,
) {
    assert_eq!(l, r);

    assert!(new_r_spv.is_none());
    let l_spv = new_l_spv.unwrap();
    try_spv_to_wgsl(&l_spv, DO_ALL);
}

fn test_mirrorpatch2_assert(
    new_l_spv: Option<Vec<u32>>,
    l: &CorrectionMap,
    new_r_spv: Option<Vec<u32>>,
    r: &CorrectionMap,
) {
    // If we add back the missing bindings, l and r should be equal.
    let mut l = l.clone();

    l.sets
        .get_mut(&0)
        .unwrap()
        .bindings
        .insert(1, CorrectionBinding::default());
    l.sets
        .get_mut(&0)
        .unwrap()
        .bindings
        .insert(3, CorrectionBinding::default());

    assert_eq!(&l, r);

    assert!(new_r_spv.is_none());
    let l_spv = new_l_spv.unwrap();
    try_spv_to_wgsl(&l_spv, DO_ALL);
}

test_mirrorpatch_with_spv_and_fn!(
    test_mirrorpatch1,
    "./mirrorpatch/test1.vert.spv",
    "./mirrorpatch/test1.frag.spv",
    test_mirrorpatch1_assert
);
test_mirrorpatch_with_spv_and_fn!(
    test_mirrorpatch2,
    "./mirrorpatch/test2.vert.spv",
    "./mirrorpatch/test2.frag.spv",
    test_mirrorpatch2_assert
);
