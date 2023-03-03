use crate::util::{ByteEncodeResult, ByteEncoded};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct PageXLogRecPtr {
    pub xlogid: u32,
    pub xrecoff: u32,
}

impl ByteEncoded for PageXLogRecPtr {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend(self.xlogid.encode());
        buf.extend(self.xrecoff.encode());
        buf
    }

    fn decode(bytes: &[u8]) -> ByteEncodeResult<Self> {
        let xlogid = u32::decode(&bytes[0..4])?;
        let xrecoff = u32::decode(&bytes[4..8])?;
        Ok(PageXLogRecPtr { xlogid, xrecoff })
    }

    fn encode_into_writer(&self, writer: &mut impl std::io::Write) -> ByteEncodeResult<()> {
        self.xlogid.encode_into_writer(writer)?;
        self.xrecoff.encode_into_writer(writer)?;
        Ok(())
    }

    fn decode_from_reader(reader: &mut impl std::io::Read) -> ByteEncodeResult<Self> {
        let xlogid = u32::decode_from_reader(reader)?;
        let xrecoff = u32::decode_from_reader(reader)?;
        Ok(PageXLogRecPtr { xlogid, xrecoff })
    }

    fn byte_size() -> u16 {
        8
    }
}
