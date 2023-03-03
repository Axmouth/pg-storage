use c2rust_bitfields::BitfieldStruct;
use crate::util::{ByteEncodeResult, ByteEncoded};

///
/// A line pointer on a buffer page.  See buffer page definitions and comments
/// for an explanation of how line pointers are used.
///
/// In some cases a line pointer is "in use" but does not have any associated
/// storage on the page.  By convention, lp_len == 0 in every line pointer
/// that does not have storage, independently of its lp_flags state.
///
#[derive(Debug, BitfieldStruct, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ItemIdData {
    // offset to tuple (from start of page)
    #[bitfield(name = "lp_off", ty = "u16", bits = "0..=14")]
    // state of line pointer, see below
    #[bitfield(name = "lp_flags", ty = "u8", bits = "15..=16")]
    // byte length of tuple
    #[bitfield(name = "lp_len", ty = "u16", bits = "17..=31")]
    lp: [u8; 4],
}

///
/// lp_flags has these possible states.  An UNUSED line pointer is available
/// for immediate re-use, the other states are not.
///
///      Redirect
/// In a REDIRECT pointer, lp_off holds offset number for next line pointer
///
pub enum LpFlags {
    /// unused (should always have lp_len=0)
    Unused = 0,
    /// used (should always have lp_len>0)
    Normal = 1,
    /// HOT redirect (should have lp_len=0)
    Redirect = 2,
    /// dead, may or may not have storage
    Dead = 3,
}

impl ByteEncoded for ItemIdData {
    fn encode(&self) -> Vec<u8> {
        self.lp.to_vec()
    }

    fn decode(bytes: &[u8]) -> ByteEncodeResult<Self> {
        let mut lp = [0_u8; 4];
        lp.copy_from_slice(bytes);

        Ok(ItemIdData {
            lp,
        })
    }

    fn encode_into_writer(&self, writer: &mut impl std::io::Write) -> ByteEncodeResult<()> {
        writer.write_all(&self.lp)?;
        Ok(())
    }

    fn decode_from_reader(reader: &mut impl std::io::Read) -> ByteEncodeResult<Self> {
        let mut lp = [0_u8; 4];
        reader.read_exact(&mut lp)?;
        Ok(ItemIdData {
            lp,
        })
    }

    fn byte_size() -> u16 {
        4
    }
}

impl ItemIdData {
    pub fn flags(&self) -> LpFlags {
        match self.lp_flags() {
            0 => LpFlags::Unused,
            1 => LpFlags::Normal,
            2 => LpFlags::Redirect,
            _ => LpFlags::Dead,
        }
    }
}