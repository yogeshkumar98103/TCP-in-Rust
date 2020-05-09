use std::fmt::{self, Display, Formatter, Result};
use std::cmp::PartialEq;

///   =================================================================
///                          ETHERNET HEADER
///   =================================================================
///
///   Destination MAC Address   : 6 bytes
///   Source MAC Address        : 6 bytes
///   IPv Type                  : 2 bytes

/// ============= MACAddress =============
pub struct MACAddress{
    pub bytes: [u8; 6]
}

impl MACAddress {
    pub fn toString(&self, hex: bool) -> String {
        if hex {
            format!("{:02X}.{:02X}.{:02X}.{:02X}.{:02X}.{:02X}", self.bytes[0], self.bytes[1], self.bytes[2],
                    self.bytes[3], self.bytes[4], self.bytes[5])
        }
        else{
            format!("{}.{}.{}.{}.{}.{}", self.bytes[0], self.bytes[1], self.bytes[2],
                    self.bytes[3], self.bytes[4], self.bytes[5])
        }
    }
}

impl Display for MACAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.toString(true))
    }
}

/// =============== IPvType ==============
#[derive(PartialEq)]
pub enum EtherType{
    IPv4,
    IPv6,
    Other(u16)
}

impl Display for EtherType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match &self {
            Self::IPv4 => write!(f, "IPv4"),
            Self::IPv6 => write!(f, "IPv6"),
            Self::Other(ipVersion) => write!(f, "{}", ipVersion),
        }
    }
}

/// ========== EthernetHeader ============
pub struct EthernetHeader{
    pub destination: MACAddress,
    pub source: MACAddress,
    pub ipVersion: EtherType
}

impl Display for EthernetHeader{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Ethernet Header :: IP version: {} || Destination MAC: {} || Source MAC: {}",
               &self.ipVersion,
               &self.destination,
               &self.source
        )
    }
}

impl EthernetHeader {
    pub fn from(buffer: &[u8]) -> Self {
        let header: EthernetHeader;

        // Parser Destination Address
        let mut bytes = [0u8; 6];
        bytes.copy_from_slice(&buffer[0..6]);
        let destination = MACAddress{ bytes };

        // Parser Source Address
        let mut bytes = [0u8; 6];
        bytes.copy_from_slice(&buffer[6..12]);
        let source = MACAddress{ bytes };

        // Parser IP version
        let ipVersion = u16::from_be_bytes([buffer[12], buffer[13]]);
        let ipVersion = match ipVersion {
            0x0800 => EtherType::IPv4,
            0x86dd => EtherType::IPv6,
            _      => EtherType::Other(ipVersion)
        };

        EthernetHeader {
            destination,
            source,
            ipVersion
        }
    }
    pub fn size(&self) -> usize { 14 }
}