use borsh::{BorshDeserialize, BorshSerialize};
use kaspa_utils::networking::{ContextualNetAddress, IpAddress, NetAddress, PeerId};
use serde::{Deserialize, Serialize};

#[cfg(not(target_family = "wasm"))]
use pyo3::pyclass;

pub type RpcNodeId = PeerId;
pub type RpcIpAddress = IpAddress;
pub type RpcPeerAddress = NetAddress;
pub type RpcContextualPeerAddress = ContextualNetAddress;

#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[cfg(not(target_family = "wasm"))]
#[pyclass]
pub struct RpcPeerInfo {
    #[pyo3(get)]
    pub id: RpcNodeId,
    #[pyo3(get)]
    pub address: RpcPeerAddress,
    #[pyo3(get)]
    pub last_ping_duration: u64, // NOTE: i64 in gRPC protowire

    #[pyo3(get)]
    pub is_outbound: bool,
    #[pyo3(get)]
    pub time_offset: i64,
    #[pyo3(get)]
    pub user_agent: String,

    #[pyo3(get)]
    pub advertised_protocol_version: u32,
    #[pyo3(get)]
    pub time_connected: u64, // NOTE: i64 in gRPC protowire
    #[pyo3(get)]
    pub is_ibd_peer: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[cfg(target_family = "wasm")]
pub struct RpcPeerInfo {
    pub id: RpcNodeId,
    pub address: RpcPeerAddress,
    pub last_ping_duration: u64, // NOTE: i64 in gRPC protowire

    pub is_outbound: bool,
    pub time_offset: i64,
    pub user_agent: String,

    pub advertised_protocol_version: u32,
    pub time_connected: u64, // NOTE: i64 in gRPC protowire
    pub is_ibd_peer: bool,
}
