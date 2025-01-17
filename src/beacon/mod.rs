mod serializer;
mod deserializer;
mod flags;

use std::{fmt::Display, net::IpAddr, time::Duration};
use serde_cbor::Value;

pub type NodeIdentifier = String;

/// A beacon sent periodically to advertize a DTN node
#[derive(Debug, Clone)]
pub struct Beacon {
    /// Beacon version number (5 currently)
    pub version: u8,

    /// Node identifier advertized by this beacon
    pub node_id: Option<NodeIdentifier>,

    /// Sequence number of this beacon
    /// It is incremented by 1 each time a beacon is emitted
    pub sequence_number: u64,

    /// Services available on this node
    /// Services can be convergence layers, application agents or other
    /// informations such as geographical location, battery level or more
    pub services: Vec<Service>,

    /// Duration between two beacon advertizments
    pub period: Option<Duration>
}

impl Beacon {

    /// Create a new v8 beacon
    pub fn new() -> Self {
        Self { 
            version: 8,
            node_id: None, 
            sequence_number: 0,
            services: Vec::new(),
            period: None
        }
    }

    /// Get next beacon
    /// Clone current beacon and increment sequence number
    pub fn next(&self) -> Self {
        let mut next = self.clone();
        next.sequence_number = next.sequence_number.wrapping_add(1);
        next
    }

    /// Get beacon as bytes
    pub fn as_bytes(&self) -> Result<Vec<u8>, serde_cbor::Error> {
        serde_cbor::to_vec(&self)
    }

    /// Parse beacon from bytes
    pub fn parse(bytes: &[u8]) -> Result<Self, serde_cbor::Error> {
        serde_cbor::from_slice(bytes)
    }
}

#[derive(Debug, Clone)]
pub enum Service {
    /// A TCP Convergence Layer v4 (RFC9174)
    /// First parameter is TCP port to connect to
    TCPCLv4(u16),

    /// A TCP Convergence Layer v3 (RFC7242)
    /// First parameter is TCP port to connect to
    TCPCLv3(u16),

    /// A Minimal TCP Convergence-Layer (draft-ietf-dtn-mtcpcl-01)
    /// First parameter us TCP port to connect to
    #[allow(clippy::upper_case_acronyms)]
    MTCPCL(u16),

    /// Geo location of node
    /// (latitude, longitude)
    GeoLocation(f32, f32),

    /// Physical address of node
    Address(String),

    /// An Unknown service
    /// (Service flag, service value)
    Unknown(u8, Value)
}

/// This service is not a convergence layer and cannot be converted into cla address
#[derive(Debug)]
pub struct NotClaError;

impl Display for NotClaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "This service is not a convergence layer")
    }
}

impl Service {
    pub fn is_cla(&self) -> bool {
        matches!(self, Service::TCPCLv4(_)|Service::TCPCLv3(_)|Service::MTCPCL(_))
    }

    pub fn as_cla_address(&self, source_address: IpAddr) -> Result<String, NotClaError> {
        match self {
            Service::TCPCLv4(port) => Ok(format!("tcpclv4:{}:{}", format_ip(source_address), port)),
            Service::TCPCLv3(port) => Ok(format!("tcpclv3:{}:{}", format_ip(source_address), port)),
            Service::MTCPCL(port) => Ok(format!("mtcp:{}:{}", format_ip(source_address), port)),
            _ => Err(NotClaError)
        }
    }
}

fn format_ip(ip: IpAddr) -> String {
    match ip {
        IpAddr::V4(ipv4_addr) => ipv4_addr.to_string(),
        IpAddr::V6(ipv6_addr) => if let Some(ipv4_addr) = ipv6_addr.to_ipv4() {
            ipv4_addr.to_string()
        } else {
            format!("[{}]", ipv6_addr)
        },
    }
}