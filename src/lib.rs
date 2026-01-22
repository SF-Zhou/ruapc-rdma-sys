//! # RDMA verbs bindings for ruapc
//!
//! This crate provides type-safe device management and low-level FFI bindings
//! to libibverbs (RDMA verbs) with JSON serialization support.
//!
//! ## Device Discovery
//!
//! The high-level [`Devices`] API provides safe device enumeration and querying:
//!
//! ```rust,no_run
//! use ruapc_rdma_sys::Devices;
//!
//! let devices = Devices::available()?;
//! println!("Found {} RDMA device(s)", devices.len());
//!
//! for device in &devices {
//!     let info = device.info();
//!     println!("  {}: {} ports", info.name, info.ports.len());
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Device Filtering
//!
//! Use [`DeviceConfig`] to filter devices by name, GID type, or port state:
//!
//! ```rust,no_run
//! use ruapc_rdma_sys::{Devices, DeviceConfig, GidType};
//! use std::collections::HashSet;
//!
//! let mut config = DeviceConfig::default();
//! config.gid_type_filter = HashSet::from([GidType::RoCEv2]);
//! config.skip_inactive_port = true;
//!
//! let devices = Devices::open(&config)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Public API
//!
//! ### Device Management
//! - [`Devices`]: Collection of RDMA devices with filtering support
//! - [`Device`]: Opened RDMA device with allocated protection domain
//! - [`DeviceInfo`]: Device metadata including name, GUID, ports, and capabilities
//! - [`Port`]: Port information with GID list
//! - [`Gid`]: Global Identifier entry with type (IB/RoCE)
//!
//! ### Configuration
//! - [`DeviceConfig`]: Device/port/GID filtering options
//! - [`GidType`]: IB/RoCE GID type enumeration
//!
//! ### Custom Types
//! - [`Guid`]: 64-bit device identifier with colon-separated formatting
//! - [`FwVer`]: Firmware version wrapper
//! - [`LinkLayer`]: Link layer type (InfiniBand/Ethernet)
//! - [`WRID`]: Work completion ID with type encoding
//! - [`WCType`]: Work completion operation type (Recv/SendData/SendImm)
//!
//! ### FFI Wrapper Functions
//! - [`ibv_poll_cq`]: Poll completion queue for work completions
//! - [`ibv_post_send`]: Post send work request to a queue pair
//! - [`ibv_post_recv`]: Post receive work request to a queue pair
//! - [`ibv_req_notify_cq`]: Request completion queue event notifications
//!
//! ## Generated Bindings
//!
//! All other types and functions from libibverbs are included via the generated
//! bindings in `$OUT_DIR/bindings.rs`. See [build.rs] for details on how custom
//! type replacements are applied during build.

#![allow(dead_code)]
#![allow(deref_nullptr)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc, clippy::too_many_arguments)]

// Make derive macros available for generated bindings
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Re-export pthread wrapper types BEFORE including bindings
// This allows build.rs to replace types in generated bindings
pub use types::{pthread_cond_t, pthread_mutex_t};

// Include generated bindings (only once - in lib.rs)
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

mod error;
pub use error::{Error, ErrorKind, Result};

mod config;
pub use config::{DeviceConfig, GidType};

mod devices;
pub use devices::{Device, DeviceInfo, Devices, Gid, Port};

mod ffi;
pub use ffi::{ibv_poll_cq, ibv_post_recv, ibv_post_send, ibv_req_notify_cq};

mod types;
pub use types::{FwVer, Guid, LinkLayer, WCType, WRID};
