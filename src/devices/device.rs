//! # RDMA device handle
//!
//! This module contains the [`Device`] type which represents an opened RDMA device
//! with an allocated protection domain (PD).
//!
//! ## Resource Management
//!
//! The `Device` type automatically manages the lifecycle of associated RDMA resources:
//! - `ibv_context` is closed via `ibv_close_device` on drop
//! - `ibv_pd` is deallocated via `ibv_dealloc_pd` on drop
//!
//! This ensures proper cleanup even when errors occur during initialization or use.

use std::{ffi::CStr, os::unix::ffi::OsStrExt, path::Path};

use super::{raw::*, types::*};
use crate::{DeviceConfig, ErrorKind, GidType, Guid, Result};

/// RDMA device handle.
///
/// Represents an opened RDMA device with an allocated protection domain.
/// The device is used to create queue pairs, register memory, and perform
/// RDMA operations.
///
/// # Examples
///
/// ```rust,no_run
/// # use ruapc_rdma_sys::Devices;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let devices = Devices::available()?;
/// let device = devices.first().unwrap();
/// println!("Device name: {}", device.info().name);
/// # Ok(())
/// # }
/// ```
pub struct Device {
    protection_domain: RawProtectionDomain,
    context: RawContext,
    device: *mut crate::ibv_device,
    info: DeviceInfo,
}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

impl Device {
    /// Returns the device name from a raw device pointer.
    ///
    /// # Safety
    ///
    /// The `device` pointer must be valid and obtained from `ibv_get_device_list`.
    pub(crate) unsafe fn device_name(device: *mut crate::ibv_device) -> String {
        // SAFETY: caller guarantees device pointer is valid
        unsafe {
            CStr::from_ptr((*device).name.as_ptr())
                .to_string_lossy()
                .to_string()
        }
    }

    /// Opens a device by raw pointer and initializes its protection domain.
    pub(crate) fn open(
        device: *mut crate::ibv_device,
        index: usize,
        config: &DeviceConfig,
    ) -> Result<Self> {
        let name = unsafe { Self::device_name(device) };
        let guid = Guid::from_be(unsafe { crate::ibv_get_device_guid(device) });
        let ibdev_path = unsafe {
            Path::new(std::ffi::OsStr::from_bytes(
                CStr::from_ptr((*device).ibdev_path.as_ptr()).to_bytes(),
            ))
        }
        .to_path_buf();

        let context = RawContext(unsafe {
            let ctx = crate::ibv_open_device(device);
            if ctx.is_null() {
                return Err(ErrorKind::IBOpenDeviceFail.with_errno());
            }
            ctx
        });

        let protection_domain = RawProtectionDomain(unsafe {
            let pd = crate::ibv_alloc_pd(context.0);
            if pd.is_null() {
                return Err(ErrorKind::IBAllocPDFail.with_errno());
            }
            pd
        });

        let mut this = Self {
            protection_domain,
            context,
            device,
            info: DeviceInfo {
                index,
                name,
                guid,
                ibdev_path,
                ..Default::default()
            },
        };
        this.update_attr(config)?;

        Ok(this)
    }

    /// Updates device attributes by querying the hardware.
    pub fn update_attr(&mut self, config: &DeviceConfig) -> Result<()> {
        let device_attr = self.context.query_device()?;

        let mut ports = Vec::with_capacity(device_attr.phys_port_cnt as usize);
        for port_num in 1..=device_attr.phys_port_cnt {
            let port_attr = self.context.query_port(port_num)?;
            if port_attr.state != crate::ibv_port_state::IBV_PORT_ACTIVE
                && config.skip_inactive_port
            {
                continue;
            }

            let gids = self.collect_port_gids(port_num, &port_attr, config);
            ports.push(Port {
                port_num,
                port_attr,
                gids,
            });
        }

        self.info.device_attr = device_attr;
        self.info.ports = ports;

        Ok(())
    }

    /// Collects GIDs for a port after applying filters.
    fn collect_port_gids(
        &self,
        port_num: u8,
        port_attr: &crate::ibv_port_attr,
        config: &DeviceConfig,
    ) -> Vec<Gid> {
        let mut gids = Vec::with_capacity(port_attr.gid_tbl_len as usize);
        for gid_index in 0..port_attr.gid_tbl_len as u16 {
            let Ok(gid) = self.context.query_gid(port_num, gid_index) else {
                continue;
            };
            let Ok(gid_type) =
                self.context
                    .query_gid_type(port_num, gid_index, &self.info.ibdev_path, port_attr)
            else {
                continue;
            };

            // Apply GID type filter
            if !config.gid_type_filter.is_empty() && !config.gid_type_filter.contains(&gid_type) {
                continue;
            }

            // Skip RoCEv2 link-local addresses if configured
            if config.roce_v2_skip_link_local_addr && gid_type == GidType::RoCEv2 {
                let ip = gid.as_ipv6();
                if ip.is_unicast_link_local() {
                    continue;
                }
            }

            gids.push(Gid {
                index: gid_index,
                gid,
                gid_type,
            })
        }
        gids
    }

    /// Returns the raw device pointer.
    ///
    /// # Safety
    ///
    /// The returned pointer is only valid as long as this `Device` exists.
    pub unsafe fn device_ptr(&self) -> *mut crate::ibv_device {
        self.device
    }

    /// Returns the raw context pointer.
    ///
    /// # Safety
    ///
    /// The returned pointer is only valid as long as this `Device` exists.
    pub unsafe fn context_ptr(&self) -> *mut crate::ibv_context {
        self.context.0
    }

    /// Returns the raw protection domain pointer.
    ///
    /// # Safety
    ///
    /// The returned pointer is only valid as long as this `Device` exists.
    pub unsafe fn pd_ptr(&self) -> *mut crate::ibv_pd {
        self.protection_domain.0
    }

    /// Returns the device index.
    ///
    /// # Returns
    ///
    /// The zero-based index of this device in the system.
    pub fn index(&self) -> usize {
        self.info.index
    }

    /// Returns device information.
    ///
    /// # Returns
    ///
    /// A reference to the device's metadata and capabilities.
    pub fn info(&self) -> &DeviceInfo {
        &self.info
    }
}

impl std::fmt::Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.info, f)
    }
}
