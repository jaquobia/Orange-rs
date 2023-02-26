pub mod prot14;


#[cfg(test)]
mod Prot14Test {
    use super::prot14::Packet;
    use orange_networking::{orange_networking_derive::PacketEnumHolder, packet::{PacketEnumHolder, PacketParseable, PacketParseError}, ByteArray};


    #[test]
    fn test_packets() {
        let bytes = [
            Packet::packet_to_bytes(Packet::KeepAlive),
        ].concat();

        let p0 = Packet::bytes_to_packet(&bytes);
        match p0 {
            Ok((Packet::KeepAlive, _)) => { 
            },
            _ => { 
                eprintln!("Not the keep_alive packet");
            },
        };
    }
}
