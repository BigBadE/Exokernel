use std::env;

fn main() {
    // read env variables that were set in build script
    let bios_path = env::var("BIOS_PATH").unwrap();

    let mut cmd = std::process::Command::new(env::var("qemu").unwrap().to_owned() + "\\qemu-system-x86_64.exe");

    cmd.arg("-drive").arg(format!("format=raw,file={}", bios_path));
    cmd.arg("-serial").arg("stdio");
    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
