use crate::structure::Security::OwnerRead;

pub struct DirectoryHeader {
    pub sector: u64,
    pub security: u16,
    pub name: [char; 256]
}

pub struct FileHeader {
    pub sector: u64,
    pub security: u16,
    pub info: u8,
    pub name: [char; 256]
}

impl DirectoryHeader {
    pub fn root() -> Self {
        let mut name = [0 as char; 256];
        for i in 0..4 {
            name[i] = "root".as_bytes()[i] as char;
        }
        return DirectoryHeader {
            sector: 1,
            name,
            security: OwnerRead as u16
        }
    }
}

pub enum Types {
    File(FileHeader),
    Directory(DirectoryHeader)
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
    UserExecute = 0b000_000_001
}