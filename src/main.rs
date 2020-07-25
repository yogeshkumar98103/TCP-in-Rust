#![allow(non_snake_case, unused_variables, unused_imports, unreachable_code, dead_code, unused_must_use, unused_doc_comments)]

mod VirtualNetwork;
mod Parser;
mod TCPConnection;
mod queue;

use VirtualNetwork::VNC;
use Parser::*;
use TCPConnection::*;

use std::io::{self, Read, Write};
use std::collections::{hash_map::Entry, HashMap, VecDeque};
use std::cmp::{Eq, min};
use std::hash::Hash;
use std::sync::{Arc, Mutex, Condvar};
use std::thread::sleep;
use std::time::Duration;

type InterfaceHandler = Arc<Mutex<ConnectionManager>>;

/// ================================================
///                Connection Manager
/// ================================================
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct Quad{
    src: (IPAddress, u16),  // IPAddress + Port
    dst: (IPAddress, u16)   // IPAddress + Port
}

#[derive(Default)]
struct ConnectionManager{
    terminate       : Mutex<bool>,
    connectionMap   : Mutex<HashMap<Quad, Arc<Active>>>,
    pendingMap      : Mutex<HashMap<u16, Arc<Pending>>>,
}

#[derive(Debug)]
struct Pending {
    pendingQueue : Mutex<VecDeque<Arc<Active>>>,
    cond         : Condvar,
}

#[derive(Debug)]
struct Active {
    connection : Mutex<Connection>,
    readCond   : Condvar,
    writeCond  : Condvar,
}

// impl Drop for ConnectionManager {
//     fn drop(&mut self) {
//
//     }
// }

/// ================================================
///                    TCPListener
/// ================================================
pub struct TCPListener {
    port: u16,
    connectionManager: Arc<ConnectionManager>,
    pending: Arc<Pending>,
    terminate: bool,
}

impl TCPListener {
    /// This function blocks current thread and wait for new connection
    /// When a new connection arrives. It resumes and returns a TCPStream
    pub fn accept(&mut self) -> Option<TCPStream> {
        let mut pendingQueue = self.pending.pendingQueue.lock().unwrap();
        loop {
            if self.terminate {
                // Stop accepting new connections
                return None;
            }
            match pendingQueue.pop_front() {
                Some(connection) =>  {
                    connection.connection.lock().unwrap().isHandled = true;
                    return Some(
                        TCPStream{
                            connectionManager: self.connectionManager.clone(),
                            connection
                        }
                    );
                },
                None => {
                    pendingQueue = self.pending.cond.wait(pendingQueue).unwrap();
                }
            }
        }
    }
}

impl Drop for TCPListener {
    fn drop(&mut self) {
        // Stop accepting new connection
        println!("Listener Dropped");
        self.terminate = true;
        self.pending.cond.notify_one();

        // TODO: Optional: Send fin packets to all these connections
        // let mut pendingQueue = self.pending.pendingQueue.lock().unwrap();
        // for quad in pending {
        //
        // }

        // TODO: Remove entry in hashmap
        // drop(pendingQueue);
        let mut pendingMap = self.connectionManager.pendingMap.lock().unwrap();
        pendingMap.remove(&self.port);
    }
}

/// ================================================
///                     Interface
/// ================================================
pub struct Interface {
    thread: Option<std::thread::JoinHandle<()>>,
    connectionManager: Arc<ConnectionManager>
}

impl Interface {
    pub fn new(iface: &str, selfIP: IPAddress, otherIP: IPAddress) -> io::Result<Self> {
        let nic = VNC::new(iface, &(selfIP.toString())[..], &(otherIP.toString())[..])?;
        println!("Starting NIC as {:?}", nic);
        let connectionManager: Arc<ConnectionManager> = Arc::default();
        let thread = {
            let connectionManager = connectionManager.clone();
            std::thread::spawn(move || {
                Interface::packetLoop(nic, connectionManager).unwrap();
            })
        };

        Ok(Self{thread: Some(thread), connectionManager})
    }

    /// This Loop runs forever and looks for any incoming packets
    fn packetLoop(mut nic: VNC, connectionManager: Arc<ConnectionManager>) -> io::Result<()>{
        let mut buf = [0u8; 1500];
        loop {
            // TODO: Add timer
            let bytesRead = nic.recv(&mut buf)?;
            {
                let terminate = connectionManager.terminate.lock().unwrap();
                if *terminate {
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

                let mut connections = connectionManager.connectionMap.lock().unwrap();
                let entry = connections.entry(key);

                match entry {
                    Entry::Vacant(entry) => {
                        let mut pendingMap = connectionManager.pendingMap.lock().unwrap();
                        // If someone is listening then only open the connection
                        if let Some(pendingConnections) = pendingMap.get_mut(&tcpHeader.destinationPort) {
                            if let Some(mut connection) = Connection::new(&ipHeader, &tcpHeader, true) {
                                connection.onPacket(tcpHeader, &mut buf[..bytesRead], dataStart, &mut nic);
                                let connection = Arc::new(
                                    Active {
                                        connection: Mutex::new(connection),
                                        readCond: Condvar::new(),
                                        writeCond: Condvar::new()
                                    }
                                );

                                entry.insert(connection.clone());

                                let mut pendingQueue = pendingConnections.pendingQueue.lock().unwrap();
                                pendingQueue.push_back(connection);


                                // TODO: Wake up `accept` call as we got a new connection
                                //       Use conditional variable maybe??
                                pendingConnections.cond.notify_one();
                            }
                        }
                    },
                    Entry::Occupied(mut entry) => {
                        let mut connection = entry.get_mut().connection.lock().unwrap();
                        let (read, write, delete) = connection.onPacket(tcpHeader, &mut buf[..bytesRead], dataStart, &mut nic);

                        if delete {
                            // Completed Fin exchanges/unexpected behaiour from other side
                            connection.isHandled = false;
                            drop(connection);
                            entry.get_mut().writeCond.notify_one();
                            entry.get_mut().readCond.notify_one();
                            connections.remove(&key);
                        }
                        else{
                            drop(connection);
                            if read {
                                entry.get_mut().readCond.notify_one();
                            }

                            if write {
                                entry.get_mut().writeCond.notify_one();
                            }
                        }
                    }
                }

                // println!("{:?}", connections);
                // TODO: Remove if connection is closed
                //       connectionManager.connections.remove(&key);
            }
        }
        Ok(())
    }

    pub fn bind(&mut self, port: u16) -> io::Result<TCPListener> {
        let mut pendingMap = self.connectionManager.pendingMap.lock().unwrap();

        // TODO: Start accepting packets on this port
        match pendingMap.entry(port) {
            Entry::Vacant(v) => {
                // Create new pendingQueue for pending connections on this port
                let pending = Arc::new(
                    Pending {
                        pendingQueue: Mutex::new(VecDeque::new()),
                        cond : Condvar::new()
                    }
                );
                v.insert(pending.clone());
                Ok(TCPListener {
                    port,
                    connectionManager: self.connectionManager.clone(),
                    pending: pending,
                    terminate: false
                })
            },
            Entry::Occupied(_) => {
                // This port is already in use
                Err(io::Error::new(io::ErrorKind::AddrInUse,
                                   "Port already in use by some other application"))
            }
        }
    }
}

impl Drop for Interface {
    fn drop(&mut self) {
        let mut terminate = self.connectionManager.terminate.lock().unwrap();
        *terminate = true;
        // Join the thread running packet loop
        self.thread.take().unwrap().join();
    }
}

/// ================================================
///                     TCPStream
/// ================================================
/// This provides interface to read and write data in a connection
pub struct TCPStream{
    connectionManager: Arc<ConnectionManager>,
    connection: Arc<Active>,
}

impl Read for TCPStream{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut connection = self.connection.connection.lock().unwrap();
        loop {
            if !connection.isHandled {
                return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "Connection Aborted"));
            }

            if !connection.incoming.is_empty() {
                /// Copy bytes from `connection.incoming` buffer to `buf`
                let (head, tail) = connection.incoming.as_slices();
                let hlen = min(buf.len(), head.len());
                buf[..hlen].copy_from_slice(&head[..hlen]);
                let tlen = min(buf.len() - hlen, tail.len());
                let len = hlen + tlen;
                buf[hlen..len].copy_from_slice(&tail[..tlen]);
                drop(connection.incoming.drain(..len));
                return Ok(len);
            }

            connection = self.connection.readCond.wait(connection).unwrap();
        };
    }
}

impl Write for TCPStream{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.raw_write(buf, true)
    }

    /// This functions blocks current thread until it successfully recieves
    /// ACK for all bytes send on network.
    fn flush(&mut self) -> io::Result<()> {
        let mut connection = self.connection.connection.lock().unwrap();
        loop {
            if connection.outgoing.is_empty(){
                return Ok(());
            }
            connection = self.connection.writeCond.wait(connection).unwrap();
        };
    }
}

impl TCPStream{
    /// Use this if you want to send large data but don't want a large buffer.
    /// Push indicates other side that this is all you wanted to send as response.
    pub fn raw_write(&mut self, mut buf: &[u8], push: bool) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut bytesWritten: usize = 0;
        let mut connection = self.connection.connection.lock().unwrap();
        loop{
            // let mut connection = self.connection.connection.lock().unwrap();
            if connection.outgoing.len() < OUTGOING_BUFFER_LIMIT{
                /// Copy bytes from `buf` to `connection.outgoing`
                let len = min(buf.len(), OUTGOING_BUFFER_LIMIT - connection.outgoing.len());
                connection.outgoing.extend(buf[..len].iter());
                buf = &buf[len..];
                bytesWritten += len;
                if buf.is_empty() {
                    if push {
                        // connection.push();
                    }
                    return Ok(bytesWritten);
                }
            }

            connection = self.connection.writeCond.wait(connection).unwrap();
        }
    }

    pub fn close() {
        // TODO: Send a fin
        unimplemented!();
    }
}

impl Drop for TCPStream {
    fn drop(&mut self){
        let mut connection = self.connection.connection.lock().unwrap();
        // TODO: Send fin packets to close connection
        // connection.sendFin();

        // let mut connectionMap = self.connectionManager.connectionMap.lock().unwrap();
        // let key = connection.getQuad();
        // connectionMap.remove(&key);
    }
}

fn main() -> io::Result<()> {
    let srcIP = IPAddress::new(10, 12, 0, 1);
    let dstIP = IPAddress::new(10, 12, 0, 2);
    let mut interface = Interface::new("tun0", srcIP, dstIP)?;
    let mut listener = interface.bind(9000)?;
    let thread = std::thread::spawn(move || {
        // This handles single Connection at a time. Other connections wait
        while let Some(mut stream) = listener.accept() {
            // New Connecton
            let mut buffer = [0u8; 1000];
            println!("New Connection");
            stream.read(&mut buffer);
            println!("Recieved Request : {}", std::str::from_utf8(&buffer[..]).unwrap());
            let ret = stream.read(&mut buffer);
            match ret {
                Ok(len) => {
                    println!("Recieved Request 2 : {}", std::str::from_utf8(&buffer[..]).unwrap());
                },
                Err(error) => {
                    println!("Error Occured");
                }
            }
            // stream.write(b"Hello From Server");
        }
    });

    thread.join().unwrap();
    Ok(())
}