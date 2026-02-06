set -e

glslc test_image.frag -o test_image.spv
glslc test_nested_image.frag -o test_nested_image.spv
glslc test_nested2_image.frag -o test_nested2_image.spv
glslc test_sampler.frag -o test_sampler.spv
glslc test_nested_sampler.frag -o test_nested_sampler.spv
glslc test_nested2_sampler.frag -o test_nested2_sampler.spv
glslc test_mixed_dref.frag -o test_mixed_dref.spv
glslc test_hidden_dref.frag -o test_hidden_dref.spv
glslc test_hidden2_dref.frag -o test_hidden2_dref.spv
glslc test_hidden3_dref.frag -o test_hidden3_dref.spv
spirv-as test_wrong_type_image.spvasm -o test_wrong_type_image.spv

