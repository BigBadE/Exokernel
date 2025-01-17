use fatfs::{FileSystem, FsOptions};
use mbrman::{MBRPartitionEntry, CHS, MBR};
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs, io};
use std::collections::HashMap;

fn main() {
    let path = env::var("OUT_DIR").unwrap();
    let path = Path::new(path.as_str());
    let bios_path = path.join("bios").join("bios.img");

    let first_stage =
        convert_elf_to_bin(build("x86_64-bootloader-bios-stage-1", "kernel/arch/x86_64/bootloader/bios/stage-1",
                                 "target.json", "stage-1", path));
    let second_stage =
        convert_elf_to_bin(build("x86_64-bootloader-bios-stage-2", "kernel/arch/x86_64/bootloader/bios/stage-2",
                                 "target.json", "release", path));

    let third_stage =
        convert_elf_to_bin(build("x86_64-bootloader-bios-stage-3", "kernel/arch/x86_64/bootloader/bios/stage-3",
                                                     "target.json", "release", path));

    //Build BIOS image
    BIOSImage::new(first_stage, second_stage, third_stage).unwrap().write(bios_path.clone()).unwrap();
    println!(
        "cargo:rustc-env=BIOS_PATH={}",
        bios_path.display()
    );
}

struct BIOSImage {
    mbr: MBR,
    kernel_loader: File,
    fat_file: File
}

impl BIOSImage {
    pub fn new(boot_loader: PathBuf, kernel_loader: PathBuf, third_stage: PathBuf) -> anyhow::Result<Self> {
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

        let files = HashMap::from([
            (".check", "PASS".bytes().collect()),
            (".kernel/stage-3.bin", fs::read(&third_stage)?)
        ]);

        // Setup the FAT32 file system
        let mut fat_file = OpenOptions::new().read(true).write(true).create(true)
            .open(third_stage.parent().unwrap().join("fat.img"))?;
        const MB: u64 = 1024 * 1024;
        let size = files.values().map(|file| file.len()).sum::<usize>() as u64;
        let len = ((size + 1024 * 64 - 1) / MB + 1) * MB + MB;
        fat_file.set_len(len)?;

        // Write the FAT data
        {
            let format_options = fatfs::FormatVolumeOptions::new().volume_label(*b"Exokernel  ");
            fatfs::format_volume(&fat_file, format_options)?;
            let fat = FileSystem::new(&mut fat_file, FsOptions::new())?;

            for (path, data) in files {
                let mut current = fat.root_dir();
                let path = path.split("/").collect::<Vec<_>>();
                for dir in &path[..path.len()-1] {
                    current = current.create_dir(*dir)?;
                }
                current.create_file(path.last().unwrap())?.write_all(&*data)?;
            }
        }

        let length = (len - 1) as u32;
        mbr[2] = MBRPartitionEntry {
            boot: 0x80,
            starting_lba: 1 + mbr[1].sectors,
            sectors: length / 512 + 1,
            sys: 0xA0,
            first_chs: CHS::empty(),
            last_chs: CHS::empty(),
        };

        Ok(BIOSImage {
            mbr,
            kernel_loader,
            fat_file
        })
    }

    pub fn write(&mut self, output: PathBuf) -> anyhow::Result<()> {
        fs::create_dir_all(output.parent().unwrap())?;
        let mut output_file = File::create(output.clone())?;

        // Write MBR
        self.mbr.write_into(&mut output_file)?;

        // Write the loader
        assert_eq!(output_file.stream_position()?, 512);
        io::copy(&mut self.kernel_loader, &mut output_file)?;

        // Seek to the end of the block
        let end = 512 - output_file.stream_position()? % 512;
        output_file.seek(SeekFrom::Current(end as i64))?;

        // Write the FAT file
        self.fat_file.seek(SeekFrom::Start(0))?;
        io::copy(&mut self.fat_file, &mut output_file)?;
        Ok(())
    }
}

fn build(name: &str, path: &str, target: &str, profile: &str, out_dir: &Path) -> PathBuf {
    //Force rebuilds
    println!("cargo:rerun-if-changed={}", path);

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg("install").arg(name);
    cmd.arg("--path").arg(path);
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
        assert_ne!(path.metadata().unwrap().len(), 0, "{} executable has no length after building", name);
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
    assert_ne!(flat_binary_path.metadata().unwrap().len(), 0, "{} is empty", flat_binary_path.display());
    flat_binary_path
}