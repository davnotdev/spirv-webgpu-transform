set -e

glslc buffer_binding_array.frag -o buffer_binding_array.spv
glslc storage_binding_array.frag -o storage_binding_array.spv
glslc texture_binding_array.frag -o texture_binding_array.spv
glslc nested_texture_binding_array.frag -o nested_texture_binding_array.spv
