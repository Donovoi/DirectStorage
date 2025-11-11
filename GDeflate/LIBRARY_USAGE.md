# Using GDeflate as a Library

This guide explains how to integrate the GDeflate compression library into your projects.

## Overview

GDeflate is a GPU-optimized compression format that maintains the same compression ratios as standard DEFLATE but enables much higher throughput for decompression, especially on GPUs. This makes it ideal for applications like:

- High-speed file search tools (like ripgrep)
- Game asset loading
- Data streaming applications
- Archive utilities

## Integration Methods

### 1. C/C++ Projects

#### Using CMake

Add to your `CMakeLists.txt`:

```cmake
# Find GDeflate
find_package(GDeflate REQUIRED)

# Link to your target
target_link_libraries(your_target PRIVATE GDeflate::GDeflate)
```

#### Using pkg-config

```bash
# Compile
gcc -o myapp myapp.c $(pkg-config --cflags --libs gdeflate)

# Or in a Makefile
CFLAGS += $(shell pkg-config --cflags gdeflate)
LDFLAGS += $(shell pkg-config --libs gdeflate)
```

#### C API Example

```c
#include <gdeflate/GDeflate_c.h>
#include <stdio.h>
#include <stdlib.h>

int main() {
    const char* input = "Hello, world!";
    size_t input_size = strlen(input);
    
    // Calculate output buffer size
    size_t max_compressed = gdeflate_compress_bound(input_size);
    uint8_t* compressed = malloc(max_compressed);
    size_t compressed_size = max_compressed;
    
    // Compress
    gdeflate_result result = gdeflate_compress(
        compressed, &compressed_size,
        (const uint8_t*)input, input_size,
        6,  // compression level
        0   // flags
    );
    
    if (result == GDEFLATE_SUCCESS) {
        printf("Compressed %zu bytes to %zu bytes\n", 
               input_size, compressed_size);
    }
    
    // Decompress
    uint8_t* decompressed = malloc(input_size);
    result = gdeflate_decompress(
        decompressed, input_size,
        compressed, compressed_size,
        0  // num_workers (0 = auto)
    );
    
    if (result == GDEFLATE_SUCCESS) {
        printf("Decompression successful!\n");
    }
    
    free(compressed);
    free(decompressed);
    return 0;
}
```

#### C++ API Example

```cpp
#include <gdeflate/GDeflate.h>
#include <vector>
#include <iostream>

int main() {
    std::string input = "Hello, world!";
    std::vector<uint8_t> input_data(input.begin(), input.end());
    
    // Prepare output buffer
    size_t max_size = GDeflate::CompressBound(input_data.size());
    std::vector<uint8_t> compressed(max_size);
    size_t compressed_size = max_size;
    
    // Compress
    bool success = GDeflate::Compress(
        compressed.data(), &compressed_size,
        input_data.data(), input_data.size(),
        6,  // compression level
        0   // flags
    );
    
    if (success) {
        compressed.resize(compressed_size);
        std::cout << "Compressed " << input_data.size() 
                  << " bytes to " << compressed_size << " bytes\n";
    }
    
    // Decompress
    std::vector<uint8_t> decompressed(input_data.size());
    success = GDeflate::Decompress(
        decompressed.data(), decompressed.size(),
        compressed.data(), compressed.size(),
        0  // num_workers (0 = auto)
    );
    
    if (success) {
        std::cout << "Decompression successful!\n";
    }
    
    return 0;
}
```

### 2. Rust Projects

Add to your `Cargo.toml`:

```toml
[dependencies]
gdeflate = "1.0"
```

Example usage:

```rust
use gdeflate::{compress, decompress};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = b"Hello, world!";
    
    // Compress
    let compressed = compress(input, 6, 0)?;
    println!("Compressed {} to {} bytes", input.len(), compressed.len());
    
    // Decompress
    let decompressed = decompress(&compressed, input.len(), 0)?;
    assert_eq!(input, decompressed.as_slice());
    
    Ok(())
}
```

### 3. Integration with ripgrep

To add GDeflate decompression support to ripgrep or similar search tools:

#### Step 1: Add GDeflate as a dependency

```toml
[dependencies]
gdeflate = { path = "path/to/GDeflate/rust" }
```

#### Step 2: Create a decompressor

```rust
use std::io::{self, Read};
use gdeflate::decompress;

pub struct GDeflateReader<R> {
    inner: R,
    decompressed: Vec<u8>,
    position: usize,
}

impl<R: Read> GDeflateReader<R> {
    pub fn new(mut inner: R) -> io::Result<Self> {
        // Read all compressed data
        let mut compressed = Vec::new();
        inner.read_to_end(&mut compressed)?;
        
        // For now, we need to know the uncompressed size
        // In a real implementation, you'd store this in a header
        let uncompressed_size = estimate_size(&compressed);
        
        // Decompress
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

fn estimate_size(compressed: &[u8]) -> usize {
    // This is a placeholder - in practice, store size in file header
    compressed.len() * 4
}
```

#### Step 3: Register the decompressor

In ripgrep's preprocessor configuration:

```rust
// Add to the list of supported compression formats
if file_extension == "gdz" || detect_gdeflate_format(&data) {
    Box::new(GDeflateReader::new(file)?)
}
```

## Building and Installing

### Building from Source

```bash
cd GDeflate
cmake --preset Release
cmake --build --preset Release
sudo cmake --install build/Linux/Release
```

### Using in Your Build System

After installation, the library is available through:
- CMake: `find_package(GDeflate)`
- pkg-config: `pkg-config --cflags --libs gdeflate`
- Direct linking: `-lgdeflate -lpthread`

## File Format Considerations

When creating compressed files for search tools, consider:

1. **Store uncompressed size**: Add a header with the uncompressed size to avoid guessing
2. **Chunk large files**: Split large files into chunks for parallel decompression
3. **File extension**: Use `.gdz` or `.gdeflate` for easy detection

Example file format:

```
[4 bytes: magic number "GDZ\0"]
[8 bytes: uncompressed size]
[N bytes: GDeflate compressed data]
```

## Performance Tips

1. **Compression level**: Use level 6-9 for a good balance of speed and ratio
2. **Multi-threading**: Set `num_workers` to 0 for automatic thread count
3. **Buffer sizing**: Use `compress_bound()` to allocate the right buffer size
4. **Streaming**: For large files, compress/decompress in chunks

## Platform Support

- **Linux**: Full support with pthread
- **Windows**: Full support including DirectStorage integration
- **macOS**: Full support with pthread

## License

GDeflate is licensed under Apache-2.0. See LICENSE file for details.
