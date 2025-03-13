// uefi_bootloader/src/main.rs
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use uefi::prelude::*;
use uefi::table::boot::{MemoryType, AllocateType};
use uefi::proto::media::file::{File, FileMode, FileAttribute, FileInfo};
use uefi::proto::media::fs::SimpleFileSystem;
use log::info;

// Architecture-specific modules
#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "aarch64")]
mod aarch64;

// Common module for shared functionality
mod common;

// This function is called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Entry point for the UEFI bootloader
// Updated efi_main function
#[entry]
fn efi_main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    // Initialize UEFI services
    uefi_services::init(&mut system_table).expect("Failed to initialize UEFI services");
    
    // Print architecture-specific welcome message
    #[cfg(target_arch = "x86_64")]
    info!("64-bit x86_64 UEFI bootloader started");
    
    #[cfg(target_arch = "aarch64")]
    info!("64-bit ARM64 UEFI bootloader started");
    
    // Set up graphics
    let mut boot_info = match setup_graphics(&mut system_table) {
        Some(info) => info,
        None => {
            info!("Failed to set up graphics");
            return Status::DEVICE_ERROR;
        }
    };
    
    // Load the kernel specific to this architecture
    #[cfg(target_arch = "x86_64")]
    let kernel_path = "\\EFI\\KERNEL\\KERNEL_X64.ELF";
    
    #[cfg(target_arch = "aarch64")]
    let kernel_path = "\\EFI\\KERNEL\\KERNEL_ARM64.ELF";
    
    // Load the appropriate kernel
    match load_kernel(image_handle, &mut system_table, kernel_path) {
        Ok(kernel_entry) => {
            info!("Kernel loaded successfully, jumping to entry point");
            
            // Perform architecture-specific preparations
            #[cfg(target_arch = "x86_64")]
            x86_64::prepare_jump_to_kernel(&mut system_table);
            
            #[cfg(target_arch = "aarch64")]
            aarch64::prepare_jump_to_kernel(&mut system_table);
            
            // Prepare boot parameters
            let boot_params_size = core::mem::size_of::<common::BootInfo>();
            let boot_params_addr = system_table
                .boot_services()
                .allocate_pool(MemoryType::RUNTIME_DATA, boot_params_size)
                .expect("Failed to allocate memory for boot parameters");
            
            // Exit UEFI boot services before jumping to the kernel
            let (_runtime, memory_map) = system_table
                .exit_boot_services(image_handle, &mut [])
                .expect("Failed to exit boot services");
            
            // Update boot info with memory map details
            boot_info.memory_map_addr = memory_map.buffer().as_ptr() as u64;
            boot_info.memory_map_size = memory_map.len() * memory_map.entry_size();
            boot_info.memory_map_entry_size = memory_map.entry_size();
            
            // Copy boot info to allocated memory
            unsafe {
                core::ptr::write_volatile(
                    boot_params_addr as *mut common::BootInfo,
                    boot_info,
                );
            }
            
            // Jump to the kernel, passing the boot info structure
            let kernel_entry: fn(*const common::BootInfo) -> ! = 
                unsafe { core::mem::transmute(kernel_entry) };
            
            kernel_entry(boot_params_addr as *const common::BootInfo);
        }
        Err(status) => {
            info!("Failed to load kernel: {:?}", status);
            return status;
        }
    }
    
    // We should never reach here
    Status::SUCCESS
}

// Add to main.rs before jumping to the kernel
fn setup_graphics(system_table: &mut SystemTable<Boot>) -> Option<common::BootInfo> {
    let boot_services = system_table.boot_services();
    
    // Get the GOP (Graphics Output Protocol)
    let gop_handle = boot_services
        .locate_protocol::<uefi::proto::console::gop::GraphicsOutput>()
        .ok()?;
    
    let gop = unsafe { &mut *gop_handle.get() };
    
    // Get current graphics mode info
    let mode_info = gop.current_mode_info();
    let framebuffer = gop.frame_buffer();
    
    // Create a boot info structure
    let mut boot_info = common::BootInfo::new(0, 0, 0); // We'll fill memory map details later
    
    // Set framebuffer info
    boot_info.framebuffer_addr = framebuffer.as_mut_ptr() as u64;
    boot_info.framebuffer_width = mode_info.resolution().0;
    boot_info.framebuffer_height = mode_info.resolution().1;
    boot_info.framebuffer_stride = mode_info.stride();
    
    Some(boot_info)
}

// Function to load the kernel from the boot volume
fn load_kernel(
    image_handle: Handle, 
    system_table: &mut SystemTable<Boot>,
    kernel_path: &str
) -> Result<u64, Status> {
    // Get the file system protocol
    let boot_services = system_table.boot_services();
    
    // Open the UEFI file system of the boot drive
    let fs = boot_services
        .locate_protocol::<SimpleFileSystem>()
        .map_err(|_| Status::NOT_FOUND)?;
    
    let fs = unsafe { &mut *fs.get() };
    
    // Open the root directory
    let mut root = fs.open_volume().map_err(|_| Status::DEVICE_ERROR)?;
    
    // Open the kernel file
    let mut kernel_file = root
        .open(
            kernel_path,
            FileMode::Read,
            FileAttribute::empty(),
        )
        .map_err(|_| Status::NOT_FOUND)?
        .into_regular_file()
        .ok_or(Status::INVALID_PARAMETER)?;
    
    // Get the kernel file size
    let file_info_size = kernel_file
        .get_info::<FileInfo>(&mut [])
        .unwrap_err()
        .data()
        .unwrap();
    
    let mut file_info_buffer = vec![0u8; file_info_size];
    let file_info = kernel_file
        .get_info::<FileInfo>(&mut file_info_buffer)
        .map_err(|_| Status::DEVICE_ERROR)?;
    
    let file_size = file_info.file_size() as usize;
    
    // Allocate memory for the kernel ELF file
    let elf_buffer_addr = boot_services
        .allocate_pages(
            AllocateType::AnyPages,
            MemoryType::LOADER_DATA,
            (file_size + 0xFFF) / 0x1000, // Round up to the next page
        )
        .map_err(|_| Status::OUT_OF_RESOURCES)?;
    
    // Read the kernel into memory
    let mut buffer = unsafe { core::slice::from_raw_parts_mut(elf_buffer_addr as *mut u8, file_size) };
    kernel_file
        .read(&mut buffer)
        .map_err(|_| Status::DEVICE_ERROR)?;
    
    // Parse the ELF header
    let elf_header = unsafe { &*(elf_buffer_addr as *const elf::ElfHeader) };
    
    if !elf_header.is_valid() {
        return Err(Status::INVALID_PARAMETER);
    }
    
    // Process program headers to load segments
    let ph_offset = elf_header.e_phoff;
    let ph_size = elf_header.e_phentsize as usize;
    let ph_count = elf_header.e_phnum as usize;
    
    for i in 0..ph_count {
        let ph_addr = elf_buffer_addr + ph_offset + (i * ph_size) as u64;
        let ph = unsafe { &*(ph_addr as *const elf::ProgramHeader) };
        
        // Only load loadable segments
        if ph.p_type != elf::PT_LOAD {
            continue;
        }
        
        // Calculate pages needed
        let pages = (ph.p_memsz + 0xFFF) / 0x1000;
        
        // Allocate memory for the segment
        let segment_addr = boot_services
            .allocate_pages(
                AllocateType::AnyPages,
                MemoryType::LOADER_DATA,
                pages as usize,
            )
            .map_err(|_| Status::OUT_OF_RESOURCES)?;
        
        // Copy segment data
        let src = elf_buffer_addr + ph.p_offset;
        let dst = segment_addr;
        let size = ph.p_filesz as usize;
        
        unsafe {
            core::ptr::copy_nonoverlapping(
                src as *const u8,
                dst as *mut u8,
                size,
            );
            
            // Zero out the rest of the segment if memory size > file size
            if ph.p_memsz > ph.p_filesz {
                core::ptr::write_bytes(
                    (dst + ph.p_filesz) as *mut u8,
                    0,
                    (ph.p_memsz - ph.p_filesz) as usize,
                );
            }
        }
    }
    
    // Return the entry point
    Ok(elf_header.entry_point)
}

// Simple ELF header parser
mod elf {
    #[repr(C)]
    pub struct ElfHeader {
        e_ident: [u8; 16],
        e_type: u16,
        e_machine: u16,
        e_version: u32,
        pub entry_point: u64,
        e_phoff: u64,
        e_shoff: u64,
        e_flags: u32,
        e_ehsize: u16,
        e_phentsize: u16,
        e_phnum: u16,
        e_shentsize: u16,
        e_shnum: u16,
        e_shstrndx: u16,
    }
    
    impl ElfHeader {
        pub fn is_valid(&self) -> bool {
            // Check ELF magic number
            self.e_ident[0] == 0x7F &&
            self.e_ident[1] == b'E' &&
            self.e_ident[2] == b'L' &&
            self.e_ident[3] == b'F' &&
            // Check 64-bit
            self.e_ident[4] == 2 &&
            // Check little-endian
            self.e_ident[5] == 1 &&
            // Check executable
            self.e_type == 2
        }
    }
}