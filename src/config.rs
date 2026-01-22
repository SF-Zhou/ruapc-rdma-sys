use std::collections::HashSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Global Identifier (GID) type for InfiniBand/RoCE networks.
///
/// Different GID types represent different network layer protocols:
/// - IB: Native InfiniBand
/// - RoCEv1: RDMA over Converged Ethernet version 1
/// - RoCEv2: RDMA over Converged Ethernet version 2
/// - Other: Custom or unrecognized GID type
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, clap::ValueEnum,
)]
pub enum GidType {
    /// Native InfiniBand.
    #[clap(name = "IB")]
    IB,
    /// RDMA over Converged Ethernet version 1.
    #[clap(name = "RoCEv1")]
    RoCEv1,
    /// RDMA over Converged Ethernet version 2.
    #[clap(name = "RoCEv2")]
    RoCEv2,
    /// Custom or unrecognized GID type.
    #[clap(skip)]
    Other(String),
}

/// Device-level configuration for RDMA device filtering.
///
/// Controls which devices, ports, and GID types are selected
/// for RDMA operations.
#[derive(Debug, Clone, Default)]
pub struct DeviceConfig {
    /// Set of device names to include. Empty means all devices.
    pub device_filter: HashSet<String>,
    /// Set of GID types to include. Empty means all types.
    pub gid_type_filter: HashSet<GidType>,
    /// Whether to skip inactive ports during device enumeration.
    pub skip_inactive_port: bool,
    /// For RoCE v2, whether to skip link-local addresses.
    pub roce_v2_skip_link_local_addr: bool,
}

impl DeviceConfig {
    /// Creates a new builder for DeviceConfig.
    pub fn builder() -> DeviceConfigBuilder {
        DeviceConfigBuilder::default()
    }

    /// Adds a device name to the filter.
    pub fn with_device(mut self, device: impl Into<String>) -> Self {
        self.device_filter.insert(device.into());
        self
    }

    /// Adds a GID type to the filter.
    pub fn with_gid_type(mut self, gid_type: GidType) -> Self {
        self.gid_type_filter.insert(gid_type);
        self
    }

    /// Sets whether to skip inactive ports.
    pub fn with_skip_inactive(mut self, skip: bool) -> Self {
        self.skip_inactive_port = skip;
        self
    }

    /// Sets whether to skip RoCEv2 link-local addresses.
    pub fn with_skip_link_local(mut self, skip: bool) -> Self {
        self.roce_v2_skip_link_local_addr = skip;
        self
    }
}

/// Builder for [`DeviceConfig`].
///
/// Provides a fluent interface for constructing device configurations.
#[derive(Debug, Default)]
pub struct DeviceConfigBuilder {
    config: DeviceConfig,
}

impl DeviceConfigBuilder {
    /// Adds a device name to the filter.
    pub fn device(mut self, device: impl Into<String>) -> Self {
        self.config.device_filter.insert(device.into());
        self
    }

    /// Adds multiple device names to the filter.
    pub fn devices<I, S>(mut self, devices: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.config
            .device_filter
            .extend(devices.into_iter().map(Into::into));
        self
    }

    /// Adds a GID type to the filter.
    pub fn gid_type(mut self, gid_type: GidType) -> Self {
        self.config.gid_type_filter.insert(gid_type);
        self
    }

    /// Adds multiple GID types to the filter.
    pub fn gid_types<I>(mut self, gid_types: I) -> Self
    where
        I: IntoIterator<Item = GidType>,
    {
        self.config.gid_type_filter.extend(gid_types);
        self
    }

    /// Sets whether to skip inactive ports.
    pub fn skip_inactive(mut self, skip: bool) -> Self {
        self.config.skip_inactive_port = skip;
        self
    }

    /// Sets whether to skip RoCEv2 link-local addresses.
    pub fn skip_link_local(mut self, skip: bool) -> Self {
        self.config.roce_v2_skip_link_local_addr = skip;
        self
    }

    /// Builds the final [`DeviceConfig`].
    pub fn build(self) -> DeviceConfig {
        self.config
    }
}
