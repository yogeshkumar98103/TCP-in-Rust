#![allow(non_snake_case, unused_variables, unused_imports, unreachable_code, dead_code, unused_must_use)]

mod VirtualNetwork;
mod Parser;
mod TCPState;

use std::io::{self, Read, Write, Result};
use VirtualNetwork::VNC;
use std::collections::HashMap;
use Parser::{IPAddress, IPProtocol};
use std::cmp::Eq;
use std::hash::Hash;
use crate::Parser::TCPHeader;
use crate::TCPState::Connection;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
struct Quad{
    src: (IPAddress, u16),
    dst: (IPAddress, u16)
}

fn main() -> Result<()> {
    let mut nic = VNC::new("tun0", "10.12.0.2", "10.12.0.1")?;
    println!("Starting with NIC: {:?}", nic);
    let mut buf = [0u8; 1504];
    let mut connections: HashMap<Quad, TCPState::Connection> = HashMap::new();
    loop {
        let bytes_read = nic.recv(&mut buf);
        if let Ok(bytesRead) = bytes_read{
            let ipHeader = Parser::IPHeader::from(&buf);
            if let Some(ipHeader) = ipHeader {
                if ipHeader.protocol != IPProtocol::Tcp as u8 {continue;}
                let tcpHeaderStart = ipHeader.size();
                let tcpHeader = Parser::TCPHeader::from(&buf[tcpHeaderStart..]);
                let dataStart = tcpHeaderStart + tcpHeader.size();
                // println!("{}:{} -> {}:{}", ipHeader.sourceIP, tcpHeader.sourcePort, ipHeader.destinationIP, tcpHeader.destinationPort);

                let key = Quad{
                    src: (ipHeader.sourceIP, tcpHeader.sourcePort),
                    dst: (ipHeader.destinationIP, tcpHeader.destinationPort)
                };

                match connections.get_mut(&key) {
                    Some(connection) => {
                        let res = connection.onPacket(tcpHeader, &mut buf[..bytesRead], dataStart, &mut nic);
                        if res {
                            connections.remove(&key);
                        }
                    },
                    None => {
                        if tcpHeader.syn {
                            // Open New Connection
                            let mut connection = Connection::new(&ipHeader, &tcpHeader);
                            let res = connection.onPacket(tcpHeader, &mut buf[..bytesRead], dataStart, &mut nic);
                            if !res {
                                connections.insert(key, connection);
                            }
                        }
                        else {
                            println!("Here");
                        }
                    }
                };
            }
        }
    }

    Ok(())
}
