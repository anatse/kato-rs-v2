use anyhow::Result;
use std::{
    fs::File,
    io::BufReader,
    net::{SocketAddr, ToSocketAddrs},
    path::Path,
    sync::Arc,
};
use tokio::net::TcpStream;
use tokio_rustls::{
    TlsConnector,
    rustls::{
        RootCertStore,
        client::ClientConfig,
        pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime},
    },
};
use tracing::debug;

use super::{KafkaErrors, KafkaStream, PROTO_PLAIN, PROTO_SSL};

use tokio_rustls::rustls::client::danger::{ServerCertVerified, ServerCertVerifier};

/// Struct implies dangerous SSL configuration - empty certificate verifier
#[derive(Debug, Clone, Copy)]
struct Verifier {}

impl ServerCertVerifier for Verifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, tokio_rustls::rustls::Error> {
        debug!("Called dangerous verifier");
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &tokio_rustls::rustls::DigitallySignedStruct,
    ) -> Result<
        tokio_rustls::rustls::client::danger::HandshakeSignatureValid,
        tokio_rustls::rustls::Error,
    > {
        Ok(tokio_rustls::rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &tokio_rustls::rustls::DigitallySignedStruct,
    ) -> Result<
        tokio_rustls::rustls::client::danger::HandshakeSignatureValid,
        tokio_rustls::rustls::Error,
    > {
        Ok(tokio_rustls::rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<tokio_rustls::rustls::SignatureScheme> {
        tokio_rustls::rustls::crypto::aws_lc_rs::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
            .to_vec()
    }
}

/// COnfiguraion for kafka connection
#[derive(Debug, Clone)]
pub struct KafkaConfig {
    bootstrap: Vec<String>,
    cert: Option<String>,
    key: Option<String>,
    ca_certs: Option<String>,
    protocol: i16,
    verify_certs: bool,
}

/// Implies kafka connection configuration
impl KafkaConfig {
    /// Create new configuration based on given parameters
    pub fn new(
        bootstrap: String,
        protocol: String,
        verify_certs: bool,
        cert: Option<String>,
        key: Option<String>,
        ca_certs: Option<String>,
    ) -> Result<Self> {
        let hp = bootstrap
            .split(",")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        if hp.is_empty() {
            Err(KafkaErrors::ConfigError("Bootstrap is empty".into()).into())
        } else {
            let proto = match protocol.to_lowercase().as_str() {
                "plaintext" => PROTO_PLAIN,
                "ssl" => match (&cert, &key) {
                    (Some(c), Some(k)) if Path::new(c).exists() && Path::new(k).exists() => {
                        PROTO_SSL
                    }
                    _ => {
                        return Err(KafkaErrors::ConfigError(
                            "For SSL protocol certificate and key files must be configured".into(),
                        )
                        .into());
                    }
                },
                other => {
                    return Err(KafkaErrors::ConfigError(format!(
                        "Unknown protocol type: {}",
                        other
                    ))
                    .into());
                }
            };
            KafkaConfig::ctor(hp, proto, verify_certs, cert, key, ca_certs)
        }
    }

    pub fn ctor(
        bootstrap: Vec<String>,
        protocol: i16,
        verify_certs: bool,
        cert: Option<String>,
        key: Option<String>,
        ca_certs: Option<String>,
    ) -> Result<Self> {
        Ok(Self {
            bootstrap,
            cert,
            verify_certs,
            key,
            ca_certs,
            protocol,
        })
    }

    pub(crate) async fn connect_to_broker(&self, address: SocketAddr) -> Result<KafkaStream> {
        debug!("Trying to connect to {:?}", address);
        let con = TcpStream::connect(address).await;
        if let Ok(stream) = con {
            debug!("Connected");
            if self.protocol == PROTO_PLAIN {
                return Ok(KafkaStream::PlainText(stream));
            } else if let (Some(cert), Some(key)) = (&self.cert, &self.key) {
                let certs = KafkaConfig::load_certs(cert)?;
                let pk = KafkaConfig::load_private_key(key)?;
                let config = if !self.verify_certs {
                    ClientConfig::builder()
                        .dangerous()
                        .with_custom_certificate_verifier(Arc::new(Verifier {}))
                        .with_client_auth_cert(certs, pk)?
                } else {
                    let mut root_store = RootCertStore::empty();
                    if let Some(ca_certs) = &self.ca_certs {
                        let root_certs = KafkaConfig::load_certs(ca_certs)?;
                        for cert in &root_certs {
                            root_store.add(cert.clone())?;
                        }
                    }

                    ClientConfig::builder()
                        .with_root_certificates(root_store)
                        .with_client_auth_cert(certs, pk)?
                };

                let rc_config = Arc::new(config);
                let config = TlsConnector::from(rc_config);

                // For rustls 0.23+, create ServerName from the IP address string
                let tls_stream = {
                    let host_ip = address.ip().to_string();
                    let server_name = ServerName::try_from(host_ip).map_err(|_| {
                        tokio_rustls::rustls::Error::InvalidCertificate(
                            tokio_rustls::rustls::CertificateError::BadEncoding,
                        )
                    })?;
                    config.connect(server_name, stream).await?
                };

                return Ok(KafkaStream::Tls(Box::new(tls_stream)));
            }
        }

        Err(KafkaErrors::ConnectError.into())
    }

    /// Connect to kafka broker. Connect using plain-text or ssl depends on configuration
    pub async fn connect(&self) -> Result<KafkaStream> {
        for bs in &self.bootstrap {
            let opt_socket_addr = bs.to_socket_addrs()?.next();
            if let Some(socket_addr) = opt_socket_addr
                && let Ok(stream) = self.connect_to_broker(socket_addr).await
            {
                return Ok(stream);
            }
        }

        Err(KafkaErrors::ConnectError.into())
    }

    /// Load certificates from file
    pub fn load_certs(filename: &str) -> Result<Vec<CertificateDer<'static>>> {
        let certfile = File::open(filename)?;
        let mut reader = BufReader::new(certfile);
        let certs: Result<Vec<CertificateDer<'static>>, _> =
            rustls_pemfile::certs(&mut reader).collect();
        Ok(certs?)
    }

    /// Load private key from file
    pub fn load_private_key(filename: &str) -> Result<PrivateKeyDer<'static>> {
        let keyfile = File::open(filename)?;
        let mut reader = BufReader::new(keyfile);

        loop {
            match rustls_pemfile::read_one(&mut reader)? {
                Some(rustls_pemfile::Item::Pkcs1Key(key)) => return Ok(key.into()),
                Some(rustls_pemfile::Item::Pkcs8Key(key)) => return Ok(key.into()),
                Some(rustls_pemfile::Item::Sec1Key(key)) => return Ok(key.into()),
                None => break,
                _ => {}
            }
        }

        Err(KafkaErrors::LoadCertError(format!(
            "no keys found in {:?} (encrypted keys not supported)",
            filename
        ))
        .into())
    }
}
