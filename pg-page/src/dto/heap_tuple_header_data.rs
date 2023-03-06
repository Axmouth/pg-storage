use crate::util::{ByteEncodeResult, ByteEncoded, GetByteSliceExt, ByteEncodeError};

use super::item_pointer_data::ItemPointerData;

pub struct MinimalTupleData {
    /// actual length of minimal tuple
    pub t_len: u32,

    /// Fields below here must match HeapTupleHeaderData!
    pub mt_padding: Vec<u8>,

    /// number of attributes + various flags
    pub t_infomask2: u16,
    /// various flag bits, see below
    pub t_infomask: u16,
    /// sizeof header incl. bitmap, padding

    /// ^ - 23 bytes - ^
    pub t_hoff: u8,
    /// bitmap of NULLs
    pub t_bits: Vec<u8>,
    // MORE DATA FOLLOWS AT END OF STRUCT
}

///
/// Heap tuple header.  To avoid wasting space, the fields should be
/// laid out in such a way as to avoid structure padding.
///
/// Datums of composite types (row types) share the same general structure
/// as on-disk tuples, so that the same routines can be used to build and
/// examine them.  However the requirements are slightly different: a Datum
/// does not need any transaction visibility information, and it does need
/// a length word and some embedded type information.  We can achieve this
/// by overlaying the xmin/cmin/xmax/cmax/xvac fields of a heap tuple
/// with the fields needed in the Datum case.  Typically, all tuples built
/// in-memory will be initialized with the Datum fields; but when a tuple is
/// about to be inserted in a table, the transaction fields will be filled,
/// overwriting the datum fields.
///
/// The overall structure of a heap tuple looks like:
///            fixed fields (HeapTupleHeaderData struct)
///            nulls bitmap (if HEAP_HASNULL is set in t_infomask)
///            alignment padding (as needed to make user data MAXALIGN'd)
///            object ID (if HEAP_HASOID_OLD is set in t_infomask, not created
///            anymore)
///            user data fields
///
/// We store five "virtual" fields Xmin, Cmin, Xmax, Cmax, and Xvac in three
/// physical fields.  Xmin and Xmax are always really stored, but Cmin, Cmax
/// and Xvac share a field.  This works because we know that Cmin and Cmax
/// are only interesting for the lifetime of the inserting and deleting
/// transaction respectively.  If a tuple is inserted and deleted in the same
/// transaction, we store a "combo" command id that can be mapped to the real
/// cmin and cmax, but only by use of local state within the originating
/// backend.  See combocid.c for more details.  Meanwhile, Xvac is only set by
/// old-style VACUUM FULL, which does not have any command sub-structure and so
/// does not need either Cmin or Cmax.  (This requires that old-style VACUUM
/// FULL never try to move a tuple whose Cmin or Cmax is still interesting,
/// ie, an insert-in-progress or delete-in-progress tuple.)
///
/// A word about t_ctid: whenever a new tuple is stored on disk, its t_ctid
/// is initialized with its own TID (location).  If the tuple is ever updated,
/// its t_ctid is changed to point to the replacement version of the tuple.  Or
/// if the tuple is moved from one partition to another, due to an update of
/// the partition key, t_ctid is set to a special value to indicate that
/// (see ItemPointerSetMovedPartitions).  Thus, a tuple is the latest version
/// of its row iff XMAX is invalid or
/// t_ctid points to itself (in which case, if XMAX is valid, the tuple is
/// either locked or deleted).  One can follow the chain of t_ctid links
/// to find the newest version of the row, unless it was moved to a different
/// partition.  Beware however that VACUUM might
/// erase the pointed-to (newer) tuple before erasing the pointing (older)
/// tuple.  Hence, when following a t_ctid link, it is necessary to check
/// to see if the referenced slot is empty or contains an unrelated tuple.
/// Check that the referenced tuple has XMIN equal to the referencing tuple's
/// XMAX to verify that it is actually the descendant version and not an
/// unrelated tuple stored into a slot recently freed by VACUUM.  If either
/// check fails, one may assume that there is no live descendant version.
///
/// t_ctid is sometimes used to store a speculative insertion token, instead
/// of a real TID.  A speculative token is set on a tuple that's being
/// inserted, until the inserter is sure that it wants to go ahead with the
/// insertion.  Hence a token should only be seen on a tuple with an XMAX
/// that's still in-progress, or invalid/aborted.  The token is replaced with
/// the tuple's real TID when the insertion is confirmed.  One should never
/// see a speculative insertion token while following a chain of t_ctid links,
/// because they are not used on updates, only insertions.
///
/// Following the fixed header fields, the nulls bitmap is stored (beginning
/// at t_bits).  The bitmap is *not* stored if t_infomask shows that there
/// are no nulls in the tuple.  If an OID field is present (as indicated by
/// t_infomask), then it is stored just before the user data, which begins at
/// the offset shown by t_hoff.  Note that t_hoff must be a multiple of
/// MAXALIGN.
///
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct HeapTupleHeaderData {
    /// insert XID stamp
    pub t_xmin: u32,
    /// delete XID stamp
    pub t_xmax: u32,
    // TODO: t_field3
    pub t_field3: u32,
    /// current TID of this or newer tuple (or a
	/// speculative insertion token)
    pub t_ctid: ItemPointerData,
    /// number of attributes, plus various flag bits
    pub t_infomask2: u16,
    /// various flag bits
    pub t_infomask: u16,
    /// offset to user data
    pub t_hoff: u8,
    /// Data
    pub data: Vec<u8>,
}

///
/// datum_typeid cannot be a domain over composite, only plain composite,
/// even if the datum is meant as a value of a domain-over-composite type.
/// This is in line with the general principle that CoerceToDomain does not
/// change the physical representation of the base type value.
///
/// Note: field ordering is chosen with thought that Oid might someday
/// widen to 64 bits.
///
pub struct DatumTupleFields {
    /// varlena header (do not touch directly!)
    pub datum_len_: u32,
    /// -1, or identifier of a record type
    pub datum_typmod: u32,
    /// composite type OID, or RECORDOID
    pub datum_typeid: u32,
}

pub enum TField3 {
    /// current TID of this or newer row version
    /// inserting or deleting command ID, or both
    CommandId(u32),
    /// XID for VACUUM operation moving a row version
    /// old-style VACUUM FULL xact ID
    Xvac(u32),
}

impl ByteEncoded for HeapTupleHeaderData {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend(self.t_xmin.encode());
        buf.extend(self.t_xmax.encode());
        buf.extend(self.t_field3.encode());
        buf.extend(self.t_infomask2.encode());
        buf.extend(self.t_infomask.encode());
        buf.extend(self.t_hoff.encode());
        buf.extend(self.data.encode());
        buf
    }

    fn decode(bytes: &[u8]) -> ByteEncodeResult<Self> {
        let t_xmin = u32::decode(bytes.get_byte_slice(0, 4)?)?;
        let t_xmax = u32::decode(bytes.get_byte_slice(4, 8)?)?;
        let t_field3 = u32::decode(bytes.get_byte_slice(8, 12)?)?;
        let t_ctid = ItemPointerData::decode(bytes.get_byte_slice(12, 18)?)?;
        let t_infomask2 = u16::decode(bytes.get_byte_slice(18, 20)?)?;
        let t_infomask = u16::decode(bytes.get_byte_slice(20, 22)?)?;
        let t_hoff = u8::decode(bytes.get_byte_slice(22, 23)?)?;
        let data = bytes.get(23..).ok_or(ByteEncodeError::NotEnoughBytes { expected: 23, actual: bytes.len() })?.to_vec();
        Ok(HeapTupleHeaderData {
            t_xmin,
            t_xmax,
            t_field3,
            t_ctid,
            t_infomask2,
            t_infomask,
            t_hoff,
            data,
        })
    }

    fn encode_into_writer(&self, writer: &mut impl std::io::Write) -> ByteEncodeResult<()> {
        self.t_xmin.encode_into_writer(writer)?;
        self.t_xmax.encode_into_writer(writer)?;
        self.t_field3.encode_into_writer(writer)?;
        self.t_infomask2.encode_into_writer(writer)?;
        self.t_infomask.encode_into_writer(writer)?;
        self.t_hoff.encode_into_writer(writer)?;
        Ok(())
    }

    fn decode_from_reader(reader: &mut impl std::io::Read) -> ByteEncodeResult<Self> {
        let t_xmin = u32::decode_from_reader(reader)?;
        let t_xmax = u32::decode_from_reader(reader)?;
        let t_field3 = u32::decode_from_reader(reader)?;
        let t_ctid = ItemPointerData::decode_from_reader(reader)?;
        let t_infomask2 = u16::decode_from_reader(reader)?;
        let t_infomask = u16::decode_from_reader(reader)?;
        let t_hoff = u8::decode_from_reader(reader)?;
        let mut data = vec![];
        reader.read_to_end(&mut data)?;
        Ok(HeapTupleHeaderData {
            t_xmin,
            t_xmax,
            t_field3,
            t_ctid,
            t_infomask2,
            t_infomask,
            t_hoff,
            data,
        })
    }
}

impl HeapTupleHeaderData {
    pub fn visible_to_tx(&self, xid: u32) -> bool {
        todo!()
    }
}

///
/// information stored in t_infomask:
/// has null attribute(s)
const HEAP_HASNULL: u16 = 0x0001;

/// has variable-width attribute(s)
const HEAP_HASVARWIDTH: u16 = 0x0002;

/// has external stored attribute(s)
const HEAP_HASEXTERNAL: u16 = 0x0004;

/// has an object-id field  
const HEAP_HASOID_OLD: u16 = 0x0008;

/// xmax is a key-shared locker
const HEAP_XMAX_KEYSHR_LOCK: u16 = 0x0010;

/// t_cid is a combo CID
const HEAP_COMBOCID: u16 = 0x0020;

/// xmax is exclusive locker
const HEAP_XMAX_EXCL_LOCK: u16 = 0x0040; // xmax, if valid, is only a locker
const HEAP_XMAX_LOCK_ONLY: u16 = 0x0080;

/// xmax is a shared locker
const HEAP_XMAX_SHR_LOCK: u16 = (HEAP_XMAX_EXCL_LOCK | HEAP_XMAX_KEYSHR_LOCK);

const HEAP_LOCK_MASK: u16 = (HEAP_XMAX_SHR_LOCK | HEAP_XMAX_EXCL_LOCK | HEAP_XMAX_KEYSHR_LOCK);

/// t_xmin committed
const HEAP_XMIN_COMMITTED: u16 = 0x0100;

/// t_xmin invalid/aborted
const HEAP_XMIN_INVALID: u16 = 0x0200;
const HEAP_XMIN_FROZEN: u16 = (HEAP_XMIN_COMMITTED | HEAP_XMIN_INVALID);

/// t_xmax committed
const HEAP_XMAX_COMMITTED: u16 = 0x0400;

/// t_xmax invalid/aborted
const HEAP_XMAX_INVALID: u16 = 0x0800;

/// t_xmax is a MultiXactId
const HEAP_XMAX_IS_MULTI: u16 = 0x1000;

/// this is UPDATEd version of row
const HEAP_UPDATED: u16 = 0x2000;

/// moved to another place by pre-9.0
/// VACUUM FULL; kept for binary
/// upgrade support
const HEAP_MOVED_OFF: u16 = 0x4000;

/// moved from another place by pre-9.0
/// VACUUM FULL; kept for binary
/// upgrade support  
const HEAP_MOVED_IN: u16 = 0x8000;
const HEAP_MOVED: u16 = (HEAP_MOVED_OFF | HEAP_MOVED_IN);
/// visibility-related bits
const HEAP_XACT_MASK: u16 = 0xFFF0;
