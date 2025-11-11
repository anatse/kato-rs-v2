# Kafka-lib

Kafka-lib is a lightweight asynchronous Rust native Kafka driver. It provides low-level operations for interacting with Kafka clusters.

## Purpose

This library provides a thin, async-native wrapper around Kafka protocol operations. It's designed to be:
- Lightweight and efficient
- Async-first with Tokio runtime support
- Support both plaintext and SSL connections
- Provide direct access to Kafka APIs

## Key Components

### Kafka Operations (`kops` module)
- `BrokerPool`: Manages connections to multiple Kafka brokers
- `KafkaConfig`: Handles connection configuration including SSL settings
- `KafkaStream`: Unified interface for both plaintext and TLS connections
- Error handling with the `KafkaErrors` enum

### Supported Operations
- Topic management (create, list, describe)
- Message production and consumption
- Consumer group management
- ACL operations
- Offset management
- Metadata operations

## Connection Configuration

The library supports both plaintext and SSL connections:
- `KafkaConfig` handles all connection parameters
- Certificate and key file management for SSL connections
- Configurable isolation levels and offset strategies

## Error Handling

The library uses the `KafkaErrors` enum for comprehensive error handling, including:
- Configuration errors
- Connection errors
- Topic/broker not found errors
- General operational errors

## Usage

This library is designed to be used by higher-level applications like kato-rs to provide Kafka functionality. It provides the core protocol implementation while allowing applications to build their own interfaces and user experiences.