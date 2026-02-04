set -e

glslc isnanisinf.frag -o isnanisinf.spv
glslc isnanisinf_vectored.frag -o isnanisinf_vectored.spv
glslc isnanisinf_immediate.frag -o isnanisinf_immediate.spv

