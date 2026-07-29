use super::*;

// Q: Hey what happens when you stack corrections?
// A: I don't want to think about it... I will start thinking after a refactor...

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum CorrectionType {
    /// A combined image sampler has been split, a new `sampler` object should be inserted.
    SplitCombined,
    /// A mixed depth texture / sampler has been duplicated, insert the same object again with a `Regular` bind type.
    SplitDrefRegular,
    /// A mixed depth texture / sampler has been duplicated, insert the same object again with a
    /// `Comparison` bind type.
    SplitDrefComparison,
    /// A storage cube texture has been converted into a storage texture 2D array, change the dimension.
    ConvertStorageCube,
    /// A binding array has been split into new variables. Insert the same resource again.
    /// For an `N` sized array, expect `N-1` entries.
    SplitBindingArray,
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ImmediatesSetMode {
    /// Place the immediates binding at certain set.
    #[default]
    Absolute,
    /// Place the immediates binding at the max set or up to a certain set.
    MaxUpTo,
}

/// Lookup a set and a binding for a list of [`CorrectionType`].
/// In order, insert a new variable for each, see [`CorrectionType`] for what type of object should
/// be inserted for each variant.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CorrectionMap {
    pub sets: Option<HashMap<u32, CorrectionSet>>,
    /// Represents both an input/output where the immediate uniforms should/is written to.
    /// Control the set immediates should be written to by setting before patching.
    /// If this is [`None`], this becomes the max set plus one.
    pub immediates_set: Option<u32>,
    /// Works as an input setting controlling the behaviour of `immediates_set`.
    /// This is valuable when dealing with WebGPU's `maxBindGroup` especially if your sets
    /// previously followed the maximum.
    pub immediates_set_mode: Option<ImmediatesSetMode>,
}
