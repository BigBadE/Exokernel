#![cfg_attr(std, no_std)]

pub mod partition;
pub mod structure;

pub trait Read {
    fn read() {

    }
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

}

#[cfg(feature = "std")]
impl Write for std::fs::File {

}

#[cfg(feature = "std")]
impl Seek for std::fs::File {

}