// kernel/src/main.rs
#![no_std]
#![no_main]

use core::panic::PanicInfo;

// The boot info structure must match the one in the bootloader
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

// This is our kernel entry point
#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    // Draw a simple pattern on the framebuffer to show we're alive
    let fb = unsafe {
        core::slice::from_raw_parts_mut(
            boot_info.framebuffer_addr as *mut u32,
            boot_info.framebuffer_width * boot_info.framebuffer_height,
        )
    };

    // Define colors
    let red = 0x00FF0000;
    let green = 0x0000FF00;
    let blue = 0x000000FF;
    let white = 0x00FFFFFF;

    // Fill screen with a gradient
    for y in 0..boot_info.framebuffer_height {
        for x in 0..boot_info.framebuffer_width {
            let offset = y * boot_info.framebuffer_stride + x;
            
            if offset < fb.len() {
                // Create a color pattern
                let color = match (x / 80, y / 80) {
                    (0, _) => red,
                    (1, _) => green,
                    (2, _) => blue,
                    _ => white,
                };
                
                fb[offset] = color;
            }
        }
    }

    // Hang forever - we have nowhere to return to
    loop {
        // CPU-specific way to halt
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

// This function is called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}