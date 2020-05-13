struct TCPServer {

}

impl TCPServer {

}

trait TCPServerApplication {
}

struct Application {
    dataBuffer: [u8; 1000],
    listener: TCP::Listener,
}

impl ServerApplication {
    fn new() -> io::Result<Application> {
        Application{
            dataBuffer: [0; 1000],
            listener
        }
    }

    /// This is triggered when a data packet is recieved.
    fn onRecieve(&self, data: &[u8]){
        /// This is the data in packet recieved.
        /// Server can store and use these according to protocol they are using.
        /// When they have recieved suffient data to make sense of request,
        /// they can send packets to respond.
        print!("{}", data);
    }

    /// This is triggered when connection is closed
    fn onClose(){

    }
}

impl TCPServerApplication for ServerApplication {

}

/// Server
fn main(){
    let interface = TCP::Interface::new();
    let listener = interface.bind(8080)?;
    while let Ok(mut stream) = listener.accept() {
        let mut buf = [0; 512];
        // New Connection
        || {
            let n = stream.read(&mut buf[..]).unwrap();
            eprintln!("read {}b of data", n);
            if n == 0 {
                println!("no more data!");
                break;
            } else {
                println!("{}", std::str::from_utf8(&buf[..n]).unwrap());
            }
        }
    }
}