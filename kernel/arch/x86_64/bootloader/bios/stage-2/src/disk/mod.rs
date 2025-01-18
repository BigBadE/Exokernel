pub mod dap;
pub mod disk;
pub mod fat;
pub mod file;

pub trait Read {
    unsafe fn read_exact(&mut self, len: usize) -> &[u8];
    fn read_exact_into(&mut self, len: usize, buf: &mut dyn AlignedBuffer);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekFrom {
    Start(u64),
}

pub trait Seek {
    fn seek(&mut self, pos: SeekFrom) -> u64;
}

#[repr(align(2))]
pub struct AlignedArrayBuffer<const LEN: usize> {
    pub buffer: [u8; LEN],
}

pub trait AlignedBuffer {
    fn slice(&self) -> &[u8];
    fn slice_mut(&mut self) -> &mut [u8];
}

impl<const LEN: usize> AlignedBuffer for AlignedArrayBuffer<LEN> {
    fn slice(&self) -> &[u8] {
        &self.buffer[..]
    }
    fn slice_mut(&mut self) -> &mut [u8] {
        &mut self.buffer[..]
    }
}
