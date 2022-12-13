use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;
use fs::partition::Partition;
use fs::Write;

fn main() {
    let path = env::var("OUT_DIR").unwrap();
    let bios_path = Path::new(path.as_str()).join("bios");

    build("x86_64-bootloader-bios", "x86_64-unknown-none");
    //Build BIOS image
    let bios = BIOSImage::new(
        PathBuf::from(env::var("CARGO_BIN_FILE_x86_64-BOOTLOADER-BIOS-STAGE-1_x86_64-bootloader-bios-stage-1").unwrap()),
        PathBuf::from(env::var("CARGO_BIN_FILE_x86_64-BOOTLOADER-BIOS-STAGE-2_x86_64-bootloader-bios-stage-2").unwrap()),
        |partition| {});
}

struct BIOSImage {}

impl BIOSImage {
    pub fn new(bootloader: PathBuf, kernelloader: PathBuf, setup: fn(Partition<File>)) -> Self {
        return BIOSImage {};
    }
}

// From rust-osdev/bootloader build.rs
fn build(name: &'static str, target: &'static str) -> PathBuf {
    let out_dir = env::var("OUT_DIR").unwrap();
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg("build").arg("--locked")
        .arg("--target").arg(target)
        .arg("-Zbuild-std=core")
        .arg("-Zbuild-std-features=compiler-builtins-mem");
    cmd.env_remove("RUSTFLAGS");
    cmd.env_remove("CARGO_ENCODED_RUSTFLAGS");
    cmd.env_remove("RUSTC_WORKSPACE_WRAPPER"); // used by clippy
    let status = cmd
        .status()
        .expect(format!("failed to run cargo install for {}", name).as_str());
    let elf_path = if status.success() {
        let path = Path::new(out_dir.as_str()).join("bin").join(name);
        assert!(path.exists(), "{} executable does not exist after building", name);
        path
    } else {
        panic!("failed to build {}", name);
    };

    return convert_elf_to_bin(elf_path);
}

// From rust-osdev/bootloader build.rs
fn convert_elf_to_bin(elf_path: PathBuf) -> PathBuf {
    let flat_binary_path = elf_path.with_extension("bin");

    let llvm_tools = llvm_tools::LlvmTools::new().expect("failed to get llvm tools");
    let objcopy = llvm_tools
        .tool(&llvm_tools::exe("llvm-objcopy"))
        .expect("LlvmObjcopyNotFound");

    // convert first stage to binary
    let mut cmd = Command::new(objcopy);
    cmd.arg("-I").arg("elf64-x86-64");
    cmd.arg("-O").arg("binary");
    cmd.arg("--binary-architecture=i386:x86-64");
    cmd.arg(&elf_path);
    cmd.arg(&flat_binary_path);
    let output = cmd
        .output()
        .expect("failed to execute llvm-objcopy command");
    if !output.status.success() {
        panic!(
            "objcopy failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    return flat_binary_path;
}