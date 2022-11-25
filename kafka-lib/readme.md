# Library to communicate with Kafka
This is very tiny library. The purpose is implements only neccessasy functions for kafka admin tool.
The library uses its own kafka protocol implementation. It uses structures and encode/decode functions 
defined in [kafka-protocol](https://docs.rs/kafka-protocol/0.3.0/kafka_protocol/).

Additional information about kafka protocol here: [Guide to kafka protocol](https://cwiki.apache.org/confluence/display/KAFKA/A+Guide+To+The+Kafka+Protocol)

## Objects and functions

- KafkaConfig - configuration to connect to kafka brokers.
- BrokerPool - provides functions to call kafka protocol. Manipulate with pool of connections to brokers.

### Service classes (enums)
- KafkaErrors - defines errors
- KafkaIsolationLevel - defines isolation level when fetching data from broker
- KafkaListOffsets - defines type of offsets when calling ListOffsets API

## Architecture
kafka-lib use tokio stream and rustls to access kafka API throught TCP protocol. All APIs are asynchronous.
All functions perform as pair of requests: send request and receive response. Library do not check correlations between
requests and responses. Because of asynchronous nature of tokio's stream operations, it is likely to get wrong answer.
In cases where we use this library, this situation is unlikely. Beaware of this if you decide to use this library in other cases.

### Logging
For logging it uses tracing crate. For desktop application logging is not very important, but can be very helpful in come cases.

## Usage

### Connect to kafka plaintext
```rust
    // Defines connection config
    let cfg = KafkaConfig::new(
        "localhost:9092".to_string(),
        "plaintext".to_string(),
        false,
        None,
        None,
        None,
    )
    .unwrap();

    // Configuration must be shareable in case when we need several broker pools
    let cfg = Arc::new(cfg);
    // Create broker pool with the configuration and client name
    let mut pool = BrokerPool::new(cfg, Duration::from_secs(30), "kafka-tool");
    // Initialize broker pool. This functoin reads api versions and fills metadata
    pool.init().await.unwrap();

    debug!("Successfully initialized brokers pool");
```

### Connect to kafka SSL

Prepare certificate and primary key - kafka.cer and kafka.pkcs8 files

```rust
    const CERT_FILENAME: &str = "kafka.cer";
    const KEY_FILENAME: &str = "kafka.pkcs8";

    // Defines connection config
    let cfg = KafkaConfig::new(
        "localhost:9093".to_string(),
        "ssl".to_string(),
        false,
        Some(CERT_FILENAME.to_string()),
        Some(KEY_FILENAME.to_string()),
        Some(CERT_FILENAME.to_string())).unwrap();

    // Configuration must be shareable in case when we need several broker pools
    let cfg = Arc::new(cfg);
    // Create broker pool with the configuration and client name
    let mut pool = BrokerPool::new(cfg, Duration::from_secs(30), "kafka-tool");
    // Initialize broker pool. This functoin reads api versions and fills metadata
    pool.init().await.unwrap();

    debug!("Successfully initialized brokers pool");
```

### Provided API
All the API provide throught the BrokerPool object. All functions are asynchronous.

#### Functions
There are few functions provided by the library. It is possible to add other if neccessary. 
All the functions has names relevant to Kafka protocol API Keys

For more information about kafka API see [Kafka protocol guide](https://kafka.apache.org/protocol.html)
- refresh_metadata
- get_api_ver
- get_metadata
- describe_acl
- create_acl
- produce
- fetch_from_topics
- list_offsets
- get_watermarks
- offset_fetch
- create_topic
- list_groups
- describe_groups
- delete_acls
- delete_topic

##### Add new function
1. Make sure than kafka-protocol crate already has desired structures. For example, for API Key ElectLeaders 
   there are two structures - ElectLeadersRequest and ElectLeadersResponse must be defined.
2. Define function using `kafka_call` macro. This macro generate private asynchronous function to call Kafka API. There are three
   parameters in the function:
    - request - prepared request structure
    - version - version, for more information see kafka API documentation
    - broker id - index in brokers list in metadata. It is must for some APPs, such as list offset. If no different to which broker
      request will be send, then use -1 as broker_id 
   ```rust
   kafka_call!(_describe_groups, DescribeGroupsRequest, DescribeGroupsResponse);
   ```
3. Define public function which provides more simple input and output for client. For example:
   ```rust
    pub async fn describe_groups(&mut self, groups: &[String]) -> anyhow::Result<Vec<(ResponseHeader, DescribeGroupsResponse)>> {
        let mut rq = DescribeGroupsRequest::default();
        rq.groups = groups.iter().map(|grp| GroupId(BrokerPool::to_strbytes(grp))).collect::<Vec<GroupId>>();
        let version = self.get_version(DescribeGroupsRequest::KEY)?.max_version;
        let broker_idx = self.get_broker_indexes().await;
        let mut res = Vec::new();
        for idx in broker_idx {
            res.push(self._describe_groups(&rq, version, idx).await?);
        }
        Ok(res)
    }
   ```