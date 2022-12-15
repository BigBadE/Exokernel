use crate::helper::LoadingLinkedList;
use crate::{Read, Seek, Write};
use crate::structure::Types;

pub struct BandTop<'a, T: Read + Write + Seek> {
    file_loader: LoadingLinkedList<'a, T, Types>,
    writer: &'a T
}

impl<'a, T: Read + Write + Seek> BandTop<'a, T> {
    //ID is 0-9
    pub fn new(writer: &'a T, drive_sectors: u16, id: u8) -> Self {
        return BandTop {
            file_loader: LoadingLinkedList::new(writer, read_next::<T>),
            writer
        }
    }
}

fn read_next<T: Read + Write + Seek>(writer: &T) -> Types {

}