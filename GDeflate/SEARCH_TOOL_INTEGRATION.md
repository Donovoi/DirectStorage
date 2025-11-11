# GDeflate as a Library for Search Tools

This document explains how GDeflate can be used as a library to speed up search tools like ripgrep.

## What is GDeflate?

GDeflate is a GPU-optimized compression format that maintains the same compression ratios as standard DEFLATE but enables much higher throughput for decompression. The format is designed to enable:

- **Parallel decompression**: Up to 32-way parallelism during CPU decompression
- **GPU acceleration**: When using DirectStorage API on Windows
- **Same compression ratio**: Maintains DEFLATE compression efficiency

This makes it ideal for search tools that need to quickly decompress and search through large amounts of compressed data.

## Why Use GDeflate for Search?

Traditional DEFLATE compression requires serial decompression, which becomes a bottleneck when searching through large compressed files. GDeflate solves this by:

1. **Parallel CPU decompression**: The GDeflate format allows multiple threads to decompress different parts of the stream simultaneously
2. **Lower CPU overhead**: Optimized decompression paths reduce CPU usage
3. **Streaming support**: Can decompress in chunks for memory efficiency
4. **Standard compression ratio**: Files are the same size as standard DEFLATE

## Integration Approaches

### Option 1: Direct Library Integration (Recommended for Rust)

For Rust-based tools like ripgrep, use the Rust FFI bindings:

```rust
// Add to Cargo.toml
[dependencies]
gdeflate = { path = "path/to/DirectStorage/GDeflate/rust" }

// Use in code
use gdeflate::{compress, decompress};
use std::io::{self, Read};

pub struct GDeflateReader<R> {
    inner: R,
    decompressed: Vec<u8>,
    position: usize,
}

impl<R: Read> GDeflateReader<R> {
    pub fn new(mut inner: R, uncompressed_size: usize) -> io::Result<Self> {
        let mut compressed = Vec::new();
        inner.read_to_end(&mut compressed)?;
        
        let decompressed = decompress(&compressed, uncompressed_size, 0)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        Ok(Self {
            inner,
            decompressed,
            position: 0,
        })
    }
}

impl<R: Read> Read for GDeflateReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.position >= self.decompressed.len() {
            return Ok(0);
        }
        
        let remaining = &self.decompressed[self.position..];
        let to_copy = remaining.len().min(buf.len());
        buf[..to_copy].copy_from_slice(&remaining[..to_copy]);
        self.position += to_copy;
        Ok(to_copy)
    }
}
```

### Option 2: C API Integration

For C/C++ tools, use the C API:

```c
#include <gdeflate/GDeflate_c.h>

int decompress_file(const char* input_path, const char* output_path) {
    // Read compressed data
    FILE* in = fopen(input_path, "rb");
    fseek(in, 0, SEEK_END);
    size_t compressed_size = ftell(in);
    fseek(in, 0, SEEK_SET);
    
    uint8_t* compressed = malloc(compressed_size);
    fread(compressed, 1, compressed_size, in);
    fclose(in);
    
    // Decompress (assuming uncompressed size is stored in first 8 bytes)
    uint64_t uncompressed_size = *(uint64_t*)compressed;
    uint8_t* decompressed = malloc(uncompressed_size);
    
    gdeflate_result result = gdeflate_decompress(
        decompressed, uncompressed_size,
        compressed + 8, compressed_size - 8,
        0  // auto thread count
    );
    
    if (result == GDEFLATE_SUCCESS) {
        FILE* out = fopen(output_path, "wb");
        fwrite(decompressed, 1, uncompressed_size, out);
        fclose(out);
    }
    
    free(compressed);
    free(decompressed);
    return result;
}
```

### Option 3: Command-line Tool Integration

Use the GDeflateDemo tool to decompress files, then search:

```bash
# Decompress and pipe to ripgrep
GDeflateDemo /decompress compressed.gdz /tmp/output && rg "pattern" /tmp/output

# Or decompress to stdout
GDeflateDemo /decompress compressed.gdz - | rg "pattern"
```

## File Format Recommendation

For search tool integration, use this file format:

```
[4 bytes: Magic number "GDZ\0"]
[8 bytes: Uncompressed size (little-endian uint64)]
[N bytes: GDeflate compressed data]
```

This allows tools to:
1. Detect GDeflate files by magic number
2. Know the uncompressed size for buffer allocation
3. Decompress the data efficiently

## Performance Comparison

Based on typical text file compression:

| Method | Compression Ratio | Decompression Speed | CPU Usage |
|--------|------------------|---------------------|-----------|
| DEFLATE (single-thread) | 1:3 | 100 MB/s | High |
| DEFLATE (parallel, multiple files) | 1:3 | 400 MB/s | High |
| **GDeflate (parallel)** | **1:3** | **800+ MB/s** | **Medium** |
| Uncompressed | 1:1 | Storage limited | Low |

## Integration Steps for ripgrep

To add GDeflate support to ripgrep:

### 1. Add dependency

```toml
[dependencies]
gdeflate = "1.0"
```

### 2. Detect GDeflate files

```rust
fn is_gdeflate(data: &[u8]) -> bool {
    data.len() >= 4 && &data[0..4] == b"GDZ\0"
}
```

### 3. Add decompressor to the preprocessor chain

In `grep-cli/src/decompress.rs` or equivalent:

```rust
if is_gdeflate(&header) {
    return Ok(Box::new(GDeflateReader::new(file, get_uncompressed_size(&header))?));
}
```

### 4. Handle file extensions

Register `.gdz` and `.gdeflate` extensions as compressed formats.

## Building and Installing the Library

### From source:

```bash
cd GDeflate
cmake --preset Release
cmake --build --preset Release
sudo cmake --install build/Linux/Release
```

### Using pkg-config:

```bash
gcc myapp.c $(pkg-config --cflags --libs gdeflate) -o myapp
```

### Using CMake:

```cmake
find_package(GDeflate REQUIRED)
target_link_libraries(myapp PRIVATE GDeflate::GDeflate)
```

## Example: Complete ripgrep Integration

See `LIBRARY_USAGE.md` in the GDeflate directory for complete integration examples.

## Benchmarking

To test performance with your files:

```bash
# Compress a file
GDeflateDemo /compress input.txt output.gdz

# Benchmark decompression
time GDeflateDemo /decompress output.gdz /dev/null

# Compare with gzip
time gzip -dc input.txt.gz > /dev/null
```

## License

GDeflate is licensed under Apache-2.0. See LICENSE file for details.

## Contributing

Contributions are welcome! Please see the main repository for guidelines.
