use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::io;
use std::io::prelude::*;
use std::collections::{HashMap, VecDeque};
use super::*;

// #[derive(Default)]
// struct InterfaceHandler {
//     manager: Mutex<ConnectionManager>,
//     pendingConnections: Condvar,
//     recievingConnections: Condvar,
// }
//
// #[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
// pub struct Quad{
//     src: (IPAddress, u16),
//     dst: (IPAddress, u16)
// }
//
// #[derive(Default)]
// struct ConnectionManager {
//     terminate: bool,
//     connections: HashMap<Quad, tcp::Connection>,
//     pending: HashMap<u16, VecDeque<Quad>>,
// }
//


pub struct Interface {
    connections
}

// impl Drop for Interface {
//     fn drop(&mut self) {
//         self.interfaceHandler.as_mut().unwrap().manager.lock().unwrap().terminate = true;
//
//         drop(self.interfaceHandler.take());
//         self.jointHandler.take().expect("interface dropped more than once")
//             .join()
//             .unwrap()
//             .unwrap();
//     }
// }
// pub struct TCPListener{
//     port: u16,
//     handler: Arc<InterfaceHandler>
// }

pub struct TCPStream{

}

impl Read for TCPStream{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {

    }
}

impl Write for TCPStream{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {

    }

    fn flush(&mut self) -> io::Result<()> {

    }
}

