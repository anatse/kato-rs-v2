# Kafka-Lib API Documentation

This document describes the functions available in the Kafka-Lib library with detailed parameters, return values, and sequence diagrams.

## 1. Broker Pool Functions

### `new(cfg: Arc<KafkaConfig>, idle_time: Duration, client_id: &'static str) -> BrokerPool`

**Description**: Creates a new brokers pool using configuration and idle time. Idle time is used to free connection resources if it was not used for specified duration.

**Parameters**:

- `cfg`: Arc\<KafkaConfig> - Kafka connection configuration
- `idle_time`: Duration - Duration after which idle connections are closed
- `client_id`: &'static str - Client identifier string

**Returns**: `BrokerPool` - A new instance of BrokerPool

**Sequence Diagram**:

```plantuml
@startuml
participant "User" as user
participant "BrokerPool" as bp
participant "Background Task" as bg

user -> bp: new(cfg, idle_time, client_id)
bp -> bp: Initialize internal fields
bp -> bg: spawn background task for connection cleanup
bp -> user: BrokerPool instance
@enduml
```

### `init(&mut self) -> Result<()>`

**Description**: Initializes the broker pool by connecting to bootstrap servers, loading API versions, and metadata.

**Parameters**: None (self reference)

**Returns**: `Result<()>` - Success or error

**Sequence Diagram**:

```plantuml
@startuml
participant "User" as user
participant "BrokerPool" as bp
participant "KafkaConfig" as cfg
participant "Broker" as b
participant "Kafka API" as api

user -> bp: init()
bp -> bp: add_start_broker()
bp -> cfg: connect()
cfg -> bp: KafkaStream
bp -> b: Create new broker instance
bp -> bp: get_api_ver()
bp -> api: ApiVersionsRequest
api -> bp: ApiVersionsResponse
bp -> bp: get_metadata()
bp -> api: MetadataRequest
api -> bp: MetadataResponse
bp -> bp: update_metadata()
bp -> user: Ok(())
@enduml
```

### `produce<T: AsRef<str>>(&mut self, topic: T, key: T, payload: T, headers: &HashMap<String, String>, create_timestamp: bool) -> Result<(ResponseHeader, ProduceResponse)>`

**Description**: Sends a message to a Kafka topic.

**Parameters**:

- `topic`: T - Topic name (where T: AsRef\<str>)
- `key`: T - Message key (where T: AsRef\<str>)
- `payload`: T - Message payload (where T: AsRef\<str>)
- `headers`: &HashMap\<String, String> - Message headers
- `create_timestamp`: bool - Whether to create a timestamp

**Returns**: `Result\<(ResponseHeader, ProduceResponse)>` - Response header and produce response

**Sequence Diagram**:

```plantuml
@startuml
participant "User" as user
participant "BrokerPool" as bp
participant "Kafka API" as api
participant "Broker" as broker

user -> bp: produce(topic, key, payload, headers, create_timestamp)
bp -> bp: prepare ProduceRequest
bp -> bp: find leader broker for topic
bp -> broker: send request
broker -> api: ProduceRequest
api -> broker: ProduceResponse
broker -> bp: (ResponseHeader, ProduceResponse)
bp -> user: (ResponseHeader, ProduceResponse)
@enduml
```

### `fetch_from_topics<T: AsRef<str>, I: Into<i8>>(&mut self, topic: &[T], timeout_ms: i32, isolation_level: I, session_id: i32, fetch_offset: i64) -> Result<(ResponseHeader, FetchResponse)>`

**Description**: Fetches messages from given topics using specified offset and isolation level.

**Parameters**:

- `topic`: &[T] - Array of topic names (where T: AsRef\<str>)
- `timeout_ms`: i32 - Maximum time to wait for data
- `isolation_level`: I - Isolation level (where I: Into\<i8>)
- `session_id`: i32 - Session identifier
- `fetch_offset`: i64 - Offset to fetch from

**Returns**: `Result<(ResponseHeader, FetchResponse)>` - Response header and fetch response

**Sequence Diagram**:

```plantuml
@startuml
participant "User" as user
participant "BrokerPool" as bp
participant "Metadata" as md
participant "Kafka API" as api
participant "Broker" as broker

user -> bp: fetch_from_topics(topics, timeout, isolation_level, session_id, offset)
bp -> md: retrieve metadata
md -> bp: topic metadata
bp -> bp: prepare FetchRequest
bp -> broker: send request
broker -> api: FetchRequest
api -> broker: FetchResponse
broker -> bp: (ResponseHeader, FetchResponse)
bp -> user: (ResponseHeader, FetchResponse)
@enduml
```

### `create_topic<T: AsRef<str>>(&mut self, name: T, partition_count: i32, replica_count: i16, configs: &[(T, T)]) -> Result<(ResponseHeader, CreateTopicsResponse)>`

**Description**: Creates a new Kafka topic with specified parameters.

**Parameters**:

- `name`: T - Topic name (where T: AsRef\<str>)
- `partition_count`: i32 - Number of partitions
- `replica_count`: i16 - Number of replicas
- `configs`: &[(T, T)] - Configuration key-value pairs

**Returns**: `Result<(ResponseHeader, CreateTopicsResponse)>` - Response header and create topics response

**Sequence Diagram**:

```plantuml
@startuml
participant "User" as user
participant "BrokerPool" as bp
participant "Kafka API" as api
participant "Broker" as broker

user -> bp: create_topic(name, partitions, replicas, configs)
bp -> bp: prepare CreateTopicsRequest
bp -> broker: send request
broker -> api: CreateTopicsRequest
api -> broker: CreateTopicsResponse
broker -> bp: (ResponseHeader, CreateTopicsResponse)
bp -> user: (ResponseHeader, CreateTopicsResponse)
@enduml
```

### `delete_topic<T: AsRef<str>>(&mut self, topic: T) -> Result\<(ResponseHeader, DeleteTopicsResponse)>`

**Description**: Deletes a Kafka topic.

**Parameters**:

- `topic`: T - Topic name (where T: AsRef\<str>)

**Returns**: `Result<(ResponseHeader, DeleteTopicsResponse)>` - Response header and delete topics response

**Sequence Diagram**:

```plantuml
@startuml
participant "User" as user
participant "BrokerPool" as bp
participant "Kafka API" as api
participant "Broker" as broker

user -> bp: delete_topic(topic)
bp -> bp: prepare DeleteTopicsRequest
bp -> broker: send request
broker -> api: DeleteTopicsRequest
api -> broker: DeleteTopicsResponse
broker -> bp: (ResponseHeader, DeleteTopicsResponse)
bp -> user: (ResponseHeader, DeleteTopicsResponse)
@enduml
```

### `list_offsets<T: AsRef<str>, I: Into<i8>>(&mut self, topic: T, partition: i32, isolation_level: I, timestamp: KafkaListOffsets) -> Result<(ResponseHeader, ListOffsetsResponse)>`

**Description**: Gets offset for a topic and partition using isolation level and timestamp.

**Parameters**:

- `topic`: T - Topic name (where T: AsRef\<str>)
- `partition`: i32 - Partition number
- `isolation_level`: I - Isolation level (where I: Into\<i8>)
- `timestamp`: KafkaListOffsets - Timestamp for offset lookup

**Returns**: `Result<(ResponseHeader, ListOffsetsResponse)>` - Response header and list offsets response

**Sequence Diagram**:

```plantuml
@startuml
participant "User" as user
participant "BrokerPool" as bp
participant "Metadata" as md
participant "Kafka API" as api
participant "Broker" as broker

user -> bp: list_offsets(topic, partition, isolation_level, timestamp)
bp -> bp: get_leader_for_topic()
md -> bp: leader broker ID
bp -> bp: prepare ListOffsetsRequest
bp -> broker: send request (to leader)
broker -> api: ListOffsetsRequest
api -> broker: ListOffsetsResponse
broker -> bp: (ResponseHeader, ListOffsetsResponse)
bp -> user: (ResponseHeader, ListOffsetsResponse)
@enduml
```

### `get_watermarks<T: AsRef<str>>(&mut self, topic: T, partition: i32, isolation_level: KafkaIsolationLevel) -> Result<(i64, i64)>`

**Description**: Retrieves start and end offset (watermarks) for a given topic and partition.

**Parameters**:

- `topic`: T - Topic name (where T: AsRef\<str>)
- `partition`: i32 - Partition number
- `isolation_level`: KafkaIsolationLevel - Isolation level for the request

**Returns**: `Result<(i64, i64)>` - Tuple with (start offset, end offset)

**Sequence Diagram**:

```plantuml
@startuml
participant "User" as user
participant "BrokerPool" as bp
participant "list_offsets" as lo

user -> bp: get_watermarks(topic, partition, isolation_level)
bp -> lo: list_offsets(Beginning)
lo -> bp: start offset response
bp -> lo: list_offsets(End)
lo -> bp: end offset response
bp -> user: (start_offset, end_offset)
@enduml
```

### `offset_fetch<T: AsRef<str>>(&mut self, group_id: T, topic: T, partitions: &[i32]) -> Result<(ResponseHeader, OffsetFetchResponse)>`

**Description**: Fetches offsets for consumer group and topic partitions.

**Parameters**:

- `group_id`: T - Consumer group ID (where T: AsRef\<str>)
- `topic`: T - Topic name (where T: AsRef\<str>)
- `partitions`: &[i32] - List of partition IDs

**Returns**: `Result<(ResponseHeader, OffsetFetchResponse)>` - Response header and offset fetch response

**Sequence Diagram**:

```plantuml
@startuml
participant "User" as user
participant "BrokerPool" as bp
participant "Kafka API" as api
participant "Broker" as broker

user -> bp: offset_fetch(group_id, topic, partitions)
bp -> bp: prepare OffsetFetchRequest
bp -> broker: send request
broker -> api: OffsetFetchRequest
api -> broker: OffsetFetchResponse
broker -> bp: (ResponseHeader, OffsetFetchResponse)
bp -> user: (ResponseHeader, OffsetFetchResponse)
@enduml
```

### `describe_acl(&mut self, rq: DescribeAclsRequest) -> Result<(ResponseHeader, DescribeAclsResponse)>`

**Description**: Gets ACLs from Kafka broker using filter data in request.

**Parameters**:

- `rq`: DescribeAclsRequest - Describe ACLs request object

**Returns**: `Result<(ResponseHeader, DescribeAclsResponse)>` - Response header and describe ACLs response

**Sequence Diagram**:

```plantuml
@startuml
participant "User" as user
participant "BrokerPool" as bp
participant "Kafka API" as api
participant "Broker" as broker

user -> bp: describe_acl(request)
bp -> bp: get API version
bp -> broker: send request
broker -> api: DescribeAclsRequest
api -> broker: DescribeAclsResponse
broker -> bp: (ResponseHeader, DescribeAclsResponse)
bp -> user: (ResponseHeader, DescribeAclsResponse)
@enduml
```

### `create_acl(&mut self, rq: CreateAclsRequest) -> Result<(ResponseHeader, CreateAclsResponse)>`

**Description**: Creates ACLs in Kafka.

**Parameters**:

- `rq`: CreateAclsRequest - Create ACLs request object

**Returns**: `Result\<(ResponseHeader, CreateAclsResponse)>` - Response header and create ACLs response

**Sequence Diagram**:

```plantuml
@startuml
participant "User" as user
participant "BrokerPool" as bp
participant "Kafka API" as api
participant "Broker" as broker

user -> bp: create_acl(request)
bp -> bp: get API version
bp -> broker: send request
broker -> api: CreateAclsRequest
api -> broker: CreateAclsResponse
broker -> bp: (ResponseHeader, CreateAclsResponse)
bp -> user: (ResponseHeader, CreateAclsResponse)
@enduml
```

### `delete_acls(&mut self, rq: DeleteAclsRequest) -> Result<(ResponseHeader, DeleteAclsResponse)>`

**Description**: Deletes ACLs in Kafka.

**Parameters**:

- `rq`: DeleteAclsRequest - Delete ACLs request object

**Returns**: `Result<(ResponseHeader, DeleteAclsResponse)>` - Response header and delete ACLs response

**Sequence Diagram**:

```plantuml
@startuml
participant "User" as user
participant "BrokerPool" as bp
participant "Kafka API" as api
participant "Broker" as broker

user -> bp: delete_acls(request)
bp -> bp: get API version
bp -> broker: send request
broker -> api: DeleteAclsRequest
api -> broker: DeleteAclsResponse
broker -> bp: (ResponseHeader, DeleteAclsResponse)
bp -> user: (ResponseHeader, DeleteAclsResponse)
@enduml
```

### `list_groups(&mut self) -> Result<Vec<(ResponseHeader, ListGroupsResponse)>>`

**Description**: Lists all consumer groups in the cluster.

**Parameters**: None (self reference)

**Returns**: `Result<Vec<(ResponseHeader, ListGroupsResponse)>>` - Vector of response headers and list groups responses

**Sequence Diagram**:

```plantuml
@startuml
participant "User" as user
participant "BrokerPool" as bp
participant "Kafka API" as api
participant "Broker" as broker

user -> bp: list_groups()
bp -> bp: get_broker_indexes()
bp -> broker: send ListGroupsRequest to all brokers
broker -> api: ListGroupsRequest
api -> broker: ListGroupsResponse
broker -> bp: (ResponseHeader, ListGroupsResponse)
bp -> bp: collect responses
bp -> user: Vec\<(ResponseHeader, ListGroupsResponse)>
@enduml
```

### `describe_groups(&mut self, groups: &[String]) -> Result<Vec<(ResponseHeader, DescribeGroupsResponse)>>`

**Description**: Describes specific consumer groups.

**Parameters**:

- `groups`: &[String] - Slice of group names to describe

**Returns**: `Result<Vec<(ResponseHeader, DescribeGroupsResponse)>>` - Vector of response headers and describe groups responses

**Sequence Diagram**:

```plantuml
@startuml
participant "User" as user
participant "BrokerPool" as bp
participant "Kafka API" as api
participant "Broker" as broker

user -> bp: describe_groups(groups)
bp -> bp: prepare DescribeGroupsRequest
bp -> bp: get_broker_indexes()
bp -> broker: send DescribeGroupsRequest to all brokers
broker -> api: DescribeGroupsRequest
api -> broker: DescribeGroupsResponse
broker -> bp: (ResponseHeader, DescribeGroupsResponse)
bp -> bp: collect responses
bp -> user: Vec\<(ResponseHeader, DescribeGroupsResponse)>
@enduml
```

## 2. Configuration Functions

### `KafkaConfig::new(bootstrap: String, protocol: String, verify_certs: bool, cert: Option<String>, key: Option<String>, ca_certs: Option<String>) -> Result<Self>`

**Description**: Creates a new Kafka configuration based on given parameters.

**Parameters**:

- `bootstrap`: String - Bootstrap server addresses
- `protocol`: String - Protocol type ("plaintext" or "ssl")
- `verify_certs`: bool - Whether to verify SSL certificates
- `cert`: Option\<String> - Path to client certificate file (for SSL)
- `key`: Option\<String> - Path to client key file (for SSL)
- `ca_certs`: Option\<String> - Path to CA certificates file (for SSL)

**Returns**: `Result<Self>` - New KafkaConfig instance or error

**Sequence Diagram**:

```plantuml
@startuml
participant "User" as user
participant "KafkaConfig" as cfg

user -> cfg: new(bootstrap, protocol, verify_certs, cert, key, ca_certs)
cfg -> cfg: validate protocol
cfg -> cfg: verify certificate files exist (for SSL)
cfg -> cfg: create configuration
cfg -> user: Ok(KafkaConfig) or Err
@enduml
```

### `KafkaConfig::connect(&self) -> Result<KafkaStream>`

**Description**: Connects to a Kafka broker using the configuration.

**Parameters**: None (self reference)

**Returns**: `Result<KafkaStream>` - KafkaStream connection or error

**Sequence Diagram**:

```plantuml
@startuml
participant "User" as user
participant "KafkaConfig" as cfg
participant "TCP/TLS" as tcp

user -> cfg: connect()
cfg -> cfg: iterate bootstrap addresses
cfg -> tcp: attempt TCP connection
alt successful connection
  alt SSL protocol
    cfg -> tcp: establish TLS connection
    tcp -> cfg: TLS stream
  else Plaintext protocol
    tcp -> cfg: TCP stream
  end
  cfg -> user: Ok(KafkaStream)
else failed connection
  cfg -> user: Err(ConnectionError)
end
@enduml
```

### `KafkaConfig::connect_to_broker(&self, address: SocketAddr) -> Result<KafkaStream>`

**Description**: Connects to a specific broker at the given address.

**Parameters**:

- `address`: SocketAddr - Socket address of the broker

**Returns**: `Result<KafkaStream>` - KafkaStream connection or error

**Sequence Diagram**:

```plantuml
@startuml
participant "User" as user
participant "KafkaConfig" as cfg
participant "TCP/TLS" as tcp

user -> cfg: connect_to_broker(address)
cfg -> tcp: TCP connection to address
alt SSL protocol
  cfg -> tcp: establish TLS connection with certificates
  tcp -> cfg: TLS stream
else Plaintext protocol
  tcp -> cfg: TCP stream
end
cfg -> user: Ok(KafkaStream)
@enduml
```

## 3. Utility Functions

### `get_leader_for_topic(&self, topic: &TopicName, partition: i32) -> Result<i32>`

**Description**: Gets the leader broker ID for a specific topic and partition using loaded metadata.

**Parameters**:

- `topic`: &TopicName - Topic name reference
- `partition`: i32 - Partition number

**Returns**: `Result<i32>` - Leader broker ID or error

**Sequence Diagram**:

```plantuml
@startuml
participant "User" as user
participant "BrokerPool" as bp
participant "Metadata" as md

user -> bp: get_leader_for_topic(topic, partition)
bp -> md: search for topic in metadata
alt topic found
  md -> bp: find partition data
  alt partition found
    bp -> user: leader_id
  else partition not found
    bp -> user: PartitionNotFound error
  end
else topic not found
  bp -> user: PartitionNotFound error
end
@enduml
```

### `refresh_metadata(&mut self) -> Result<()>`

**Description**: Refreshes metadata from Kafka broker.

**Parameters**: None (self reference)

**Returns**: `Result<()>` - Success or error

**Sequence Diagram**:

```plantuml
@startuml
participant "User" as user
participant "BrokerPool" as bp
participant "Kafka API" as api
participant "Broker" as broker

user -> bp: refresh_metadata()
bp -> bp: get_version(MetadataRequest::KEY)
bp -> bp: prepare GetMetadataRequest
bp -> broker: send request
broker -> api: MetadataRequest
api -> broker: MetadataResponse
broker -> bp: update metadata
bp -> bp: update brokers list
bp -> user: Ok(())
@enduml
```
