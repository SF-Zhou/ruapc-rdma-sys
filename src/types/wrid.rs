//! Work completion ID with type information
//!
//! The WRID (Work Request ID) encodes both a type and an ID in a single
//! 64-bit value for efficient work completion matching.

/// Work completion ID with type information
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WRID(pub u64);

/// Type of work completion
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WCType {
    /// Receive work completion
    Recv = 0,
    /// Send data work completion
    SendData = 1,
    /// Send with immediate data work completion
    SendImm = 2,
}

impl WRID {
    /// Number of bits used for type information
    pub const TYPE_BITS: u32 = 62;
    /// Mask to extract type bits from WRID
    pub const TYPE_MASK: u64 = ((1 << (u64::BITS - Self::TYPE_BITS)) - 1) << Self::TYPE_BITS;

    /// Creates a new WRID with the specified type and ID
    pub fn new(wc_type: WCType, id: u64) -> Self {
        assert!(id & Self::TYPE_MASK == 0, "ID too large");
        Self(((wc_type as u64) << Self::TYPE_BITS) | id)
    }

    /// Creates a WRID for a receive operation
    pub fn recv(id: u64) -> Self {
        Self::new(WCType::Recv, id)
    }

    /// Creates a WRID for a send data operation
    pub fn send_data(id: u64) -> Self {
        Self::new(WCType::SendData, id)
    }

    /// Creates a WRID for a send with immediate data operation
    pub fn send_imm(id: u64) -> Self {
        Self::new(WCType::SendImm, id)
    }

    /// Returns the type of work completion
    pub fn get_type(&self) -> WCType {
        match (self.0 & Self::TYPE_MASK) >> Self::TYPE_BITS {
            0 => WCType::Recv,
            1 => WCType::SendData,
            2 => WCType::SendImm,
            _ => unreachable!(),
        }
    }

    /// Returns the ID portion of the WRID
    pub fn get_id(&self) -> u64 {
        self.0 & !Self::TYPE_MASK
    }
}

impl std::fmt::Debug for WRID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.get_type() {
            WCType::Recv => write!(f, "Recv({})", self.get_id()),
            WCType::SendData => write!(f, "SendData({})", self.get_id()),
            WCType::SendImm => write!(f, "SendImm({})", self.get_id()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrid_recv() {
        let id = 12345u64;
        let wrid = WRID::recv(id);
        assert_eq!(wrid.get_type(), WCType::Recv);
        assert_eq!(wrid.get_id(), id);
    }

    #[test]
    fn test_wrid_send_data() {
        let id = 67890u64;
        let wrid = WRID::send_data(id);
        assert_eq!(wrid.get_type(), WCType::SendData);
        assert_eq!(wrid.get_id(), id);
    }

    #[test]
    fn test_wrid_send_imm() {
        let id = 54321u64;
        let wrid = WRID::send_imm(id);
        assert_eq!(wrid.get_type(), WCType::SendImm);
        assert_eq!(wrid.get_id(), id);
    }

    #[test]
    fn test_wrid_new() {
        let wrid = WRID::new(WCType::Recv, 1000);
        assert_eq!(wrid.get_type(), WCType::Recv);
        assert_eq!(wrid.get_id(), 1000);

        let wrid = WRID::new(WCType::SendData, 2000);
        assert_eq!(wrid.get_type(), WCType::SendData);
        assert_eq!(wrid.get_id(), 2000);

        let wrid = WRID::new(WCType::SendImm, 3000);
        assert_eq!(wrid.get_type(), WCType::SendImm);
        assert_eq!(wrid.get_id(), 3000);
    }

    #[test]
    fn test_wrid_id_overflow() {
        let large_id = 1u64 << 62;
        let result = std::panic::catch_unwind(|| {
            WRID::new(WCType::Recv, large_id);
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_wrid_debug() {
        let wrid = WRID::recv(123);
        let debug_str = format!("{:?}", wrid);
        assert_eq!(debug_str, "Recv(123)");

        let wrid = WRID::send_data(456);
        let debug_str = format!("{:?}", wrid);
        assert_eq!(debug_str, "SendData(456)");

        let wrid = WRID::send_imm(789);
        let debug_str = format!("{:?}", wrid);
        assert_eq!(debug_str, "SendImm(789)");
    }

    #[test]
    fn test_wrid_type_mask() {
        let mask = WRID::TYPE_MASK;
        let expected_mask: u64 = 0xC000000000000000;
        assert_eq!(mask, expected_mask);
    }

    #[test]
    fn test_wrid_encoding() {
        let wrid = WRID::recv(0x1234);
        let value = wrid.0;
        assert_eq!(value & WRID::TYPE_MASK, 0);
        assert_eq!(value & !WRID::TYPE_MASK, 0x1234);

        let wrid = WRID::send_data(0x5678);
        let value = wrid.0;
        assert_eq!((value & WRID::TYPE_MASK) >> WRID::TYPE_BITS, 1);
        assert_eq!(value & !WRID::TYPE_MASK, 0x5678);

        let wrid = WRID::send_imm(0x9ABC);
        let value = wrid.0;
        assert_eq!((value & WRID::TYPE_MASK) >> WRID::TYPE_BITS, 2);
        assert_eq!(value & !WRID::TYPE_MASK, 0x9ABC);
    }
}
