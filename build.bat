@echo off
REM Simple build script for MelonOS

echo Building UEFI bootloader with nightly toolchain...
cd uefi_bootloader
cargo +nightly build --target x86_64-unknown-uefi --release
if %ERRORLEVEL% neq 0 (
    echo Failed to build bootloader
    cd ..
    exit /b %ERRORLEVEL%
)
cd ..

echo Building test kernel...
cd kernel
set ARCH=x86_64

REM Ensure .cargo directory exists
if not exist .cargo mkdir .cargo

REM Use the appropriate config
if "%ARCH%"=="x86_64" (
    echo Using x86_64 config
    if exist .cargo\config.toml.x86_64 (
        copy /Y .cargo\config.toml.x86_64 .cargo\config.toml
    )
    
    REM Copy the linker script to the root directory
    copy /Y link.ld ..\link.ld
    
    cargo +nightly build --release
    if %ERRORLEVEL% neq 0 (
        echo Failed to build kernel
        cd ..
        exit /b %ERRORLEVEL%
    )
    set KERNEL_NAME=KERNEL_X64.ELF
)
cd ..

echo Setting up disk image files...
if not exist esp mkdir esp
if not exist esp\EFI mkdir esp\EFI
if not exist esp\EFI\BOOT mkdir esp\EFI\BOOT
if not exist esp\EFI\KERNEL mkdir esp\EFI\KERNEL

copy /Y uefi_bootloader\target\x86_64-unknown-uefi\release\uefi_bootloader.efi esp\EFI\BOOT\BOOTX64.EFI
copy /Y kernel\target\x86_64-unknown-none\release\kernel esp\EFI\KERNEL\%KERNEL_NAME%

echo Build completed successfully. Files ready at esp\ directory.
echo To test in QEMU, run the following command:
echo qemu-system-x86_64 -drive file=fat:rw:esp,format=raw -bios OVMF.fd -m 128M