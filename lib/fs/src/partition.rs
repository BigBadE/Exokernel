use crate::{Read, Seek, Write};
use crate::structure::DirectoryHeader;

pub struct Partition<T: Read + Write + Seek> {
    writer: T,
    root: DirectoryHeader
}

impl<T: Read + Write + Seek> Partition<T> {
    pub fn new(writer: T) -> Self {
        return Partition {
            writer,
            root: DirectoryHeader::root()
        }
    }
}