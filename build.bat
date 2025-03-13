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

copy uefi_bootloader\target\x86_64-unknown-uefi\release\uefi-bootloader.efi esp\EFI\BOOT\BOOTX64.EFI
copy kernel\target\%ARCH%-unknown-none\release\test-kernel esp\EFI\KERNEL\%KERNEL_NAME%

echo Files prepared. Run disk creation separately if needed.