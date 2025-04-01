#![no_std]
#![no_main]
#![feature(naked_functions)]

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};
use core::cell::UnsafeCell;

// Kernel header structure - must match the bootloader's expectation
#[repr(C)]
struct KernelHeader {
    // Magic number to verify we have a valid kernel
    magic: u64,
    // Entry point address - relative to the start of the kernel
    entry_point: u64,
    // Size of the kernel in memory
    size: u64,
}

// Define our kernel's magic number: "GRAPEOS" in hex
const KERNEL_MAGIC: u64 = 0x4752415045_4F53_00;

// Global state for our simple event system
static SYSTEM_RUNNING: AtomicBool = AtomicBool::new(false);

// Event types for our reactive OS
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum EventType {
    SystemInit = 0,
    KeyPress = 1,
    Timer = 2,
    SystemShutdown = 3,
}

// A basic event structure
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct Event {
    event_type: EventType,
    priority: u8,  // Higher number = higher priority
    timestamp: u64,
}

// Create a static kernel header at the start of the kernel
// Using unsafe for attributes that require it in newer Rust nightly
#[unsafe(link_section = ".kernel_header")]
#[unsafe(no_mangle)]
static KERNEL_HEADER: KernelHeader = KernelHeader {
    magic: KERNEL_MAGIC,
    // Just store a fixed offset for now - we'll calculate the real one at link time
    entry_point: 0x1000, // This will be a simple offset from the header 
    size: 0, // Filled in by the build process
};

// TextDisplay trait for console output
trait TextDisplay {
    fn clear(&mut self);
    fn write_str(&mut self, s: &str);
    fn write_char(&mut self, c: char);
}

// A simple console that writes directly to VGA memory
struct VgaConsole {
    buffer: *mut u16,
    row: UnsafeCell<usize>,
    col: UnsafeCell<usize>,
}

// Mark VgaConsole as safe to share between threads
// (though we won't be using threads in our simple kernel)
unsafe impl Sync for VgaConsole {}

// Implementation of VGA console
impl VgaConsole {
    // Initialize a new VGA console
    fn new() -> Self {
        Self {
            buffer: 0xB8000 as *mut u16, // Standard VGA buffer address
            row: UnsafeCell::new(0),
            col: UnsafeCell::new(0),
        }
    }
    
    // Move to the next line
    fn newline(&mut self) {
        // Get current row and col
        let row = unsafe { *self.row.get() };
        
        // Reset column to 0
        unsafe { *self.col.get() = 0 };
        
        // Increment row
        let new_row = row + 1;
        
        if new_row >= 25 {
            // Simple scrolling - move everything up one line
            for y in 1..25 {
                for x in 0..80 {
                    unsafe {
                        let current = *self.buffer.add(y * 80 + x);
                        *self.buffer.add((y - 1) * 80 + x) = current;
                    }
                }
            }
            // Clear the last line
            for x in 0..80 {
                unsafe { 
                    *self.buffer.add(24 * 80 + x) = 0x0720; // Space with gray on black
                }
            }
            unsafe { *self.row.get() = 24 };
        } else {
            unsafe { *self.row.get() = new_row };
        }
    }
}

// Implement TextDisplay for VgaConsole
impl TextDisplay for VgaConsole {
    fn clear(&mut self) {
        for i in 0..(80 * 25) {
            unsafe { 
                *self.buffer.add(i) = 0x0720; // Space with gray on black
            }
        }
        
        // Reset cursor position
        unsafe {
            *self.row.get() = 0;
            *self.col.get() = 0;
        }
    }
    
    fn write_char(&mut self, c: char) {
        // Get current row and col
        let row = unsafe { *self.row.get() };
        let col = unsafe { *self.col.get() };
        
        match c {
            '\n' => self.newline(),
            '\r' => unsafe { *self.col.get() = 0 },
            _ => {
                // Write the character to the buffer
                let char_with_attr = 0x0700 | (c as u16); // Gray on black
                unsafe { 
                    *self.buffer.add(row * 80 + col) = char_with_attr;
                }
                
                // Advance cursor
                let new_col = col + 1;
                if new_col >= 80 {
                    self.newline();
                } else {
                    unsafe { *self.col.get() = new_col };
                }
            }
        }
    }
    
    fn write_str(&mut self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }
}

// Event dispatcher - the heart of our reactive system
struct EventDispatcher {
    console: VgaConsole,
}

impl EventDispatcher {
    fn new() -> Self {
        let console = VgaConsole::new();
        Self { console }
    }
    
    // Dispatch an event to the appropriate handler
    fn dispatch_event(&mut self, event: Event) {
        // In a real system, we would have a queue and priority-based scheduling
        match event.event_type {
            EventType::SystemInit => self.handle_system_init(event),
            EventType::KeyPress => self.handle_key_press(event),
            EventType::Timer => self.handle_timer(event),
            EventType::SystemShutdown => self.handle_system_shutdown(event),
        }
    }
    
    // Event handlers
    fn handle_system_init(&mut self, _event: Event) {
        self.console.clear();
        self.console.write_str("GrapeOS Kernel Initialized\n\r");
        self.console.write_str("---------------------------\n\r");
        self.console.write_str("Welcome to the Reactive Operating System!\n\r");
        
        SYSTEM_RUNNING.store(true, Ordering::SeqCst);
    }
    
    fn handle_key_press(&mut self, _event: Event) {
        self.console.write_str("Key press detected\n\r");
    }
    
    fn handle_timer(&mut self, _event: Event) {
        self.console.write_str(".");
    }
    
    fn handle_system_shutdown(&mut self, _event: Event) {
        self.console.write_str("\n\rShutting down...\n\r");
        SYSTEM_RUNNING.store(false, Ordering::SeqCst);
    }
    
    // Create a new event
    fn create_event(&self, event_type: EventType, priority: u8) -> Event {
        Event {
            event_type,
            priority,
            timestamp: 0, // For now, just use a dummy timestamp
        }
    }
}

// Kernel main function - called by the bootloader
#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    // Initialize the core system
    let mut dispatcher = EventDispatcher::new();
    
    // Send system initialization event
    let init_event = dispatcher.create_event(EventType::SystemInit, 10);
    dispatcher.dispatch_event(init_event);
    
    // Main event loop - in a real system, this would be driven by hardware events
    for _i in 0..10 {
        // Create a timer event every second
        let timer_event = dispatcher.create_event(EventType::Timer, 5);
        dispatcher.dispatch_event(timer_event);
        
        // Simulate a delay - in a real system, this would be handled by the CPU's timer
        for _ in 0..5000000 {
            // Simple delay loop
            core::hint::spin_loop();
        }
    }
    
    // Simulate a key press
    let key_event = dispatcher.create_event(EventType::KeyPress, 8);
    dispatcher.dispatch_event(key_event);
    
    // Shut down the system
    let shutdown_event = dispatcher.create_event(EventType::SystemShutdown, 10);
    dispatcher.dispatch_event(shutdown_event);
    
    // In a real OS, we would power off the machine here
    loop {
        // Halt the CPU until the next interrupt
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

// Required panic handler
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // In a real kernel, we would log the panic and possibly display it
    loop {
        core::hint::spin_loop();
    }
}