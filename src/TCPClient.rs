use crate::Parser::*;

impl Interface {
    /// Create new instance of Interface
    /// TODO: Fix this selfIP and otherIP for linux
    pub fn new(iface: &str, selfIP: IPAddress, otherIP: IPAddress) -> io::Result<Self> {
        let mut iph = IPHeader::default();
        iph.sourceIP = selfIP;
        let nic = VNC::new(iface, &(selfIP.toString())[..], &(otherIP.toString())[..])?;
        Ok(Self {
            iph, nic,
            tcph: TCPHeader::default(),
            send: SendSequenceSpace::new(0),    // TODO: Generate iss properly
            recv: RecvSequenceSpace::default(),
            state: TCPState::Closed,
            buf: [0; 1500]
        })
    }

    /// Open TCP in passive mode
    /// Start listening on a given port number
    /// TODO: Return a detailed error rather than just bool
    pub fn openPassive(&mut self, port: u16) -> bool{
        return true;
        // self.tcph.syn = true;
        // self.tcph.ack = true;
        // self.tcph.acknowledgementNumber = self.recv.nxt;
        // self.tcph.sequenceNumber = self.send.iss;
        // self.state = TCPState::SynRcvd;
        // self.write(&[]);
        //
        // // Reset control bits
        // self.tcph.syn = false;
        // self.tcph.ack = false;
    }

    /// Open TCP in active mode
    /// Try to establish connection
    /// TODO: Return a detailed error rather than just bool
    fn openActive(&mut self, dstIP: IPAddress, dstPort: u16, srcPort: u16) -> bool {
        self.iph.destinationIP = dstIP;
        self.tcph.destinationPort = dstPort;
        // TODO: Allocate Source Port automatically
        self.tcph.sourcePort = srcPort;

        // Send syn packet
        self.sendSyn();
        while(self.)
        // Retransmit till we recieve [syn, ack]

        return true;
    }

    fn sendSyn(&mut self){
        self.tcph.syn = true;
        self.tcph.sequenceNumber = self.send.iss;
        self.tcph.acknowledgementNumber = 0;
        self.tcph.window = self.send.wnd;
        self.write(&[]);

        // Reset Control Bits
        tcph.syn = false;

        // Change state
        self.state = TCPState::SynSnt;
    }

    /// Given the payload data, this function performs following operations:-
    /// 1. Calculate Checksum of `self.tcph` and `self.iph` headers
    /// 2. Serialize headers to `self.buf`
    /// 3. Transmit `self.buff` using `self.nic`
    fn write(&mut self, data: &[u8]) {
        self.tcph.calcChecksum(self.iph.sourceIP, self.iph.destinationIP, data);
        self.iph.serialize(&mut buff[..]);
        self.tcph.serialize(&mut buff[self.iph.size()..]);
        let hsize = self.iph.size() + self.tcph.size();
        let size = hsize + data.len();
        buff[hsize..size].copy_from_slice(data);
        nic.send(&buff[..size]);
    }

    fn send(&mut self, data: &[u8], push: bool) {

    }

    fn disconnect(&mut self){

    }
}


const DEFAULT_WINDOW_SIZE: u16 = 10;

///
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
    CloseWait,
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
    wnd: u16,

    /// send urgent pointer
    up: bool,

    /// segment sequence number used for last window update
    wl1: u32,

    /// segment acknowledgment number used for last window update
    wl2: u32,

    /// initial send sequence number
    iss: u32
}

impl SendSequenceSpace{
    fn new(iss: u32) -> Self {
        Self{
            una: iss,
            nxt: iss + 1,
            wnd: DEFAULT_WINDOW_SIZE,
            up: false,
            wl1: 0,
            wl2: 0,
            iss
        }
    }
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

impl RecvSequenceSpace{
    fn init(&mut self, irs: u32){
        self.irs = irs;
        self.wnd = DEFAULT_WINDOW_SIZE;
        self.up = false;
        self.nxt = irs + 1;
    }
}

pub struct Interface {
    iph: IPHeader,
    tcph: TCPHeader,
    state: TCPState,
    send: SendSequenceSpace,
    recv: RecvSequenceSpace,
    nic: VNC,
    buf: [u8; 1500]
}

impl Interface {
    /// Create new instance of Interface
    /// TODO: Fix this selfIP and otherIP for linux
    pub fn new(iface: &str, selfIP: IPAddress, otherIP: IPAddress) -> Self {
        let iph = IPHeader::default();
        iph.sourceIP = selfIP;
        Self {
            iph,
            tcph: TCPHeader::default(),
            send: SendSequenceSpace::new(0),    // TODO: Generate iss properly
            recv: RecvSequenceSpace::default(),
            state: TCPState::Closed,
            nic: VNC::new(iface, &(selfIP.toString())[..], &(otherIP.toString())[..]),
            buf: [0; 1500]
        }
    }

    /// Open TCP in passive mode
    /// Start listening on a given port number
    pub fn openPassive(&mut self, port: u16){
        // self.tcph.syn = true;
        // self.tcph.ack = true;
        // self.tcph.acknowledgementNumber = self.recv.nxt;
        // self.tcph.sequenceNumber = self.send.iss;
        // self.state = TCPState::SynRcvd;
        // self.write(&[]);
        //
        // // Reset control bits
        // self.tcph.syn = false;
        // self.tcph.ack = false;
    }

    /// Open TCP in active mode
    /// Try to establish connection
    /// TODO: Return a detailed error rather than just bool
    fn openActive(&mut self, dstIP: IPAddress, dstPort: u16, srcPort: u16) -> bool {
        self.iph.destinationIP = dstIP;
        self.tcph.destinationPort = dstPort;
        // TODO: Allocate Source Port automatically
        self.tcph.sourcePort = srcPort;

        // Send syn packet
        self.sendSyn();
    }

    fn sendSyn(&mut self){
        self.tcph.syn = true;
        self.tcph.sequence

        tcph.syn = false;
    }

    /// Given the payload data, this function performs following operations:-
    /// 1. Calculate Checksum of `self.tcph` and `self.iph` headers
    /// 2. Serialize headers to `self.buf`
    /// 3. Transmit `self.buff` using `self.nic`
    fn write(&mut self, data: &[u8]) {
        self.tcph.calcChecksum(self.iph.sourceIP, self.iph.destinationIP, data);
        self.iph.serialize(&mut buff[..]);
        self.tcph.serialize(&mut buff[self.iph.size()..]);
        let hsize = self.iph.size() + self.tcph.size();
        let size = hsize + data.len();
        buff[hsize..size].copy_from_slice(data);
        nic.send(&buff[..size]);
    }

    fn send(&mut self, data: &[u8], push: bool) {

    }

    fn disconnect(&mut self){

    }
}