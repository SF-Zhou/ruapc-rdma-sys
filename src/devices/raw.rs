//! # Raw FFI wrappers with RAII cleanup
//!
//! This module contains low-level wrappers around libibverbs FFI types that
//! handle automatic resource cleanup via Drop implementations.
//!
//! ## Types
//!
//! - [`RawDeviceList`]: Wrapper for device list from `ibv_get_device_list`
//! - [`RawContext`]: Wrapper for `ibv_context` from `ibv_open_device`
//! - [`RawProtectionDomain`]: Wrapper for `ibv_pd` from `ibv_alloc_pd`
//!
//! ## Resource Safety
//!
//! All wrapper types automatically clean up their underlying FFI resources when
//! dropped, preventing resource leaks even during error conditions.

use std::{ops::Deref, path::Path};

use crate::{Error, ErrorKind, GidType, LinkLayer, Result};

/// GID type string values from sysfs.
const GID_TYPE_IB_ROCE_V1: &str = "IB/RoCE v1\n";
const GID_TYPE_ROCE_V2: &str = "RoCE v2\n";

/// Raw device list wrapper with automatic cleanup.
///
/// Wraps the pointer returned by `ibv_get_device_list` and ensures
/// proper cleanup via `ibv_free_device_list` when dropped.
pub struct RawDeviceList {
    ptr: *mut *mut crate::ibv_device,
    num_devices: usize,
}

impl RawDeviceList {
    /// Returns the list of available RDMA devices.
    ///
    /// # Errors
    ///
    /// Returns an error if device list retrieval fails or no devices are found.
    pub fn available() -> Result<Self> {
        let mut num_devices: libc::c_int = 0;
        let ptr = unsafe { crate::ibv_get_device_list(&mut num_devices) };
        if ptr.is_null() {
            return Err(ErrorKind::IBGetDeviceListFail.with_errno());
        }
        if num_devices == 0 {
            return Err(ErrorKind::IBDeviceNotFound.into());
        }
        Ok(Self {
            ptr,
            num_devices: num_devices as usize,
        })
    }
}

impl Drop for RawDeviceList {
    fn drop(&mut self) {
        unsafe { crate::ibv_free_device_list(self.ptr) };
    }
}

impl Deref for RawDeviceList {
    type Target = [*mut crate::ibv_device];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr, self.num_devices) }
    }
}

unsafe impl Send for RawDeviceList {}
unsafe impl Sync for RawDeviceList {}

/// Raw context wrapper with automatic cleanup.
///
/// Wraps an `ibv_context` pointer and ensures proper cleanup via
/// `ibv_close_device` when dropped.
pub struct RawContext(pub *mut crate::ibv_context);

impl Drop for RawContext {
    fn drop(&mut self) {
        let _ = unsafe { crate::ibv_close_device(self.0) };
    }
}

impl RawContext {
    /// Executes a query FFI function and converts return code to Result.
    ///
    /// # Arguments
    ///
    /// * `query_fn` - FFI query function that returns 0 on success
    /// * `error_kind` - Error kind to use on failure
    ///
    /// # Safety
    ///
    /// The query function must correctly handle all arguments.
    unsafe fn query_with_errno<F>(&self, query_fn: F, error_kind: ErrorKind) -> Result<()>
    where
        F: FnOnce() -> libc::c_int,
    {
        let ret = query_fn();
        if ret == 0 {
            Ok(())
        } else {
            Err(error_kind.with_errno())
        }
    }

    /// Queries device attributes.
    ///
    /// # Errors
    ///
    /// Returns an error if the query operation fails.
    pub fn query_device(&self) -> Result<crate::ibv_device_attr> {
        let mut device_attr = crate::ibv_device_attr::default();
        unsafe {
            self.query_with_errno(
                || crate::ibv_query_device(self.0, &mut device_attr),
                ErrorKind::IBQueryDeviceFail,
            )?
        };
        Ok(device_attr)
    }

    /// Queries port attributes.
    ///
    /// # Errors
    ///
    /// Returns an error if the query operation fails.
    pub fn query_port(&self, port_num: u8) -> Result<crate::ibv_port_attr> {
        let mut port_attr = std::mem::MaybeUninit::<crate::ibv_port_attr>::uninit();
        unsafe {
            self.query_with_errno(
                || crate::ibv_query_port(self.0, port_num, port_attr.as_mut_ptr() as _),
                ErrorKind::IBQueryPortFail,
            )?;
            Ok(port_attr.assume_init())
        }
    }

    /// Queries a GID for the specified port and index.
    ///
    /// # Errors
    ///
    /// Returns an error if the query operation fails.
    pub fn query_gid(&self, port_num: u8, gid_index: u16) -> Result<crate::ibv_gid> {
        let mut gid = crate::ibv_gid::default();
        let ret = unsafe { crate::ibv_query_gid(self.0, port_num as _, gid_index as _, &mut gid) };
        if ret == 0 && !gid.is_null() {
            Ok(gid)
        } else {
            Err(ErrorKind::IBQueryGidFail.with_errno())
        }
    }

    /// Queries the GID type from sysfs.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from sysfs fails.
    pub fn query_gid_type(
        &self,
        port_num: u8,
        gid_index: u16,
        ibdev_path: &Path,
        port_attr: &crate::ibv_port_attr,
    ) -> Result<GidType> {
        let path = ibdev_path.join(format!("ports/{port_num}/gid_attrs/types/{gid_index}"));
        match std::fs::read_to_string(path) {
            Ok(content) => {
                if content == GID_TYPE_IB_ROCE_V1 {
                    match port_attr.link_layer {
                        LinkLayer::InfiniBand => Ok(GidType::IB),
                        LinkLayer::Ethernet => Ok(GidType::RoCEv1),
                        _ => Ok(GidType::Other(content.trim().to_string())),
                    }
                } else if content == GID_TYPE_ROCE_V2 {
                    Ok(GidType::RoCEv2)
                } else {
                    Ok(GidType::Other(content.trim().to_string()))
                }
            }
            Err(err) => Err(Error::new(ErrorKind::IBQueryGidTypeFail, err.to_string())),
        }
    }
}

unsafe impl Send for RawContext {}
unsafe impl Sync for RawContext {}

/// Raw protection domain wrapper with automatic cleanup.
///
/// A protection domain (PD) is a security mechanism that isolates
/// memory regions and queue pairs from each other.
///
/// Wraps an `ibv_pd` pointer and ensures proper cleanup via
/// `ibv_dealloc_pd` when dropped.
pub struct RawProtectionDomain(pub *mut crate::ibv_pd);

impl Drop for RawProtectionDomain {
    fn drop(&mut self) {
        let _ = unsafe { crate::ibv_dealloc_pd(self.0) };
    }
}

unsafe impl Send for RawProtectionDomain {}
unsafe impl Sync for RawProtectionDomain {}
