use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    net::{SocketAddr, ToSocketAddrs},
    sync::{atomic::AtomicI32, Arc},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use bytes::{Bytes, BytesMut};
use kafka_protocol::{
    messages::{
        api_versions_response::ApiVersion,
        create_topics_request::{CreatableTopic, CreateableTopicConfigBuilder},
        fetch_request::{FetchPartition, FetchTopicBuilder},
        list_offsets_request::{
            ListOffsetsPartition, ListOffsetsPartitionBuilder, ListOffsetsTopicBuilder,
        },
        offset_fetch_request::{
            OffsetFetchRequestGroupBuilder, OffsetFetchRequestTopicBuilder,
            OffsetFetchRequestTopicsBuilder,
        },
        produce_request, ApiVersionsRequest, ApiVersionsResponse, BrokerId, CreateAclsRequest,
        CreateAclsResponse, CreateTopicsRequest, CreateTopicsResponse, DeleteAclsRequest,
        DeleteAclsResponse, DeleteTopicsRequest, DeleteTopicsResponse, DescribeAclsRequest,
        DescribeAclsResponse, DescribeGroupsRequest, DescribeGroupsResponse, FetchRequest,
        FetchResponse, GroupId, ListGroupsRequest, ListGroupsResponse, ListOffsetsRequest,
        ListOffsetsResponse, MetadataRequest, MetadataResponse, OffsetFetchRequest,
        OffsetFetchResponse, ProduceRequest, ProduceResponse, RequestHeader, ResponseHeader,
        TopicName,
    },
    protocol::{Decodable, Encodable, Request, StrBytes},
    records::{Record, RecordBatchEncoder, RecordEncodeOptions, TimestampType},
};
use tokio::{
    select,
    sync::Mutex,
    task::JoinHandle,
    time::{self, Instant},
};
use tracing::{debug, error, info, warn};

use super::{
    config::KafkaConfig, versions::request_header_version, versions::response_header_version,
    KafkaErrors, KafkaIsolationLevel, KafkaListOffsets, KafkaStream,
};

/// Contains broker iinformation and essential functions
#[derive(Debug)]
pub struct Broker {
    /// Socket address of the broker
    address: SocketAddr,
    #[allow(dead_code)]
    /// Broker identifier
    broker_id: i32,
    /// Stream connected to broker. The stream must be accessed outside of main flow
    stream: Mutex<Option<KafkaStream>>,
    /// Last broker's access time
    access_time: Option<Instant>,
    /// Configuration. Used to connect/reconnect to broker
    config: Arc<KafkaConfig>,
}

impl Broker {
    #[allow(dead_code)]
    /// Create and connect to broker using broker's id, host, port and kafka configuration
    async fn connect(&mut self) -> anyhow::Result<()> {
        let stream = self.config.connect_to_broker(self.address).await?;
        self.update_access_time();
        let mut broker = self.stream.lock().await;
        *broker = Some(stream);
        Ok(())
    }

    #[allow(dead_code)]
    async fn disconnect(&mut self) {
        let mut broker = self.stream.lock().await;
        *broker = None;
    }

    #[inline(always)]
    fn update_access_time(&mut self) {
        self.access_time = Some(Instant::now());
    }

    async fn write_i32(&mut self, value: i32) -> anyhow::Result<()> {
        self.update_access_time();
        let mut stream = self.stream.lock().await;
        if stream.is_none() {
            *stream = Some(self.config.connect_to_broker(self.address).await?);
        }
        match &mut *stream {
            Some(ks) => ks.write_i32(value).await,
            None => Err(KafkaErrors::ConnectError.into()),
        }
    }

    async fn write_all(&mut self, value: &[u8]) -> anyhow::Result<()> {
        self.update_access_time();
        let mut stream = self.stream.lock().await;
        if stream.is_none() {
            *stream = Some(self.config.connect_to_broker(self.address).await?);
        }
        match &mut *stream {
            Some(ks) => ks.write_all(value).await,
            None => Err(KafkaErrors::ConnectError.into()),
        }
    }

    async fn read_i32(&mut self) -> anyhow::Result<i32> {
        self.update_access_time();
        let mut stream = self.stream.lock().await;
        if stream.is_none() {
            *stream = Some(self.config.connect_to_broker(self.address).await?);
        }
        match &mut *stream {
            Some(ks) => ks.read_i32().await,
            None => Err(KafkaErrors::ConnectError.into()),
        }
    }

    async fn read_exact(&mut self, value: &mut [u8]) -> anyhow::Result<usize> {
        self.update_access_time();
        let mut stream = self.stream.lock().await;
        if stream.is_none() {
            *stream = Some(self.config.connect_to_broker(self.address).await?);
        }
        match &mut *stream {
            Some(ks) => ks.read_exact(value).await,
            None => Err(KafkaErrors::ConnectError.into()),
        }
    }
}

/// Contains all information used to communicate with kafka throught the kafka protocol
/// Implements Kafka API protocol functions
#[derive(Debug)]
pub struct BrokerPool {
    ///  Kafka connection configuration
    config: Arc<KafkaConfig>,
    /// Broker list
    brokers: Arc<Mutex<HashMap<i32, Broker>>>,
    #[allow(dead_code)]
    /// Idle task join handle
    idle_task: JoinHandle<()>,
    /// Client id
    client_id: &'static str,
    /// Correlation container
    correlation: AtomicI32,
    /// Metadata
    metadata: Option<MetadataResponse>,
    /// Api Versions
    api_ver: Option<ApiVersionsResponse>,
}

/// Implies basic functions and constructors for brokers pool
impl BrokerPool {
    /// Creates new brokers pool using configuration and idle time. Idle time used to free conenction resources if it was not used for specified duration
    pub fn new(cfg: Arc<KafkaConfig>, idle_time: Duration, client_id: &'static str) -> Self {
        let brokers = Arc::new(Mutex::new(HashMap::<i32, Broker>::new()));
        let background_broker_handle = brokers.clone();
        let handle = tokio::spawn(async move {
            let mut interval = time::interval(idle_time);
            loop {
                select! {
                    _ = interval.tick() => {
                        info!("Trying to clear brokers");
                        let ro_brokers = background_broker_handle.lock().await;
                        let now = Instant::now();
                        for (_, broker) in &*ro_brokers {
                            if let Some(access_time) = broker.access_time {
                                if now.duration_since(access_time) > idle_time {
                                    // Broker was accessed too long time ago. Close broker connection
                                    let mut stream = broker.stream.lock().await;
                                    *stream = None;
                                }
                            } else {
                                // Broker was never accessed, close broker connection
                                let mut stream = broker.stream.lock().await;
                                *stream = None;
                            };
                        };
                    }
                    else => {
                        warn!("Some error occurred");
                        break;
                    }
                }
            }
        });

        Self {
            config: cfg,
            brokers,
            idle_task: handle,
            client_id,
            correlation: AtomicI32::new(1),
            metadata: None,
            api_ver: None,
        }
    }

    /// Update brokers list using given metadata from kafka
    pub async fn update_metadata(&mut self, md: &MetadataResponse) {
        let mut brokers = self.brokers.lock().await;

        //brokers.clear();
        let mut new_brokers = HashMap::new();
        md.brokers.iter().for_each(|(id, bmd)| {
            let broker_id = **id;
            match (bmd.host.to_string(), bmd.port as u16).to_socket_addrs() {
                Err(err) => {
                    error!("Error converting to socket address: {:?}", err);
                }
                Ok(mut addr_iter) => {
                    if let Some(_) = brokers.get(&broker_id) {
                        new_brokers.insert(broker_id, brokers.remove(&broker_id).unwrap());
                    } else {
                        if let Some(addr) = addr_iter.next() {
                            let found_idx = match brokers.iter().find(|(_, v)| v.address == addr) {
                                Some((idx, _)) => *idx,
                                _ => -1,
                            };

                            if found_idx != -1 {
                                new_brokers.insert(broker_id, brokers.remove(&found_idx).unwrap());
                            } else {
                                new_brokers.insert(
                                    broker_id,
                                    Broker {
                                        address: addr,
                                        broker_id,
                                        stream: Mutex::new(None),
                                        access_time: None,
                                        config: self.config.clone(),
                                    },
                                );
                            }
                        }
                    }
                }
            };
        });

        *brokers = new_brokers;
    }

    /// Get api version for given api key
    pub fn get_version(&self, api_key: i16) -> anyhow::Result<&ApiVersion> {
        if let Some(api_ver) = &self.api_ver {
            Ok(&api_ver.api_keys[api_key as usize])
        } else {
            Err(KafkaErrors::NotInitialized.into())
        }
    }

    /// Get leader broker id for topic and partition, using loaded metadata
    pub fn get_leader_for_topic(&self, topic: &TopicName, partition: i32) -> anyhow::Result<i32> {
        if let Some(md) = &self.metadata {
            let found = md
                .topics
                .get(topic)
                .iter()
                .filter_map(|md| {
                    md.partitions
                        .iter()
                        .find(|p| p.partition_index == partition)
                        .map(|p| *p.leader_id)
                })
                .collect::<Vec<i32>>();
            if found.len() == 1 {
                Ok(found[0])
            } else {
                Err(KafkaErrors::PartitionNotFound(partition, topic.to_string()).into())
            }
        } else {
            Err(KafkaErrors::PartitionNotFound(partition, topic.to_string()).into())
        }
    }

    pub async fn get_broker_indexes(&self) -> Vec<i32> {
        self.brokers
            .lock()
            .await
            .iter()
            .map(|(idx, _)| *idx)
            .collect()
    }

    pub fn metadata(&self) -> &Option<MetadataResponse> {
        &self.metadata
    }
}

/// Macro to generate async function to make kafka protocol call and process response
/// # Example
/// ```ignore
/// /// Defines function _get_api_version to get api versions from kafka
/// kafka_call!(_get_api_version, ApiVersionsRequest, ApiVersionsResponse);
/// ```
/// And it will generate function
/// ```ignore
/// async fn _get_api_version(&mut self, client_id: &'static str, rq: &ApiVersionsRequest, version: i16) -> anyhow::Result<(ResponseHeader, ApiVersionsResponse)> {
///    /// Here is generated neccessary code
/// }
/// ```
macro_rules! kafka_call {
    ($func_name:ident, $str_rq:ty, $str_rs:ty) => {
        /// Call kafka with arguments:
        ///  - rq - request
        ///  - version - request version
        ///  - broker_id - broker id in broker's array
        #[inline(always)]
        async fn $func_name(
            &mut self,
            rq: &$str_rq,
            version: i16,
            broker_id: i32,
        ) -> anyhow::Result<(ResponseHeader, $str_rs)> {
            debug!(
                "Start calling kafka protocol for {:?}",
                stringify!($func_name)
            );

            let mut buf = BytesMut::new();
            let request_header = self.prepare_request(<$str_rq>::KEY, self.client_id, version);
            request_header
                .encode(&mut buf, request_header_version(<$str_rq>::KEY, version))
                .map_err(BrokerPool::map_proto_err)?;
            rq.encode(&mut buf, version)
                .map_err(BrokerPool::map_proto_err)?;

            // Prepare broker to call
            let mut brokers = self.brokers.lock().await;
            if brokers.is_empty() {
                return Err(KafkaErrors::NotInitialized.into());
            }

            debug!("Brokers: {:?}", brokers.keys());
            let broker = if broker_id == -1 {
                brokers.iter_mut().map(|(_, v)| v).next()
            } else {
                brokers.get_mut(&broker_id)
            };
            match broker {
                None => Err(KafkaErrors::BrokerNotFound.into()),
                Some(broker) => {
                    broker.write_i32(buf.len() as i32).await?;
                    broker.write_all(&buf[..]).await?;
                    let rlen = broker.read_i32().await? as usize;
                    let mut response_buf = BytesMut::zeroed(rlen as usize);
                    broker.read_exact(&mut response_buf[..]).await?;
                    let resp_header = ResponseHeader::decode(
                        &mut response_buf,
                        response_header_version(<$str_rq>::KEY, version),
                    )
                    .map_err(BrokerPool::map_proto_err)?;
                    <$str_rs>::decode(&mut response_buf, version)
                        .map_err(BrokerPool::map_proto_err)
                        .map(|v| (resp_header, v))
                }
            }
        }
    };
}

#[allow(dead_code)]
/// Implies utility
impl BrokerPool {
    /// Convert nonstatic reference to string to StrBytes
    /// This function uses unsafe operation because depends on StrBytes type
    fn to_strbytes<T: AsRef<str>>(value: T) -> StrBytes {
        unsafe { StrBytes::from_utf8_unchecked(Bytes::from(value.as_ref().as_bytes().to_vec())) }
    }

    /// Map kafka protool error to KafkaError and furter to anyhow error
    fn map_proto_err<T: Debug>(err: T) -> anyhow::Error {
        KafkaErrors::SomeError(format!("{:?}", err)).into()
    }

    /// Convert map to list of kafka message headers
    fn map_to_headers(map: &HashMap<String, String>) -> Vec<(StrBytes, Option<Bytes>)> {
        map.iter()
            .map(|(k, v)| {
                (
                    BrokerPool::to_strbytes(k),
                    Some(Bytes::from(v.as_bytes().to_vec())),
                )
            })
            .collect()
    }

    /// Prepares request based on api key, cient id and api version
    fn prepare_request<T: Into<i16>>(
        &self,
        api_key: T,
        client_id: &'static str,
        api_version: i16,
    ) -> RequestHeader {
        let mut req_header = RequestHeader::default();
        req_header.request_api_version = api_version;
        req_header.request_api_key = api_key.into();
        req_header.client_id = Some(StrBytes::from_str(&client_id));
        req_header.correlation_id = self
            .correlation
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        req_header
    }
}

#[allow(dead_code)]
/// Implies asynchronous kafka protocol
impl BrokerPool {
    #[inline]
    async fn add_start_broker(&mut self) -> anyhow::Result<()> {
        debug!("add starting broker...");
        let stream = self.config.connect().await?;
        let addr = stream.addr()?;

        let mut brokers = self.brokers.lock().await;
        let broker = brokers.insert(
            0,
            Broker {
                address: addr,
                broker_id: 0,
                stream: Mutex::new(Some(stream)),
                access_time: None,
                config: self.config.clone(),
            },
        );

        debug!("Starting broker added: {:?}, {:?}", brokers, broker);
        Ok(())
    }

    pub async fn init(&mut self) -> anyhow::Result<()> {
        self.add_start_broker().await?;
        let md_ver = if let Some(av) = &self.api_ver {
            av.api_keys[MetadataRequest::KEY as usize].max_version
        } else {
            // Load api versions info
            let (_, api_ver) = self.get_api_ver().await?;
            let md_ver = api_ver.api_keys[MetadataRequest::KEY as usize].max_version;
            self.api_ver = Some(api_ver);
            md_ver
        };

        if self.metadata.is_none() {
            // Load metadata
            let (_, md) = self.get_metadata(md_ver).await?;
            self.update_metadata(&md).await;
            self.metadata = Some(md);
        }

        Ok(())
    }

    /// Refresh metadata from kafka broker
    pub async fn refresh_metadata(&mut self) -> anyhow::Result<()> {
        let md_ver = self.get_version(MetadataRequest::KEY)?.max_version;
        let (_, md) = self.get_metadata(md_ver).await?;
        self.update_metadata(&md).await;
        self.metadata = Some(md);
        Ok(())
    }

    kafka_call!(_get_api_ver, ApiVersionsRequest, ApiVersionsResponse);
    /// Get api versions from kafka broker
    pub async fn get_api_ver(&mut self) -> anyhow::Result<(ResponseHeader, ApiVersionsResponse)> {
        let mut ver = ApiVersionsRequest::default();
        ver.client_software_version = StrBytes::from_str("1.0");
        ver.client_software_name = StrBytes::from_str(self.client_id);
        match self._get_api_ver(&ver, 3, 0).await {
            Ok(res) => Ok(res),
            Err(_) => self._get_api_ver(&ver, 0, -1).await,
        }
    }

    kafka_call!(_get_metadata, MetadataRequest, MetadataResponse);
    /// Get metadata from kafka broker
    pub async fn get_metadata(
        &mut self,
        version: i16,
    ) -> anyhow::Result<(ResponseHeader, MetadataResponse)> {
        let mut rq = MetadataRequest::default();
        rq.topics = None;
        self._get_metadata(&rq, version, -1).await
    }

    kafka_call!(_describe_acl, DescribeAclsRequest, DescribeAclsResponse);
    /// Get ACLs from kafka broker using filter data in request
    pub async fn describe_acl(
        &mut self,
        rq: DescribeAclsRequest,
    ) -> anyhow::Result<(ResponseHeader, DescribeAclsResponse)> {
        // get version
        self._describe_acl(
            &rq,
            self.get_version(DescribeAclsRequest::KEY)?.max_version,
            -1,
        )
        .await
    }

    kafka_call!(_create_acl, CreateAclsRequest, CreateAclsResponse);
    /// Create ACL in kafka
    pub async fn create_acl(
        &mut self,
        rq: CreateAclsRequest,
    ) -> anyhow::Result<(ResponseHeader, CreateAclsResponse)> {
        self._create_acl(
            &rq,
            self.get_version(CreateAclsRequest::KEY)?.max_version,
            -1,
        )
        .await
    }

    kafka_call!(_produce, ProduceRequest, ProduceResponse);
    /// Send single request to kafka with given parameters and following default parameters:
    /// - acks - all
    /// - transactional - false
    /// - control - false
    /// - timestamp_type - TimestampType::Creation
    /// - record batch encoding version - 2
    pub async fn produce<T: AsRef<str>>(
        &mut self,
        topic: T,
        key: T,
        payload: T,
        headers: &HashMap<String, String>,
        create_timestamp: bool,
    ) -> anyhow::Result<(ResponseHeader, ProduceResponse)> {
        let mut rq = ProduceRequest::default();
        rq.acks = -1;
        let topic_name = TopicName(BrokerPool::to_strbytes(topic));
        let mut td = produce_request::TopicProduceData::default();
        let mut pd = produce_request::PartitionProduceData::default();
        let timestamp = if create_timestamp {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_millis()
                .try_into()?
        } else {
            -1
        };

        // Create record with default values
        let mut record = Record {
            transactional: false,
            control: false,
            partition_leader_epoch: -1,
            producer_id: -1,
            producer_epoch: -1,
            timestamp_type: TimestampType::Creation,
            offset: 0,
            sequence: -1,
            timestamp,
            key: Some(Bytes::from(key.as_ref().to_string())),
            value: Some(Bytes::from(payload.as_ref().to_string())),
            headers: Default::default(),
        };
        BrokerPool::map_to_headers(headers)
            .into_iter()
            .for_each(|(k, v)| {
                record.headers.insert(k, v);
            });
        let mut buf = BytesMut::new();
        let opts = RecordEncodeOptions {
            version: 2,
            compression: kafka_protocol::records::Compression::Gzip,
        };
        RecordBatchEncoder::encode(&mut buf, vec![record].iter(), &opts)
            .map_err(BrokerPool::map_proto_err)?;
        pd.index = 0;
        pd.records = Some(buf.into());
        td.partition_data = vec![pd];

        rq.topic_data.insert(topic_name, td);
        self._produce(&rq, self.get_version(ProduceRequest::KEY)?.max_version, -1)
            .await
    }

    kafka_call!(_fetch, FetchRequest, FetchResponse);
    /// Function fetch data from given topic using specified offset and request for messages that satisfy desire isolation level
    pub async fn fetch_from_topics<T: AsRef<str>, I: Into<i8>>(
        &mut self,
        topic: &[T],
        timeout_ms: i32,
        isolation_level: I,
        session_id: i32,
        fetch_offset: i64,
    ) -> anyhow::Result<(ResponseHeader, FetchResponse)> {
        // prepare topic data
        if let Some(md) = &self.metadata {
            let topics = topic
                .iter()
                .map(|t| TopicName(BrokerPool::to_strbytes(t)))
                .filter_map(|tn| md.topics.get(&tn).map(|tmd| (tn, tmd)))
                .filter_map(|(tn, tmd)| {
                    FetchTopicBuilder::default()
                        .topic(tn)
                        .topic_id(tmd.topic_id.clone())
                        .partitions(
                            tmd.partitions
                                .iter()
                                .map(|p| {
                                    let mut par = FetchPartition::default();
                                    par.partition = p.partition_index;
                                    par.fetch_offset = fetch_offset;
                                    par
                                })
                                .collect(),
                        )
                        .unknown_tagged_fields(BTreeMap::new())
                        .build()
                        .ok()
                })
                .collect::<Vec<_>>();

            let mut rq = FetchRequest::default();
            rq.replica_id = BrokerId(-1);
            rq.max_wait_ms = timeout_ms;
            rq.isolation_level = isolation_level.into();
            rq.topics = topics;
            rq.session_id = session_id;
            let version = self.get_version(FetchRequest::KEY)?.max_version;
            self._fetch(&rq, version, -1).await
        } else {
            Err(KafkaErrors::NotInitialized.into())
        }
    }

    kafka_call!(_list_offsets, ListOffsetsRequest, ListOffsetsResponse);
    /// Get offset for topic and partition using isolation level and timestamp shifting.
    /// Timestamp can be one of KafkaListOffsets enum values. To more information see
    /// https://cwiki.apache.org/confluence/pages/viewpage.action?pageId=65868090
    ///
    /// To get watermarks use get_watermarks function
    pub async fn list_offsets<T: AsRef<str>, I: Into<i8>>(
        &mut self,
        topic: T,
        partition: i32,
        isolation_level: I,
        timestamp: KafkaListOffsets,
    ) -> anyhow::Result<(ResponseHeader, ListOffsetsResponse)> {
        let topic_name = TopicName(BrokerPool::to_strbytes(topic));
        let leader = self.get_leader_for_topic(&topic_name, partition)?;

        let mut rq = ListOffsetsRequest::default();
        let topic = ListOffsetsTopicBuilder::default()
            .name(topic_name)
            .partitions(vec![ListOffsetsPartitionBuilder::default()
                .partition_index(partition)
                .max_num_offsets(1)
                .timestamp(timestamp.into())
                .current_leader_epoch(-1)
                .unknown_tagged_fields(Default::default())
                .build()
                .unwrap_or_else(|e| {
                    debug!("Error build partition {:?}", e);
                    let pd = ListOffsetsPartition::default();
                    pd
                })])
            .unknown_tagged_fields(Default::default())
            .build()?;

        rq.topics = vec![topic];
        rq.isolation_level = isolation_level.into();
        let version = self.get_version(ListOffsetsRequest::KEY)?.max_version;
        self._list_offsets(&rq, version, leader).await
    }

    /// Function retrieves start and end offset for given topic and partiotn for messages that satisfy the isolation level
    pub async fn get_watermarks<T: AsRef<str>>(
        &mut self,
        topic: T,
        partition: i32,
        isolation_level: KafkaIsolationLevel,
    ) -> anyhow::Result<(i64, i64)> {
        let (_, start) = self
            .list_offsets(
                &topic,
                partition,
                isolation_level,
                KafkaListOffsets::Beginning,
            )
            .await?;
        let (_, end) = self
            .list_offsets(&topic, partition, isolation_level, KafkaListOffsets::End)
            .await?;
        Ok((
            start.topics[0].partitions[0].offset,
            end.topics[0].partitions[0].offset,
        ))
    }

    kafka_call!(_offset_fetch, OffsetFetchRequest, OffsetFetchResponse);
    /// Function call kafka OffsetFetch protocol function
    pub async fn offset_fetch<T: AsRef<str>>(
        &mut self,
        group_id: T,
        topic: T,
        partitions: &[i32],
    ) -> anyhow::Result<(ResponseHeader, OffsetFetchResponse)> {
        let mut rq = OffsetFetchRequest::default();
        let topic_name = TopicName(BrokerPool::to_strbytes(topic));
        let topic = vec![OffsetFetchRequestTopicBuilder::default()
            .name(topic_name.clone())
            .partition_indexes(partitions.to_vec())
            .unknown_tagged_fields(Default::default())
            .build()?];

        rq.topics = Some(topic);
        rq.require_stable = true;
        let version = self.get_version(OffsetFetchRequest::KEY)?.max_version;
        if version > 7 {
            rq.group_id = GroupId(BrokerPool::to_strbytes(group_id));
        } else {
            rq.groups = vec![OffsetFetchRequestGroupBuilder::default()
                .topics(Some(vec![OffsetFetchRequestTopicsBuilder::default()
                    .name(topic_name)
                    .partition_indexes(partitions.to_vec())
                    .unknown_tagged_fields(Default::default())
                    .build()?]))
                .build()?];
        }

        self._offset_fetch(&rq, version, -1).await
    }

    kafka_call!(_create_topic, CreateTopicsRequest, CreateTopicsResponse);
    /// Function creates topic in kafka using name partitions count, count of replicas and additional configs
    pub async fn create_topic<T: AsRef<str>>(
        &mut self,
        name: T,
        partition_count: i32,
        replica_count: i16,
        configs: &[(T, T)],
    ) -> anyhow::Result<(ResponseHeader, CreateTopicsResponse)> {
        let mut rq = CreateTopicsRequest::default();
        let topic_name = TopicName(BrokerPool::to_strbytes(name));
        let mut topic = CreatableTopic::default();
        topic.num_partitions = partition_count;
        topic.replication_factor = replica_count;
        for (k, v) in configs {
            let config = CreateableTopicConfigBuilder::default()
                .value(if v.as_ref().is_empty() {
                    None
                } else {
                    Some(BrokerPool::to_strbytes(v))
                })
                .unknown_tagged_fields(Default::default())
                .build()?;
            topic.configs.insert(BrokerPool::to_strbytes(k), config);
        }

        rq.topics.insert(topic_name, topic);
        let version = self.get_version(CreateTopicsRequest::KEY)?.max_version;
        self._create_topic(&rq, version, -1).await
    }

    kafka_call!(_list_groups, ListGroupsRequest, ListGroupsResponse);
    /// List group consumers
    pub async fn list_groups(
        &mut self,
    ) -> anyhow::Result<Vec<(ResponseHeader, ListGroupsResponse)>> {
        let rq = ListGroupsRequest::default();
        let version = self.get_version(ListGroupsRequest::KEY)?.max_version;
        let broker_idx = self.get_broker_indexes().await;
        let mut res = Vec::new();
        for idx in broker_idx {
            res.push(self._list_groups(&rq, version, idx).await?);
        }

        Ok(res)
    }

    kafka_call!(
        _describe_groups,
        DescribeGroupsRequest,
        DescribeGroupsResponse
    );
    pub async fn describe_groups(
        &mut self,
        groups: &[String],
    ) -> anyhow::Result<Vec<(ResponseHeader, DescribeGroupsResponse)>> {
        let mut rq = DescribeGroupsRequest::default();
        rq.groups = groups
            .iter()
            .map(|grp| GroupId(BrokerPool::to_strbytes(grp)))
            .collect::<Vec<GroupId>>();
        let version = self.get_version(DescribeGroupsRequest::KEY)?.max_version;
        let broker_idx = self.get_broker_indexes().await;
        let mut res = Vec::new();
        for idx in broker_idx {
            res.push(self._describe_groups(&rq, version, idx).await?);
        }
        Ok(res)
    }

    kafka_call!(_delete_acls, DeleteAclsRequest, DeleteAclsResponse);
    pub async fn delete_acls(
        &mut self,
        rq: DeleteAclsRequest,
    ) -> anyhow::Result<(ResponseHeader, DeleteAclsResponse)> {
        let version = self.get_version(DeleteAclsRequest::KEY)?.max_version;
        self._delete_acls(&rq, version, -1).await
    }

    kafka_call!(_delete_topic, DeleteTopicsRequest, DeleteTopicsResponse);
    pub async fn delete_topic<T: AsRef<str>>(
        &mut self,
        topic: T,
    ) -> anyhow::Result<(ResponseHeader, DeleteTopicsResponse)> {
        let version = self.get_version(DeleteTopicsRequest::KEY)?.max_version;
        let mut rq = DeleteTopicsRequest::default();
        rq.topic_names = vec![TopicName(BrokerPool::to_strbytes(topic))];
        self._delete_topic(&rq, version, -1).await
    }
}
