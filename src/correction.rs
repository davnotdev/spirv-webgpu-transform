use super::*;

// Q: Hey what happens when you stack corrections?
// A: I don't want to think about it... I will start thinking after a refactor...

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
    /// A storage cube texture has been converted into a storage texture 2D array (change dimension).
    ConvertStorageCubeTexture = 3,
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

/// Lookup a set and a binding for a list of [`CorrectionType`].
/// In order, insert a new variable for each, see [`CorrectionType`] for what type of object should
/// be inserted for each variant.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CorrectionMap {
    pub sets: HashMap<u32, CorrectionSet>,
}
