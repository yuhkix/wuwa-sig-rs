# wuwa-sig-rs

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows-lightgrey.svg)](https://www.microsoft.com/windows)
[![Game](https://img.shields.io/badge/Game-Wuthering%20Waves-purple.svg)](https://wutheringwaves.kurogames.com/)
[![Reversed Rooms](https://img.shields.io/badge/Discord-Reversed%20Rooms-pink.svg)](https://discord.gg/reversedrooms)

A high-performance, thread-safe Rust library for memory scanning and hooking Windows applications, specifically designed for bypassing PAK file verification in WuWa (Wuthering Waves).

## 🚀 Features

- **High-Performance Memory Scanning**: Optimized pattern matching algorithms with caching
- **Thread-Safe Hook Management**: State tracking and safe hook application/removal
- **Structured Logging**: Configurable log levels with colored output and timestamps
- **Safe Memory Access**: Bounds checking and error handling for all unsafe operations
- **Performance Optimized**: Minimal allocations, efficient algorithms, and caching
- **Modular Architecture**: Clean separation of concerns for easy maintenance
- **Comprehensive Documentation**: Detailed API documentation and examples

## 🏗️ Architecture

The library is organized into several focused modules:

- **`config`**: Configuration management with validation and defaults
- **`constants`**: Application constants and magic numbers
- **`error`**: Comprehensive error handling with thiserror integration
- **`hooks`**: Thread-safe hook management with state tracking
- **`logger`**: High-performance structured logging system
- **`memory`**: Optimized memory scanning and pattern matching
- **`safety`**: Safe abstractions for unsafe operations

## 📦 Installation

### Prerequisites

- **Windows 10/11** (x64)
- **Rust 1.75+** with Cargo
- **Visual Studio Build Tools** (for Windows API bindings)
- **Administrator privileges** (for DLL injection operations)

### Building

```bash
# Clone the repository
git clone https://github.com/yuhkix/wuwa-sig-rs.git
cd wuwa-sig-rs

# Build in debug mode
cargo build

# Build in release mode
cargo build --release
```

The compiled DLL will be available at `target/release/wuwa_sig_rs.dll`.

## 🛡️ Safety

This library uses unsafe code for low-level memory operations and Windows API calls. All unsafe operations are wrapped in safe abstractions with:

- **Bounds checking** for memory access
- **Null pointer validation** before dereferencing
- **Panic recovery** for critical operations
- **Comprehensive error handling** with detailed error messages

## ⚡ Performance

The library is optimized for performance with:

- **Cached module scanning** to avoid repeated system calls
- **Optimized pattern matching** with different algorithms for different pattern types
- **Efficient memory access patterns** with minimal copying
- **Thread-safe operations** without unnecessary locking
- **Minimal allocations** in hot paths

## 🧪 Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test memory::tests
```

## 🔒 Security & Legal Considerations

- **Educational Purpose**: This software is provided for legitimate game modification research
- **Game Terms**: Users must comply with Wuthering Waves Terms of Service
- **Administrator Access**: DLL injection requires elevated privileges
- **Process Isolation**: Always ensure proper game process boundaries
- **Responsible Use**: Intended for development and research purposes only

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

This software is provided for educational and legitimate game modification research purposes only. Users are responsible for ensuring compliance with applicable laws, game Terms of Service, and regulations. The authors are not liable for any misuse of this software or violations of game policies.

## 🙏 Acknowledgments

- T[Ranny](https://git.xeondev.com/Ranny) - Fixing 🧑‍🦼 and updated to work for 2.7
- [Interceptor-rs](https://git.xeondev.com/ReversedRoomsMisc/interceptor-rs) - Function hooking framework
- [ILHook](https://github.com/regomne/ilhook-rs) - Low-level hooking utilities
- [Windows-rs](https://github.com/microsoft/windows-rs) - Windows API bindings

---

**Made with ❤️ in Rust**

*For developers and researchers working on legitimate game enhancement projects.*