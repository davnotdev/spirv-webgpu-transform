cc spv_webgpu_transform.c -L../../target/debug/ -lspirv_webgpu_transform_ffi -I../ -o spv_webgpu_transform 

if ! command -v glslc &> /dev/null; then
    echo "glslc not found, skipping shader compilation."
else
    glslc bad.frag -o bad.spv
fi
