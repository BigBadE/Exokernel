use crate::{Read, Seek, Write};
use crate::structure::DirectoryHeader;

pub struct Partition<T: Read + Write + Seek> {
    writer: T,
    readonly: bool
}

impl<T: Read + Write + Seek> Partition<T> {
    pub fn new(writer: T, readonly: bool) -> Self {
        return Partition {
            writer,
            readonly
        }
    }
}