# SPIRV WebGPU Transforms

[![Version Badge](https://img.shields.io/crates/v/spirv-webgpu-transform)](https://crates.io/crates/spirv-webgpu-transform)
[![Docs Badge](https://img.shields.io/docsrs/spirv-webgpu-transform/latest)](https://docs.rs/spirv-webgpu-transform/latest/spirv_webgpu_transform/)
[![License Badge](https://img.shields.io/crates/l/spirv-webgpu-transform)](LICENSE)
[![Downloads Badge](https://img.shields.io/crates/d/spirv-webgpu-transform)](https://crates.io/crates/spirv-webgpu-transform)

When porting native games to the web using WebGPU, it becomes neccessary to translate shaders (typically from SPIR-V) to WGSL using [`naga`](https://github.com/gfx-rs/wgpu) or [`tint`](https://dawn.googlesource.com/dawn).
Unfortunately, the WGSL specification lacks support for many features that have shader programmers have become accustomed to.
This project aims to transform common but unsupported SPIRV shaders into a form that `naga` and `tint` can transpile.

## Feature Summary

At the moment, the following transformations are supported:

| Feature                   | `spirv-val` | `naga` | `tint` |
| ------------------------- | ----------- | ------ | ------ |
| Combined Image Samplers   | ✅          | ✅     | ✅     |
| Mixed Depth / Comparison  | ✅          | ⚠️\*   | ❌     |
| Push Constants (WIP)      | ❌          | ❌     | ❌     |

> \* Simple cases are OK.
> With some [special patches](https://github.com/davnotdev/wgpu/tree/trunk-naga-patches), `naga` can process these.

## Combined Image Samplers

It is commonly known that [WebGpu does not support combined image samplers](https://github.com/gpuweb/gpuweb/issues/770).
This makes adding WebGpu support for existing OpenGL or Vulkan renderers impossible without workarounds.
This is one such workaround.
By reading and modifying SPIRV byte code, combined image samplers can be split into their respective texture and sampler.
Special edge cases such as the use of combined image samplers in function parameters and nested functions are also handled.

```glsl
layout(set = 0, binding = 0) uniform sampler2D u_texture;
layout(set = 0, binding = 1) uniform sampler2DArray u_texture_array;

// is converted into...

layout(set = 0, binding = 0) uniform texture2D u_texture;
layout(set = 0, binding = 1) uniform sampler u_sampler;

// *texture2DArray doesn't exist in glsl, but in wgsl, this would be texture_2d_array<f32>
layout(set = 0, binding = 2) uniform texture2DArray u_texture_array;
layout(set = 0, binding = 3) uniform sampler u_sampler;
```

### Additional Notes

- Translating `sampler2D[N]` and `sampler2DArray[N]` is NOT supported.
- After being split, the SPIR-V will not translate back to GLSL "one-to-one", the translation back to GLSL using either `naga` or `tint` creates a combined image sampler!
- This implementation has not been updated to use the current function nesting implementation, so extremely strange function nesting patterns may cause issues

### Tests

| Test                | `spirv-val` | Naga   | Tint |
| ------------------- | ----------- | ------ | ---- |
| `test.frag`         | ✅          | ✅     | ✅   |
| `test_nested.frag`  | ✅          | ✅     | ✅   |
| `test_arrayed.frag` | ✅          | ✅     | ✅   |
| `test_mixed.frag`   | ✅          | ✅     | ✅   |

## Mixed Depth / Comparison

The WGSL spec differentiates between `sampler` and `sampler_comparison` as well as `texture2d<T>` and `texture_depth_2d`.
In GLSL and SPIRV land, the rules on which can be used where are MUCH softer.
In fact, in SPIRV, "whether or not depth comparisons are actually done is a property of the sampling opcode, not of this (image type) type declaration."
WGSL technically does allow for the mixing of these types to some degree, but both `naga` and `tint` have trouble or simply CANNOT mix the two.
For that reason, we need to decouple "regular" and "comparison" samplers and textures.

```glsl
layout(set = 0, binding = 0) uniform sampler u_mixed_sampler;
layout(set = 0, binding = 1) uniform texture2D u_mixed_texture;

void main() {
    float g0 = textureProj(sampler2DShadow(u_mixed_texture, u_mixed_sampler), vec4(0.0, 0.0, 0.0, 0.0));
    vec4 g1 = textureLod(sampler2D(u_mixed_texture, u_mixed_sampler), vec2(0.0, 0.0), 0);
}

// is *ROUGHLY* converted into ...

layout(set = 0, binding = 0) uniform sampler u_mixed_sampler;
layout(set = 0, binding = 1) uniform sampler u_comparison_sampler;
layout(set = 0, binding = 2) uniform texture2D u_mixed_texture;
layout(set = 0, binding = 3) uniform texture2D u_comparison_texture;

void main() {
    float g0 = textureProj(sampler2DShadow(u_comparison_texture, u_comparison_texture), vec4(0.0, 0.0, 0.0, 0.0));
    vec4 g1 = textureLod(sampler2D(u_mixed_texture, u_mixed_sampler), vec2(0.0, 0.0), 0);
}
```

### Tests

| Test                              | `spirv-val` | Naga   | Tint |
| --------------------------------- | ----------- | ------ | ---- |
| `test_image.frag`                 | ✅          | ✅     | ✅   |
| `test_wrong_type_image.spvasm`    | ✅          | ✅     | ✅   |
| `test_sampler.frag`               | ✅          | ✅     | ✅   |
| `test_mixed_dref.frag`            | ✅          | ❌\*    | ❌   |
| `test_nested_sampler.frag`        | ✅          | ❌\*    | ❌   |
| `test_nested2_sampler.frag`       | ✅          | ❌\*    | ❌   |
| `test_nested_image.frag`          | ✅          | ❌\*    | ❌   |
| `test_nested2_image.frag`         | ✅          | ❌\*    | ❌   |
| `test_hidden_dref.frag`           | ✅          | ❌\*    | ❌   |
| `test_hidden2_dref.frag`          | ✅          | ❌\*    | ❌   |
| `test_hidden3_dref.frag`          | ❌          | ❌      | ❌   |

> \* With some [special patches](https://github.com/davnotdev/wgpu/tree/trunk-naga-patches), `naga` can process these.

## Push Constants

> Coming Soon (if the WebGPU spec doesn't move fast enough)

## Library Usage

Add this to your `Cargo.toml`:

```
spirv-webgpu-transform = "0.1"
```

I recommend having a look at [`src/bin/spv_webgpu_transform.rs`](src/bin/spv_webgpu_transform.rs).

```rust
let spv = spirv_webgpu_transform::u8_slice_to_u32_vec(&spv_bytes);

// Tells you which bindings need to be corrected.
let mut out_correction_map = None;

// Using `combimgsampsplitter` as an example.
spirv_webgpu_transform::combimgsampsplitter(&spv, &mut out_correction_map).unwrap()

let out_spv_bytes = spirv_webgpu_transform::u32_slice_to_u8_vec(&out_spv);
```

## CLI Usage

```bash
# Using the `combimg` operation as an example.

cargo install spirv-webgpu-transform
spv_webgpu_transform combimg in.spv out.spv
# or
git clone https://github.com/davnotdev/spirv-webgpu-transform
cd spirv-webgpu-transform
cargo r -- combimg in.spv out.spv
```

## FFI Usage

See [`ffi/bin/spv_webgpu_transform.c`](ffi/bin/spv_webgpu_transform.c) 

---

> Enjoy!
