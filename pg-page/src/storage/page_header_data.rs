use crate::fs_format::{ByteEncodeResult, ByteEncoded};

use super::page_xl_log_rex_ptr::PageXLogRecPtr;

///
/// disk page organization
///
/// space management information generic to any page
///
///    pd_lsn              - identifies xlog record for last change to this page.
///    pd_checksum         - page checksum, if set.
///    pd_flags            - flag bits.
///    pd_lower            - offset to start of free space.
///    pd_upper            - offset to end of free space.
///    pd_special          - offset to start of special space.
///    pd_pagesize_version - size in bytes and page layout version number.
///    pd_prune_xid        - oldest XID among potentially prunable tuples on page.
///
/// The LSN is used by the buffer manager to enforce the basic rule of WAL:
/// "thou shalt write xlog before data".  A dirty buffer cannot be dumped
/// to disk until xlog has been flushed at least as far as the page's LSN.
///
/// pd_checksum stores the page checksum, if it has been set for this page;
/// zero is a valid value for a checksum. If a checksum is not in use then
/// we leave the field unset. This will typically mean the field is zero
/// though non-zero values may also be present if databases have been
/// pg_upgraded from releases prior to 9.3, when the same byte offset was
/// used to store the current timelineid when the page was last updated.
/// Note that there is no indication on a page as to whether the checksum
/// is valid or not, a deliberate design choice which avoids the problem
/// of relying on the page contents to decide whether to verify it. Hence
/// there are no flag bits relating to checksums.
///
/// pd_prune_xid is a hint field that helps determine whether pruning will be
/// useful.  It is currently unused in index pages.
///
/// The page version number and page size are packed together into a single
/// uint16 field.  This is for historical reasons: before PostgreSQL 7.3,
/// there was no concept of a page version number, and doing it this way
/// lets us pretend that pre-7.3 databases have page version number zero.
/// We constrain page sizes to be multiples of 256, leaving the low eight
/// bits available for a version number.
///
/// Minimum possible page size is perhaps 64B to fit page header, opaque space
/// and a minimal tuple; of course, in reality you want it much bigger, so
/// the constraint on pagesize mod 256 is not an important restriction.
/// On the high end, we can only support pages up to 32KB because lp_off/lp_len
/// are 15 bits.
///
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct PageHeaderData {
    /// LSN: next byte after last byte of WAL record for last change to this page
    pub pd_lsn: PageXLogRecPtr,
    /// Page checksum
    pub pd_checksum: u16,
    /// Flag bits
    pub pd_flags: u16,
    /// Offset to start of free space
    pub pd_lower: u16,
    /// Offset to end of free space
    pub pd_upper: u16,
    /// Offset to start of special space
    pub pd_special: u16,
    /// Page size and layout version number information
    pub pd_pagesize_version: u16,
    /// Oldest unpruned XMAX on page, or zero if none
    pub pd_prune_xid: u32,
}

impl ByteEncoded for PageHeaderData {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend(self.pd_lsn.encode());
        buf.extend(self.pd_checksum.encode());
        buf.extend(self.pd_flags.encode());
        buf.extend(self.pd_lower.encode());
        buf.extend(self.pd_upper.encode());
        buf.extend(self.pd_special.encode());
        buf.extend(self.pd_pagesize_version.encode());
        buf.extend(self.pd_prune_xid.encode());
        buf
    }

    fn decode(bytes: &[u8]) -> ByteEncodeResult<Self> {
        let mut reader = std::io::Cursor::new(bytes);
        Ok(Self {
            pd_lsn: PageXLogRecPtr::decode_from_reader(&mut reader)?,
            pd_checksum: u16::decode_from_reader(&mut reader)?,
            pd_flags: u16::decode_from_reader(&mut reader)?,
            pd_lower: u16::decode_from_reader(&mut reader)?,
            pd_upper: u16::decode_from_reader(&mut reader)?,
            pd_special: u16::decode_from_reader(&mut reader)?,
            pd_pagesize_version: u16::decode_from_reader(&mut reader)?,
            pd_prune_xid: u32::decode_from_reader(&mut reader)?,
        })
    }

    fn encode_into_writer(&self, writer: &mut impl std::io::Write) -> ByteEncodeResult<()> {
        Ok(writer.write_all(&self.encode())?)
    }

    fn decode_from_reader(reader: &mut impl std::io::Read) -> ByteEncodeResult<Self> {
        let mut buf = [0; 24];
        reader.read_exact(&mut buf)?;
        Self::decode(&buf)
    }

    fn byte_size() -> u16 {
        24
    }
}

impl PageHeaderData {
    pub fn page_size(&self) -> usize {
        (self.pd_pagesize_version & 0xFF00) as usize
    }

    pub fn page_version(&self) -> u16 {
        self.pd_pagesize_version & 0x00FF
    }
}