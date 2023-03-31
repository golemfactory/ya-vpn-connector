use crate::packet_conv::{packet_ether_to_ip_slice, packet_ip_wrap_to_ether};

pub fn packet_ip_wrap_to_ether(
    frame: &[u8],
    src_mac: Option<&[u8; 6]>,
    dst_mac: Option<&[u8; 6]>,
) -> Result<Vec<u8>, Error> {
    if frame.is_empty() {
        return Err(Error::Other(
            "Error when wrapping IP packet: Empty packet".into(),
        ));
    }
    if let Err(err) = IpPacket::peek(frame) {
        return Err(Error::PacketMalformed(format!(
            "Error when wrapping IP packet {err}"
        )));
    }

    let mut eth_packet = vec![0u8; frame.len() + 14];
    if let Some(dst_mac) = dst_mac {
        eth_packet[EtherField::DST_MAC].copy_from_slice(dst_mac);
    } else {
        const DEFAULT_DST_MAC: &[u8; 6] = &[0x02, 0x02, 0x02, 0x02, 0x02, 0x02];
        eth_packet[EtherField::DST_MAC].copy_from_slice(DEFAULT_DST_MAC);
    }
    if let Some(src_mac) = src_mac {
        eth_packet[EtherField::SRC_MAC].copy_from_slice(src_mac);
    } else {
        const DEFAULT_SRC_MAC: &[u8; 6] = &[0x01, 0x01, 0x01, 0x01, 0x01, 0x01];
        eth_packet[EtherField::SRC_MAC].copy_from_slice(DEFAULT_SRC_MAC);
    }
    match IpPacket::packet(frame) {
        IpPacket::V4(_pkt) => {
            const ETHER_TYPE_IPV4: &[u8; 2] = &[0x08, 0x00];
            eth_packet[EtherField::ETHER_TYPE].copy_from_slice(ETHER_TYPE_IPV4);
        }
        IpPacket::V6(_pkt) => {
            const ETHER_TYPE_IPV6: &[u8; 2] = &[0x86, 0xdd];
            eth_packet[EtherField::ETHER_TYPE].copy_from_slice(ETHER_TYPE_IPV6);
        }
    };
    eth_packet[EtherField::PAYLOAD].copy_from_slice(&frame[0..]);
    Ok(eth_packet)
}

pub fn packet_ether_to_ip_slice(frame: &[u8]) -> Result<&[u8], Error> {
    const MIN_IP_HEADER_LENGTH: usize = 20;
    if frame.len() <= 14 + MIN_IP_HEADER_LENGTH {
        return Err(Error::Other(format!(
            "Error when creating IP packet from ether packet: Packet too short. Packet length {}",
            frame.len()
        )));
    }
    let ip_frame = &frame[EtherField::PAYLOAD];
    if let Err(err) = IpPacket::peek(ip_frame) {
        return Err(Error::PacketMalformed(format!(
            "Error when creating IP packet from ether packet {err}"
        )));
    }
    Ok(ip_frame)
}

