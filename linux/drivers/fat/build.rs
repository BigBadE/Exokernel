use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Path to shared headers (in linux/include)
    let linux_include = manifest_dir.parent().unwrap().parent().unwrap()
        .join("include");

    // Compile the real Linux FAT driver with our shim headers
    cc::Build::new()
        // Real Linux kernel FAT driver files (unmodified from kernel v6.6)
        .file("linux-src/fs/fat/inode.c")
        .file("linux-src/fs/fat/dir.c")
        .file("linux-src/fs/fat/file.c")
        .file("linux-src/fs/fat/cache.c")
        .file("linux-src/fs/fat/fatent.c")
        .file("linux-src/fs/fat/misc.c")
        .file("linux-src/fs/fat/namei_vfat.c")
        .file("linux-src/fs/fat/namei_msdos.c")
        .file("linux-src/fs/fat/nfs.c")
        // Our shim headers that provide Linux kernel API (from linux/include)
        .include(&linux_include)
        // FAT driver internal header
        .include(manifest_dir.join("linux-src/fs/fat"))
        // UAPI headers
        .include(manifest_dir.join("linux-src/include"))
        // Compiler flags to simulate kernel environment
        .flag("-ffreestanding")
        .flag("-fno-stack-protector")
        .flag("-fno-builtin")
        // Warnings
        .flag("-Wall")
        .flag("-Wno-unused-parameter")
        .flag("-Wno-sign-compare")
        .flag("-Wno-unused-function")
        .flag("-Wno-unused-variable")
        .flag("-Wno-pointer-sign")
        .flag("-Wno-missing-field-initializers")
        .flag("-Wno-implicit-function-declaration")
        // Defines for kernel/shim compatibility
        .define("__KERNEL__", None)
        .define("CONFIG_X86_64", None)
        .define("CONFIG_FAT_DEFAULT_CODEPAGE", "437")
        .define("CONFIG_FAT_DEFAULT_IOCHARSET", "\"iso8859-1\"")
        .define("CONFIG_VFAT_FS", None)
        .define("CONFIG_MSDOS_FS", None)
        .define("CONFIG_FAT_FS", None)
        .define("CONFIG_FAT_DEFAULT_UTF8", "0")
        // Disable features we don't support
        .define("CONFIG_FAT_TEST", None)  // Disable kunit tests
        .compile("linux_fat");

    // Tell cargo to link the compiled library
    println!("cargo:rustc-link-lib=static=linux_fat");
    println!("cargo:rustc-link-search=native={}", out_dir.display());

    // Rerun if any source files change
    println!("cargo:rerun-if-changed=linux-src/fs/fat/");
    println!("cargo:rerun-if-changed=include/linux/");
    println!("cargo:rerun-if-changed=include/asm/");
}
