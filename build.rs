use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use fs::partition::Partition;

fn main() {
    let path = env::var("OUT_DIR").unwrap();
    let path = Path::new(path.as_str());
    let bios_path = path.join("bios");

    //Build BIOS image
    BIOSImage::new(
        PathBuf::from(env::var("CARGO_BIN_FILE_x86_64-BOOTLOADER-BIOS-STAGE-1_x86_64-bootloader-bios-stage-1").unwrap()),
        PathBuf::from(env::var("CARGO_BIN_FILE_x86_64-BOOTLOADER-BIOS-STAGE-2_x86_64-bootloader-bios-stage-2").unwrap()),
        |partition| {}).write(bios_path.join("bios.img"));
}

struct BIOSImage {}

impl BIOSImage {
    pub fn new(bootloader: PathBuf, kernelloader: PathBuf, setup: fn(Partition<File>)) -> Self {
        return BIOSImage {};
    }

    pub fn write(&self, output: PathBuf) {

    }
}