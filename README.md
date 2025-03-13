# MelonOS

A ground-up operating system built with security, privacy, and cross-platform compatibility at its core.

## Vision

MelonOS is a completely new operating system built from first principles. Instead of modifying existing kernels or building on top of established operating systems, MelonOS takes a clean-slate approach with modern hardware and security requirements in mind.

The driving principles behind this project:
- **Privacy**: Don't track me, don't sell my data, online/offline, self-hosted, control over everything.
- **Memory Safety First**: Built entirely in Rust for inherent memory safety and performance.
- **Universal Compatibility**: Designed to run across all 64-bit major CPU architectures.
- **Security by Design**: API-centric security model that protects the core system.
- **Permission-Based Access**: Applications must explicitly request and receive permission to access hardware and system resources.
- **Complete Isolation**: Process isolation and sandboxing to prevent security breaches.
- **Device Ecosystem**: Seamless and secure communication between your personal devices.
- **Original Architecture**: Not based on Linux, Windows, or any existing OS foundation.

## Technical Architecture

### Bootloader

The project starts with a 64-bit UEFI bootloader that:
- Operates exclusively in 64-bit mode
- Supports multiple CPU architectures through conditional compilation
- Initialises essential hardware components
- Sets up graphics and memory maps before kernel handoff

### Kernel

The Rust-based kernel provides:
- Advanced memory management
- Process isolation and scheduling
- Core security primitives
- Resource allocation

### API Layer

All system interactions happen through a well-defined API layer:
- Applications can't directly access hardware or system resources
- All resource access requires explicit permission grants
- User control over permission settings

### Driver System

A hardware abstraction layer that:
- Manages device access
- Handles hardware diversity
- Provides consistent interfaces across platforms

### Inter-device Communication

A protocol for secure device communication that:
- Enables data sharing between trusted devices (think of Apple ecosystem)
- Implements end-to-end encryption
- Verifies device identity and authorisation

## Development Roadmap

1. UEFI Bootloader implementation
2. Basic Rust kernel with memory management
3. Process isolation and scheduling
4. API security layer
5. Hardware abstraction and drivers
6. Command-line interface
7. Inter-device communication protocol
8. Graphical user interface

## Current Status

This project is in the initial development phase. Currently focusing on:
- UEFI bootloader implementation
- Rust kernel foundation

## Contributing

Contributions are welcome, especially in the following areas:
- UEFI/bootloader expertise
- Rust systems programming
- Hardware driver development
- Security design and implementation
- Cross-platform architecture

## FAQ

**Q: Why build a completely new OS instead of modifying Linux or another existing system?**

A: While existing operating systems offer many advantages, they also carry legacy design decisions that impact security, performance, and architecture. By starting fresh, we can implement modern security practices from the ground up and create a system specifically designed for today's computing needs.

**Q: Is this a practical project or mainly educational?**

A: Both. While ambitious, the project has practical goals of creating a more secure, private computing environment. It's also an excellent platform for exploring modern OS design, Rust systems programming, and security architecture.

**Q: How will you handle the massive driver ecosystem needed for hardware support?**

A: This is indeed one of the biggest challenges. The initial focus will be on supporting a limited set of common hardware, with the driver system designed to be extensible. As the project matures, we'll develop a framework that makes it easier to implement drivers for additional hardware.
