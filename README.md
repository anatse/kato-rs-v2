# Kato-rs-v2

A Rust-based Kafka administration and monitoring tool with a graphical user interface. This project provides a desktop application for managing Apache Kafka clusters, allowing users to view topics, consumer groups, ACLs, and interact with Kafka brokers.

## Project Overview

**Kato-rs** is a modern, native Rust implementation of a Kafka tool that provides comprehensive administration and monitoring capabilities. It features a graphical user interface built with FLTK and a custom Kafka driver implementation for efficient communication with Kafka clusters.

## Architecture

The project is organized as a Cargo workspace with two main crates:

### 1. kafka-lib

A lightweight, async native Rust Kafka driver that implements the Kafka protocol. Key features include:

- **Protocol Implementation**: Direct implementation of Kafka wire protocol using `kafka-protocol`
- **Connection Management**: Handles both plaintext and SSL connections to Kafka brokers
- **Connection Pooling**: Efficient broker connection pooling with automatic connection reuse
- **API Support**: Implements core Kafka APIs including:
  - Metadata retrieval
  - Produce/Fetch operations
  - Consumer group management
  - ACL operations
  - Topic creation/deletion
  - Offset management
- **Error Handling**: Comprehensive error handling with custom error types

### 2. kato-rs

The main application that provides the GUI interface to interact with Kafka clusters:

- **GUI Framework**: Built with FLTK for native cross-platform desktop experience
- **Configuration Management**: Stores server configurations and preferences using Sled database
- **Multi-tab Interface**: Supports multiple Kafka cluster connections simultaneously
- **Data Visualization**: Displays topics, consumer groups, ACLs, and message data
- **Real-time Operations**: Fetches and displays live Kafka cluster information

## Features

- **Kafka Cluster Management**: Connect to and manage multiple Kafka clusters
- **Topic Operations**: View topic details, partitions, and message watermarks
- **Consumer Group Management**: View and monitor consumer groups and their offsets
- **ACL Management**: View and manage Access Control Lists
- **Message Browsing**: Browse and search through topic messages
- **Configuration Persistence**: Store and load server configurations and UI preferences
- **Secure Connections**: Support for SSL/TLS connections to Kafka brokers
- **Async Architecture**: Non-blocking operations for responsive UI

## Prerequisites

- Rust 1.70+ (with edition 2024 support)
- Cargo
- Build tools for your platform (gcc/clang on Unix-like systems, Visual Studio on Windows)

## Building

```bash
# Clone the repository
git clone <repository-url>
cd kato-rs-v2

# Build the project
cargo build

# Build in release mode
cargo build --release
```

## Running

```bash
# Run the application
cargo run

# Or run the release build
./target/release/kato-rs
```

## Configuration

The application stores its configuration in a Sled database at `~/.kato-rs/` on your user's home directory. Configuration includes:

- Kafka server connection details (bootstrap servers, SSL settings)
- Topic preferences (formatting, display options)
- Window preferences (position, size)

## Dependencies

### Runtime Dependencies
- `tokio`: Async runtime for concurrent operations
- `fltk`: Cross-platform GUI toolkit
- `kafka-protocol`: Kafka protocol implementation
- `serde`: Serialization/deserialization
- `sled`: Embedded database for configuration
- `tracing`: Application telemetry and logging

### Build Dependencies
- `fl2rust`: FLTK interface builder

## Development

### Project Structure

```
kato-rs-v2/
├── kafka-lib/          # Kafka protocol implementation
│   ├── src/
│   │   ├── kops/       # Kafka operations (broker, config, versions)
│   │   └── lib.rs
│   └── Cargo.toml
├── kato-rs/            # Main application
│   ├── src/
│   │   ├── config/     # Configuration management
│   │   ├── gui/        # Graphical user interface
│   │   └── main.rs
│   └── Cargo.toml
├── Cargo.toml          # Workspace manifest
└── README.md
```

### Key Components

- **BrokerPool**: Manages connections to multiple Kafka brokers
- **KafkaConfig**: Handles connection configuration (hosts, SSL settings)
- **KUI**: Main GUI component that orchestrates the user interface
- **Config**: Manages application configuration persistence

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests if applicable
5. Ensure all tests pass (`cargo test`)
6. Commit your changes (`git commit -am 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## License

This project is licensed under the terms specified in the LICENSE file (to be added).

## Roadmap

- Enhanced message filtering and search capabilities
- Improved topic creation and management UI
- Support for more Kafka APIs
- Message publishing functionality
- Performance optimizations
- Additional authentication methods
- Export/import configurations