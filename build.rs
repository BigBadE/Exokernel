use std::fs::File;
use std::path::Path;
use fs::Write;

fn main() {
    let path = env!("OUT_DIR");
    let bios_path = Path::new(path).join("bios");

    //Build BIOS image
}

struct BIOSImage {

}

impl BIOSImage {
    pub fn new() -> Self {
        return BIOSImage {

        }
    }
}