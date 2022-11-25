/// Function used to determine relations between ApiKey and Response header version
/// Because not generated automatically from kafka_protocol it copied from kafka source manually
pub(crate) fn request_header_version(api_key: i16, version: i16) -> i16 {
    match api_key {
        0 =>
        // Produce
        {
            if version >= 9 {
                2
            } else {
                1
            }
        }
        1 =>
        // Fetch
        {
            if version >= 12 {
                2
            } else {
                1
            }
        }
        2 =>
        // ListOffsets
        {
            if version >= 6 {
                2
            } else {
                1
            }
        }
        3 =>
        // Metadata
        {
            if version >= 9 {
                2
            } else {
                1
            }
        }
        4 =>
        // LeaderAndIsr
        {
            if version >= 4 {
                2
            } else {
                1
            }
        }
        5 =>
        // StopReplica
        {
            if version >= 2 {
                2
            } else {
                1
            }
        }
        6 =>
        // UpdateMetadata
        {
            if version >= 6 {
                2
            } else {
                1
            }
        }
        7 =>
        // ControlledShutdown
        // Version 0 of ControlledShutdownRequest has a non-standard request header
        // which does not include clientId.  Version 1 of ControlledShutdownRequest
        // and later use the standard request header.
        {
            if version == 0 {
                0
            } else if version >= 3 {
                2
            } else {
                1
            }
        }
        8 =>
        // OffsetCommit
        {
            if version >= 8 {
                2
            } else {
                1
            }
        }
        9 =>
        // OffsetFetch
        {
            if version >= 6 {
                2
            } else {
                1
            }
        }
        10 =>
        // FindCoordinator
        {
            if version >= 3 {
                2
            } else {
                1
            }
        }
        11 =>
        // JoinGroup
        {
            if version >= 6 {
                2
            } else {
                1
            }
        }
        12 =>
        // Heartbeat
        {
            if version >= 4 {
                2
            } else {
                1
            }
        }
        13 =>
        // LeaveGroup
        {
            if version >= 4 {
                2
            } else {
                1
            }
        }
        14 =>
        // SyncGroup
        {
            if version >= 4 {
                2
            } else {
                1
            }
        }
        15 =>
        // DescribeGroups
        {
            if version >= 5 {
                2
            } else {
                1
            }
        }
        16 =>
        // ListGroups
        {
            if version >= 3 {
                2
            } else {
                1
            }
        }
        17 =>
        // SaslHandshake
        {
            1
        }
        18 =>
        // ApiVersions
        {
            if version >= 3 {
                2
            } else {
                1
            }
        }
        19 =>
        // CreateTopics
        {
            if version >= 5 {
                2
            } else {
                1
            }
        }
        20 =>
        // DeleteTopics
        {
            if version >= 4 {
                2
            } else {
                1
            }
        }
        21 =>
        // DeleteRecords
        {
            if version >= 2 {
                2
            } else {
                1
            }
        }
        22 =>
        // InitProducerId
        {
            if version >= 2 {
                2
            } else {
                1
            }
        }
        23 =>
        // OffsetForLeaderEpoch
        {
            if version >= 4 {
                2
            } else {
                1
            }
        }
        24 =>
        // AddPartitionsToTxn
        {
            if version >= 3 {
                2
            } else {
                1
            }
        }
        25 =>
        // AddOffsetsToTxn
        {
            if version >= 3 {
                2
            } else {
                1
            }
        }
        26 =>
        // EndTxn
        {
            if version >= 3 {
                2
            } else {
                1
            }
        }
        27 =>
        // WriteTxnMarkers
        {
            if version >= 1 {
                2
            } else {
                1
            }
        }
        28 =>
        // TxnOffsetCommit
        {
            if version >= 3 {
                2
            } else {
                1
            }
        }
        29 =>
        // DescribeAcls
        {
            if version >= 2 {
                2
            } else {
                1
            }
        }
        30 =>
        // CreateAcls
        {
            if version >= 2 {
                2
            } else {
                1
            }
        }
        31 =>
        // DeleteAcls
        {
            if version >= 2 {
                2
            } else {
                1
            }
        }
        32 =>
        // DescribeConfigs
        {
            if version >= 4 {
                2
            } else {
                1
            }
        }
        33 =>
        // AlterConfigs
        {
            if version >= 2 {
                2
            } else {
                1
            }
        }
        34 =>
        // AlterReplicaLogDirs
        {
            if version >= 2 {
                2
            } else {
                1
            }
        }
        35 =>
        // DescribeLogDirs
        {
            if version >= 2 {
                2
            } else {
                1
            }
        }
        36 =>
        // SaslAuthenticate
        {
            if version >= 2 {
                2
            } else {
                1
            }
        }
        37 =>
        // CreatePartitions
        {
            if version >= 2 {
                2
            } else {
                1
            }
        }
        38 =>
        // CreateDelegationToken
        {
            if version >= 2 {
                2
            } else {
                1
            }
        }
        39 =>
        // RenewDelegationToken
        {
            if version >= 2 {
                2
            } else {
                1
            }
        }
        40 =>
        // ExpireDelegationToken
        {
            if version >= 2 {
                2
            } else {
                1
            }
        }
        41 =>
        // DescribeDelegationToken
        {
            if version >= 2 {
                2
            } else {
                1
            }
        }
        42 =>
        // DeleteGroups
        {
            if version >= 2 {
                2
            } else {
                1
            }
        }
        43 =>
        // ElectLeaders
        {
            if version >= 2 {
                2
            } else {
                1
            }
        }
        44 =>
        // IncrementalAlterConfigs
        {
            if version >= 1 {
                2
            } else {
                1
            }
        }
        45 =>
        // AlterPartitionReassignments
        {
            2
        }
        46 =>
        // ListPartitionReassignments
        {
            2
        }
        47 =>
        // OffsetDelete
        {
            1
        }
        48 =>
        // DescribeClientQuotas
        {
            if version >= 1 {
                2
            } else {
                1
            }
        }
        49 =>
        // AlterClientQuotas
        {
            if version >= 1 {
                2
            } else {
                1
            }
        }
        50 =>
        // DescribeUserScramCredentials
        {
            2
        }
        51 =>
        // AlterUserScramCredentials
        {
            2
        }
        52 =>
        // Vote
        {
            2
        }
        53 =>
        // BeginQuorumEpoch
        {
            1
        }
        54 =>
        // EndQuorumEpoch
        {
            1
        }
        55 =>
        // DescribeQuorum
        {
            2
        }
        56 =>
        // AlterPartition
        {
            2
        }
        57 =>
        // UpdateFeatures
        {
            2
        }
        58 =>
        // Envelope
        {
            2
        }
        59 =>
        // FetchSnapshot
        {
            2
        }
        60 =>
        // DescribeCluster
        {
            2
        }
        61 =>
        // DescribeProducers
        {
            2
        }
        62 =>
        // BrokerRegistration
        {
            2
        }
        63 =>
        // BrokerHeartbeat
        {
            2
        }
        64 =>
        // UnregisterBroker
        {
            2
        }
        65 =>
        // DescribeTransactions
        {
            2
        }
        66 =>
        // ListTransactions
        {
            2
        }
        67 =>
        // AllocateProducerIds
        {
            2
        }
        _ => 0,
    }
}

/// Function keeps correlation between api version and related response header version
pub(crate) fn response_header_version(api_key: i16, _version: i16) -> i16 {
    match api_key {
        0 =>
        // Produce
        {
            if _version >= 9 {
                1
            } else {
                0
            }
        }
        1 =>
        // Fetch
        {
            if _version >= 12 {
                1
            } else {
                0
            }
        }
        2 =>
        // ListOffsets
        {
            if _version >= 6 {
                1
            } else {
                0
            }
        }
        3 =>
        // Metadata
        {
            if _version >= 9 {
                1
            } else {
                0
            }
        }
        4 =>
        // LeaderAndIsr
        {
            if _version >= 4 {
                1
            } else {
                0
            }
        }
        5 =>
        // StopReplica
        {
            if _version >= 2 {
                1
            } else {
                0
            }
        }
        6 =>
        // UpdateMetadata
        {
            if _version >= 6 {
                1
            } else {
                0
            }
        }
        7 =>
        // ControlledShutdown
        {
            if _version >= 3 {
                1
            } else {
                0
            }
        }
        8 =>
        // OffsetCommit
        {
            if _version >= 8 {
                1
            } else {
                0
            }
        }
        9 =>
        // OffsetFetch
        {
            if _version >= 6 {
                1
            } else {
                0
            }
        }
        10 =>
        // FindCoordinator
        {
            if _version >= 3 {
                1
            } else {
                0
            }
        }
        11 =>
        // JoinGroup
        {
            if _version >= 6 {
                1
            } else {
                0
            }
        }
        12 =>
        // Heartbeat
        {
            if _version >= 4 {
                1
            } else {
                0
            }
        }
        13 =>
        // LeaveGroup
        {
            if _version >= 4 {
                1
            } else {
                0
            }
        }
        14 =>
        // SyncGroup
        {
            if _version >= 4 {
                1
            } else {
                0
            }
        }
        15 =>
        // DescribeGroups
        {
            if _version >= 5 {
                1
            } else {
                0
            }
        }
        16 =>
        // ListGroups
        {
            if _version >= 3 {
                1
            } else {
                0
            }
        }
        17 =>
        // SaslHandshake
        {
            0
        }
        18 =>
        // ApiVersions
        // ApiVersionsResponse always includes a v0 header.
        // See KIP-511 for details.
        {
            0
        }
        19 =>
        // CreateTopics
        {
            if _version >= 5 {
                1
            } else {
                0
            }
        }
        20 =>
        // DeleteTopics
        {
            if _version >= 4 {
                1
            } else {
                0
            }
        }
        21 =>
        // DeleteRecords
        {
            if _version >= 2 {
                1
            } else {
                0
            }
        }
        22 =>
        // InitProducerId
        {
            if _version >= 2 {
                1
            } else {
                0
            }
        }
        23 =>
        // OffsetForLeaderEpoch
        {
            if _version >= 4 {
                1
            } else {
                0
            }
        }
        24 =>
        // AddPartitionsToTxn
        {
            if _version >= 3 {
                1
            } else {
                0
            }
        }
        25 =>
        // AddOffsetsToTxn
        {
            if _version >= 3 {
                1
            } else {
                0
            }
        }
        26 =>
        // EndTxn
        {
            if _version >= 3 {
                1
            } else {
                0
            }
        }
        27 =>
        // WriteTxnMarkers
        {
            if _version >= 1 {
                1
            } else {
                0
            }
        }
        28 =>
        // TxnOffsetCommit
        {
            if _version >= 3 {
                1
            } else {
                0
            }
        }
        29 =>
        // DescribeAcls
        {
            if _version >= 2 {
                1
            } else {
                0
            }
        }
        30 =>
        // CreateAcls
        {
            if _version >= 2 {
                1
            } else {
                0
            }
        }
        31 =>
        // DeleteAcls
        {
            if _version >= 2 {
                1
            } else {
                0
            }
        }
        32 =>
        // DescribeConfigs
        {
            if _version >= 4 {
                1
            } else {
                0
            }
        }
        33 =>
        // AlterConfigs
        {
            if _version >= 2 {
                1
            } else {
                0
            }
        }
        34 =>
        // AlterReplicaLogDirs
        {
            if _version >= 2 {
                1
            } else {
                0
            }
        }
        35 =>
        // DescribeLogDirs
        {
            if _version >= 2 {
                1
            } else {
                0
            }
        }
        36 =>
        // SaslAuthenticate
        {
            if _version >= 2 {
                1
            } else {
                0
            }
        }
        37 =>
        // CreatePartitions
        {
            if _version >= 2 {
                1
            } else {
                0
            }
        }
        38 =>
        // CreateDelegationToken
        {
            if _version >= 2 {
                1
            } else {
                0
            }
        }
        39 =>
        // RenewDelegationToken
        {
            if _version >= 2 {
                1
            } else {
                0
            }
        }
        40 =>
        // ExpireDelegationToken
        {
            if _version >= 2 {
                1
            } else {
                0
            }
        }
        41 =>
        // DescribeDelegationToken
        {
            if _version >= 2 {
                1
            } else {
                0
            }
        }
        42 =>
        // DeleteGroups
        {
            if _version >= 2 {
                1
            } else {
                0
            }
        }
        43 =>
        // ElectLeaders
        {
            if _version >= 2 {
                1
            } else {
                0
            }
        }
        44 =>
        // IncrementalAlterConfigs
        {
            if _version >= 1 {
                1
            } else {
                0
            }
        }
        45 =>
        // AlterPartitionReassignments
        {
            1
        }
        46 =>
        // ListPartitionReassignments
        {
            1
        }
        47 =>
        // OffsetDelete
        {
            0
        }
        48 =>
        // DescribeClientQuotas
        {
            if _version >= 1 {
                1
            } else {
                0
            }
        }
        49 =>
        // AlterClientQuotas
        {
            if _version >= 1 {
                1
            } else {
                0
            }
        }
        50 =>
        // DescribeUserScramCredentials
        {
            1
        }
        51 =>
        // AlterUserScramCredentials
        {
            1
        }
        52 =>
        // Vote
        {
            1
        }
        53 =>
        // BeginQuorumEpoch
        {
            0
        }
        54 =>
        // EndQuorumEpoch
        {
            0
        }
        55 =>
        // DescribeQuorum
        {
            1
        }
        56 =>
        // AlterPartition
        {
            1
        }
        57 =>
        // UpdateFeatures
        {
            1
        }
        58 =>
        // Envelope
        {
            1
        }
        59 =>
        // FetchSnapshot
        {
            1
        }
        60 =>
        // DescribeCluster
        {
            1
        }
        61 =>
        // DescribeProducers
        {
            1
        }
        62 =>
        // BrokerRegistration
        {
            1
        }
        63 =>
        // BrokerHeartbeat
        {
            1
        }
        64 =>
        // UnregisterBroker
        {
            1
        }
        65 =>
        // DescribeTransactions
        {
            1
        }
        66 =>
        // ListTransactions
        {
            1
        }
        67 =>
        // AllocateProducerIds
        {
            1
        }
        _ => 0,
    }
}
