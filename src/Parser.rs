pub mod EthernetHeaderParser;
pub mod IPHeaderParser;
pub mod TCPHeaderParser;
pub mod IPTrafficClass;

pub use EthernetHeaderParser::{MACAddress, EtherType, EthernetHeader};
pub use IPHeaderParser::{IPVersion, IPAddress, IPHeader};
pub use TCPHeaderParser::TCPHeader;
pub use IPTrafficClass::IPProtocol;