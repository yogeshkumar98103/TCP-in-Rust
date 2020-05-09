use crate::Parser;
use crate::Parser::*;
use crate::VirtualNetwork::VNC;
use crate::TCPState::TCPState::{SynRcvd, Listen, Closed};

const DEFAULT_WINDOW_SIZE: u16 = 10;

///                                            Transmission Control Protocol
///                                                 Functional Specification
///
///
///
///
///                               +---------+ ---------\      active OPEN
///                               |  CLOSED |            \    -----------
///                               +---------+<---------\   \   create TCB
///                                 |     ^              \   \  snd SYN
///                    passive OPEN |     |   CLOSE        \   \
///                    ------------ |     | ----------       \   \
///                     create TCB  |     | delete TCB         \   \
///                                 V     |                      \   \
///                               +---------+            CLOSE    |    \
///                               |  LISTEN |          ---------- |     |
///                               +---------+          delete TCB |     |
///                    rcv SYN      |     |     SEND              |     |
///                   -----------   |     |    -------            |     V
///  +---------+      snd SYN,ACK  /       \   snd SYN          +---------+
///  |         |<-----------------           ------------------>|         |
///  |   SYN   |                    rcv SYN                     |   SYN   |
///  |   RCVD  |<-----------------------------------------------|   SENT  |
///  |         |                    snd ACK                     |         |
///  |         |------------------           -------------------|         |
///  +---------+   rcv ACK of SYN  \       /  rcv SYN,ACK       +---------+
///    |           --------------   |     |   -----------
///    |                  x         |     |     snd ACK
///    |                            V     V
///    |  CLOSE                   +---------+
///    | -------                  |  ESTAB  |
///    | snd FIN                  +---------+
///    |                   CLOSE    |     |    rcv FIN
///    V                  -------   |     |    -------
///  +---------+          snd FIN  /       \   snd ACK          +---------+
///  |  FIN    |<-----------------           ------------------>|  CLOSE  |
///  | WAIT-1  |------------------                              |   WAIT  |
///  +---------+          rcv FIN  \                            +---------+
///    | rcv ACK of FIN   -------   |                            CLOSE  |
///    | --------------   snd ACK   |                           ------- |
///    V        x                   V                           snd FIN V
///  +---------+                  +---------+                   +---------+
///  |FINWAIT-2|                  | CLOSING |                   | LAST-ACK|
///  +---------+                  +---------+                   +---------+
///    |                rcv ACK of FIN |                 rcv ACK of FIN |
///    |  rcv FIN       -------------- |    Timeout=2MSL -------------- |
///    |  -------              x       V    ------------        x       V
///     \ snd ACK                 +---------+delete TCB         +---------+
///      ------------------------>|TIME WAIT|------------------>| CLOSED  |
///                               +---------+                   +---------+
///
///                       TCP Connection State Diagram

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TCPState{
    Closed,
    Listen,
    SynRcvd,
    SynSnt,
    Estab,
    FinWait1,
    FinWait2,
    // CloseWait,
    Closing,
    LastAck,
    TimeWait
}

///  ===> Send Sequence Space
///
///            1         2          3          4
///           ----------|----------|----------|----------
///                  SND.UNA    SND.NXT    SND.UNA
///                                       +SND.WND
///
/// 1 - old sequence numbers which have been acknowledged
/// 2 - sequence numbers of unacknowledged data
/// 3 - sequence numbers allowed for new data transmission
/// 4 - future sequence numbers which are not yet allowed
///

#[derive(Debug, Copy, Clone, Default)]
struct SendSequenceSpace{
    /// send unacknowledged
    una: u32,

    /// send next
    nxt: u32,

    /// send window
    wnd: u32,

    /// send urgent pointer
    up: bool,

    /// segment sequence number used for last window update
    wl1: u32,

    /// segment acknowledgment number used for last window update
    wl2: u32,

    /// initial send sequence number
    iss: u32
}


///   ===> Receive Sequence Space
///
///                  1          2          3
///              ----------|----------|----------
///                     RCV.NXT    RCV.NXT
///                               +RCV.WND
///
///   1 - old sequence numbers which have been acknowledged
///   2 - sequence numbers allowed for new reception
///   3 - future sequence numbers which are not yet allowed
///

#[derive(Debug, Copy, Clone, Default)]
struct RecvSequenceSpace{
    /// receive next
    nxt: u32,

    /// receive window
    wnd: u16,

    /// receive urgent pointer
    up: bool,

    /// initial receive sequence number
    irs: u32,
}


pub struct Connection{
    state: TCPState,
    send: SendSequenceSpace,
    recv: RecvSequenceSpace,
    tcph: TCPHeader,
    iph: IPHeader,
}

impl Connection{
    pub fn new(iph: &Parser::IPHeader, tcph: &Parser::TCPHeader) -> Connection {
        let iss = 0;
        Connection{
            state: Listen,
            send: SendSequenceSpace{
                iss: iss,
                una: iss,
                nxt: iss + 1,
                wnd: 10,
                up: false,
                wl1: 0,
                wl2: 0
            },
            recv: RecvSequenceSpace{
                irs: tcph.sequenceNumber,
                nxt: tcph.sequenceNumber + 1,
                wnd: tcph.window,
                up : false
            },
            tcph: TCPHeader::new(tcph.destinationPort, tcph.sourcePort, iss, DEFAULT_WINDOW_SIZE),
            iph: IPHeader::new(iph.destinationIP, iph.sourceIP, IPProtocol::Tcp, 64, 20)
        }
    }

    fn handleReset(&mut self, buff: &mut [u8], tcph: Parser::TCPHeader, nic: &mut VNC) {
        self.tcph.rst = true;
        self.write(nic, buff, &[]);
    }

    fn handleListen(&mut self, buff: &mut [u8], nic: &mut VNC) {
        self.tcph.syn = true;
        self.tcph.ack = true;
        self.tcph.acknowledgementNumber = self.recv.nxt;
        self.tcph.sequenceNumber = self.send.iss;
        self.state = TCPState::SynRcvd;
        self.write(nic, buff, &[]);

        // Reset control bits
        self.tcph.syn = false;
        self.tcph.ack = false;
    }

    fn handleSynRcvd(&mut self, buff: &mut [u8], tcph: Parser::TCPHeader, nic: &mut VNC) {
        if tcph.ack {
            self.state = TCPState::Estab;
        }
    }

    fn handleLastAck(&mut self, buff: &mut [u8], tcph: Parser::TCPHeader, nic: &mut VNC) {
        if tcph.ack {
            self.state = TCPState::Closed;
        }
    }

    fn handleEstab(&mut self, buff: &mut [u8], tcph: Parser::TCPHeader, dataStart: usize, nic: &mut VNC) {
        // Temporarily print data as char
        let data = String::from_utf8_lossy(&buff[dataStart..]);
        print!("{}", data);
        let dataSize= buff.len() - dataStart;

        if tcph.fin {
            // Request for Closing Connection
            self.recv.nxt = Self::addWrapping(self.recv.nxt, 1);
            // self.state = TCPState::CloseWait;
        }
        else {
            self.recv.nxt = Self::addWrapping(self.recv.nxt, dataSize);
        }

        // Send Acknoledgement
        self.tcph.ack = true;
        self.tcph.sequenceNumber = tcph.acknowledgementNumber;
        self.tcph.acknowledgementNumber = self.recv.nxt;
        self.write(nic, buff, &[]);

        // Reset Control bits
        self.tcph.ack = false;

        if tcph.fin {
            // TCPState::CloseWait;
            self.tcph.fin = true;
            self.tcph.ack = true;
            self.send.nxt = Self::addWrapping(self.send.nxt, 1);
            self.write(nic, buff, &[]);
            self.state = TCPState::LastAck;
        }
    }

    fn addWrapping(num: u32, add: usize) -> u32{
        ((num as usize + add) % (1usize << 32)) as u32
    }

    fn verifyPacket(&self, tcph: &TCPHeader, segLength: u32) -> bool{
        // ===> Check 1: valid acknowledgement
        //          send.una < ack <= send.nxt
        let ack = tcph.acknowledgementNumber;
        if !(ack == self.send.nxt || Connection::checkBetween(self.send.una, ack, self.send.nxt)) {return false;}

        // ==> Check 2: valid sequence number
        //          recv.nxt <= seq < recv.nxt + recv.wnd
        //          recv.nxt <= seq + len - 1 < recv.nxt + recv.wnd
        let seq = tcph.sequenceNumber;
        let c1 = self.recv.nxt == seq || Connection::checkBetween(self.recv.nxt, seq, self.recv.nxt + self.recv.wnd as u32);
        let c2 = self.recv.nxt == seq + segLength - 1 || Connection::checkBetween(self.recv.nxt, seq + segLength - 1, self.recv.nxt + self.recv.wnd as u32);
        if segLength == 0 {
            if self.recv.wnd == 0 { return seq == self.recv.nxt; }
            else{ return c1; }
        }
        else{
            return self.recv.wnd != 0 && c1 && c2;
        }
    }

    /// start < x < end
    fn checkBetween(start: u32, x: u32, end: u32) -> bool{
        if start < end {
             start < x && x < end
        }
        else{
             start < x || x < end
        }
    }

    pub fn onPacket(&mut self, tcph: Parser::TCPHeader, buff: &mut [u8], dataStart: usize, nic: &mut VNC) -> bool{
        // println!("Recieved {} bytes.", buff.len() - dataStart);
        // println!("{:02X?}\n", &buff[..]);
        //
        // println!("State: {:?}", self.state);
        if !(self.state == TCPState::Listen || self.verifyPacket(&tcph, (buff.len() - dataStart) as u32)) {
            self.handleReset(buff, tcph, nic);
            return false;
        }

        match self.state {
            TCPState::Listen    => self.handleListen(buff, nic),
            TCPState::SynRcvd   => self.handleSynRcvd(buff, tcph, nic),
            TCPState::SynSnt    => {},
            TCPState::Estab     => self.handleEstab(buff, tcph, dataStart, nic),
            TCPState::FinWait1  => {},
            TCPState::FinWait2  => {},
            TCPState::Closing   => {},
            TCPState::LastAck   => self.handleLastAck(buff, tcph, nic),
            TCPState::TimeWait  => {},
            _ => {}
        };

        return self.state == TCPState::Closed
    }

    fn write(&mut self, nic: &mut VNC, buff: &mut [u8], data: &[u8]) {
        self.tcph.calcChecksum(self.iph.sourceIP, self.iph.destinationIP, data);
        self.iph.serialize(&mut buff[..]);
        self.tcph.serialize(&mut buff[self.iph.size()..]);
        let hsize = self.iph.size() + self.tcph.size();
        let size = hsize + data.len();
        buff[hsize..size].copy_from_slice(data);
        // println!("Send {} bytes.\n{:02X?}\n", data.len(), &buff[..]);
        nic.send(&buff[..size]);
    }
}