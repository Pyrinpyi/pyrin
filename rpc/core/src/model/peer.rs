use borsh::{BorshDeserialize, BorshSerialize};
use pyo3::pyclass;
use kaspa_utils::networking::{ContextualNetAddress, IpAddress, NetAddress, PeerId};
use serde::{Deserialize, Serialize};

pub type RpcNodeId = PeerId;
pub type RpcIpAddress = IpAddress;
pub type RpcPeerAddress = NetAddress;
pub type RpcContextualPeerAddress = ContextualNetAddress;

#[derive(Clone, Debug, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
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
