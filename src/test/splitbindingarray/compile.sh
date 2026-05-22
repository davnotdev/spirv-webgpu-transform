set -e

glslc -O0 buffer_binding_array.frag -o buffer_binding_array.spv
glslc -O0 storage_binding_array.frag -o storage_binding_array.spv
glslc -O0 texture_binding_array.frag -o texture_binding_array.spv
glslc -O0 sampler_binding_array.frag -o sampler_binding_array.spv
glslc -O0 nested_texture_binding_array.frag -o nested_texture_binding_array.spv
glslc -O0 sampler_stub.frag -o sampler_stub.spv
