//! RDMA GID (Global Identifier) type with serialization support
//!
//! The GID is a 128-bit identifier used for addressing in RDMA networks.
//! It can be represented as an IPv6 address.

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{borrow::Cow, net::Ipv6Addr};

pub use crate::ibv_gid;

impl ibv_gid {
    /// Returns the raw GID bytes
    pub fn as_raw(&self) -> &[u8; 16] {
        unsafe { &self.raw }
    }

    /// Returns the GID as a 128-bit integer
    pub fn as_bits(&self) -> u128 {
        u128::from_be_bytes(unsafe { self.raw })
    }

    /// Returns the GID as an IPv6 address
    pub fn as_ipv6(&self) -> Ipv6Addr {
        Ipv6Addr::from_bits(self.as_bits())
    }

    /// Returns the subnet prefix portion of the GID
    pub fn subnet_prefix(&self) -> u64 {
        u64::from_be(unsafe { self.global.subnet_prefix })
    }

    /// Returns the interface ID portion of the GID
    pub fn interface_id(&self) -> u64 {
        u64::from_be(unsafe { self.global.interface_id })
    }

    /// Checks if the GID is null (zero interface ID)
    pub fn is_null(&self) -> bool {
        self.interface_id() == 0
    }
}

impl std::fmt::Debug for ibv_gid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ipv6())
    }
}

impl Serialize for ibv_gid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.as_ipv6().to_string())
    }
}

impl<'de> Deserialize<'de> for ibv_gid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let s = String::deserialize(deserializer)?;
        let addr = s
            .parse::<Ipv6Addr>()
            .map_err(|_| D::Error::custom("invalid IPv6 address format"))?;
        let bits = addr.to_bits();
        let mut gid = ibv_gid::default();
        gid.global.subnet_prefix = ((bits >> 64) as u64).to_be();
        gid.global.interface_id = (bits as u64).to_be();
        Ok(gid)
    }
}

impl JsonSchema for ibv_gid {
    fn schema_name() -> Cow<'static, str> {
        "GID".into()
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "pattern": "^(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}$",
            "description": "IPv6 address format GID"
        })
    }
}
