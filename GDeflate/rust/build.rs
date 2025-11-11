use std::env;
use std::path::PathBuf;

fn main() {
    let gdeflate_src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("..")
        .join("GDeflate");
    let libdeflate_src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("..")
        .join("3rdparty")
        .join("libdeflate");

    // Build libdeflate
    let mut libdeflate_build = cc::Build::new();
    libdeflate_build
        .files(&[
            "lib/adler32.c",
            "lib/crc32.c",
            "lib/deflate_compress.c",
            "lib/deflate_decompress.c",
            "lib/gdeflate_compress.c",
            "lib/gdeflate_decompress.c",
            "lib/gzip_compress.c",
            "lib/gzip_decompress.c",
            "lib/utils.c",
            "lib/zlib_compress.c",
            "lib/zlib_decompress.c",
            "lib/x86/cpu_features.c",
        ].iter().map(|f| libdeflate_src.join(f)))
        .include(&libdeflate_src)
        .warnings(false);

    // Platform-specific flags
    if cfg!(target_os = "windows") {
        libdeflate_build.define("_CRT_SECURE_NO_WARNINGS", None);
    }

    libdeflate_build.compile("deflate");

    // Build GDeflate
    let mut gdeflate_build = cc::Build::new();
    gdeflate_build
        .cpp(true)
        .std("c++17")
        .files(&[
            "GDeflateCompress.cpp",
            "GDeflateDecompress.cpp",
            "GDeflate_c.cpp",
        ].iter().map(|f| gdeflate_src.join(f)))
        .include(&gdeflate_src)
        .include(&libdeflate_src)
        .warnings(false);

    // Platform-specific settings
    if cfg!(target_os = "windows") {
        gdeflate_build
            .define("_CRT_SECURE_NO_WARNINGS", None);
    }

    gdeflate_build.compile("gdeflate");

    // Link pthread on Unix
    if !cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=pthread");
    }

    // Re-run build script if sources change
    println!("cargo:rerun-if-changed=../GDeflate/GDeflateCompress.cpp");
    println!("cargo:rerun-if-changed=../GDeflate/GDeflateDecompress.cpp");
    println!("cargo:rerun-if-changed=../GDeflate/GDeflate_c.cpp");
    println!("cargo:rerun-if-changed=../GDeflate/GDeflate.h");
    println!("cargo:rerun-if-changed=../GDeflate/GDeflate_c.h");
}
