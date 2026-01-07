set -e

glslc splitcombined/test.frag -o splitcombined/test.spv
glslc splitcombined/test_arrayed.frag -o splitcombined/test_arrayed.spv
glslc splitcombined/test_nested.frag -o splitcombined/test_nested.spv
glslc splitcombined/test_mixed.frag -o splitcombined/test_mixed.spv

glslc splitdref/test_image.frag -o splitdref/test_image.spv
glslc splitdref/test_nested_image.frag -o splitdref/test_nested_image.spv
glslc splitdref/test_nested2_image.frag -o splitdref/test_nested2_image.spv
glslc splitdref/test_sampler.frag -o splitdref/test_sampler.spv
glslc splitdref/test_nested_sampler.frag -o splitdref/test_nested_sampler.spv
glslc splitdref/test_nested2_sampler.frag -o splitdref/test_nested2_sampler.spv
glslc splitdref/test_mixed_dref.frag -o splitdref/test_mixed_dref.spv
glslc splitdref/test_hidden_dref.frag -o splitdref/test_hidden_dref.spv
glslc splitdref/test_hidden2_dref.frag -o splitdref/test_hidden2_dref.spv
glslc splitdref/test_hidden3_dref.frag -o splitdref/test_hidden3_dref.spv
spirv-as splitdref/test_wrong_type_image.spvasm -o splitdref/test_wrong_type_image.spv

glslc mirrorpatch/test1.vert -o mirrorpatch/test1.vert.spv
glslc mirrorpatch/test1.frag -o mirrorpatch/test1.frag.spv
glslc mirrorpatch/test2.vert -o mirrorpatch/test2.vert.spv
glslc mirrorpatch/test2.frag -o mirrorpatch/test2.frag.spv

glslc isnanisinfpatch/isnanisinf.frag  -o isnanisinfpatch/isnanisinf.spv
glslc isnanisinfpatch/isnanisinf_vectored.frag  -o isnanisinfpatch/isnanisinf_vectored.spv
