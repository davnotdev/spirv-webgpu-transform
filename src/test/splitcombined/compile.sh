set -e

glslc -O0 test.frag -o test.spv
glslc -O0 test_arrayed.frag -o test_arrayed.spv
glslc -O0 test_nested.frag -o test_nested.spv
glslc -O0 test_mixed.frag -o test_mixed.spv

