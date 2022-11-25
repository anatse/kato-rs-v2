mod broker;
mod config;
mod versions;

use std::net::SocketAddr;

use anyhow::Result;
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_rustls::client::TlsStream;

pub use broker::BrokerPool;
pub use config::KafkaConfig;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum KafkaErrors {
    #[error("Some error occurred: {0}")]
    SomeError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Cannot connect to kafka server")]
    ConnectError,

    #[error("Error load certificate or private key: {0}")]
    LoadCertError(String),

    #[error("Connection not initialized, must call function BrokerPool::init() first")]
    NotInitialized,

    #[error("Topic not found")]
    TopicNotFound,

    #[error("Broker not found")]
    BrokerNotFound,

    #[error("Partition with index {0} not found in topic {1}")]
    PartitionNotFound(i32, String),
}

/// Enum defines common functions for network stream
#[derive(Debug)]
pub enum KafkaStream {
    PlainText(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl KafkaStream {
    fn addr(&self) -> Result<SocketAddr> {
        match self {
            KafkaStream::PlainText(stream) => stream.peer_addr().map_err(From::from),
            KafkaStream::Tls(stream) => stream.get_ref().0.peer_addr().map_err(From::from),
        }
    }
}

/// Implies write behavour for KafkaStream object
impl KafkaStream {
    async fn write_i32(&mut self, value: i32) -> Result<()> {
        match self {
            KafkaStream::PlainText(stream) => stream.write_i32(value).await.map_err(|e| e.into()),
            KafkaStream::Tls(stream) => stream.write_i32(value).await.map_err(|e| e.into()),
        }
    }

    async fn write_all(&mut self, value: &[u8]) -> Result<()> {
        match self {
            KafkaStream::PlainText(stream) => stream.write_all(value).await.map_err(|e| e.into()),
            KafkaStream::Tls(stream) => stream.write_all(value).await.map_err(|e| e.into()),
        }
    }
}

/// Implies read behavour for KafkaStream object
impl KafkaStream {
    async fn read_i32(&mut self) -> Result<i32> {
        match self {
            KafkaStream::PlainText(stream) => stream.read_i32().await.map_err(|e| e.into()),
            KafkaStream::Tls(stream) => stream.read_i32().await.map_err(|e| e.into()),
        }
    }

    // async fn read_i32(&mut self) -> Result<i32> {
    //     let mut buf = [0u8; 4];
    //     let len = self.read_exact(&mut buf).await?;
    //     println!("read lenth of len: {}", len);
    //     let res = i32::from_be_bytes(buf);
    //     Ok(res)
    // }

    async fn read_exact(&mut self, value: &mut [u8]) -> Result<usize> {
        match self {
            KafkaStream::PlainText(stream) => stream.read_exact(value).await.map_err(|e| e.into()),
            KafkaStream::Tls(stream) => stream.read_exact(value).await.map_err(|e| e.into()),
        }
    }
}

/// Protocol definition constant - PLAIN TEXT
const PROTO_PLAIN: i16 = 0;
/// Protocol definition constant -SSL
const PROTO_SSL: i16 = 1;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum KafkaIsolationLevel {
    ReadUncommitted,
    ReadCommitted,
}

impl Into<i8> for KafkaIsolationLevel {
    fn into(self) -> i8 {
        match self {
            KafkaIsolationLevel::ReadUncommitted => 0,
            KafkaIsolationLevel::ReadCommitted => 1,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum KafkaListOffsets {
    Beginning,
    End,
    Stored,
    Invalid,
    TailBase,
    ForTime(i64),
}

impl Into<i64> for KafkaListOffsets {
    fn into(self) -> i64 {
        match self {
            KafkaListOffsets::Beginning => -2,
            KafkaListOffsets::End => -1,
            KafkaListOffsets::Stored => -1000,
            KafkaListOffsets::Invalid => -1001,
            KafkaListOffsets::TailBase => -2000,
            KafkaListOffsets::ForTime(time) => time,
        }
    }
}
