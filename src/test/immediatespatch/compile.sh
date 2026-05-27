set -e

glslc -O0 immediates.frag -o immediates.spv
glslc -O0 mat2_direct.frag -o mat2_direct.spv
glslc -O0 array_of_mat2.frag -o array_of_mat2.spv
glslc -O0 nested_struct.frag -o nested_struct.spv
glslc -O0 row_major.frag -o row_major.spv
