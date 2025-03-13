// uefi_bootloader/src/aarch64.rs
use uefi::prelude::*;

pub fn prepare_jump_to_kernel(_system_table: &mut SystemTable<Boot>) {
    // ARM64 specific preparations
    // Disable interrupts
    unsafe {
        core::arch::asm!("msr daifset, #2");
    }
}