use std::env;

fn main() {
    // read env variables that were set in build script
    let bios_path = env::var("BIOS_PATH").unwrap();

    let mut cmd = std::process::Command::new(env::var("qemu").unwrap().to_owned() + "\\qemu-system-x86_64.exe");

    cmd.arg("-drive").arg(format!("format=raw,file={}", bios_path));
    cmd.arg("-serial").arg("stdio");

    //QEMU GDB debug
    //cmd.arg("-s").arg("-S");
    //BIOS debug
    //cmd.arg("-chardev").arg("stdio,id=seabios");
    //cmd.arg("-device").arg("isa-debugcon,iobase=0x402,chardev=seabios");
    //cmd.arg("-trace").arg("enable=ide_*");

    println!("Running {}", bios_path);
    let mut child = cmd.spawn().unwrap();
    child.wait().unwrap();
}
