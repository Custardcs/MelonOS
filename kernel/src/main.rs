#![no_std]
#![no_main]

use core::panic::PanicInfo;

// Color constants for debugging
const COLOR_RED:   u32 = 0x00FF0000;
const COLOR_GREEN: u32 = 0x0000FF00;
const COLOR_BLUE:  u32 = 0x000000FF;
const COLOR_WHITE: u32 = 0x00FFFFFF;

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

// Extremely verbose debugging function
fn debug_framebuffer(boot_info: &'static BootInfo) {
    // Log framebuffer details via color patterns
    let fb = unsafe {
        core::slice::from_raw_parts_mut(
            boot_info.framebuffer_addr as *mut u32,
            boot_info.framebuffer_width * boot_info.framebuffer_height
        )
    };

    // Diagnostic color pattern
    for y in 0..boot_info.framebuffer_height {
        for x in 0..boot_info.framebuffer_width {
            let offset = y * boot_info.framebuffer_stride + x;
            
            // Create a diagnostic grid
            let color = match (x / 80, y / 80) {
                (0, 0) => COLOR_RED,    // Top-left: red
                (1, 0) => COLOR_GREEN,  // Top-middle: green
                (2, 0) => COLOR_BLUE,   // Top-right: blue
                _ => COLOR_WHITE        // Rest: white
            };

            if offset < fb.len() {
                fb[offset] = color;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    // Validate boot info
    if boot_info.framebuffer_addr == 0 {
        // Emergency red screen if no framebuffer
        let emergency_fb = unsafe {
            core::slice::from_raw_parts_mut(
                0 as *mut u32,
                1024 * 768  // Assume standard resolution
            )
        };
        
        for i in 0..1024*768 {
            emergency_fb[i] = COLOR_RED;
        }
        
        loop {
            // Halt with emergency indicator
            unsafe { core::arch::asm!("hlt"); }
        }
    }

    // Perform diagnostic display
    debug_framebuffer(boot_info);

    // Hang forever with debug information visible
    loop {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("hlt");
        }

        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfe");
        }
    }
}

// Panic handler with color-coded emergency
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Emergency blue screen on panic
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}