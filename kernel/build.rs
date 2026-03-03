fn main() {
    // Get the path to the compiled os binary (set by artifact dependencies)
    let os_path = std::env::var("CARGO_BIN_FILE_OS_os")
        .expect("CARGO_BIN_FILE_OS_os not set - os artifact dependency required");

    // Export path for kernel to include the os binary
    println!("cargo:rustc-env=INIT_ELF_PATH={}", os_path);

    // Re-run if os binary changes
    println!("cargo:rerun-if-changed={}", os_path);
}
