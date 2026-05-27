// SPIR-V / Vulkan buffer layout rules.
//
// The conversion this patch performs is std430 -> std140:
// bump array / struct base alignment to 16, and bump `ArrayStride` / `MatrixStride` to >=16.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LayoutRule {
    /// "Standard Storage Buffer Layout"
    /// Push constants use this one.
    #[allow(dead_code)]
    Std430,
    /// "Standard Uniform Buffer Layout"
    /// Arrays and structs have their base alignment rounded up to a multiple of 16.
    Std140,
}

#[derive(Debug, Clone)]
pub(super) struct Type {
    pub id: u32,
    pub kind: TypeKind,
}

#[derive(Debug, Clone)]
pub(super) enum TypeKind {
    Scalar { width_bytes: u32 },
    Vector { component: Box<Type>, count: u32 },
    Matrix { column: Box<Type>, cols: u32 },
    Array { element: Box<Type>, len: u32 },
    Struct { members: Vec<Type> },
}

pub(super) struct StructLayout {
    pub member_offsets: Vec<u32>,
    pub size: u32,
    #[allow(dead_code)]
    pub align: u32,
}

const fn round_up(x: u32, a: u32) -> u32 {
    (x + a - 1) & !(a - 1)
}

pub(super) fn base_align(t: &TypeKind, rule: LayoutRule) -> u32 {
    let inner = match t {
        // §15.6.4: "A scalar has a base alignment equal to its scalar alignment."
        TypeKind::Scalar { width_bytes } => *width_bytes,
        // §15.6.4: vec2 = 2N, vec3 / vec4 = 4N (where N is the scalar alignment of the component type).
        TypeKind::Vector { component, count } => match count {
            2 => 2 * base_align(&component.kind, rule),
            3 | 4 => 4 * base_align(&component.kind, rule),
            n => panic!("Unsupported vector component count: {}", n),
        },
        // §15.6.4: "A column-major matrix has a base alignment equal to the base alignment of the column vector type."
        // RowMajor is handled at MatrixStride emission time; the type itself describes columns.
        TypeKind::Matrix { column, .. } => base_align(&column.kind, rule),
        // §15.6.4: "An array has a base alignment equal to the base alignment of its element type" — modulo the std140 round-up below.
        TypeKind::Array { element, .. } => base_align(&element.kind, rule),
        // §15.6.4: "A structure has a base alignment equal to the largest base alignment of any of its members" — modulo the std140 round-up below.
        TypeKind::Struct { members } => members
            .iter()
            .map(|m| base_align(&m.kind, rule))
            .max()
            .unwrap_or(4),
    };
    // §15.6.4 "Standard Uniform Buffer Layout":
    //   - Array's base alignment is rounded up to a multiple of 16.
    //   - Struct's base alignment is rounded up to a multiple of 16.
    // The Standard Storage Buffer Layout (and push constants) omit this rule.
    match (rule, t) {
        (LayoutRule::Std140, TypeKind::Array { .. } | TypeKind::Struct { .. }) => inner.max(16),
        _ => inner,
    }
}

pub(super) fn size_of(t: &TypeKind, rule: LayoutRule) -> u32 {
    match t {
        TypeKind::Scalar { width_bytes } => *width_bytes,
        // Tight component packing.
        // vec3 occupies 3N bytes; the alignment padding for the next member is supplied by the consumer at offset assignment time.
        TypeKind::Vector { component, count } => count * size_of(&component.kind, rule),
        TypeKind::Matrix { column, cols } => {
            let col_count = column_vec_count(column);
            let scalar_w = column_scalar_width(column);
            cols * matrix_stride(col_count, scalar_w, rule)
        }
        TypeKind::Array { element, len } => array_stride(&element.kind, rule) * len,
        TypeKind::Struct { members } => layout_struct(members, rule).size,
    }
}

// "An array's `ArrayStride` is equal to its element's consumed size rounded up to the array's base alignment."  Under Standard
// Uniform Buffer Layout the element's base alignment is itself ≥16, so the resulting stride is also ≥16.
pub(super) fn array_stride(elem: &TypeKind, rule: LayoutRule) -> u32 {
    let align = base_align(elem, rule);
    let raw = round_up(size_of(elem, rule), align);
    match rule {
        LayoutRule::Std140 => raw.max(16),
        LayoutRule::Std430 => raw,
    }
}

// `column_vec_count` is the component count of one column for a ColMajor matrix, or one row for a RowMajor matrix.
// The stride is the size of that column / row rounded up to its own vector base alignment and to 16 under std140.
pub(super) fn matrix_stride(column_vec_count: u32, scalar_w: u32, rule: LayoutRule) -> u32 {
    let vec_align = match column_vec_count {
        1 => scalar_w,
        2 => 2 * scalar_w,
        3 | 4 => 4 * scalar_w,
        n => panic!("Unsupported matrix column dimension: {}", n),
    };
    let raw = round_up(column_vec_count * scalar_w, vec_align);
    match rule {
        LayoutRule::Std140 => raw.max(16),
        LayoutRule::Std430 => raw,
    }
}

// Compute member offsets and total size for an `OpTypeStruct`.
//
// "The members are assigned consecutive offsets starting from zero, with each member's offset adjusted upwards to satisfy its base alignment."
// "The structure's size is the offset of the last member, plus the size of the last member, rounded up to a multiple of the structure's base alignment."
pub(super) fn layout_struct(members: &[Type], rule: LayoutRule) -> StructLayout {
    let mut offset = 0u32;
    let mut offsets = Vec::with_capacity(members.len());
    let mut align = 4u32;

    for m in members {
        let a = base_align(&m.kind, rule);
        offset = round_up(offset, a);
        offsets.push(offset);
        offset += size_of(&m.kind, rule);
        align = align.max(a);
    }

    let size = round_up(offset, align);
    StructLayout {
        member_offsets: offsets,
        size,
        align,
    }
}

pub(super) fn column_vec_count(column: &Type) -> u32 {
    match &column.kind {
        TypeKind::Vector { count, .. } => *count,
        TypeKind::Scalar { .. } => 1,
        _ => panic!("Matrix column type must be a scalar or vector"),
    }
}

pub(super) fn column_scalar_width(column: &Type) -> u32 {
    match &column.kind {
        TypeKind::Vector { component, .. } => match &component.kind {
            TypeKind::Scalar { width_bytes } => *width_bytes,
            _ => panic!("Matrix column vector component must be scalar"),
        },
        TypeKind::Scalar { width_bytes } => *width_bytes,
        _ => panic!("Matrix column type must be a scalar or vector"),
    }
}
