use std::io;

use bytes::{BufMut, Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

/// A Tun Packet to be sent or received on the TUN interface.
#[derive(Debug)]
pub struct TapPacket(Bytes);

impl TapPacket {
    /// Create a new `TunPacket` based on a byte slice.
    pub fn new(bytes: Vec<u8>) -> TapPacket {
        //let proto = infer_proto(&bytes);
        TapPacket(Bytes::from(bytes))
    }

    /// Return this packet's bytes.
    pub fn get_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_bytes(self) -> Bytes {
        self.0
    }
}

/// A TunPacket Encoder/Decoder.
pub struct TapPacketCodec();

impl TapPacketCodec {
    /// Create a new `TapPacketCodec` specifying whether the underlying
    ///  tunnel Device has enabled the packet information header.
    pub fn new() -> TapPacketCodec {
        TapPacketCodec()
    }
}

impl Default for TapPacketCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder for TapPacketCodec {
    type Item = TapPacket;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if buf.is_empty() {
            return Ok(None);
        }

        let pkt = buf.split_to(buf.len());

        // reserve enough space for the next packet
        buf.reserve(90000);

        // if the packet information is enabled we have to ignore the first 4 bytes
        /*if self.0 {
            let _ = pkt.split_to(4);
        }*/

        //  let proto = infer_proto(pkt.as_ref());
        Ok(Some(TapPacket(pkt.freeze())))
    }
}

impl Encoder<TapPacket> for TapPacketCodec {
    type Error = io::Error;

    fn encode(&mut self, item: TapPacket, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.reserve(item.get_bytes().len() + 4);
        match item {
            TapPacket(bytes) => dst.put(bytes),
        }
        Ok(())
    }
}
