use crate::{Read, Seek, Write};
use crate::structure::DirectoryHeader;

#[cfg(not(feature = "readonly"))]
pub struct Partition<T: Read + Write + Seek> {
    writer: T,
}

#[cfg(not(feature = "readonly"))]
impl<T: Read + Write + Seek> Partition<T> {
    pub fn new(writer: T) -> Self {
        return Partition {
            writer
        }
    }
}

pub struct ReadPartition<T: Read + Seek> {
    reader: T
}

impl<T: Read + Seek> ReadPartition<T> {
    pub fn new(reader: T) -> Self {
        return ReadPartition {
            reader
        }
    }
}