// uefi_bootloader/src/common.rs
use uefi::prelude::*;
use uefi::table::boot::MemoryDescriptor;

// Structure to pass information to the kernel
#[repr(C)]
pub struct BootInfo {
    memory_map_addr: u64,
    memory_map_size: usize,
    memory_map_entry_size: usize,
    pub framebuffer_addr: u64,
    pub framebuffer_width: usize,
    pub framebuffer_height: usize,
    pub framebuffer_stride: usize,
}

impl BootInfo {
    pub fn new(
        memory_map_addr: u64,
        memory_map_size: usize,
        memory_map_entry_size: usize,
    ) -> Self {
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
    
    pub fn memory_map<'a>(&self) -> &'a [MemoryDescriptor] {
        unsafe {
            core::slice::from_raw_parts(
                self.memory_map_addr as *const MemoryDescriptor,
                self.memory_map_size / self.memory_map_entry_size,
            )
        }
    }
}