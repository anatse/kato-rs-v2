# Kato-rs Application

Kato-rs is a Kafka tool written in Rust that provides both a command-line interface and a graphical user interface for interacting with Kafka clusters.

## Purpose

This application provides a user-friendly interface for managing and monitoring Kafka clusters, allowing users to:
- Connect to multiple Kafka clusters
- Browse topics and consumer groups
- Manage server configurations with persistent storage
- Use both GUI and command-line interfaces
- Handle SSL connections with certificate authentication

## Architecture

### Configuration Management
- Uses sled embedded database for persistent storage of configurations
- Manages server connections, topic preferences, and window preferences
- Supports multiple Kafka cluster configurations

### GUI Interface
- Built with the FLTK toolkit
- Provides tree-based navigation for servers, brokers, and topics
- Tab-based interface for managing multiple connections
- Certificate browsing and SSL configuration UI

### Core Modules
- `config/`: Handles persistent storage and configuration management
- `gui/`: Implements the graphical user interface
- Integration with kafka-lib for Kafka operations

## Features

- Persistent server configuration storage
- SSL/TLS connection support with certificate management
- Multiple Kafka cluster support
- GUI interface with server browsing
- Asynchronous operations using Tokio
- Comprehensive logging with tracing

## Configuration Storage

The application stores various types of configuration in the `~/.kato-rs/` directory:
- Server configurations (bootstrap servers, certificates, etc.)
- Topic preferences (formatting, message count, etc.)
- Window preferences (position, size, etc.)

## Development

- Built with Rust using async/await for non-blocking operations
- Uses FLTK for GUI components
- sled database for configuration persistence
- Follows Rust best practices for error handling and async programming