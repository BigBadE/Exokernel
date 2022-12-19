#![cfg_attr(not(std), no_std)]

pub mod partition;
pub mod structure;
pub mod allocator;
pub mod bands;
pub mod helper;

#[cfg(feature = "std")]
extern crate std;

pub trait Read {
    fn read(&mut self, buffer: &mut [u8]) -> Option<usize>;
}

pub trait Write {
    fn write() {

    }
}

pub trait Seek {
    fn seek() {

    }
}

#[cfg(feature = "std")]
impl Read for std::fs::File {
    fn read(&mut self, buffer: &mut [u8]) -> Option<usize> {
        return std::io::Read::read(self, buffer).ok();
    }
}

#[cfg(feature = "std")]
impl Write for std::fs::File {

}

#[cfg(feature = "std")]
impl Seek for std::fs::File {

}