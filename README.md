# wuwa-sig-rs

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows-lightgrey.svg)](https://www.microsoft.com/windows)
[![Game](https://img.shields.io/badge/Game-Wuthering%20Waves-purple.svg)](https://wutheringwaves.kurogames.com/)
[![Reversed Rooms](https://img.shields.io/badge/Discord-Reversed%20Rooms-pink.svg)](https://discord.gg/D3SXfNGBhq)

A specialized Rust-based DLL injection library designed to bypass signature verification checks in the Wuthering Waves game client. This project provides advanced memory pattern scanning and function hooking capabilities for legitimate game modification research and development purposes.

## 🎮 Purpose

**wuwa-sig-rs** is specifically designed to:
- Bypass the signature verification function in Wuthering Waves
- Enable legitimate game modification development and testing
- Provide a foundation for game enhancement research
- Support community-driven game improvement projects

## 🚀 Features

- **Signature Bypass**: Advanced pattern scanning to locate and hook signature verification functions
- **Memory Pattern Matching**: Sophisticated byte pattern detection with customizable masks
- **Function Interception**: Seamless function replacement using the Interceptor-rs framework
- **Game Client Integration**: Specifically designed for Wuthering Waves client architecture
- **Console Debugging**: Rich, colorized console output for development and monitoring
- **Performance Optimized**: Written in Rust for maximum performance and memory safety

## 🛠️ Prerequisites

- **Windows 10/11** (x64)
- **Rust 1.75+** with Cargo
- **Visual Studio Build Tools** (for Windows API bindings)
- **Administrator privileges** (for DLL injection operations)
- **Wuthering Waves game client** (for testing and development)

## 📦 Installation

### 1. Clone the Repository

```bash
git clone https://github.com/yuhkix/wuwa-sig-rs.git
cd wuwa-sig-rs
```

### 2. Install Dependencies

```bash
cargo build --release
```

### 3. Build the DLL

```bash
cargo build --release --target x86_64-pc-windows-msvc
```

The compiled DLL will be available at `target/x86_64-pc-windows-msvc/release/wuwa_sig_rs.dll`.

## 🏗️ Architecture

The project is built with a modular architecture specifically designed for game signature bypass:

- **Signature Scanner**: Locates verification functions in game memory
- **Bypass Engine**: Manages function interception and replacement
- **Game Integration**: Handles Wuthering Waves client-specific logic
- **Console Interface**: Provides development feedback and debugging information
- **Memory Utilities**: Offers low-level memory manipulation for game processes

## 🎯 Wuthering Waves Integration

This library is specifically designed to work with the Wuthering Waves game client:

- **Target Process**: `Client-Win64-Shipping.exe`
- **Signature Function**: `f_pak_file_check` verification routine
- **Memory Layout**: Optimized for the game's memory structure
- **Hook Timing**: Waits for proper game initialization before applying bypasses

## 🔒 Security & Legal Considerations

- **Educational Purpose**: This software is provided for legitimate game modification research
- **Game Terms**: Users must comply with Wuthering Waves Terms of Service
- **Administrator Access**: DLL injection requires elevated privileges
- **Process Isolation**: Always ensure proper game process boundaries
- **Responsible Use**: Intended for development and research purposes only

## 🤝 Contributing

We welcome contributions from the Wuthering Waves modding community! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Setup

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/signature-bypass`
3. Commit your changes: `git commit -m 'feat: improve signature detection'`
4. Push to the branch: `git push origin feature/signature-bypass`
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

This software is provided for educational and legitimate game modification research purposes only. Users are responsible for ensuring compliance with applicable laws, game Terms of Service, and regulations. The authors are not liable for any misuse of this software or violations of game policies.

## 🆘 Support

- **Issues**: [GitHub Issues](https://github.com/yuhkix/wuwa-sig-rs/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yuhkix/wuwa-sig-rs/discussions)
- **Documentation**: [Wiki](https://github.com/yuhkix/wuwa-sig-rs/wiki)

## 🙏 Acknowledgments

- [Interceptor-rs](https://git.xeondev.com/ReversedRoomsMisc/interceptor-rs) - Function hooking framework
- [ILHook](https://github.com/regomne/ilhook-rs) - Low-level hooking utilities
- [Windows-rs](https://github.com/microsoft/windows-rs) - Windows API bindings

---

**Made with ❤️ in Rust**

*For developers and researchers working on legitimate game enhancement projects.*