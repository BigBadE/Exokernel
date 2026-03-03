use std::path::PathBuf;

fn main() {
    // Get the path to the compiled kernel binary (set by artifact dependencies)
    let kernel_path = std::env::var("CARGO_BIN_FILE_KERNEL_kernel")
        .expect("CARGO_BIN_FILE_KERNEL_kernel not set");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // Create UEFI disk image
    let uefi_path = out_dir.join("uefi.img");
    let kernel_path = PathBuf::from(kernel_path);
    bootloader::UefiBoot::new(&kernel_path)
        .create_disk_image(&uefi_path)
        .expect("Failed to create UEFI disk image");

    // BIOS bootloader compilation is currently broken in the bootloader crate
    // See: https://github.com/rust-osdev/bootloader/issues
    // For now, just use UEFI which works fine
    let bios_path = out_dir.join("bios.img");
    // Create empty placeholder so the build doesn't fail
    std::fs::write(&bios_path, b"BIOS not supported - use UEFI").ok();

    // Export paths for src/main.rs
    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());
}
