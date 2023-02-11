use std::{mem, ptr};
use crate::{Read, Seek, SeekFrom};
use crate::structure::Types::{Directory, File};

const OFFSET: usize = mem::size_of::<GenericHeader>();

#[repr(C)]
pub struct DirectoryHeader {
    generic_header: GenericHeader,
    first_start: u64,
    second_start: u64,
}

impl DirectoryHeader {
    pub fn read<T: Read>(reader: &mut T, buffer: &mut [u8; 512]) -> Option<DirectoryHeader> {
        match reader.read(buffer) {
            None => return None,
            _ => {}
        }
        reader.read_len(buffer, 8 + 256 + 8 + 2 + 8 + 8 + 8);

        return Some(DirectoryHeader::open(buffer));
    }

    fn open(buffer: &[u8; 512]) -> DirectoryHeader {
        unsafe {
            return DirectoryHeader {
                generic_header: GenericHeader::new(buffer),
                first_start: ptr::read(buffer[OFFSET] as *const u64),
                second_start: ptr::read(buffer[OFFSET + 8] as *const u64),
            };
        }
    }

    pub fn find_child<T: Read + Seek>(&self, reader: &mut T, buffer: &mut [u8; 512], name: &str) -> Option<Types> {
        let mut child = self.first_start;
        'outer: loop {
            if child == 0 {
                return None;
            }
            reader.seek(SeekFrom::Start(child & 0x7FFFFFFFFFFFFFFF));
            reader.read(buffer);
            let header = GenericHeader::new(buffer);
            let mut i = 0;
            for character in name.bytes() {
                if header.file_name[i] != character {
                    child = header.next_start;
                    continue 'outer;
                }
                i += 1;
            }
            if i == name.len() {
                return if child & 0x8000000000 == 1 {
                    Some(Directory(DirectoryHeader::open(buffer)))
                } else {
                    Some(File(FileHeader::open(buffer)))
                }
            }
        }
    }
}

#[repr(C)]
pub struct FileHeader {
    generic_header: GenericHeader,
    file_size: u64
}

impl FileHeader {
    pub fn read<T: Read>(reader: &mut T, buffer: &mut [u8; 512]) -> Option<FileHeader> {
        match reader.read(buffer) {
            None => return None,
            _ => {}
        }
        reader.read_len(buffer, 8 + 256 + 2 + 8 + 8 + 8);
        return Some(FileHeader::open(buffer));
    }

    fn open(buffer: &[u8; 512]) -> Self {
        unsafe {
            return FileHeader {
                generic_header: GenericHeader::new(buffer),
                file_size: ptr::read(buffer[OFFSET] as *const u64),
            };
        }
    }
}

#[repr(C)]
pub struct GenericHeader {
    next_start: u64,
    file_name: [u8; 256],
    security: u16,
    parent_start: u64,
    second_next_start: u64,
}

impl GenericHeader {
    fn new(buffer: &[u8; 512]) -> Self {
        unsafe {
            return GenericHeader {
                next_start: ptr::read(buffer[0] as *const u64),
                file_name: ptr::read(buffer[0x8] as *const [u8; 256]),
                security: ptr::read(buffer[0x108] as *const u16),
                parent_start: ptr::read(buffer[0x10A] as *const u64),
                second_next_start: ptr::read(buffer[0x112] as *const u64),
            }
        }
    }
}

pub enum Types {
    File(FileHeader),
    Directory(DirectoryHeader),
}

pub enum Security {
    OwnerRead = 0b100_000_000,
    OwnerWrite = 0b010_000_000,
    OwnerExecute = 0b001_000_000,
    GroupRead = 0b000_100_000,
    GroupWrite = 0b000_010_000,
    GroupExecute = 0b000_001_000,
    UserRead = 0b000_000_100,
    UserWrite = 0b000_000_010,
    UserExecute = 0b000_000_001,
}