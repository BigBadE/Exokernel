fn main() {
    // read env variables that were set in build script
    let bios_path = env!("BIOS_PATH");

    let mut cmd = std::process::Command::new(env!("qemu").to_owned() + "\\qemu-system-x86_64");

    cmd.arg("-drive").arg(format!("format=raw,file={}", bios_path));
    cmd.arg("-serial").arg("stdio");
    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
