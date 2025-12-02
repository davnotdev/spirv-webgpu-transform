#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include "../spirv_webgpu_transform.h"

#define BAD_FILE_PATH "./bad.spv"

void print_set_binding(TransformCorrectionMap map, uint32_t set, uint32_t binding);

int main() {
    // 1. Read the SPIRV file
    FILE* file = fopen(BAD_FILE_PATH, "rb");
    fseek(file, 0, SEEK_END);
    int spirv_bytes = ftell(file);
    fseek(file, 0, SEEK_SET);
    uint8_t* spirv = (uint8_t*)malloc(spirv_bytes);
    fread(spirv, 1, spirv_bytes, file);
    fclose(file);
    
    // 2. Run the transformations
    TransformCorrectionMap correction_map = SPIRV_WEBGPU_TRANFORM_CORRECTION_MAP_NULL;

    uint32_t* comb_out_spv;
    uint32_t comb_out_count;
    spirv_webgpu_transform_combimgsampsplitter_alloc((uint32_t*)spirv, spirv_bytes / 4, &comb_out_spv, &comb_out_count, &correction_map);

    uint32_t* dref_out_spv;
    uint32_t dref_out_count;
    spirv_webgpu_transform_drefsplitter_alloc(comb_out_spv, comb_out_count, &dref_out_spv, &dref_out_count, &correction_map);

    // 3. Observe the patched variables
    print_set_binding(correction_map, 0, 0);
    print_set_binding(correction_map, 0, 1);
    print_set_binding(correction_map, 1, 0);

    // Fluke values should return None
    print_set_binding(correction_map, 1, 1);
    print_set_binding(correction_map, 2, 0);

    // 4. Free memory
    spirv_webgpu_transform_drefsplitter_free(dref_out_spv);
    spirv_webgpu_transform_combimgsampsplitter_free(comb_out_spv);
    spirv_webgpu_transform_correction_map_free(correction_map);
    free(spirv);
}

void print_set_binding(TransformCorrectionMap map, uint32_t set, uint32_t binding) {
    uint16_t* corrections;
    uint32_t correction_count;
    TransformCorrectionStatus status = spirv_webgpu_transform_correction_map_index(map, set, binding, &corrections, &correction_count);

    printf("For set %d, binding %d:\n", set, binding);

    if (status == SPIRV_WEBGPU_TRANSFORM_CORRECTION_STATUS_NONE)  {
        printf("\tNone\n");
    } else {
        printf("\tSome\n");
    }

    printf("\t");
    for (int i = 0; i < correction_count; i++) {
        switch (corrections[i]) {
            case SPIRV_WEBGPU_TRANSFORM_CORRECTION_TYPE_SPLIT_COMBINED:
                printf("SPLIT_COMBINED ");
                break;
            case SPIRV_WEBGPU_TRANSFORM_CORRECTION_TYPE_SPLIT_DREF_REGULAR:
                printf("SPLIT_DREF_REGULAR ");
                break;
            case SPIRV_WEBGPU_TRANSFORM_CORRECTION_TYPE_SPLIT_DREF_COMPARISON:
                printf("SPLIT_DREF_COMPARISON ");
                break;
        }
    }
    printf("\n");
}
