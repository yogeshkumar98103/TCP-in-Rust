extern crate byteorder;
use std::fmt::{self, Display, Formatter, Result};
use std::cmp::PartialEq;
use std::ops::BitAnd;
use super::IPProtocol;
use self::byteorder::{ByteOrder, BigEndian, ReadBytesExt, WriteBytesExt};
use crate::Parser::IPHeaderParser::IPVersion::IPv4;

///   =================================================================
///                             IP HEADER
///   =================================================================
///
///    0               1               2               3
///    0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |Version| Header|  Service Type |         Total Length          |
///   |       | Length|               |                               |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |          Identification       |Flags|     Fragment Offset     |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |       TTL     |    Protocol   |        Header Checksum        |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                         Source IP Addr                        |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                       Destination IP Addr                     |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                    Options                    |    Padding    |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                             data                              |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

#[derive(PartialEq)]
pub enum IPVersion{
    IPv4 = 4,
    IPv6 = 6,
}

impl Display for IPVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match &self {
            Self::IPv4 => write!(f, "IPv4"),
            Self::IPv6 => write!(f, "IPv6"),
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct IPAddress{
    pub bytes: [u8; 4]
}

impl IPAddress {
    pub fn toString(&self) -> String {
        format!("{}.{}.{}.{}", self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3])
    }
}

impl Display for IPAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.toString())
    }
}

pub enum IPServiceType {
    MinimizeDelay           = 0b00010000,
    MaximizeThroughput      = 0b00001000,
    MaximizeReliability     = 0b00000100,
    MinimizeMonetaryCost    = 0b00000010,
}

pub enum IPFlags {
    DontFragment = 0b01000000,
    MoreFragment = 0b00100000
}

pub struct IPHeader{
    pub version: IPVersion,
    pub headerLength: u8,
    pub totalLength: u16,
    pub identification: u16,
    pub fragmentOffset: u16,
    pub ttl: u8,
    pub protocol: u8,
    pub headerChecksum: u16,
    pub sourceIP: IPAddress,
    pub destinationIP: IPAddress,
    pub optionsLen: u8,
    pub options: [u8; 40],

    // IP Flags
    pub dontFragment: bool,
    pub morefragments: bool,

    // IP Service Type
    pub minimizeDelay: bool,
    pub maximizeThroughput: bool,
    pub maximizeReliability: bool,
    pub minimizeMonetaryCost: bool,
}

impl IPHeader {
    pub fn from(buffer: &[u8]) -> Option<Self> {
        // Parser IP Version
        let version = (buffer[0] & 0b11110000) >> 4;
        let version = match version {
            4 => IPVersion::IPv4,
            6 => IPVersion::IPv6,
            _ => return None
        };

        if !IPHeader::verifyChecksum(buffer) {
            return None;
        }

        // Parser Header Length
        let headerLength = buffer[0] & 0b00001111;
        let serviceType = buffer[1];
        let totalLength = u16::from_be_bytes([buffer[2], buffer[3]]);
        let identification = u16::from_be_bytes([buffer[4], buffer[5]]);
        let flags = buffer[6] & 0b11100000;
        let fragmentOffset = u16::from_be_bytes([buffer[6], buffer[7]]) & 0x1FFF;
        let ttl = buffer[8];
        let protocol = buffer[9];
        let headerChecksum = u16::from_be_bytes([buffer[10], buffer[11]]);

        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&buffer[12..16]);
        let sourceIP = IPAddress{ bytes };

        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&buffer[16..20]);
        let destinationIP = IPAddress{ bytes };

        // TODO:  Parse Options

        Some(IPHeader {
            version, headerLength, totalLength, identification,
            fragmentOffset, ttl, protocol, headerChecksum, sourceIP, destinationIP,
            optionsLen: 0, options: [0; 40],

            dontFragment: 0 != flags & IPFlags::DontFragment as u8,
            morefragments: 0 != flags & IPFlags::MoreFragment as u8,
            minimizeDelay: 0 != flags & IPServiceType::MinimizeDelay as u8,
            maximizeThroughput: 0 != flags & IPServiceType::MaximizeThroughput as u8,
            maximizeReliability: 0 != flags & IPServiceType::MaximizeReliability as u8,
            minimizeMonetaryCost: 0 != flags & IPServiceType::MinimizeMonetaryCost as u8,
        })
    }

    pub fn new(sourceIP: IPAddress, destinationIP: IPAddress, protocol: IPProtocol, ttl: u8, payloadLen: u16) -> Self {
        IPHeader {
            version: IPVersion::IPv4,
            headerLength: 5,
            totalLength: 20 + payloadLen,
            identification: 0,
            fragmentOffset: 0,
            protocol: protocol as u8,
            headerChecksum: 0,
            optionsLen: 0,
            options: [0;40],

            sourceIP,
            destinationIP,
            ttl,

            dontFragment: true,
            morefragments: false,
            minimizeDelay: false,
            maximizeThroughput: false,
            maximizeReliability: false,
            minimizeMonetaryCost: false
        }
    }

    pub fn serialize(&mut self, buffer: &mut [u8]){
        buffer[0] = (4 << 4) | (self.headerLength);
        buffer[1] = self.getServiceType();

        BigEndian::write_u16(&mut buffer[2..], self.totalLength);
        BigEndian::write_u16(&mut buffer[4..], self.identification);

        let value = ((self.getFlags() as u16) << 12) | self.fragmentOffset;
        BigEndian::write_u16(&mut buffer[6..], value);
        buffer[8] = self.ttl;
        buffer[9] = self.protocol;
        self.calcHeaderChecksum();
        BigEndian::write_u16(&mut buffer[10..], self.headerChecksum);
        buffer[12..16].copy_from_slice(&self.sourceIP.bytes);
        buffer[16..20].copy_from_slice(&self.destinationIP.bytes);
    }

    fn calcHeaderChecksum(&mut self){
        let sum: u32 = [
            BigEndian::read_u16(&[(4 << 4) | self.headerLength, self.getServiceType()]),
            self.totalLength,
            self.identification,
            ((self.getFlags() as u16) << 12) | self.fragmentOffset,
            BigEndian::read_u16(&[self.ttl, self.protocol]),
            BigEndian::read_u16(&self.sourceIP.bytes[0..2]),
            BigEndian::read_u16(&self.sourceIP.bytes[2..4]),
            BigEndian::read_u16(&self.destinationIP.bytes[0..2]),
            BigEndian::read_u16(&self.destinationIP.bytes[2..4])
        ].iter().map(|x| u32::from(*x)).sum();

        // let options = self.options();
        // for i in 0..(options.len()/2) {
        //     sum += u32::from( BigEndian::read_u16(&options[i*2..i*2 + 2]) );
        // }

        let carryAdd = (sum & 0xFFFF) + (sum >> 16);
        self.headerChecksum = !( ((carryAdd & 0xFFFF) + (carryAdd >> 16)) as u16 )
    }

    pub fn verifyChecksum(buf: &[u8]) -> bool {
        let headerSize: usize = ((buf[0] & 0x0F) * 4) as usize;
        let mut sum = 0;
        for i in (0..headerSize).step_by(2){
            sum += u16::from_be_bytes([buf[i], buf[i+1]]) as u32
        }

        sum = (sum >> 16) + (sum & 0x0000FFFF);
        sum = (sum >> 16) + (sum & 0x0000FFFF);
        sum + 1 == (1 << 16)
    }

    pub fn getFlags(&self) -> u8{
        let mut value: u8 = 0;
        if self.dontFragment {value |= IPFlags::DontFragment as u8;}
        if self.morefragments {value |= IPFlags::MoreFragment as u8;}
        value
    }

    pub fn getServiceType(&self) -> u8 {
        let mut value: u8 = 0;
        if self.maximizeReliability {value |= IPServiceType::MaximizeReliability as u8;}
        if self.maximizeThroughput {value |= IPServiceType::MaximizeThroughput as u8;}
        if self.minimizeMonetaryCost {value |= IPServiceType::MinimizeMonetaryCost as u8;}
        if self.minimizeDelay {value |= IPServiceType::MinimizeDelay as u8;}
        value
    }

    pub fn size(&self) -> usize { self.headerLength as usize * 4}
}