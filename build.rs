use std::env;
use std::fs::{DirBuilder, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use mbrman::{CHS, MBR, MBRPartitionEntry};
use fs::partition::Partition;

fn main() {
    let path = env::var("OUT_DIR").unwrap();
    let path = Path::new(path.as_str());
    let bios_path = path.clone().join("bios").join("bios.img");

    //Build BIOS image
    BIOSImage::new(path,
                   build("x86_64-bootloader-bios-stage-1", "kernel/arch/x86_64/bootloader/bios/stage-1", "target.json", "stage-1", path),
                   build("x86_64-bootloader-bios-stage-2", "kernel/arch/x86_64/bootloader/bios/stage-2", "target.json", "release", path),
                   |partition| {}).unwrap().write(bios_path.clone()).unwrap();
    println!(
        "cargo:rustc-env=BIOS_PATH={}",
        bios_path.display()
    );
}

struct BIOSImage {
    mbr: MBR,
}

impl BIOSImage {
    pub fn new(out_dir: &Path, boot_loader: PathBuf, kernel_loader: PathBuf, setup: fn(Partition<File>)) -> anyhow::Result<Self> {
        let boot_loader = convert_elf_to_bin(boot_loader);
        let mut boot_loader = File::open(boot_loader)?;
        let mut mbr = MBR::read_from(&mut boot_loader, 512)?;
        let kernel_loader = File::open(kernel_loader)?;
        mbr[1] = MBRPartitionEntry {
            boot: 0x80,
            starting_lba: 1,
            sectors: (kernel_loader.metadata()?.len() - 1) as u32 / 512 + 1,
            sys: 0x20,
            first_chs: CHS::empty(),
            last_chs: CHS::empty(),
        };

        let disk = out_dir.join("bios.tmp");
        {
            let mut disk = File::create(disk.clone())?;
            disk.write(&[0xC, 0xA, 0x5, 0xC, 0xA, 0xD, 0xE])?;
            let partition = Partition::new(disk, false);
            setup(partition);
        }

        mbr[2] = MBRPartitionEntry {
            boot: 0x80,
            starting_lba: 1 + mbr[1].sectors,
            sectors: (File::open(disk)?.metadata()?.len() - 1) as u32 / 512 + 1,
            sys: 0xA0,
            first_chs: CHS::empty(),
            last_chs: CHS::empty(),
        };

        return Ok(BIOSImage {
            mbr
        });
    }

    pub fn write(&mut self, output: PathBuf) -> anyhow::Result<()> {
        std::fs::create_dir_all(output.parent().unwrap())?;
        Ok(self.mbr.write_into(&mut File::create(output)?).unwrap())
    }
}

fn build(name: &str, path: &str, target: &str, profile: &str, out_dir: &Path) -> PathBuf {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg("install").arg(name.clone());
    cmd.arg("--path").arg(path.clone());
    cmd.arg("--locked");
    cmd.arg("--target").arg(format!("{}/{}", path, target));
    cmd.arg("--root").arg(out_dir);
    cmd.arg("--profile").arg(profile);
    cmd.env_remove("RUSTFLAGS");
    cmd.env_remove("CARGO_ENCODED_RUSTFLAGS");
    let status = cmd
        .status()
        .expect(format!("failed to run cargo install for {}", name).as_str());
    if status.success() {
        let path = out_dir.join("bin").join(name);
        assert!(
            path.exists(),
            "{} executable does not exist after building", name
        );
        path
    } else {
        panic!("failed to build {}", name);
    }
}

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
    flat_binary_path
}