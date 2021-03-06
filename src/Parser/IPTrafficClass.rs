use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IPProtocol {
    ///IPv6 Hop-by-Hop Option [RFC8200]
    IPv6HeaderHopByHop = 0,
    ///Internet Control Message [RFC792]
    Icmp = 1,
    ///Internet Group Management [RFC1112]
    Igmp = 2,
    ///Gateway-to-Gateway [RFC823]
    Ggp = 3,
    ///IPv4 encapsulation [RFC2003]
    IPv4 = 4,
    ///Stream [RFC1190][RFC1819]
    Stream = 5,
    ///Transmission Control [RFC793]
    Tcp = 6,
    ///CBT [Tony_Ballardie]
    Cbt = 7,
    ///Exterior Gateway Protocol [RFC888][David_Mills]
    Egp = 8,
    ///any private interior gateway (used by Cisco for their IGRP) [Internet_Assigned_Numbers_Authority]
    Igp = 9,
    ///BBN RCC Monitoring [Steve_Chipman]
    BbnRccMon = 10,
    ///Network Voice Protocol [RFC741][Steve_Casner]
    NvpII = 11,
    ///PUP
    Pup = 12,
    ///ARGUS (deprecated) [Robert_W_Scheifler]
    Argus = 13,
    ///EMCON [<mystery contact>]
    Emcon = 14,
    ///Cross Net Debugger [Haverty, J., "XNET Formats for Internet Protocol Version 4", IEN 158, October 1980.][Jack_Haverty]
    Xnet = 15,
    ///Chaos [J_Noel_Chiappa]
    Chaos = 16,
    ///User Datagram [RFC768][Jon_Postel]
    Udp = 17,
    ///Multiplexing [Cohen, D. and J. Postel, "Multiplexing Protocol", IEN 90, USC/Information Sciences Institute, May 1979.][Jon_Postel]
    Mux = 18,
    ///DCN Measurement Subsystems [David_Mills]
    DcnMeas = 19,
    ///Host Monitoring [RFC869][Bob_Hinden]
    Hmp = 20,
    ///Packet Radio Measurement [Zaw_Sing_Su]
    Prm = 21,
    ///XEROX NS IDP
    XnsIdp = 22,
    ///Trunk-1 [Barry_Boehm]
    Trunk1 = 23,
    ///Trunk-2 [Barry_Boehm]
    Trunk2 = 24,
    ///Leaf-1 [Barry_Boehm]
    Leaf1 = 25,
    ///Leaf-2 [Barry_Boehm]
    Leaf2 = 26,
    ///Reliable Data Protocol [RFC908][Bob_Hinden]
    Rdp = 27,
    ///Internet Reliable Transaction [RFC938][Trudy_Miller]
    Irtp = 28,
    ///ISO Transport Protocol Class 4 [RFC905][<mystery contact>]
    IsoTp4 = 29,
    ///Bulk Data Transfer Protocol [RFC969][David_Clark]
    NetBlt = 30,
    ///MFE Network Services Protocol [Shuttleworth, B., "A Documentary of MFENet, a National Computer Network", UCRL-52317, Lawrence Livermore Labs, Livermore, California, June 1977.][Barry_Howard]
    MfeNsp = 31,
    ///MERIT Internodal Protocol [Hans_Werner_Braun]
    MeritInp = 32,
    ///Datagram Congestion Control Protocol [RFC4340]
    Dccp = 33,
    ///Third Party Connect Protocol [Stuart_A_Friedberg]
    ThirdPartyConnectProtocol = 34,
    ///Inter-Domain Policy Routing Protocol [Martha_Steenstrup]
    Idpr = 35,
    ///XTP [Greg_Chesson]
    Xtp = 36,
    ///Datagram Delivery Protocol [Wesley_Craig]
    Ddp = 37,
    ///IDPR Control Message Transport Proto [Martha_Steenstrup]
    IdprCmtp = 38,
    ///TP++ Transport Protocol [Dirk_Fromhein]
    TpPlusPlus = 39,
    ///IL Transport Protocol [Dave_Presotto]
    Il = 40,
    ///IPv6 encapsulation [RFC2473]
    Ipv6 = 41,
    ///Source Demand Routing Protocol [Deborah_Estrin]
    Sdrp = 42,
    ///Routing Header for IPv6 [Steve_Deering]
    IPv6RouteHeader = 43,
    ///Fragment Header for IPv6 [Steve_Deering]
    IPv6FragmentationHeader = 44,
    ///Inter-Domain Routing Protocol [Sue_Hares]
    Idrp = 45,
    ///Reservation Protocol [RFC2205][RFC3209][Bob_Braden]
    Rsvp = 46,
    ///Generic Routing Encapsulation [RFC2784][Tony_Li]
    Gre = 47,
    ///Dynamic Source Routing Protocol [RFC4728]
    Dsr = 48,
    ///BNA [Gary Salamon]
    Bna = 49,
    ///Encap Security Payload [RFC4303]
    IPv6EncapSecurityPayload = 50,
    ///Authentication Header [RFC4302]
    IPv6AuthenticationHeader = 51,
    ///Integrated Net Layer Security  TUBA [K_Robert_Glenn]
    Inlsp = 52,
    ///IP with Encryption (deprecated) [John_Ioannidis]
    Swipe = 53,
    ///NBMA Address Resolution Protocol [RFC1735]
    Narp = 54,
    ///IP Mobility [Charlie_Perkins]
    Mobile = 55,
    ///Transport Layer Security Protocol using Kryptonet key management [Christer_Oberg]
    Tlsp = 56,
    ///SKIP [Tom_Markson]
    Skip = 57,
    ///ICMP for IPv6 [RFC8200]
    IPv6Icmp = 58,
    ///No Next Header for IPv6 [RFC8200]
    IPv6NoNextHeader = 59,
    ///Destination Options for IPv6 [RFC8200]
    IPv6DestinationOptions = 60,
    ///any host internal protocol [Internet_Assigned_Numbers_Authority]
    AnyHostInternalProtocol = 61,
    ///CFTP [Forsdick, H., "CFTP", Network Message, Bolt Beranek and Newman, January 1982.][Harry_Forsdick]
    Cftp = 62,
    ///any local network [Internet_Assigned_Numbers_Authority]
    AnyLocalNetwork = 63,
    ///SATNET and Backroom EXPAK [Steven_Blumenthal]
    SatExpak = 64,
    ///Kryptolan [Paul Liu]
    Krytolan = 65,
    ///MIT Remote Virtual Disk Protocol [Michael_Greenwald]
    Rvd = 66,
    ///Internet Pluribus Packet Core [Steven_Blumenthal]
    Ippc = 67,
    ///any distributed file system [Internet_Assigned_Numbers_Authority]
    AnyDistributedFileSystem = 68,
    ///SATNET Monitoring [Steven_Blumenthal]
    SatMon = 69,
    ///VISA Protocol [Gene_Tsudik]
    Visa = 70,
    ///Internet Packet Core Utility [Steven_Blumenthal]
    Ipcv = 71,
    ///Computer Protocol Network Executive [David Mittnacht]
    Cpnx = 72,
    ///Computer Protocol Heart Beat [David Mittnacht]
    Cphb = 73,
    ///Wang Span Network [Victor Dafoulas]
    Wsn = 74,
    ///Packet Video Protocol [Steve_Casner]
    Pvp = 75,
    ///Backroom SATNET Monitoring [Steven_Blumenthal]
    BrSatMon = 76,
    ///SUN ND PROTOCOL-Temporary [William_Melohn]
    SunNd = 77,
    ///WIDEBAND Monitoring [Steven_Blumenthal]
    WbMon = 78,
    ///WIDEBAND EXPAK [Steven_Blumenthal]
    WbExpak = 79,
    ///ISO Internet Protocol [Marshall_T_Rose]
    IsoIp = 80,
    ///VMTP [Dave_Cheriton]
    Vmtp = 81,
    ///SECURE-VMTP [Dave_Cheriton]
    SecureVmtp = 82,
    ///VINES [Brian Horn]
    Vines = 83,
    ///Transaction Transport Protocol or Internet Protocol Traffic Manager [Jim_Stevens]
    TtpOrIptm = 84,
    ///NSFNET-IGP [Hans_Werner_Braun]
    NsfnetIgp = 85,
    ///Dissimilar Gateway Protocol [M/A-COM Government Systems, "Dissimilar Gateway Protocol Specification, Draft Version", Contract no. CS901145, November 16, 1987.][Mike_Little]
    Dgp = 86,
    ///TCF [Guillermo_A_Loyola]
    Tcf = 87,
    ///EIGRP [RFC7868]
    Eigrp = 88,
    ///OSPFIGP [RFC1583][RFC2328][RFC5340][John_Moy]
    Ospfigp = 89,
    ///Sprite RPC Protocol [Welch, B., "The Sprite Remote Procedure Call System", Technical Report, UCB/Computer Science Dept., 86/302, University of California at Berkeley, June 1986.][Bruce Willins]
    SpriteRpc = 90,
    ///Locus Address Resolution Protocol [Brian Horn]
    Larp = 91,
    ///Multicast Transport Protocol [Susie_Armstrong]
    Mtp = 92,
    ///AX.25 Frames [Brian_Kantor]
    Ax25 = 93,
    ///IP-within-IP Encapsulation Protocol [John_Ioannidis]
    Ipip = 94,
    ///Mobile Internetworking Control Pro. (deprecated) [John_Ioannidis]
    Micp = 95,
    ///Semaphore Communications Sec. Pro. [Howard_Hart]
    SccSp = 96,
    ///Ethernet-within-IP Encapsulation [RFC3378]
    EtherIp = 97,
    ///Encapsulation Header [RFC1241][Robert_Woodburn]
    Encap = 98,
    ///GMTP [[RXB5]]
    Gmtp = 100,
    ///Ipsilon Flow Management Protocol [Bob_Hinden][November 1995, 1997.]
    Ifmp = 101,
    ///PNNI over IP [Ross_Callon]
    Pnni = 102,
    ///Protocol Independent Multicast [RFC7761][Dino_Farinacci]
    Pim = 103,
    ///ARIS [Nancy_Feldman]
    Aris = 104,
    ///SCPS [Robert_Durst]
    Scps = 105,
    ///QNX [Michael_Hunter]
    Qnx = 106,
    ///Active Networks [Bob_Braden]
    ActiveNetworks = 107,
    ///IP Payload Compression Protocol [RFC2393]
    IpComp = 108,
    ///Sitara Networks Protocol [Manickam_R_Sridhar]
    SitraNetworksProtocol = 109,
    ///Compaq Peer Protocol [Victor_Volpe]
    CompaqPeer = 110,
    ///IPX in IP [CJ_Lee]
    IpxInIp = 111,
    ///Virtual Router Redundancy Protocol [RFC5798]
    Vrrp = 112,
    ///PGM Reliable Transport Protocol [Tony_Speakman]
    Pgm = 113,
    ///any 0-hop protocol [Internet_Assigned_Numbers_Authority]
    AnyZeroHopProtocol = 114,
    ///Layer Two Tunneling Protocol [RFC3931][Bernard_Aboba]
    Layer2TunnelingProtocol = 115,
    ///D-II Data Exchange (DDX) [John_Worley]
    Ddx = 116,
    ///Interactive Agent Transfer Protocol [John_Murphy]
    Iatp = 117,
    ///Schedule Transfer Protocol [Jean_Michel_Pittet]
    Stp = 118,
    ///SpectraLink Radio Protocol [Mark_Hamilton]
    Srp = 119,
    ///UTI [Peter_Lothberg]
    Uti = 120,
    ///Simple Message Protocol [Leif_Ekblad]
    SimpleMessageProtocol = 121,
    ///Simple Multicast Protocol (deprecated) [Jon_Crowcroft][draft-perlman-simple-multicast]
    Sm = 122,
    ///Performance Transparency Protocol [Michael_Welzl]
    Ptp = 123,
    ///ISIS over IPv4 [Tony_Przygienda]
    IsisOverIpv4 = 124,
    ///FIRE [Criag_Partridge]
    Fire = 125,
    ///Combat Radio Transport Protocol [Robert_Sautter]
    Crtp = 126,
    ///Combat Radio User Datagram [Robert_Sautter]
    Crudp = 127,
    ///SSCOPMCE [Kurt_Waber]
    Sscopmce = 128,
    ///IPLT [[Hollbach]]
    Iplt = 129,
    ///Secure Packet Shield [Bill_McIntosh]
    Sps = 130,
    ///Private IP Encapsulation within IP [Bernhard_Petri]
    Pipe = 131,
    ///Stream Control Transmission Protocol [Randall_R_Stewart]
    Sctp = 132,
    ///Fibre Channel [Murali_Rajagopal][RFC6172]
    Fc = 133,
    ///RSVP-E2E-IGNORE [RFC3175]
    RsvpE2eIgnore = 134,
    ///MobilityHeader [RFC6275]
    MobilityHeader = 135,
    ///UDPLite [RFC3828]
    UdpLite = 136,
    /// [RFC4023]
    MplsInIp = 137,
    ///MANET Protocols [RFC5498]
    Manet = 138,
    ///Host Identity Protocol [RFC7401]
    Hip = 139,
    ///Shim6 Protocol [RFC5533]
    Shim6 = 140,
    ///Wrapped Encapsulating Security Payload [RFC5840]
    Wesp = 141,
    ///Robust Header Compression [RFC5858]
    Rohc = 142,
    ///Use for experimentation and testing
    ExperimentalAndTesting0 = 253,
    ///Use for experimentation and testing
    ExperimentalAndTesting1 = 254
}