use crate::util::{ByteEncodeResult, ByteEncoded};

use super::block_id_data::BlockIdData;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ItemPointerData {
    /// block number
    pub ip_blkid: BlockIdData,
    /// offset in page
    pub ip_posid: u16,
}

impl ByteEncoded for ItemPointerData {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend(self.ip_blkid.encode());
        buf.extend(self.ip_posid.encode());
        buf
    }

    fn decode(bytes: &[u8]) -> ByteEncodeResult<Self> {
        let ip_blkid = BlockIdData::decode(&bytes[0..4])?;
        let ip_posid = u16::decode(&bytes[4..6])?;
        Ok(ItemPointerData { ip_blkid, ip_posid })
    }

    fn encode_into_writer(&self, writer: &mut impl std::io::Write) -> ByteEncodeResult<()> {
        self.ip_blkid.encode_into_writer(writer)?;
        self.ip_posid.encode_into_writer(writer)?;
        Ok(())
    }

    fn decode_from_reader(reader: &mut impl std::io::Read) -> ByteEncodeResult<Self> {
        let ip_blkid = BlockIdData::decode_from_reader(reader)?;
        let ip_posid = u16::decode_from_reader(reader)?;
        Ok(ItemPointerData { ip_blkid, ip_posid })
    }

    fn byte_size() -> u16 {
        6
    }
}
