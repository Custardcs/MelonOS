// uefi_bootloader/src/elf.rs
#[repr(C)]
pub struct ElfHeader {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    pub entry_point: u64,
    pub e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
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

// Program header type: loadable segment
pub const PT_LOAD: u32 = 1;