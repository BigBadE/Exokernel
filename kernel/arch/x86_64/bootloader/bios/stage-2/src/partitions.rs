#[repr(C)]
pub struct PartitionTableEntry {
    ignored: u64,
    pub sector: u32,
    pub length: u32
}