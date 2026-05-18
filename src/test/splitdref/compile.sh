set -e

glslc -O0 test_image.frag -o test_image.spv
glslc -O0 test_nested_image.frag -o test_nested_image.spv
glslc -O0 test_nested2_image.frag -o test_nested2_image.spv
glslc -O0 test_sampler.frag -o test_sampler.spv
glslc -O0 test_nested_sampler.frag -o test_nested_sampler.spv
glslc -O0 test_nested2_sampler.frag -o test_nested2_sampler.spv
glslc -O0 test_mixed_dref.frag -o test_mixed_dref.spv
glslc -O0 test_hidden_dref.frag -o test_hidden_dref.spv
glslc -O0 test_hidden2_dref.frag -o test_hidden2_dref.spv
glslc -O0 test_hidden3_dref.frag -o test_hidden3_dref.spv
glslc -O0 test_cross_dref.frag -o test_cross_dref.spv
spirv-as test_wrong_type_image.spvasm -o test_wrong_type_image.spv

