use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, exit};

/// Create a FAT32 disk image for testing the filesystem
fn create_test_disk(disk_path: &str) {

    // Only create if it doesn't exist
    if Path::new(disk_path).exists() {
        return;
    }

    println!("Creating test FAT32 disk image: {disk_path}");

    // Create a 32MB disk image
    let size = 32 * 1024 * 1024;
    let mut file = fs::File::create(disk_path).expect("Failed to create disk image");
    file.set_len(size).expect("Failed to set disk size");

    // Format as FAT32 using mkfs.fat if available
    let status = Command::new("mkfs.fat")
        .args(["-F", "32", "-n", "EXOTEST", disk_path])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("Formatted disk as FAT32");

            // Mount and add a test file (Linux only)
            let mount_dir = "/tmp/exo_mount";
            let _ = fs::create_dir_all(mount_dir);

            // Try to mount and add test files (requires sudo, may fail)
            let mount_status = Command::new("sudo")
                .args(["mount", "-o", "loop", disk_path, mount_dir])
                .status();

            if mount_status.map(|s| s.success()).unwrap_or(false) {
                // Create a test file
                let test_file = format!("{mount_dir}/HELLO.TXT");
                if let Ok(mut f) = fs::File::create(&test_file) {
                    let _ = f.write_all(b"Hello from the Exokernel filesystem!\n");
                }

                // Create a directory with a file
                let _ = fs::create_dir(format!("{mount_dir}/DOCS"));
                if let Ok(mut f) = fs::File::create(format!("{mount_dir}/DOCS/README.TXT")) {
                    let _ = f.write_all(b"This is a test file in a subdirectory.\n");
                }

                // Unmount
                let _ = Command::new("sudo").args(["umount", mount_dir]).status();
                println!("Added test files to disk image");
            } else {
                println!("Note: Could not mount disk to add test files (may need sudo)");
            }
        }
        _ => {
            println!("Warning: mkfs.fat not available, disk will be unformatted");
        }
    }
}

fn main() {
    let uefi_path = env!("UEFI_PATH");
    let bios_path = env!("BIOS_PATH");

    let args: Vec<String> = env::args().collect();
    let prog = &args[0];

    // Choose whether to start the UEFI or BIOS image
    let uefi = match args.get(1).map(|s| s.to_lowercase()).as_deref() {
        Some("uefi") => true,
        Some("bios") => false,
        Some("-h") | Some("--help") => {
            println!("Usage: {prog} [uefi|bios]");
            println!("  uefi  - boot using OVMF (UEFI)");
            println!("  bios  - boot using legacy BIOS (default)");
            exit(0);
        }
        None => true, // Default to UEFI (BIOS currently broken in bootloader crate)
        _ => {
            eprintln!("Usage: {prog} [uefi|bios]");
            exit(1);
        }
    };

    // Create test disk for filesystem testing
    let disk_path = "target/test_disk.img";
    create_test_disk(disk_path);

    // Cross-platform QEMU executable detection
    let qemu_exe = if cfg!(windows) {
        env::var("qemu")
            .map(|path| format!("{}\\qemu-system-x86_64.exe", path))
            .unwrap_or_else(|_| "qemu-system-x86_64.exe".to_string())
    } else {
        "qemu-system-x86_64".to_string()
    };

    let mut cmd = Command::new(&qemu_exe);
    // Open a window for QEMU display
    cmd.arg("-serial").arg("stdio");  // Serial output still goes to terminal
    cmd.arg("-m").arg("256M");        // 256MB RAM

    // Add virtio-blk device using PCI transport
    // The kernel will enumerate PCI and find the device's BAR address
    cmd.arg("-device")
        .arg("virtio-blk-pci,drive=disk0");
    cmd.arg("-drive")
        .arg(format!("id=disk0,if=none,format=raw,file={disk_path}"));

    if uefi {
        let prebuilt = Prebuilt::fetch(Source::LATEST, "target/ovmf")
            .expect("Failed to fetch OVMF prebuilt");

        let code = prebuilt.get_file(Arch::X64, FileType::Code);
        let vars = prebuilt.get_file(Arch::X64, FileType::Vars);

        cmd.arg("-drive")
            .arg(format!("format=raw,file={uefi_path}"));
        cmd.arg("-drive").arg(format!(
            "if=pflash,format=raw,unit=0,file={},readonly=on",
            code.display()
        ));
        cmd.arg("-drive").arg(format!(
            "if=pflash,format=raw,unit=1,file={},snapshot=on",
            vars.display()
        ));
        println!("Running UEFI image: {uefi_path}");
    } else {
        cmd.arg("-drive")
            .arg(format!("format=raw,file={bios_path}"));
        println!("Running BIOS image: {bios_path}");
    }

    println!("Virtio block device attached: {disk_path}");
    let mut child = cmd.spawn().expect("Failed to start qemu-system-x86_64");
    child.wait().expect("Failed to wait on QEMU");
}
