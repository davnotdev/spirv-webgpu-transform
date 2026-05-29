set -e

glslc -O0 storagecube.frag -o storagecube.spv
glslc -O0 storagecube_nested.frag -o storagecube_nested.spv
glslc -O0 storagecube_immediate.frag -o storagecube_immediate.spv

