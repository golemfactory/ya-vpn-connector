use std::io;

use bytes::{BufMut, Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

//Note tap packet support is experimental only

#[derive(Debug)]
pub struct AnyPacket(Bytes);

impl AnyPacket {
    /// Create a new `AnyPacket` based on a byte slice.
    pub fn new(bytes: Vec<u8>) -> AnyPacket {
        //let proto = infer_proto(&bytes);
        AnyPacket(Bytes::from(bytes))
    }
    pub fn new_from_bytes(bytes: Bytes) -> AnyPacket {
        //let proto = infer_proto(&bytes);
        AnyPacket(bytes)
    }

    pub fn new_from_slice(bytes: &[u8]) -> AnyPacket {
        //let proto = infer_proto(&bytes);
        AnyPacket(Bytes::from(bytes))
    }

    /// Return this packet's bytes.
    pub fn get_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_bytes(self) -> Bytes {
        self.0
    }
}

/// A AnyPacket Encoder/Decoder.
pub struct AnyPacketCodec();

impl AnyPacketCodec {
    /// Create a new `TapPacketCodec` specifying whether the underlying
    ///  tunnel Device has enabled the packet information header.
    pub fn new() -> AnyPacketCodec {
        AnyPacketCodec()
    }
}

impl Default for AnyPacketCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder for AnyPacketCodec {
    type Item = AnyPacket;
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
        Ok(Some(AnyPacket(pkt.freeze())))
    }
}

impl Encoder<AnyPacket> for AnyPacketCodec {
    type Error = io::Error;

    fn encode(&mut self, item: AnyPacket, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.reserve(item.get_bytes().len() + 4);
        match item {
            AnyPacket(bytes) => dst.put(bytes),
        }
        Ok(())
    }
}
