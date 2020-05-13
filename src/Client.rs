#[path = "TCPClient.rs"]
mod TCPClient;

use TCPClient::*;

struct Client {
    connected: bool,
    interface: TCP::ClientInterface,
    buffer: [u8; 1000],
    idx: i32
}

impl Client {
    fn new() -> Client{
        Client{
            connected: false,
            interface: TCP::ClientInterface::new(),
            buffer: [0; 1000],
            idx: 0
        }
    }

    fn connect(&mut self, serverIP: IPAddress, serverPort: u16){
        let successful = interface.connect(serverIP, serverPort);
        if successful {
            self.connected = true;
        }
    }

    fn send(&mut self, request: String) -> String {
        let mut stream = self.interface.send(request.as_bytes());
        stream.addListener(|data| {
            let len = stream.data.len();
            if(len > 1000){
                panic!("Recieved huge data");
            }
            self.buffer[idx..len].copy_from_slice(stream.data);
        });

        return String::from_u8(buffer);
    }

    fn disconnect(&mut self){
        if self.connected {
            self.interface.disconnect();
            self.connected = false;
        }
    }
}

/// Client
fn main(){
    /// Create a new client
    let mut client = Client::new();

    /// Connect to server on given IP Address and Port
    let serverIP = IPAddress::new(127, 0, 0, 1);
    client.connect(serverIP, 8080);

    /// Send a request using send method. It will return the response
    let response = client.send("get page.txt");
    println!("{}", response);

    /// Close connection
    client.disconnect();
}