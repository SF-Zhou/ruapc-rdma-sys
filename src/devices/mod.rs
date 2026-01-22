//! # RDMA device management
//!
//! This module provides a high-level, type-safe API for enumerating, opening,
//! and querying RDMA devices on the system.
//!
//! ## Module Organization
//!
//! - [`mod.rs`](self): Devices collection and public API
//! - [`device.rs`](device): Single Device handle implementation
//! - [`types.rs`](types): Public data types (DeviceInfo, Port, Gid)
//! - [`raw.rs`](raw): FFI wrappers with RAII cleanup
//!
//! ## Example
//!
//! ```rust,no_run
//! use ruapc_rdma_sys::Devices;
//!
//! // Discover all available devices
//! let devices = Devices::available()?;
//!
//! for device in &devices {
//!     let info = device.info();
//!     println!("Device: {}", info.name);
//!     println!("  GUID: {}", info.guid);
//!
//!     for port in &info.ports {
//!         println!("  Port {}: {} GIDs", port.port_num, port.gids.len());
//!         for gid in &port.gids {
//!             println!("    GID[{}]: {:?}", gid.index, gid.gid_type);
//!         }
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod device;
mod raw;
mod types;

pub use device::Device;
pub use types::{DeviceInfo, Gid, Port};

use std::{ops::Deref, sync::Arc};

use crate::{DeviceConfig, ErrorKind, Result};

use raw::RawDeviceList;

/// A collection of RDMA devices available on the system.
///
/// Provides access to all available RDMA devices after filtering
/// based on configuration.
#[derive(Clone)]
pub struct Devices(pub Vec<Arc<Device>>);

impl Devices {
    /// Returns a list of available RDMA devices with default configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No RDMA devices are found
    /// - Device opening fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use ruapc_rdma_sys::Devices;
    /// let devices = Devices::available().unwrap();
    /// println!("Found {} RDMA device(s)", devices.len());
    /// ```
    pub fn available() -> Result<Devices> {
        Self::open(&Default::default())
    }

    /// Returns the number of devices in this collection.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if this collection contains no devices.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Opens RDMA devices based on the provided configuration.
    ///
    /// Allows filtering devices by name, GID type, and other criteria.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for device filtering
    ///
    /// # Errors
    ///
    /// Returns an error if device enumeration or opening fails.
    pub fn open(config: &DeviceConfig) -> Result<Devices> {
        let list = RawDeviceList::available()?;
        let mut devices = Vec::with_capacity(list.len());
        for &device in list.iter() {
            // Early filter by device name to avoid expensive device opening
            if !config.device_filter.is_empty() {
                let name = unsafe { Device::device_name(device) };
                if !config.device_filter.contains(&name) {
                    continue;
                }
            }

            let index = devices.len();
            let device = Device::open(device, index, config)?;
            devices.push(Arc::new(device));
        }
        if devices.is_empty() {
            Err(ErrorKind::IBDeviceNotFound.into())
        } else {
            Ok(Devices(devices))
        }
    }
}

impl Deref for Devices {
    type Target = [Arc<Device>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> IntoIterator for &'a Devices {
    type Item = &'a Arc<Device>;
    type IntoIter = std::slice::Iter<'a, Arc<Device>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_devices() {
        let devices = Devices::available().unwrap();
        assert!(!devices.is_empty());
        for device in &devices {
            println!("{:#?}", device);

            let json = serde_json::to_string_pretty(&device.info()).unwrap();
            let der = serde_json::from_str::<DeviceInfo>(&json).unwrap();
            let ser = serde_json::to_string_pretty(&der).unwrap();
            assert_eq!(json, ser);
        }
    }
}
