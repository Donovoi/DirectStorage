#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include "GDeflate/GDeflate_c.h"

int main() {
    printf("Testing GDeflate C API\n");
    printf("Version: %s\n\n", gdeflate_version());
    
    const char* input = "Hello, world! This is a test of GDeflate compression. "
                       "The quick brown fox jumps over the lazy dog.";
    size_t input_size = strlen(input);
    
    printf("Input: %s\n", input);
    printf("Input size: %zu bytes\n\n", input_size);
    
    // Calculate maximum compressed size
    size_t max_compressed = gdeflate_compress_bound(input_size);
    printf("Maximum compressed size: %zu bytes\n", max_compressed);
    
    // Allocate buffers
    uint8_t* compressed = (uint8_t*)malloc(max_compressed);
    uint8_t* decompressed = (uint8_t*)malloc(input_size + 1);
    
    if (!compressed || !decompressed) {
        fprintf(stderr, "Memory allocation failed\n");
        return 1;
    }
    
    // Compress
    size_t compressed_size = max_compressed;
    gdeflate_result result = gdeflate_compress(
        compressed, &compressed_size,
        (const uint8_t*)input, input_size,
        6,  // compression level
        0   // flags
    );
    
    if (result != GDEFLATE_SUCCESS) {
        fprintf(stderr, "Compression failed with error: %d\n", result);
        free(compressed);
        free(decompressed);
        return 1;
    }
    
    printf("Compressed size: %zu bytes (%.1f%% of original)\n\n", 
           compressed_size, (compressed_size * 100.0) / input_size);
    
    // Decompress
    result = gdeflate_decompress(
        decompressed, input_size,
        compressed, compressed_size,
        0  // num_workers (0 = auto)
    );
    
    if (result != GDEFLATE_SUCCESS) {
        fprintf(stderr, "Decompression failed with error: %d\n", result);
        free(compressed);
        free(decompressed);
        return 1;
    }
    
    decompressed[input_size] = '\0';
    printf("Decompressed: %s\n", decompressed);
    
    // Verify
    if (memcmp(input, decompressed, input_size) == 0) {
        printf("\n✓ Success! Data matches after round-trip compression.\n");
    } else {
        fprintf(stderr, "\n✗ Error! Data does not match.\n");
        free(compressed);
        free(decompressed);
        return 1;
    }
    
    free(compressed);
    free(decompressed);
    return 0;
}
