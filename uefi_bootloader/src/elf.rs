// // uefi_bootloader/src/elf.rs
// #[repr(C)]
// pub struct ElfHeader {
//     e_ident: [u8; 16],
//     e_type: u16,
//     e_machine: u16,
//     e_version: u32,
//     pub entry_point: u64,
//     pub e_phoff: u64,
//     e_shoff: u64,
//     e_flags: u32,
//     e_ehsize: u16,
//     pub e_phentsize: u16,
//     pub e_phnum: u16,
//     e_shentsize: u16,
//     e_shnum: u16,
//     e_shstrndx: u16,
// }

// impl ElfHeader {
//     pub fn is_valid(&self) -> bool {
//         // Check ELF magic number
//         self.e_ident[0] == 0x7F &&
//         self.e_ident[1] == b'E' &&
//         self.e_ident[2] == b'L' &&
//         self.e_ident[3] == b'F' &&
//         // Check 64-bit
//         self.e_ident[4] == 2 &&
//         // Check little-endian
//         self.e_ident[5] == 1 &&
//         // Check executable
//         self.e_type == 2
//     }
// }

// #[repr(C)]
// pub struct ProgramHeader {
//     pub p_type: u32,
//     pub p_flags: u32,
//     pub p_offset: u64,
//     pub p_vaddr: u64,
//     pub p_paddr: u64,
//     pub p_filesz: u64,
//     pub p_memsz: u64,
//     pub p_align: u64,
// }

// // Program header type: loadable segment
// pub const PT_LOAD: u32 = 1;


// uefi_bootloader/src/elf.rs
use log::info;

#[repr(C)]
pub struct ElfHeader {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub entry_point: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

impl ElfHeader {
    pub fn is_valid(&self) -> bool {
        // Dump the first few bytes for debugging
        info!("ELF header bytes: {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}", 
            self.e_ident[0], self.e_ident[1], self.e_ident[2], self.e_ident[3],
            self.e_ident[4], self.e_ident[5], self.e_ident[6], self.e_ident[7]);
        
        // Check ELF magic number (0x7F 'E' 'L' 'F')
        let magic_valid = self.e_ident[0] == 0x7F &&
            self.e_ident[1] == b'E' &&
            self.e_ident[2] == b'L' &&
            self.e_ident[3] == b'F';
        
        if !magic_valid {
            info!("Invalid ELF magic: [{:02X}, {:02X}, {:02X}, {:02X}]", 
                self.e_ident[0], self.e_ident[1], self.e_ident[2], self.e_ident[3]);
            return false;
        }
        
        // Check 64-bit (EI_CLASS = 2)
        if self.e_ident[4] != 2 {
            info!("Not a 64-bit ELF (class = {})", self.e_ident[4]);
            return false;
        }
        
        // Check little-endian (EI_DATA = 1)
        if self.e_ident[5] != 1 {
            info!("Not little-endian (data = {})", self.e_ident[5]);
            return false;
        }
        
        // Check version (EI_VERSION = 1)
        if self.e_ident[6] != 1 {
            info!("Invalid ELF version (version = {})", self.e_ident[6]);
            return false;
        }
        
        // Check executable (e_type = ET_EXEC = 2)
        if self.e_type != 2 {
            info!("Not an executable (type = {})", self.e_type);
            return false;
        }
        
        // Check for x86_64 machine type (e_machine = EM_X86_64 = 62)
        #[cfg(target_arch = "x86_64")]
        if self.e_machine != 62 {
            info!("Not an x86_64 executable (machine = {})", self.e_machine);
            return false;
        }
        
        // Check for AArch64 machine type (e_machine = EM_AARCH64 = 183)
        #[cfg(target_arch = "aarch64")]
        if self.e_machine != 183 {
            info!("Not an AArch64 executable (machine = {})", self.e_machine);
            return false;
        }
        
        // All checks passed
        info!("ELF header validation successful");
        true
    }
    
    pub fn dump_info(&self) {
        info!("ELF Header Information:");
        info!("  Magic: {:02X} {:02X} {:02X} {:02X}", 
            self.e_ident[0], self.e_ident[1], self.e_ident[2], self.e_ident[3]);
        info!("  Class: {} ({})", self.e_ident[4], 
            if self.e_ident[4] == 1 { "32-bit" } else if self.e_ident[4] == 2 { "64-bit" } else { "Unknown" });
        info!("  Data: {} ({})", self.e_ident[5], 
            if self.e_ident[5] == 1 { "Little Endian" } else if self.e_ident[5] == 2 { "Big Endian" } else { "Unknown" });
        info!("  Version: {}", self.e_ident[6]);
        info!("  OS ABI: {}", self.e_ident[7]);
        info!("  Type: {} ({})", self.e_type, 
            match self.e_type {
                1 => "Relocatable",
                2 => "Executable",
                3 => "Shared Object",
                4 => "Core",
                _ => "Unknown"
            });
        info!("  Machine: {} ({})", self.e_machine, 
            match self.e_machine {
                3 => "x86",
                20 => "PowerPC",
                21 => "PowerPC64",
                40 => "ARM",
                62 => "x86_64",
                183 => "AArch64",
                _ => "Unknown"
            });
        info!("  Entry Point: 0x{:x}", self.entry_point);
        info!("  Program Headers Offset: 0x{:x}", self.e_phoff);
        info!("  Program Header Count: {}", self.e_phnum);
        info!("  Program Header Size: {} bytes", self.e_phentsize);
    }
}

#[repr(C)]
pub struct ProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

impl ProgramHeader {
    pub fn dump_info(&self, index: usize) {
        info!("Program Header #{}", index);
        info!("  Type: {} ({})", self.p_type, match self.p_type {
            0 => "NULL",
            1 => "LOAD",
            2 => "DYNAMIC",
            3 => "INTERP",
            4 => "NOTE",
            _ => "Other"
        });
        info!("  Flags: 0x{:x} ({}{}{})", self.p_flags, 
            if self.p_flags & 4 != 0 { "r" } else { "-" },
            if self.p_flags & 2 != 0 { "w" } else { "-" },
            if self.p_flags & 1 != 0 { "x" } else { "-" });
        info!("  Offset: 0x{:x}", self.p_offset);
        info!("  Virtual Address: 0x{:x}", self.p_vaddr);
        info!("  Physical Address: 0x{:x}", self.p_paddr);
        info!("  File Size: {} bytes", self.p_filesz);
        info!("  Memory Size: {} bytes", self.p_memsz);
        info!("  Alignment: 0x{:x}", self.p_align);
    }
}

// Program header type constants
pub const PT_NULL: u32 = 0;
pub const PT_LOAD: u32 = 1;
pub const PT_DYNAMIC: u32 = 2;
pub const PT_INTERP: u32 = 3;
pub const PT_NOTE: u32 = 4;