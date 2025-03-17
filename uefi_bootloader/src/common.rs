// uefi_bootloader/src/common.rs

#[repr(C)]
pub struct BootInfo {
    pub memory_map_addr: u64,
    pub memory_map_size: usize,
    pub memory_map_entry_size: usize,
    pub framebuffer_addr: u64,
    pub framebuffer_width: usize,
    pub framebuffer_height: usize,
    pub framebuffer_stride: usize,
}

impl BootInfo {
    pub fn new(memory_map_addr: u64, memory_map_size: usize, memory_map_entry_size: usize) -> Self {
        Self {
            memory_map_addr,
            memory_map_size,
            memory_map_entry_size,
            framebuffer_addr: 0,
            framebuffer_width: 0,
            framebuffer_height: 0,
            framebuffer_stride: 0,
        }
    }
}
