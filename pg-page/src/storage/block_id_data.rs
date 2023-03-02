use crate::fs_format::{ByteEncodeResult, ByteEncoded};

///
/// BlockId:
///
/// this is a storage type for BlockNumber.  in other words, this type
/// is used for on-disk structures (e.g., in HeapTupleData) whereas
/// BlockNumber is the type on which calculations are performed (e.g.,
/// in access method code).
///
/// there doesn't appear to be any reason to have separate types except
/// for the fact that BlockIds can be SHORTALIGN'd (and therefore any
/// structures that contains them, such as ItemPointerData, can also be
/// SHORTALIGN'd).  this is an important consideration for reducing the
/// space requirements of the line pointer (ItemIdData) array on each
/// page and the header of each heap or index tuple, so it doesn't seem
/// wise to change this without good reason.
///
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct BlockIdData {
    /// block number
    pub bi_hi: u16,
    /// block number
    pub bi_lo: u16,
}

impl ByteEncoded for BlockIdData {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend(self.bi_hi.encode());
        buf.extend(self.bi_lo.encode());
        buf
    }

    fn decode(bytes: &[u8]) -> ByteEncodeResult<Self> {
        let bi_hi = u16::decode(&bytes[0..2])?;
        let bi_lo = u16::decode(&bytes[2..4])?;
        Ok(BlockIdData { bi_hi, bi_lo })
    }

    fn encode_into_writer(&self, writer: &mut impl std::io::Write) -> ByteEncodeResult<()> {
        self.bi_hi.encode_into_writer(writer)?;
        self.bi_lo.encode_into_writer(writer)?;
        Ok(())
    }

    fn decode_from_reader(reader: &mut impl std::io::Read) -> ByteEncodeResult<Self> {
        let bi_hi = u16::decode_from_reader(reader)?;
        let bi_lo = u16::decode_from_reader(reader)?;
        Ok(BlockIdData { bi_hi, bi_lo })
    }

    fn byte_size() -> u16 {
        4
    }
}
