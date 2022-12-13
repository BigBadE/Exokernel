# Building

Rust core library downloaded for your target (example for x86_64)

```rustup target add x86_64-unknown-none```

Nightly channel

```rustup override set nightly```

Install QEMU and set the environmental variable QEMU to the path to its
installation folder.

# Structure

src - Automatically runs the kernel in QEMU
kernel - Core exokernel
kernel/arch - Arch-specific code
(arch)/bootloader/bios - BIOS bootloader