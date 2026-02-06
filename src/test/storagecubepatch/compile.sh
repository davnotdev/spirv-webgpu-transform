set -e

glslc storagecube.frag -o storagecube.spv
glslc storagecube_nested.frag -o storagecube_nested.spv
glslc storagecube_immediate.frag -o storagecube_immediate.spv

