pub struct MBR {

}

impl MBR {

}

pub struct PartitionTable {
    entries: [PartitionEntry; 4]
}

pub struct PartitionEntry {
    //0x0 = not bootable, 0x80 = bootable
    bootable: u8,
    lba: u32,
    total_sectors: u32
}

impl PartitionEntry {
    pub fn new(is_bootable: bool, lba: u32, total_sectors: u32) -> Self {
        let mut bootable = 0x0;
        if is_bootable {
            bootable = 0x80;
        }
        return PartitionEntry {
            bootable,
            lba,
            total_sectors
        }
    }
}