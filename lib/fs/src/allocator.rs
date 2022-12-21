#[cfg(not(feature = "readonly"))]
pub struct Allocator {
    end: *mut u8
}