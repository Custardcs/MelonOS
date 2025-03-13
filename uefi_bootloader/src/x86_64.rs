// uefi_bootloader/src/x86_64.rs
use uefi::prelude::*;

pub fn prepare_jump_to_kernel(_system_table: &mut SystemTable<Boot>) {
    // x86_64 specific preparations
    // For example, make sure interrupts are disabled
    unsafe {
        core::arch::asm!("cli");
    }
}