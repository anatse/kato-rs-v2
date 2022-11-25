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
    rustls::{
        client::ServerCertVerifier, Certificate, ClientConfig, PrivateKey, RootCertStore,
        ServerName,
    },
    TlsConnector,
};
use tracing::debug;

use super::{KafkaErrors, KafkaStream, PROTO_PLAIN, PROTO_SSL};

/// Struct implies dangerous SSL configuration - empty certificate verifier
#[derive(Debug, Clone, Copy)]
struct Verifier {}
impl ServerCertVerifier for Verifier {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<tokio_rustls::rustls::client::ServerCertVerified, tokio_rustls::rustls::Error> {
        debug!("Called dangerous verifier: {:?}", server_name);
        Ok(tokio_rustls::rustls::client::ServerCertVerified::assertion())
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
                        .into())
                    }
                },
                other => {
                    return Err(KafkaErrors::ConfigError(format!(
                        "Unknown protocol type: {}",
                        other
                    ))
                    .into())
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
            } else {
                if let (Some(cert), Some(key)) = (&self.cert, &self.key) {
                    let certs = KafkaConfig::load_certs(cert)?;
                    let pk = KafkaConfig::load_private_key(key)?;
                    let config = if !self.verify_certs {
                        ClientConfig::builder()
                            .with_safe_defaults()
                            .with_custom_certificate_verifier(Arc::new(Verifier {}))
                            .with_single_cert(certs, pk)?
                    } else {
                        let mut root_store = RootCertStore::empty();
                        if let Some(ca_certs) = &self.ca_certs {
                            let root_certs = KafkaConfig::load_certs(ca_certs)?;
                            for cert in &root_certs {
                                root_store.add(cert)?;
                            }
                        }

                        ClientConfig::builder()
                            .with_safe_defaults()
                            .with_root_certificates(root_store)
                            .with_single_cert(certs, pk)?
                    };

                    let rc_config = Arc::new(config.into());
                    let config = TlsConnector::from(rc_config);

                    let domain_name = address
                        .to_string()
                        .chars()
                        .take_while(|&ch| ch != ':')
                        .collect::<String>();
                    let domain = ServerName::try_from(domain_name.as_str())?;
                    let tls_stream = config.connect(domain, stream).await?;

                    return Ok(KafkaStream::Tls(tls_stream));
                }
            }
        }

        Err(KafkaErrors::ConnectError.into())
    }

    /// Connect to kafka broker. Connect using plain-text or ssl depends on configuration
    pub async fn connect(&self) -> Result<KafkaStream> {
        for bs in &self.bootstrap {
            let opt_socket_addr = bs.to_socket_addrs()?.next();
            if let Some(socket_addr) = opt_socket_addr {
                if let Ok(stream) = self.connect_to_broker(socket_addr).await {
                    return Ok(stream);
                }
            }
        }

        Err(KafkaErrors::ConnectError.into())
    }

    /// Load certificates from file
    pub fn load_certs(filename: &str) -> Result<Vec<Certificate>> {
        let certfile = File::open(filename)?;
        let mut reader = BufReader::new(certfile);
        Ok(rustls_pemfile::certs(&mut reader)
            .unwrap()
            .iter()
            .map(|v| Certificate(v.clone()))
            .collect())
    }

    /// Load private key from file
    pub fn load_private_key(filename: &str) -> Result<PrivateKey> {
        let keyfile = File::open(filename)?;
        let mut reader = BufReader::new(keyfile);

        loop {
            match rustls_pemfile::read_one(&mut reader)? {
                Some(rustls_pemfile::Item::RSAKey(key)) => return Ok(PrivateKey(key)),
                Some(rustls_pemfile::Item::PKCS8Key(key)) => return Ok(PrivateKey(key)),
                Some(rustls_pemfile::Item::ECKey(key)) => return Ok(PrivateKey(key)),
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
