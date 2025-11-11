use bincode::{Decode, Encode};
use kafka_lib::KafkaErrors;
use serde::{Deserialize, Serialize};
use sled::{Db, Tree};
use tracing::error;

#[cfg(not(test))]
use tracing::info;

#[derive(Serialize, Deserialize, Debug, Decode, Encode)]
/// Kafka Server configuration
pub struct ServerConfig {
    pub bootstrap: Vec<String>,
    pub cert: Option<String>,
    pub key: Option<String>,
    pub ca_certs: Option<String>,
    pub protocol: i16,
    pub verify_certs: bool,
}

#[derive(Debug, Serialize, Deserialize, Decode, Encode)]
/// Topic Preference configuration
pub struct TopicPreference {
    pub key_format: String,
    pub data_format: String,
    pub num_last_messages: u32,
}

#[derive(Debug, Serialize, Deserialize, Decode, Encode)]
pub struct WindowPreference {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug)]
pub struct Config {
    preferences: Db,
    servers: Tree,
    topics: Tree,
    window: Tree,
}

macro_rules! config_impl {
    ($pref_type:ty) => {
        impl $pref_type {
            fn read(db: &Tree) -> Vec<(String, Self)> {
                let config = bincode::config::standard();
                db.iter()
                    .filter_map(|v| match v {
                        Err(err) => {
                            error!("Error reading {}: {:?}", stringify!(pref_type), err);
                            None
                        }
                        Ok((key, value)) => match (
                            std::str::from_utf8(key.as_ref()),
                            bincode::decode_from_slice::<Self, _>(value.as_ref(), config),
                        ) {
                            (Ok(key), Ok(srv)) => Some((key.to_owned(), srv.0)),
                            _ => None,
                        },
                    })
                    .collect()
            }

            fn find<T: AsRef<str>>(db: &Tree, name: T) -> anyhow::Result<Self> {
                let config = bincode::config::standard();
                match db.get(name.as_ref()) {
                    Ok(Some(value)) => {
                        bincode::decode_from_slice::<Self, _>(value.as_ref(), config)
                            .map(|srv| srv.0)
                            .map_err(From::from)
                    }
                    Err(error) => Err(error.into()),
                    _ => Err(KafkaErrors::SomeError(format!(
                        "{} not found",
                        stringify!($pref_type)
                    ))
                    .into()),
                }
            }

            fn add<T: AsRef<str>>(&self, db: &Tree, name: T) -> anyhow::Result<()> {
                let config = bincode::config::standard();
                let data = bincode::encode_to_vec(self, config)?;
                let _ = db.insert(name.as_ref(), data)?;
                Ok(())
            }

            fn delete<T: AsRef<str>>(db: &Tree, name: T) -> anyhow::Result<()> {
                let _ = db.remove(name.as_ref())?;
                Ok(())
            }
        }
    };
}

config_impl!(ServerConfig);
config_impl!(TopicPreference);
config_impl!(WindowPreference);

/// Implies essential functions to manipulate stored configurations
impl Config {
    #[cfg(test)]
    /// This function was defined only for test purposes. To develop use ::new() without parameters
    pub fn test_new(home_dir: &str) -> anyhow::Result<Self> {
        let config = sled::Config::default().path(format!("{}/.kato-rs", home_dir));

        let preferences = config.open()?;

        Ok(Self {
            servers: preferences.open_tree("servers")?,
            topics: preferences.open_tree("topics")?,
            window: preferences.open_tree("window")?,
            preferences,
        })
    }

    pub fn new() -> anyhow::Result<Self> {
        let home_dir = match dirs::home_dir() {
            Some(hd) => hd.to_str().unwrap().to_string(),
            None => {
                info!("User directory unknown, use current");
                ".".to_string()
            }
        };

        let config = sled::Config::default().path(format!("{}/.kato-rs", home_dir));

        let preferences = config.open()?;

        Ok(Self {
            servers: preferences.open_tree("servers")?,
            topics: preferences.open_tree("topics")?,
            window: preferences.open_tree("window")?,
            preferences,
        })
    }

    /// Read all servers from stored configuration
    pub fn read_servers(&self) -> Vec<(String, ServerConfig)> {
        ServerConfig::read(&self.servers)
    }

    /// Find srver from stored configuration by server's name
    pub fn find_server<T: AsRef<str>>(&self, name: T) -> anyhow::Result<ServerConfig> {
        ServerConfig::find(&self.servers, name)
    }

    /// Add server to stored configuration
    pub fn add_server<T: AsRef<str>>(&self, name: T, server: ServerConfig) -> anyhow::Result<()> {
        server.add(&self.servers, name)
    }

    /// Delete server from stored configuration
    pub fn delete_server<T: AsRef<str>>(&self, name: T) -> anyhow::Result<()> {
        ServerConfig::delete(&self.servers, name)
    }

    /// Add topic preference to stored configuration
    pub fn add_topic_preference<T: AsRef<str>>(
        &self,
        name: T,
        pref: &TopicPreference,
    ) -> anyhow::Result<()> {
        pref.add(&self.topics, name)
    }

    /// Find topic preference in stored configuration by name. Name may be in format [server_name]:topic_name.
    pub fn find_topic_preference<T: AsRef<str>>(&self, name: T) -> anyhow::Result<TopicPreference> {
        TopicPreference::find(&self.topics, name)
    }

    pub fn read_topic_preferences(&self) -> Vec<(String, TopicPreference)> {
        TopicPreference::read(&self.topics)
    }

    /// Delete topic preference from onfiguration by name. Name may be in format [server_name]:topic_name.
    pub fn delete_topic_preference<T: AsRef<str>>(&self, name: T) -> anyhow::Result<()> {
        TopicPreference::delete(&self.topics, name)
    }

    pub fn add_window_preference<T: AsRef<str>>(
        &self,
        name: T,
        pref: &WindowPreference,
    ) -> anyhow::Result<()> {
        pref.add(&self.window, name)
    }

    /// Find topic preference in stored configuration by name. Name may be in format [server_name]:topic_name.
    pub fn find_window_preference<T: AsRef<str>>(
        &self,
        name: T,
    ) -> anyhow::Result<WindowPreference> {
        WindowPreference::find(&self.window, name)
    }

    pub fn read_window_preferences(&self) -> Vec<(String, WindowPreference)> {
        WindowPreference::read(&self.window)
    }

    /// Delete topic preference from onfiguration by name. Name may be in format [server_name]:topic_name.
    pub fn delete_window_preference<T: AsRef<str>>(&self, name: T) -> anyhow::Result<()> {
        WindowPreference::delete(&self.window, name)
    }
}

#[cfg(test)]
/// Tests for configuration functions
mod tests {
    // Folder contains test database data
    const HOME_DIR: &str = "./test-data";

    use super::{Config, TopicPreference};
    use crate::config::ServerConfig;

    #[test]
    fn test_open_db() {
        match Config::test_new(HOME_DIR) {
            Ok(_) => {}
            Err(err) => {
                panic!("Error opening config db {:?}", err);
            }
        };
    }

    #[test]
    fn test_add_server() {
        let cfg = Config::test_new(HOME_DIR);
        assert!(cfg.is_ok());
        let cfg = cfg.unwrap();

        let server = ServerConfig {
            bootstrap: vec!["localhost:9092".to_string()],
            cert: None,
            key: None,
            ca_certs: None,
            protocol: 0, // Plaintext
            verify_certs: false,
        };
        match cfg.add_server("test-server", server) {
            Ok(_) => {}
            Err(err) => {
                panic!("Error adding server {:?}", err);
            }
        }
    }

    #[test]
    fn test_read_servers() {
        test_add_server();
        let cfg = Config::test_new(HOME_DIR).unwrap();
        let servers = cfg.read_servers();
        assert!(!servers.is_empty());
        let found = servers.iter().find(|(name, _)| name == "test-server");
        assert!(found.is_some());
        let (_, found) = found.unwrap();
        assert_eq!(found.protocol, 0);
        assert_eq!(found.bootstrap[0].as_str(), "localhost:9092");
    }

    #[test]
    fn test_delete_server() {
        test_add_server();
        let cfg = Config::test_new(HOME_DIR).unwrap();
        match cfg.delete_server("test-server") {
            Ok(_) => {}
            Err(err) => {
                panic!("Error adding server {:?}", err);
            }
        }

        // Check server exists
        match cfg.find_server("test-server") {
            Ok(_) => panic!("Server has not been deleted"),
            Err(err) => {
                assert_eq!(
                    "Some error occurred: StoredConfig not found",
                    format!("{}", err)
                );
            }
        };
    }

    #[test]
    fn test_topic_preferences() {
        let pref = TopicPreference {
            key_format: "string".to_string(),
            data_format: "string".to_string(),
            num_last_messages: 500,
        };

        let cfg = Config::test_new(HOME_DIR);
        assert!(cfg.is_ok());
        let cfg = cfg.unwrap();

        if let Err(err) = cfg.add_topic_preference("test-topic", &pref) {
            panic!("Error adding topic reference: {}", err);
        }

        let prefs = cfg.read_topic_preferences();
        assert_eq!(1, prefs.len());

        if let Err(err) = cfg.find_topic_preference("test-topic") {
            panic!("Error finding topic reference: {}", err);
        }

        if let Err(err) = cfg.delete_topic_preference("test-topic") {
            panic!("Error deleting topic reference: {}", err);
        }

        // Check server exists
        match cfg.find_topic_preference("test-topic") {
            Ok(_) => panic!("Server has not been deleted"),
            Err(err) => {
                assert_eq!(
                    "Some error occurred: TopicPreference not found",
                    format!("{}", err)
                );
            }
        };
    }
}
