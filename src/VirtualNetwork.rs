use std::io::{self, Read, Write};
use std::fs::{File, OpenOptions};
use std::process;

#[derive(Debug)]
pub struct VNC{
    fd: File
}

impl VNC {
    pub fn new(ifname: &str, sourceIP: &str, destIP: &str) -> io::Result<Self> {
        let obj = Self::with_options(&("/dev/".to_owned() + ifname), true);
        if let Ok(_) = &obj {
            let output = process::Command::new("sudo")
                .arg("ifconfig")
                .arg(ifname)
                .arg(sourceIP)
                .arg(destIP).output().expect("Ifconfig Failed");
        }
        obj
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        (&self.fd).read(buf)
    }

    pub fn send(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&self.fd).write(buf)
    }

    fn with_options(ifname: &str, packet_info: bool) -> io::Result<Self> {
        let fd = OpenOptions::new()
            .read(true)
            .write(true)
            .open(ifname)?;

        Ok(VNC { fd })
    }
}
