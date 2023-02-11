use std::{env, fs, io};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use fatfs::{FormatVolumeOptions, FsOptions};
use mbrman::{CHS, MBR, MBRPartitionEntry};

const MB: u64 = 1024 * 1024;

fn main() {
    let path = env::var("OUT_DIR").unwrap();
    let path = Path::new(path.as_str());
    let bios_path = path.clone().join("bios").join("bios.img");

    let first_stage =
        convert_elf_to_bin(build("x86_64-bootloader-bios-stage-1", "kernel/arch/x86_64/bootloader/bios/stage-1",
                                 "target.json", "stage-1", path));
    let second_stage =
        convert_elf_to_bin(build("x86_64-bootloader-bios-stage-2", "kernel/arch/x86_64/bootloader/bios/stage-2",
                                 "target.json", "release", path));

    let mut map = HashMap::new();
    map.insert("sys", convert_elf_to_bin(build("x86_64-bootloader-bios-stage-3", "kernel/arch/x86_64/bootloader/bios/stage-3",
                                                     "target.json", "release", path)));

    //Build BIOS image
    BIOSImage::new(path, first_stage, second_stage, map).unwrap().write(bios_path.clone()).unwrap();
    println!(
        "cargo:rustc-env=BIOS_PATH={}",
        bios_path.display()
    );
}

struct BIOSImage {
    mbr: MBR,
    kernel_loader: File,
    disk: File,
}

impl BIOSImage {
    pub fn new(out_dir: &Path, boot_loader: PathBuf, kernel_loader: PathBuf, include: HashMap<&str, PathBuf>) -> anyhow::Result<Self> {
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
        let disk = fs::OpenOptions::new().read(true).write(true).create(true).truncate(true).open(disk.clone()).unwrap();
        let mut len = 0;
        for (_, path) in &include {
            len += fs::metadata(path)?.len();
        }

        let len = ((len + 1024 * 64 - 1) / MB + 1) * MB;
        disk.set_len(len).unwrap();

        fatfs::format_volume(&disk, FormatVolumeOptions::new().volume_label(*b"ExokernelFS"))?;
        let fs = fatfs::FileSystem::new(&disk, FsOptions::new())?;

        let root = fs.root_dir();
        for (path, adding_path) in &include {
            let path = Path::new(path);
            let ancestors: Vec<&Path> = path.ancestors().skip(1).collect();
            for ancestor in ancestors.into_iter().rev().skip(1) {
                let ancestor = ancestor.display().to_string();
                if !ancestor.is_empty() {
                    root.create_dir(ancestor.as_str())?;
                }
            }
            let mut target = root.create_file(path.to_str().unwrap())?;
            target.truncate()?;
            io::copy(&mut File::open(adding_path)?, &mut target)?;
        }
        drop(root);
        drop(fs);

        mbr[2] = MBRPartitionEntry {
            boot: 0x80,
            starting_lba: 1 + mbr[1].sectors,
            sectors: (&disk.metadata()?.len() - 1) as u32 / 512 + 1,
            sys: 0xA0,
            first_chs: CHS::empty(),
            last_chs: CHS::empty(),
        };

        return Ok(BIOSImage {
            mbr,
            kernel_loader,
            disk,
        });
    }

    pub fn write(&mut self, output: PathBuf) -> anyhow::Result<()> {
        fs::create_dir_all(output.parent().unwrap())?;
        let mut output_file = File::create(output.clone())?;
        self.mbr.write_into(&mut output_file)?;

        assert_eq!(output_file.stream_position()?, 512);
        io::copy(&mut self.kernel_loader, &mut output_file).unwrap();
        output_file.seek(SeekFrom::Start(((self.kernel_loader.metadata()?.len() - 1) / 512 + 2) * 512))?;
        self.disk.seek(SeekFrom::Start(0))?;
        io::copy(&mut self.disk, &mut output_file).unwrap();
        return Ok(());
    }
}

fn build(name: &str, path: &str, target: &str, profile: &str, out_dir: &Path) -> PathBuf {
    //Force rebuilds
    println!("cargo:rerun-if-changed={}", path);

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

struct TestWriter {
    array: [u8; 512],
    head: u64
}

impl Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for i in 0..buf.len() {
            self.array[i+(self.head as usize)] = buf[i];
        }
        return Ok(buf.len());
    }

    fn flush(&mut self) -> io::Result<()> {
        self.array = [0; 512];
        return Ok(());
    }
}

impl Seek for TestWriter {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(head) => self.head = head,
            _ => panic!("Lazy!")
        }
        return Ok(self.head);
    }
}