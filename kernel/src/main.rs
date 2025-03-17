#![no_std]
#![no_main]

use core::panic::PanicInfo;

// BGR color constants
const COLOR_BLUE: u32 = 0x00FF0000;  // Blue in BGR format

#[repr(C)]
pub struct BootInfo {
    memory_map_addr: u64,
    memory_map_size: usize,
    memory_map_entry_size: usize,
    framebuffer_addr: u64,
    framebuffer_width: usize,
    framebuffer_height: usize,
    framebuffer_stride: usize,
}

// Main kernel entry point
#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    // Fill the screen with blue
    if boot_info.framebuffer_addr != 0 {
        let fb = unsafe {
            core::slice::from_raw_parts_mut(
                boot_info.framebuffer_addr as *mut u32,
                boot_info.framebuffer_width * boot_info.framebuffer_height
            )
        };
        
        // Simple approach - fill entire screen with blue
        for i in 0..fb.len() {
            fb[i] = COLOR_BLUE;
        }
    }
    
    // Hang forever
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

// Panic handler
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}