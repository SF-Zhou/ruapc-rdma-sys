//! Work completion (ibv_wc) helper methods
//!
//! Provides type-safe helper methods for checking work completion status
//! and extracting completion data.

use crate::WCType;

pub use crate::{ibv_wc, ibv_wc_flags, ibv_wc_status};

impl ibv_wc {
    /// Checks if this work completion is for a receive operation
    pub fn is_recv(&self) -> bool {
        self.wr_id.get_type() == WCType::Recv
    }

    /// Checks if this work completion is for a send data operation
    pub fn is_send_data(&self) -> bool {
        self.wr_id.get_type() == WCType::SendData
    }

    /// Checks if this work completion is for a send with immediate data operation
    pub fn is_send_imm(&self) -> bool {
        self.wr_id.get_type() == WCType::SendImm
    }

    /// Checks if the work completed successfully
    ///
    /// Returns true if the completion status is IBV_WC_SUCCESS
    pub fn succ(&self) -> bool {
        self.status == ibv_wc_status::IBV_WC_SUCCESS
    }

    /// Extracts immediate data from this work completion
    ///
    /// Returns Some with the immediate data value if the IBV_WC_WITH_IMM
    /// flag is set, otherwise returns None
    pub fn imm(&self) -> Option<u32> {
        if ibv_wc_flags(self.wc_flags) & ibv_wc_flags::IBV_WC_WITH_IMM != ibv_wc_flags(0) {
            Some(u32::from_be(unsafe { self.__bindgen_anon_1.imm_data }))
        } else {
            None
        }
    }
}

impl std::fmt::Debug for ibv_wc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ibv_wc")
            .field("wr_id", &self.wr_id)
            .field("status", &self.status)
            .field("opcode", &self.opcode)
            .field("vendor_err", &self.vendor_err)
            .field("byte_len", &self.byte_len)
            .field("imm_data", &self.imm())
            .finish()
    }
}
