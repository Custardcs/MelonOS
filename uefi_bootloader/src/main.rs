// uefi_bootloader/src/main.rs
#![no_std]
#![no_main]

// Add the following lines to import alloc
extern crate alloc;
use alloc::vec;

use log::info;
use uefi::data_types::CStr16;
use uefi::prelude::*;
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::table::boot::{AllocateType, MemoryType};

// Architecture-specific modules
#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "aarch64")]
mod aarch64;

// Common module for shared functionality
mod common;

// ELF parsing module
mod elf;

// Entry point for the UEFI bootloader
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

    // Load the kernel file path
    #[cfg(target_arch = "x86_64")]
    let kernel_path = b"\\EFI\\KERNEL\\KERNEL_X64.ELF\0";

    #[cfg(target_arch = "aarch64")]
    let kernel_path = b"\\EFI\\KERNEL\\KERNEL_ARM64.ELF\0";

    // Convert the path to UTF-16
    let mut kernel_path_utf16 = [0u16; 128];
    for (i, &b) in kernel_path.iter().enumerate() {
        if b == 0 {
            kernel_path_utf16[i] = 0;
            break;
        }
        kernel_path_utf16[i] = b as u16;
    }

    // Create CStr16 from the UTF-16 array
    let kernel_path_cstr = unsafe { CStr16::from_u16_with_nul_unchecked(&kernel_path_utf16) };

    // Load the appropriate kernel
    match load_kernel(image_handle, &mut system_table, kernel_path_cstr) {
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
                .allocate_pool(MemoryType::RUNTIME_SERVICES_DATA, boot_params_size)
                .expect("Failed to allocate memory for boot parameters");

            // We'll get memory map size information first
            let memory_map_info = system_table.boot_services().memory_map_size();
            let descriptor_size = memory_map_info.entry_size;

            // Allocate a buffer for the memory map in a separate scope
            // We need to pass ownership of this memory to the kernel
            let memory_map_buffer_size = memory_map_info.map_size + 4096; // Add some extra space
            let memory_map_buffer = system_table
                .boot_services()
                .allocate_pool(MemoryType::RUNTIME_SERVICES_DATA, memory_map_buffer_size)
                .expect("Failed to allocate memory map buffer");

            // We'll store the descriptor size in boot_info
            boot_info.memory_map_entry_size = descriptor_size;
            boot_info.memory_map_addr = memory_map_buffer as u64;

            // We'll update the actual size after exit_boot_services
            // For now, just copy the boot info to its location
            unsafe {
                core::ptr::write_volatile(boot_params_addr as *mut common::BootInfo, boot_info);
            }

            // Create a temporary buffer for exit_boot_services
            let mut temp_map_buf = [0u8; 16384];

            // Get the memory map into our permanent buffer before exiting boot services
            let map_size = unsafe {
                let map_buffer = core::slice::from_raw_parts_mut(
                    memory_map_buffer as *mut u8,
                    memory_map_buffer_size,
                );

                let (_size, map) = system_table
                    .boot_services()
                    .memory_map(map_buffer)
                    .expect("Failed to get memory map");

                map.len() * descriptor_size
            };

            // Update the memory map size in boot info
            unsafe {
                let boot_info_mut = &mut *(boot_params_addr as *mut common::BootInfo);
                boot_info_mut.memory_map_size = map_size;
            }

            // Now exit boot services
            let _ = system_table
                .exit_boot_services(image_handle, &mut temp_map_buf)
                .expect("Failed to exit boot services");

            // Jump to the kernel, passing the boot info structure
            unsafe {
                let kernel_entry: fn(*const common::BootInfo) -> ! =
                    core::mem::transmute(kernel_entry);

                kernel_entry(boot_params_addr as *const common::BootInfo);
            }
        }
        Err(status) => {
            info!("Failed to load kernel: {:?}", status);
            return status;
        }
    }

    // This line is unreachable but needed for the compiler
    #[allow(unreachable_code)]
    Status::SUCCESS
}

// Function to set up graphics
fn setup_graphics(system_table: &mut SystemTable<Boot>) -> Option<common::BootInfo> {
    let boot_services = system_table.boot_services();

    // Get the GOP (Graphics Output Protocol)
    unsafe {
        #[allow(deprecated)]
        let gop = boot_services
            .locate_protocol::<uefi::proto::console::gop::GraphicsOutput>()
            .ok()?;

        // Get the concrete GOP instance
        let gop = &mut *gop.get();

        // Get current graphics mode info
        let mode_info = gop.current_mode_info();

        // Get framebuffer
        let mut framebuffer = gop.frame_buffer();

        // Create a boot info structure
        let mut boot_info = common::BootInfo::new(0, 0, 0); // We'll fill memory map details later

        // Set framebuffer info
        boot_info.framebuffer_addr = framebuffer.as_mut_ptr() as u64;
        boot_info.framebuffer_width = mode_info.resolution().0;
        boot_info.framebuffer_height = mode_info.resolution().1;
        boot_info.framebuffer_stride = mode_info.stride();

        Some(boot_info)
    }
}

// Function to load the kernel from the boot volume
fn load_kernel(
    _image_handle: Handle,
    system_table: &mut SystemTable<Boot>,
    kernel_path: &CStr16,
) -> Result<u64, Status> {
    // Get the file system protocol
    let boot_services = system_table.boot_services();

    info!("Opening file system...");

    // Open the UEFI file system of the boot drive
    #[allow(deprecated)]
    let fs_proto = unsafe {
        boot_services
            .locate_protocol::<SimpleFileSystem>()
            .map_err(|_| Status::NOT_FOUND)?
    };

    let fs = unsafe { &mut *fs_proto.get() };

    // Open the root directory
    let mut root = fs.open_volume().map_err(|_| Status::DEVICE_ERROR)?;

    info!("Opening kernel file: {:?}", kernel_path);

    // Open the kernel file
    let mut kernel_file = root
        .open(kernel_path, FileMode::Read, FileAttribute::empty())
        .map_err(|_| Status::NOT_FOUND)?
        .into_regular_file()
        .ok_or(Status::INVALID_PARAMETER)?;

    info!("Getting kernel file size...");

    // Get the kernel file size
    let file_info_size = kernel_file
        .get_info::<FileInfo>(&mut [])
        .unwrap_err()
        .data()
        .unwrap_or(512);

    let mut file_info_buffer = vec![0u8; file_info_size];
    let file_info = kernel_file
        .get_info::<FileInfo>(&mut file_info_buffer)
        .map_err(|_| Status::DEVICE_ERROR)?;

    let file_size = file_info.file_size() as usize;
    info!("Kernel file size: {} bytes", file_size);

    if file_size == 0 {
        info!("Kernel file is empty");
        return Err(Status::INVALID_PARAMETER);
    }

    info!("Allocating memory for kernel...");

    // Allocate memory for the kernel ELF file - using a standard memory type
    let elf_buffer_addr = boot_services
        .allocate_pages(
            AllocateType::AnyPages,
            MemoryType::LOADER_DATA,
            (file_size + 0xFFF) / 0x1000, // Round up to the next page
        )
        .map_err(|_| Status::OUT_OF_RESOURCES)?;

    // Read the kernel into memory
    // Creating a slice from a raw pointer requires unsafe
    let mut buffer =
        unsafe { core::slice::from_raw_parts_mut(elf_buffer_addr as *mut u8, file_size) };

    info!("Reading kernel file...");
    kernel_file
        .read(&mut buffer)
        .map_err(|_| Status::DEVICE_ERROR)?;

    // Parse the ELF header - use the elf module now
    info!("Parsing ELF header...");
    let elf_header = unsafe { &*(elf_buffer_addr as *const elf::ElfHeader) };

    // Dump detailed header information for debugging
    elf_header.dump_info();

    if !elf_header.is_valid() {
        info!("Invalid ELF header - see above logs for details");
        return Err(Status::INVALID_PARAMETER);
    }

    info!(
        "Valid ELF header found. Entry point: {:x}",
        elf_header.entry_point
    );

    // Process program headers to load segments
    let ph_offset = elf_header.e_phoff;
    let ph_size = elf_header.e_phentsize as usize;
    let ph_count = elf_header.e_phnum as usize;

    info!("Processing {} program headers", ph_count);

    for i in 0..ph_count {
        let ph_addr = elf_buffer_addr + ph_offset + (i * ph_size) as u64;
        // Dereferencing a raw pointer requires unsafe
        let ph = unsafe { &*(ph_addr as *const elf::ProgramHeader) };

        // Dump program header information
        ph.dump_info(i);

        // Only load loadable segments
        if ph.p_type != elf::PT_LOAD {
            continue;
        }

        info!(
            "Loading segment {}: vaddr={:x}, size={}",
            i, ph.p_vaddr, ph.p_memsz
        );

        // Calculate pages needed
        let pages = (ph.p_memsz + 0xFFF) / 0x1000;

        // Allocate memory for the segment - using a standard memory type
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

        // Memory operations with raw pointers require unsafe
        unsafe {
            // Copy from source to destination
            core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, size);

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
    info!(
        "Kernel loaded successfully. Entry point: {:x}",
        elf_header.entry_point
    );
    Ok(elf_header.entry_point)
}
