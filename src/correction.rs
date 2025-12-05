use super::*;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CorrectionType {
    /// A combined image sampler has been split, a new `sampler` object should be inserted.
    SplitCombined = 0,
    /// A mixed depth texture / sampler has been duplicated, insert the same object again with a `Regular` bind type.
    SplitDrefRegular = 1,
    /// A mixed depth texture / sampler has been duplicated, insert the same object again with a
    /// `Comparison` bind type.
    SplitDrefComparison = 2,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CorrectionBinding {
    /// In order, what additional bindings have been appended to this one.
    pub corrections: Vec<CorrectionType>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CorrectionSet {
    pub bindings: HashMap<u32, CorrectionBinding>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CorrectionMap {
    pub sets: HashMap<u32, CorrectionSet>,
}
