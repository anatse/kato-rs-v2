# Kato-rs Project

Kato-rs is a Rust-based Kafka tool that provides both a command-line interface and a GUI for interacting with Kafka clusters. It consists of two main components:

## Components

### 1. kafka-lib
A lightweight asynchronous Rust native Kafka driver that provides low-level operations for interacting with Kafka clusters. It supports both plaintext and SSL connections, and includes functionality for:
- Broker pool management
- Topic operations (create, list, describe)
- Consumer group management
- ACL operations
- Message production and consumption
- Offset management

### 2. kato-rs
The main application that provides both CLI and GUI interfaces for working with Kafka clusters. Key features include:
- GUI interface built with FLTK
- Server configuration management with persistent storage
- Connection management for multiple Kafka clusters
- Topic browsing and management
- Certificate-based authentication for SSL connections

## Project Structure

```
kato-rs-v2/
├── kafka-lib/          # Kafka driver library
│   ├── src/
│   │   └── kops/       # Kafka operations implementation
│   └── certs/          # SSL certificates for testing
├── kato-rs/            # Main application
│   ├── src/
│   │   ├── config/     # Configuration management
│   │   └── gui/        # GUI implementation
│   └── Cargo.toml
└── target/             # Build artifacts
```

## Dependencies

- `tokio`: Async runtime
- `fltk`: GUI toolkit
- `sled`: Embedded database for configuration
- `kafka-protocol`: Kafka protocol implementation
- `tracing`: Logging framework
- `serde/bson`: Serialization/deserialization

## Development

- Use `cargo run` to build and run the application
- Use `cargo test` to run tests
- The application stores configuration in `~/.kato-rs/` using sled database

## Key Features

- Support for plaintext and SSL connections to Kafka
- GUI interface for managing multiple Kafka clusters
- Persistent storage of server configurations
- Topic and consumer group browsing
- Certificate and key file management for SSL connections