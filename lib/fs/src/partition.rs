use crate::{Read, Seek, SeekFrom, Write};
use crate::structure::{DirectoryHeader, FileHeader};

#[cfg(not(feature = "read-only"))]
pub struct Partition<T: Read + Write + Seek> {
    writer: T,
}

#[cfg(feature = "read-only")]
pub struct Partition<T: Read + Seek> {
    reader: T
}

#[cfg(not(feature = "read-only"))]
impl<T: Read + Write + Seek> Partition<T> {
    pub fn new(writer: T) -> Self {
        return Partition {
            writer
        }
    }

    pub fn open(writer: T) -> Self {
        return Partition {
            writer
        }
    }
}

#[cfg(feature = "read-only")]
impl<T: Read + Seek> Partition<T> {
    pub fn open(reader: T) -> Self {
        return Partition {
            reader
        }
    }

    pub fn read(&mut self, file: &str) -> Option<FileHeader> {
        self.reader.seek(SeekFrom::Start(0));
        let mut buffer: [u8; 512] = [0; 512];
        let mut current = 0;
        let mut last = DirectoryHeader::read(&mut self.reader, &mut buffer).expect("No root!");
        while current < file.len() {
            let i = 0;
            let start = current;
            for character in file.chars().skip(start) {
                if character == '\\' || character == '/' {
                    current = i;
                    break;
                }
                current = i;
            }

            last.
        }
        return None;
    }
}