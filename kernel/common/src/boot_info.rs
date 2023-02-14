pub struct BootInfo {
    pub video: VideoInfo
}

pub struct VideoInfo {
    pub mode: u16,
    pub width: u16,
    pub height: u16,
    pub framebuffer: u32,
    pub bytes_per_pixel: u16,
    pub bytes_per_line: u16
}