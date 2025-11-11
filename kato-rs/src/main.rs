#[allow(unused_imports)]
#[allow(dead_code)]
mod config;
mod gui;

use std::{sync::Arc, time::Duration};
use tracing_subscriber::{EnvFilter, Layer, fmt};

use kafka_lib::{BrokerPool, DescribeAclsRequest, KafkaConfig, KafkaIsolationLevel};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[macro_use]
extern crate tracing;

#[tokio::main]
async fn main() {
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("debug"))
        .unwrap();

    tracing_subscriber::registry()
        .with(fmt::layer().with_filter(filter_layer))
        .init();

    let config = config::Config::new().unwrap();
    let mut kui = gui::Kui::new(config);
    let _ = kui.run();

    const CERT_FILENAME: &str =
        "/Users/sbt-sementsov-av/projects/kafka-driver-rs/kafka-main/certs/audit2_kafka.cer";
    const KEY_FILENAME: &str =
        "/Users/sbt-sementsov-av/projects/kafka-driver-rs/kafka-main/certs/audit2_kafka.pkcs8";

    // let cfg = KafkaConfig::new(
    //     "10.21.32.11:9093".to_string(),
    //     "ssl".to_string(),
    //     false,
    //     Some(CERT_FILENAME.to_string()),
    //     Some(KEY_FILENAME.to_string()),
    //     Some(CERT_FILENAME.to_string())).unwrap();

    let cfg = KafkaConfig::new(
        "localhost:9092".to_string(),
        "plaintext".to_string(),
        false,
        Some(CERT_FILENAME.to_string()),
        Some(KEY_FILENAME.to_string()),
        Some(CERT_FILENAME.to_string()),
    )
    .unwrap();

    let cfg = Arc::new(cfg);
    let mut pool = BrokerPool::new(cfg, Duration::from_secs(30), "kafka-tool");
    pool.init().await.unwrap();

    info!("Successfully initialized brokers pool");

    let (_, ver) = pool.get_api_ver().await.unwrap();
    info!("Api versions: {:?}", ver);

    let mut dacl = DescribeAclsRequest::default();
    dacl.resource_type_filter = 1; // ANY
    dacl.pattern_type_filter = 1; // ANY
    dacl.operation = 1; // ANY
    dacl.permission_type = 1; // ANY
    let (rh, acl) = pool.describe_acl(dacl).await.unwrap();
    println!("ACLs: {:?}", acl);
    println!("ACLs: {:?}", rh);

    // let mut headers = HashMap::new();
    // headers.insert("type".to_string(), "auditEvent".to_string());

    // let topic = match k_ops.topic_by_name("test-2") {
    //     Ok(topic) => topic,
    //     Err(err) => {
    //         println!("Error get topic by name: {:?}", err);
    //         let (_, resp) = k_ops.create_topic("test-2", 3, 1, &[("compression.type", "lz4")]).await.unwrap();
    //         println!("CreateTopic: {:?}", resp);
    //         k_ops.refresh_metadata().await.unwrap();
    //         k_ops.topic_by_name("test-2").unwrap()
    //     }
    // };
    // println!("TopicUUID: {}", topic.topic_id);

    // let (rs, data) = k_ops
    //     .fetch_from_topics(&["test"], 100, KafkaIsolationLevel::ReadUncommitted, 0, 0)
    //     .await
    //     .unwrap();
    // println!("\n\n rs: {:?} \n ---- {:?}", rs, data);

    for partition in 0..6 {
        let (start_offset, end_offset) = pool
            .get_watermarks(
                "audit-metamodel-in",
                partition,
                KafkaIsolationLevel::ReadUncommitted,
            )
            .await
            .unwrap();
        info!(
            "\n***** Partition: {}, Offsets: start {}, end: {}",
            partition, start_offset, end_offset
        );
    }

    let response_list = pool.list_groups().await.unwrap();
    let mut groups_rq = Vec::new();
    for (_, groups) in response_list {
        for group in groups.groups {
            let grp_name = group.group_id.to_string();
            if grp_name.starts_with("audit-ose-mq2kafka") {
                info!(
                    "Group name: protocol: {}, state: {}, name: {}",
                    group.protocol_type, group.group_state, grp_name,
                );
                groups_rq.push(grp_name);
            }
        }
    }

    let describes = pool.describe_groups(&groups_rq).await.unwrap();
    for (_, dresp) in describes {
        for dg in dresp.groups {
            if dg.error_code == 0 {
                info!("{:?}", dg);
            }
        }
    }

    for grp in groups_rq {
        let (_, offs) = pool
            .offset_fetch(grp.as_str(), "audit-metamodel-in", &[0])
            .await
            .unwrap();
        println!("\n\nOffsets: {:?}", offs);
    }

    // Fetch data from topics
    // let mut session_id = 0;
    // let mut offset = offsets.topics[0].partitions[0].offset;
    // let (_, offs) = k_ops.offset_fetch("consumer", "test", &[0]).await.unwrap();
    // println!("\n\nOffsets: {:?}", offs);

    // for _ in 0..33 {
    //     let (rs, data) = k_ops
    //         .fetch_from_topics(
    //             &["test"],
    //             100,
    //             KafkaIsolationLevel::ReadUncommitted,
    //             session_id,
    //             offset,
    //         )
    //         .await
    //         .unwrap();
    //     println!("\n\n{:?} session: {}, offset: {} ", rs, session_id, offset);
    //     session_id = data.session_id;
    //     offset += 1;

    //     for resp in &data.responses {
    //         println!("Topic ({}, {})", resp.topic.to_string(), resp.topic_id);
    //         for par in &resp.partitions {
    //             println!(
    //                 "Partition: {}, error: {}, wm: {}",
    //                 par.partition_index, par.error_code, par.high_watermark
    //             );
    //             if let Some(rec) = &par.records {
    //                 let mut buf = rec.clone();
    //                 let rcs = RecordBatchDecoder::decode(&mut buf).unwrap();
    //                 println!("{:?}", rcs);
    //             }
    //         }
    //     }
    // }

    // for i in 0..11 {
    //     match k_ops.produce("test",
    //         format!("key-{}", i).as_str(),
    //         format!("Message with log append ctimestamp - {}", i).as_str(),
    //         &headers,
    //         true
    //     )
    //     .await {
    //         Ok((_, res)) => println!("OK - Result: {:?}", res),
    //         Err(err) => println!("Error {}", err),
    //     };
    // }
}
