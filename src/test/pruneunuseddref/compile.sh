set -e

glslc pruneunuseddref.frag -o pruneunuseddref.spv
glslc pruneunuseddref_nested.frag -o pruneunuseddref_nested.spv

