#![cfg_attr(not(std), no_std)]

pub mod partition;
pub mod structure;

#[cfg(feature = "std")]
extern crate std;

pub trait Read {
    fn read(&mut self, buffer: &mut [u8]) -> Option<usize>;

    fn read_len(&mut self, buffer: &mut [u8], len: usize) -> Option<usize>;
}

pub trait Write {
    fn write() {

    }
}

pub trait Seek {
    fn seek(&mut self, seeking: SeekFrom);
}

pub enum SeekFrom {
    Start(u64),
    End(u64)
}

#[cfg(feature = "std")]
impl Read for std::fs::File {
    fn read(&mut self, buffer: &mut [u8]) -> Option<usize> {
        return std::io::Read::read(self, buffer).ok();
    }

    fn read_len(&mut self, buffer: &mut [u8], len: usize) -> Option<usize> {
        return std::io::Read::read(self, &mut buffer[0..len]).ok();
    }
}

#[cfg(feature = "std")]
impl Write for std::fs::File {

}

#[cfg(feature = "std")]
impl Seek for std::fs::File {
    fn seek(&mut self, position: u64) {
        panic!("No seeking on files!");
    }
}