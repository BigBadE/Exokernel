fn main() {
    // Tell cargo to pass the linker script to the linker
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-arg=-T{}/linker.ld", manifest_dir);

    // Force static relocation model to generate ET_EXEC instead of ET_DYN (PIE)
    // This is crucial because our simple ELF loader doesn't process relocations
    println!("cargo:rustc-env=CARGO_ENCODED_RUSTFLAGS=-Crelocation-model=static");

    // Re-run if linker script changes
    println!("cargo:rerun-if-changed=linker.ld");
}
