extern crate byteorder;
use crate::Parser::IPAddress;
use self::byteorder::{ByteOrder, BigEndian, ReadBytesExt, WriteBytesExt};

///   =================================================================
///                             TCP HEADER
///   =================================================================
///
///    0               1               2               3
///    0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |          Source Port          |       Destination Port        |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                        Sequence Number                        |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                    Acknowledgment Number                      |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |Header |           |U|A|P|R|S|F|                               |
///   | Length| Reserved  |R|C|S|S|Y|I|            Window             |
///   |       |           |G|K|H|T|N|N|                               |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |           Checksum            |         Urgent Pointer        |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                    Options                    |    Padding    |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                             data                              |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

pub struct TCPHeader{
    pub sourcePort              :  u16,
    pub destinationPort         :  u16,
    pub sequenceNumber          :  u32,
    pub acknowledgementNumber   :  u32,
    pub headerLength            :  u8,
    pub reserved                :  u8,
    pub window                  :  u16,
    pub checksum                :  u16,
    pub urgentPointer           :  u16,
    pub options                 :  [u8; 40],

    /// Control Bits
    pub fin: bool,
    pub syn: bool,
    pub rst: bool,
    pub psh: bool,
    pub ack: bool,
    pub urg: bool,
    pub ece: bool,
    pub cwr: bool,
}

impl TCPHeader {
    pub fn from(buffer: &[u8]) -> Self{
        let sourcePort = u16::from_be_bytes([buffer[0], buffer[1]]);
        let destinationPort = u16::from_be_bytes([buffer[2], buffer[3]]);
        let sequenceNumber = u32::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
        let acknowledgementNumber = u32::from_be_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]);

        let temp = u16::from_be_bytes([buffer[12], buffer[13]]);
        let headerLength  = ((temp & 0b1111000000000000u16) >> 12) as u8;
        let reserved    = ((temp & 0b0000111111000000u16) >> 6) as u8;
        let controlBits = (temp & 0b0000000000111111) as u8;

        let window = u16::from_be_bytes([buffer[14], buffer[15]]);
        let checksum = u16::from_be_bytes([buffer[16], buffer[17]]);
        let urgentPointer = u16::from_be_bytes([buffer[18], buffer[19]]);

        // TODO: Parser option correctly
        // let mut optionsBuff = [0u8; 8];
        // optionsBuff.clone_from_slice(&buffer[20..headerLength]);

        TCPHeader {
            sourcePort, destinationPort, sequenceNumber, acknowledgementNumber,
            headerLength, reserved, window, checksum, urgentPointer,
            options: [0; 40],
            fin: 0 != controlBits & 0b00000001,
            syn: 0 != controlBits & 0b00000010,
            rst: 0 != controlBits & 0b00000100,
            psh: 0 != controlBits & 0b00001000,
            ack: 0 != controlBits & 0b00010000,
            urg: 0 != controlBits & 0b00100000,
            ece: 0 != controlBits & 0b01000000,
            cwr: 0 != controlBits & 0b10000000,
        }
    }

    pub fn new(sourcePort: u16, destinationPort: u16, sequenceNumber: u32, window: u16) -> Self {
        TCPHeader {
            sourcePort,
            destinationPort,
            sequenceNumber,
            acknowledgementNumber: 0,
            headerLength: 5,
            reserved: 0,
            window: window,
            checksum: 0,
            urgentPointer: 0,
            options: [0; 40],
            fin: false,
            syn: false,
            rst: false,
            psh: false,
            ack: false,
            urg: false,
            ece: false,
            cwr: false
        }
    }

    pub fn resetControlBits(&mut self) {
        self.fin = false;
        self.syn = false;
        self.rst = false;
        self.psh = false;
        self.ack = false;
        self.urg = false;
        self.ece = false;
        self.cwr = false;
    }

    pub fn serialize(&mut self, buffer: &mut [u8]){
        BigEndian::write_u16(buffer, self.sourcePort);
        BigEndian::write_u16(&mut buffer[2..], self.destinationPort);
        BigEndian::write_u32(&mut buffer[4..], self.sequenceNumber);
        BigEndian::write_u32(&mut buffer[8..], self.acknowledgementNumber);
        let val = ((self.headerLength as u16) << 12) | ((self.reserved as u16) << 8) | self.getControlBits() as u16;
        BigEndian::write_u16(&mut buffer[12..], val);
        BigEndian::write_u16(&mut buffer[14..], self.window);
        BigEndian::write_u16(&mut buffer[16..], self.checksum);
        BigEndian::write_u16(&mut buffer[18..], self.urgentPointer);
    }

    pub fn getControlBits(&self) -> u8{
        let mut controlBits = 0u8;
        if self.fin { controlBits |= 0b00000001u8; }
        if self.syn { controlBits |= 0b00000010u8; }
        if self.rst { controlBits |= 0b00000100u8; }
        if self.psh { controlBits |= 0b00001000u8; }
        if self.ack { controlBits |= 0b00010000u8; }
        if self.urg { controlBits |= 0b00100000u8; }
        if self.ece { controlBits |= 0b01000000u8; }
        if self.cwr { controlBits |= 0b10000000u8; }
        controlBits
    }

    pub fn calcChecksum(&mut self, sourceIP: IPAddress, destinationIP: IPAddress, payload: &[u8]) {
        // Check Length Constraint
        let tcpLength = (self.headerLength * 4) as usize + payload.len();
        // if (std::u16::MAX as usize) < tcpLength { return false; }

        // PseudoHeader => sourceIP, destinationIP, protocol, tcpLength
        let pseudoHeaderSum =
            u64::from(u16::from_be_bytes([sourceIP.bytes[0], sourceIP.bytes[1]])) +
                u64::from(u16::from_be_bytes([sourceIP.bytes[2], sourceIP.bytes[3]])) +
                u64::from(u16::from_be_bytes([destinationIP.bytes[0], destinationIP.bytes[1]])) +
                u64::from(u16::from_be_bytes([destinationIP.bytes[2], destinationIP.bytes[3]])) +
                6u64 + tcpLength as u64;

        self.checksum = self.calcChecksumPostIP(pseudoHeaderSum, payload);
        // return true;
    }

    fn calcChecksumPostIP(&self, pseudoHeaderSum: u64, payload: &[u8]) -> u16 {
        fn u32Checksum(value: u32) -> u64 {
            let mut buffer: [u8;4] = [0;4];
            BigEndian::write_u32(&mut buffer, value);
            u64::from(BigEndian::read_u16(&buffer[..2])) + u64::from(BigEndian::read_u16(&buffer[2..]))
        }

        let controlBits = self.getControlBits();

        let mut sum =
            pseudoHeaderSum +
            u64::from(self.sourcePort) +
            u64::from(self.destinationPort) +
            u32Checksum(self.sequenceNumber) +
            u32Checksum(self.acknowledgementNumber) +
            u64::from(BigEndian::read_u16(&[self.headerLength << 4, controlBits])) +
            u64::from(self.window) +
            u64::from(self.urgentPointer);

        //add the options
        // let options_len = self.options_len();
        // for i in RangeStep::new(0, options_len, 2) {
        //     sum += u64::from( BigEndian::read_u16(&self.options_buffer[i..i + 2]) );
        // }

        // Payload
        let n = (payload.len() / 2) * 2;
        for i in (0..n).step_by(2) {
            sum += u64::from( BigEndian::read_u16(&payload[i..i + 2]));
        }

        // Pad the last byte with 0
        if payload.len() % 2 == 1 {
            sum += u64::from(BigEndian::read_u16(&[payload[payload.len() - 1], 0]));
        }

        let carryAdd = (sum & 0xffff) +
                            ((sum >> 16) & 0xffff) +
                            ((sum >> 32) & 0xffff) +
                            ((sum >> 48) & 0xffff);
        let result = ((carryAdd & 0xffff) + (carryAdd >> 16)) as u16;
        !result
    }

    pub fn verifyChecksum(buf: &[u8], sourceIP: IPAddress, destinationIP: IPAddress) -> bool {
        let headerSize: usize = ((buf[13] & 0x0F) * 4) as usize;
        let mut sum = 0;
        for i in (0..headerSize).step_by(2){
            sum += u16::from_be_bytes([buf[i], buf[i+1]]) as u32
        }

        sum += u16::from_be_bytes([sourceIP.bytes[0], sourceIP.bytes[1]]) as u32;
        sum += u16::from_be_bytes([sourceIP.bytes[2], sourceIP.bytes[3]]) as u32;
        sum += u16::from_be_bytes([destinationIP.bytes[0], destinationIP.bytes[1]]) as u32;
        sum += u16::from_be_bytes([destinationIP.bytes[2], destinationIP.bytes[3]]) as u32;

        sum = (sum >> 16) + (sum & 0x0000FFFF);
        sum = (sum >> 16) + (sum & 0x0000FFFF);
        sum + 1 == (1 << 16)
    }

    pub fn size(&self) -> usize {self.headerLength as usize * 4}
}

impl Default for TCPHeader {
    fn default() -> Self {
        TCPHeader {
            sourcePort: 0,
            destinationPort: 0,
            sequenceNumber: 0,
            acknowledgementNumber: 0,
            headerLength: 5,
            fin: false, syn: false, rst: false,
            psh: false, ack: false, urg: false, ece: false,
            cwr: false,
            window: 0,
            checksum: 0,
            urgentPointer: 0,
            options: [0;40],
            reserved: 0
        }
    }
}

/// Different kinds of options that can be present in the options part of a TCP header.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TcpOptionElement {
    Nop,
    MaximumSegmentSize(u16),
    WindowScale(u8),
    SelectiveAcknowledgementPermitted,
    SelectiveAcknowledgement((u32,u32), [Option<(u32,u32)>;3]),
    ///Timestamp & echo (first number is the sender timestamp, the second the echo timestamp)
    Timestamp(u32, u32),
}

/*
 pub fn options(&self) -> &[u8] {
        &self.options_buffer[..self.options_len()]
    }

    ///Sets the options (overwrites the current options) or returns an error when there is not enough space.
    pub fn set_options(&mut self, options: &[TcpOptionElement]) -> Result<(), TcpOptionWriteError> {

        //calculate the required size of the options
        use crate::TcpOptionElement::*;
        let required_length = options.iter().fold(0, |acc, ref x| {
            acc + match x {
                Nop => 1,
                MaximumSegmentSize(_) => 4,
                WindowScale(_) => 3,
                SelectiveAcknowledgementPermitted => 2,
                SelectiveAcknowledgement(_, rest) => {
                    rest.iter().fold(10, |acc2, ref y| {
                        match y {
                            None => acc2,
                            Some(_) => acc2 + 8
                        }
                    })
                },
                Timestamp(_, _) => 10,
            }
        });

        if self.options_buffer.len() < required_length {
            Err(TcpOptionWriteError::NotEnoughSpace(required_length))
        } else {

            //reset the options to null
            self.options_buffer = [0;40];
            self._data_offset = TCP_MINIMUM_DATA_OFFSET;

            //write the options to the buffer
            //note to whoever: I would have prefered to use std::io::Cursor as it would be less error
            //                 prone. But just in case that "no std" support is added later lets
            //                 not not rewrite it just yet with cursor.
            let mut i = 0;
            for element in options {
                match element {
                    Nop => {
                        self.options_buffer[i] = TCP_OPTION_ID_NOP;
                        i += 1;
                    },
                    MaximumSegmentSize(value) => {
                        self.options_buffer[i] = TCP_OPTION_ID_MAXIMUM_SEGMENT_SIZE;
                        i += 1;
                        self.options_buffer[i] = 4;
                        i += 1;
                        BigEndian::write_u16(&mut self.options_buffer[i..i + 2], *value);
                        i += 2;
                    },
                    WindowScale(value) => {
                        self.options_buffer[i] = TCP_OPTION_ID_WINDOW_SCALE;
                        i += 1;
                        self.options_buffer[i] = 3;
                        i += 1;
                        self.options_buffer[i] = *value;
                        i += 1;
                    },
                    SelectiveAcknowledgementPermitted => {
                        self.options_buffer[i] = TCP_OPTION_ID_SELECTIVE_ACK_PERMITTED;
                        i += 1;
                        self.options_buffer[i] = 2;
                        i += 1;
                    },
                    SelectiveAcknowledgement(first, rest) => {
                        self.options_buffer[i] = TCP_OPTION_ID_SELECTIVE_ACK;
                        i += 1;

                        //write the length
                        self.options_buffer[i] = rest.iter().fold(10, |acc, ref y| {
                            match y {
                                None => acc,
                                Some(_) => acc + 8
                            }
                        });
                        i += 1;

                        //write first
                        BigEndian::write_u32(&mut self.options_buffer[i..i + 4], first.0);
                        i += 4;
                        BigEndian::write_u32(&mut self.options_buffer[i..i + 4], first.1);
                        i += 4;

                        //write the rest
                        for v in rest {
                            match v {
                                None => {},
                                Some((a,b)) => {
                                    BigEndian::write_u32(&mut self.options_buffer[i..i + 4], *a);
                                    i += 4;
                                    BigEndian::write_u32(&mut self.options_buffer[i..i + 4], *b);
                                    i += 4;
                                }
                            }
                        }
                    },
                    Timestamp(a, b) =>  {
                        self.options_buffer[i] = TCP_OPTION_ID_TIMESTAMP;
                        i += 1;
                        self.options_buffer[i] = 10;
                        i += 1;
                        BigEndian::write_u32(&mut self.options_buffer[i..i + 4], *a);
                        i += 4;
                        BigEndian::write_u32(&mut self.options_buffer[i..i + 4], *b);
                        i += 4;
                    }
                }
            }
            //set the new data offset
            if i > 0 {
                self._data_offset = (i / 4) as u8 + TCP_MINIMUM_DATA_OFFSET;
                if i % 4 != 0 {
                    self._data_offset += 1;
                }
            }
            //done
            Ok(())
        }
    }

    ///Sets the options to the data given.
    pub fn set_options_raw(&mut self, data: &[u8]) -> Result<(), TcpOptionWriteError> {
        //check length
        if self.options_buffer.len() < data.len() {
            Err(TcpOptionWriteError::NotEnoughSpace(data.len()))
        } else {
            //reset all to zero to ensure padding
            self.options_buffer = [0;40];

            //set data & data_offset
            self.options_buffer[..data.len()].copy_from_slice(data);
            self._data_offset = (data.len() / 4) as u8 + TCP_MINIMUM_DATA_OFFSET;
            if data.len() % 4 != 0 {
                self._data_offset += 1;
            }
            Ok(())
        }
    }

    ///Returns an iterator that allows to iterate through all known TCP header options.
    pub fn options_iterator(&self) -> TcpOptionsIterator {
        TcpOptionsIterator {
            options: &self.options_buffer[..self.options_len()]
        }
    }
 */
