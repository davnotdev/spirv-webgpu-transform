set -e

glslc pruneunuseddref_texture.frag -o pruneunuseddref_texture.spv
glslc pruneunuseddref_sampler.frag -o pruneunuseddref_sampler.spv
glslc pruneunuseddref_nested.frag -o pruneunuseddref_nested.spv

