use std::io::{self, Read, Write, Result};
use std::fs::{File, OpenOptions};
use std::process;

#[derive(Debug)]
pub struct VNC{
    pub fd: File
}

impl VNC {
    pub fn new(ifname: &str, sourceIP: &str, destIP: &str) -> Result<Self> {
        let obj = Self::with_options(&("/dev/".to_owned() + ifname), true);
        if let Ok(_) = &obj {
            let op = process::Command::new("sudo")
                .arg("ifconfig")
                .arg(ifname)
                .arg(sourceIP)
                .arg(destIP).spawn().expect("Ifconfig Failed");
        }
        obj
    }

    pub fn recv(&self, buf: &mut [u8]) -> Result<usize> {
        (&self.fd).read(buf)
    }

    pub fn send(&mut self, buf: &[u8]) -> Result<usize> {
        (&self.fd).write(buf)
    }

    fn with_options(ifname: &str, packet_info: bool) -> Result<Self> {
        let fd = OpenOptions::new()
            .read(true)
            .write(true)
            .open(ifname)?;

        Ok(VNC { fd })
    }
}
