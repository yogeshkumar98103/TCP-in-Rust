#![allow(non_snake_case, unused_variables, unused_imports, unreachable_code, dead_code, unused_must_use, unused_doc_comments)]

mod VirtualNetwork;
mod Parser;
mod TCPConnection;

use VirtualNetwork::VNC;
use Parser::*;
use TCPConnection::*;

use std::io::{self, Read, Write};
use std::collections::{hash_map::Entry, HashMap, VecDeque};
use std::cmp::{Eq, min};
use std::hash::Hash;
use std::sync::{Arc, Mutex, Condvar};

type InterfaceHandler = Arc<Mutex<ConnectionManager>>;

/// ================================================
///                Connection Manager
/// ================================================
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct Quad{
    src: (IPAddress, u16),
    dst: (IPAddress, u16)
}

#[derive(Default)]
struct ConnectionManager{
    terminate   : Mutex<bool>,
    connections : Mutex<HashMap<u16, Arc<ActiveConnections>>>,
    pending     : Mutex<HashMap<u16, Arc<PendingConnections>>>,
}

struct PendingConnections {
    pending : Mutex<VecDeque<Quad>>,
    cond    : Condvar,
}

struct ActiveConnections {
    connection : Mutex<Connection>,
    cond       : Condvar,
}

impl Drop for ConnectionManager {
    fn drop(&mut self) {

    }
}

/// ================================================
///                    TCPListener
/// ================================================
pub struct TCPListener {
    port: u16,
    connectionManager: Arc<ConnectionManager>
}

impl TCPListener {
    /// This function blocks current thread and wait for new connection
    /// When a new connection arrives. It resumes and returns a TCPStream
    pub fn accept(&mut self) -> io::Result<TCPStream> {
        loop {
            {
                let mut connectionManager = self.connectionManager.lock().unwrap();
                if let Some(quad) = connectionManager.pending.get_mut(&self.port).expect("Port closed while listening.").pop_front() {
                    return Ok(TCPStream{
                        quad,
                        connectionManager: self.connectionManager.clone()
                    });
                }
            };

            // TODO: Block till pending connections
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "No pending connections on this port"
            ));
        }
    }
}

impl Drop for TCPListener {
    fn drop(&mut self) {
        let mut pending: Option<VecDeque<Quad>>;
        {
            let connectionManager = self.connectionManager.lock().unwrap();
            pending = connectionManager.pending.remove(&self.port);
        }

        if let Some(pending) = pending {
            for quad in pending {
                // TODO: Optional: Send fin packets to all these connections
            }
            drop(pending);
        }
    }
}

/// ================================================
///                     Interface
/// ================================================
pub struct Interface {
    thread: std::thread::JoinHandle<()>,
    connectionManager: Arc<Mutex<ConnectionManager>>
}

impl Interface {
    pub fn new(iface: &str, selfIP: IPAddress, otherIP: IPAddress) -> io::Result<Self> {
        let nic = VNC::new(iface, &(selfIP.toString())[..], &(otherIP.toString())[..])?;
        let connectionManager: Arc<Mutex<ConnectionManager>> = Arc::default();
        let thread = {
            let connectionManager = connectionManager.clone();
            std::thread::spawn(move || Interface::packetLoop(nic, connectionManager).unwrap())
        };

        Ok(Self{thread, connectionManager})
    }

    /// This Loop runs forever and looks for any incoming packets
    fn packetLoop(mut nic: VNC, connectionManager: Arc<Mutex<ConnectionManager>>) -> io::Result<()>{
        let mut buf = [0u8; 1500];
        loop {
            // TODO: Add timer
            let bytesRead = nic.recv(&mut buf)?;

            {
                let mut connectionManager = connectionManager.lock().unwrap();
                if connectionManager.terminate {
                    // Stop accepting new packets and end this thread
                    return Ok(());
                }
            }

            let ipHeader = Parser::IPHeader::from(&buf[..bytesRead]);
            if let Some(ipHeader) = ipHeader {
                if ipHeader.protocol != IPProtocol::Tcp as u8 {continue;}
                let tcpHeaderStart = ipHeader.size();
                let tcpHeader = Parser::TCPHeader::from(&buf[tcpHeaderStart..bytesRead]);
                let dataStart = tcpHeaderStart + tcpHeader.size();
                println!("{}:{} -> {}:{}", ipHeader.sourceIP, tcpHeader.sourcePort, ipHeader.destinationIP, tcpHeader.destinationPort);

                let key = Quad{
                    src: (ipHeader.sourceIP, tcpHeader.sourcePort),
                    dst: (ipHeader.destinationIP, tcpHeader.destinationPort)
                };

                let mut connectionManager = connectionManager.lock().unwrap();
                let connectionManager = &mut *connectionManager;
                match connectionManager.connections.entry(key) {
                    Entry::Vacant(entry) => {
                        // If someone is listening then only open the connection
                        if let Some(pendingConnections) = connectionManager.pending.get_mut(&tcpHeader.destinationPort) {
                            if let Some(mut connection) = Connection::new(&ipHeader, &tcpHeader) {
                                connection.onPacket(tcpHeader, &mut buf[..bytesRead], dataStart, &mut nic);
                                entry.insert(connection);
                                pendingConnections.push_back(key);

                                // TODO: Wake up `accept` call as we got a new connection
                                //       Use conditional variable maybe??
                            }
                        }
                    },
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().onPacket(tcpHeader, &mut buf[..bytesRead], dataStart, &mut nic);
                    }
                }

                // TODO: Remove if connection is closed
                //       connectionManager.connections.remove(&key);
            }
        }
        Ok(())
    }

    pub fn bind(&mut self, port: u16) -> io::Result<TCPListener> {
        use std::collections::hash_map::Entry;
        {
            let mut connectionManager = self.connectionManager.lock().unwrap();
            // TODO: Start accepting packets on this port
            match connectionManager.pending.entry(port) {
                Entry::Vacant(v) => {
                    // Create new vector for pending connections on this port
                    v.insert(VecDeque::new());
                },
                Entry::Occupied(_) => {
                    // This port is already in use
                    return Err(io::Error::new(io::ErrorKind::AddrInUse,
                                              "Port already in use by some other application"))
                }
            }
        };

        Ok(TCPListener{
            port,
            connectionManager: self.connectionManager.clone()
        })
    }
}

impl Drop for Interface {
    fn drop(&mut self) {
        let connectionManager = self.connectionManager.lock().unwrap();
        connectionManager.terminate = true;
        // Join the thread running packet loop
        self.thread.join().unwrap();
    }
}

/// ================================================
///                     TCPStream
/// ================================================
/// This provides interface to read and write data in a connection
pub struct TCPStream{
    quad: Quad,
    connectionManager: Arc<Mutex<ConnectionManager>>
}

impl Read for TCPStream{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        {
            let mut connectionManager = self.connectionManager.lock().unwrap();
            let connection = connectionManager.connections.get_mut(&self.quad).ok_or_else( || {
                io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "Stream terminated Unexpectedly"
                )
            })?;

            if connection.incoming.is_empty() {
                // TODO: Block
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "Waiting to any incoming data"
                ));
            }

            /// Copy bytes from `connection.incoming` buffer to `buf`
            let (head, tail) = connection.incoming.as_slices();
            let hlen = min(buf.len(), head.len());
            buf.copy_from_slice(&head[..hlen]);
            let tlen = min(buf.len() - hlen, tail.len());
            buf.copy_from_slice(&tail[..tlen]);
            let len = hlen + tlen;
            drop(connection.incoming.drain(..len));
            return Ok(len);
        };

        Ok(0)
    }
}

impl Write for TCPStream{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        {
            let mut connectionManager = self.connectionManager.lock().unwrap();
            let connection = connectionManager.connections.get_mut(&self.quad).ok_or_else( || {
                io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "Stream terminated Unexpectedly"
                )
            })?;

            if connection.outgoing.len() >= OUTGOING_BUFFER_LIMIT{
                // TODO: Block
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "Outgoing Buffer is full"
                ));
            }

            /// Copy bytes from `buf` to `connection.incoming`
            let len = min(buf.len(), OUTGOING_BUFFER_LIMIT - connection.outgoing.len());
            connection.outgoing.extend(buf[..len].iter());
            return Ok(len);

            // TODO: Write remaining bytes
        };
    }

    /// This functions blocks current thread until it successfully recieves
    /// ACK for all bytes send on network.
    fn flush(&mut self) -> io::Result<()> {
        {
            let mut connectionManager = self.connectionManager.lock().unwrap();
            let connection = connectionManager.connections.get_mut(&self.quad).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "Stream terminated Unexpectedly"
                )
            })?;

            if connection.outgoing.is_empty(){
                return Ok(());
            }
            else{
                // TODO: Block
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "Sending all buffered data"
                ));
            }
        };
    }
}

impl TCPStream{
    pub fn close() {
        // TODO: Send a fin
        unimplemented!();
    }
}

impl Drop for TCPStream {
    fn drop(&mut self){
        let mut connection: Option<Connection>;
        {
            let connectionManager = self.connectionManager.lock().unwrap();
            connection = connectionManager.connections.remove(&self.key);
        }

        if let Some(connection) = connection {
            // TODO: Send fin packets to close connection
            drop(connection);
        }
    }
}

fn main() -> io::Result<()> {
    let srcIP = IPAddress::new(10, 12, 0, 1);
    let dstIP = IPAddress::new(10, 12, 0, 2);
    let mut interface = Interface::new("tun0", srcIP, dstIP)?;
    let mut listener = interface.bind(8080)?;
    std::thread::spawn(move || {
        while let Ok(stream) = listener.accept() {
            // New Connecton
        }
    });
    Ok(())
}

// mod VirtualNetwork;
// mod Parser;
// mod TCPState;
// // mod Listener;
//
// use std::io::{self, Read, Write, Result};
// use VirtualNetwork::VNC;
// use std::collections::HashMap;
// use Parser::{IPAddress, IPProtocol};
// use std::cmp::Eq;
// use std::hash::Hash;
// use crate::Parser::TCPHeader;
// use crate::TCPState::Connection;
// // use Listener::Quad;
//
// #[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
// pub struct Quad{
//     src: (IPAddress, u16),
//     dst: (IPAddress, u16)
// }
//
// fn main() -> Result<()> {
//     let mut nic = VNC::new("tun0", "10.12.0.2", "10.12.0.1")?;
//     println!("Starting with NIC: {:?}", nic);
//     let mut buf = [0u8; 1504];
//     let mut connections: HashMap<Quad, TCPState::Connection> = HashMap::new();
//     loop {
//         if let Ok(bytesRead) = nic.recv(&mut buf){
//             let ipHeader = Parser::IPHeader::from(&buf);
//             if let Some(ipHeader) = ipHeader {
//                 if ipHeader.protocol != IPProtocol::Tcp as u8 {continue;}
//                 let tcpHeaderStart = ipHeader.size();
//                 let tcpHeader = Parser::TCPHeader::from(&buf[tcpHeaderStart..]);
//                 let dataStart = tcpHeaderStart + tcpHeader.size();
//                 // println!("{}:{} -> {}:{}", ipHeader.sourceIP, tcpHeader.sourcePort, ipHeader.destinationIP, tcpHeader.destinationPort);
//
//                 let key = Quad{
//                     src: (ipHeader.sourceIP, tcpHeader.sourcePort),
//                     dst: (ipHeader.destinationIP, tcpHeader.destinationPort)
//                 };
//
//                 match connections.get_mut(&key) {
//                     Some(connection) => {
//                         let res = connection.onPacket(tcpHeader, &mut buf[..bytesRead], dataStart, &mut nic);
//                         if res {
//                             connections.remove(&key);
//                         }
//                     },
//                     None => {
//                         if tcpHeader.syn {
//                             // Open New Connection
//                             let mut connection = Connection::new(&ipHeader, &tcpHeader);
//                             let res = connection.onPacket(tcpHeader, &mut buf[..bytesRead], dataStart, &mut nic);
//                             if !res {
//                                 connections.insert(key, connection);
//                             }
//                         }
//                         else {
//                             println!("Here");
//                         }
//                     }
//                 };
//             }
//         }
//     }
//
//     Ok(())
// }
