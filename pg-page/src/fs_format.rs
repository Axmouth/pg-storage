use thiserror::Error;

#[derive(Debug, Error)]
pub enum ByteEncodeError {
    #[error("Not enough bytes to decode, expected {expected} bytes, got {actual} bytes")]
    NotEnoughBytes { expected: usize, actual: usize },
    #[error("Too many bytes to decode, expected {expected} bytes, got {actual} bytes")]
    TooManyBytes { expected: usize, actual: usize },
    #[error("Invalid size of bytes to decode, expected {expected} bytes, got {actual} bytes")]
    InvalidSize { expected: usize, actual: usize },
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("UTF8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("UTF16 error: {0}")]
    Utf16Error(#[from] std::string::FromUtf16Error),
    #[error("From UTF8 error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
}

pub type ByteEncodeResult<T> = Result<T, ByteEncodeError>;

pub trait GetByteSliceExt {
    fn get_byte_slice(&self, start: usize, end: usize) -> ByteEncodeResult<&[u8]>;
}

impl GetByteSliceExt for [u8] {
    fn get_byte_slice(&self, start: usize, end: usize) -> ByteEncodeResult<&[u8]> {
        self.get(start..end).ok_or(ByteEncodeError::NotEnoughBytes {
            expected: end,
            actual: self.len(),
        })
    }
}

pub fn read_u64(bytes: &[u8]) -> u64 {
    let mut buf = [0; 8];
    buf.copy_from_slice(bytes);
    u64::from_le_bytes(buf)
}

pub fn read_u32(bytes: &[u8]) -> u32 {
    let mut buf = [0; 4];
    buf.copy_from_slice(bytes);
    u32::from_le_bytes(buf)
}

pub fn read_u16(bytes: &[u8]) -> u16 {
    let mut buf = [0; 2];
    buf.copy_from_slice(bytes);
    u16::from_le_bytes(buf)
}

pub fn read_u8(bytes: &[u8]) -> u8 {
    let mut buf = [0; 1];
    buf.copy_from_slice(bytes);
    u8::from_le_bytes(buf)
}

pub fn read_i16(bytes: &[u8]) -> i16 {
    let mut buf = [0; 2];
    buf.copy_from_slice(bytes);
    i16::from_le_bytes(buf)
}

pub fn read_i32(bytes: &[u8]) -> i32 {
    let mut buf = [0; 4];
    buf.copy_from_slice(bytes);
    i32::from_le_bytes(buf)
}

pub fn read_i64(bytes: &[u8]) -> i64 {
    let mut buf = [0; 8];
    buf.copy_from_slice(bytes);
    i64::from_le_bytes(buf)
}

pub fn write_u64(bytes: &mut [u8], value: u64) {
    bytes.copy_from_slice(&value.to_le_bytes());
}

pub fn write_u32(bytes: &mut [u8], value: u32) {
    bytes.copy_from_slice(&value.to_le_bytes());
}

pub fn write_u16(bytes: &mut [u8], value: u16) {
    bytes.copy_from_slice(&value.to_le_bytes());
}

pub fn write_u8(bytes: &mut [u8], value: u8) {
    bytes.copy_from_slice(&value.to_le_bytes());
}

pub fn write_i8(bytes: &mut [u8], value: i8) {
    bytes.copy_from_slice(&value.to_le_bytes());
}

pub fn write_i16(bytes: &mut [u8], value: i16) {
    bytes.copy_from_slice(&value.to_le_bytes());
}

pub fn write_i32(bytes: &mut [u8], value: i32) {
    bytes.copy_from_slice(&value.to_le_bytes());
}

pub fn write_i64(bytes: &mut [u8], value: i64) {
    bytes.copy_from_slice(&value.to_le_bytes());
}

pub fn write_string(bytes: &mut [u8], value: &str) {
    bytes.copy_from_slice(value.as_bytes());
}

pub fn read_string(bytes: &[u8]) -> ByteEncodeResult<String> {
    Ok(String::from_utf8(bytes.to_vec())?)
}

pub trait ByteEncoded
where
    Self: Sized,
{
    fn encode(&self) -> Vec<u8>;
    fn decode(bytes: &[u8]) -> ByteEncodeResult<Self> {
        let mut reader = std::io::Cursor::new(bytes);
        Self::decode_from_reader(&mut reader)
    }
    fn encode_into_writer(&self, writer: &mut impl std::io::Write) -> ByteEncodeResult<()>;
    fn decode_from_reader(reader: &mut impl std::io::Read) -> ByteEncodeResult<Self>
    where
        Self: Sized;
    fn byte_size() -> u16 {
        0
    }
}

pub trait ByteEncodedSized
where
    Self: Sized + ByteEncoded,
{
    fn encode(&self) -> Vec<u8> {
        <Self as ByteEncoded>::encode(self)
    }

    fn decode(bytes: &[u8], size: usize) -> ByteEncodeResult<Self> {
        match bytes.len().cmp(&size) {
            std::cmp::Ordering::Equal => <Self as ByteEncoded>::decode(bytes),
            std::cmp::Ordering::Greater => Err(ByteEncodeError::TooManyBytes {
                expected: size,
                actual: bytes.len(),
            }),
            std::cmp::Ordering::Less => Err(ByteEncodeError::NotEnoughBytes {
                expected: size,
                actual: bytes.len(),
            }),
        }
    }

    fn encode_into_writer(&self, writer: &mut impl std::io::Write) -> ByteEncodeResult<()> {
        Ok(writer.write_all(&ByteEncoded::encode(self))?)
    }

    fn decode_from_reader(reader: &mut impl std::io::Read, size: usize) -> ByteEncodeResult<Self>
    where
        Self: Sized,
    {
        let mut buf = vec![0; size];
        reader.read_exact(&mut buf)?;
        ByteEncoded::decode(&buf)
    }
}

impl ByteEncoded for u64 {
    fn encode(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> ByteEncodeResult<Self> {
        Ok(read_u64(bytes))
    }

    fn encode_into_writer(&self, writer: &mut impl std::io::Write) -> ByteEncodeResult<()> {
        Ok(writer.write_all(&self.to_le_bytes())?)
    }

    fn decode_from_reader(reader: &mut impl std::io::Read) -> ByteEncodeResult<Self> {
        let mut buf = [0; 8];
        reader.read_exact(&mut buf)?;
        Ok(read_u64(&buf))
    }

    fn byte_size() -> u16 {
        8
    }
}

impl ByteEncoded for u32 {
    fn encode(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> ByteEncodeResult<Self> {
        Ok(read_u32(bytes))
    }

    fn encode_into_writer(&self, writer: &mut impl std::io::Write) -> ByteEncodeResult<()> {
        Ok(writer.write_all(&self.to_le_bytes())?)
    }

    fn decode_from_reader(reader: &mut impl std::io::Read) -> ByteEncodeResult<Self> {
        let mut buf = [0; 4];
        reader.read_exact(&mut buf)?;
        Ok(read_u32(&buf))
    }

    fn byte_size() -> u16 {
        4
    }
}

impl ByteEncoded for u16 {
    fn encode(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> ByteEncodeResult<Self> {
        Ok(read_u16(bytes))
    }

    fn encode_into_writer(&self, writer: &mut impl std::io::Write) -> ByteEncodeResult<()> {
        Ok(writer.write_all(&self.to_le_bytes())?)
    }

    fn decode_from_reader(reader: &mut impl std::io::Read) -> ByteEncodeResult<Self> {
        let mut buf = [0; 2];
        reader.read_exact(&mut buf)?;
        Ok(read_u16(&buf))
    }

    fn byte_size() -> u16 {
        2
    }
}

impl ByteEncoded for u8 {
    fn encode(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> ByteEncodeResult<Self> {
        Ok(read_u8(bytes))
    }

    fn encode_into_writer(&self, writer: &mut impl std::io::Write) -> ByteEncodeResult<()> {
        Ok(writer.write_all(&self.to_le_bytes())?)
    }

    fn decode_from_reader(reader: &mut impl std::io::Read) -> ByteEncodeResult<Self> {
        let mut buf = [0; 1];
        reader.read_exact(&mut buf)?;
        Ok(read_u8(&buf))
    }

    fn byte_size() -> u16 {
        1
    }
}

impl ByteEncoded for String {
    fn encode(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> ByteEncodeResult<Self> {
        Ok(String::from_utf8(bytes.to_vec())?)
    }

    fn encode_into_writer(&self, writer: &mut impl std::io::Write) -> ByteEncodeResult<()> {
        Ok(writer.write_all(self.as_bytes())?)
    }

    fn decode_from_reader(reader: &mut impl std::io::Read) -> ByteEncodeResult<Self> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(String::from_utf8(buf)?)
    }
}

impl<T> ByteEncoded for Vec<T>
where
    T: ByteEncoded + Sized,
{
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        for item in self {
            buf.extend(item.encode());
        }
        buf
    }

    fn decode(bytes: &[u8]) -> ByteEncodeResult<Self> {
        if T::byte_size() != 0 {
            if bytes.len() % T::byte_size() as usize != 0 {
                return Err(ByteEncodeError::InvalidSize {
                    expected: T::byte_size() as usize,
                    actual: bytes.len(),
                });
            }
            let mut items = Vec::new();
            for chunk in bytes.chunks(T::byte_size() as usize) {
                items.push(T::decode(chunk)?);
            }
            Ok(items)
        } else {
            let mut reader = std::io::Cursor::new(bytes);
            let mut items = Vec::new();
            while reader.position() < reader.get_ref().len() as u64 {
                items.push(T::decode_from_reader(&mut reader)?);
            }
            Ok(items)
        }
    }

    fn encode_into_writer(&self, writer: &mut impl std::io::Write) -> ByteEncodeResult<()> {
        for item in self {
            item.encode_into_writer(writer)?;
        }
        Ok(())
    }

    fn decode_from_reader(reader: &mut impl std::io::Read) -> ByteEncodeResult<Self> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let mut reader = std::io::Cursor::new(buf);
        let mut items = Vec::new();
        while reader.position() < reader.get_ref().len() as u64 {
            items.push(T::decode_from_reader(&mut reader)?);
        }
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::*;

    #[test]
    fn test_item() {
        let item = HeapTupleHeaderData {
            t_xmin: 1,
            t_xmax: 2,
            t_cid: 3,
            t_xvac: 4,
            t_ctid: ItemPointerData {
                ip_blkid: BlockIdData { bi_hi: 5, bi_lo: 6 },
                ip_posid: 6,
            },
            t_infomask2: 7,
            t_infomask: 8,
            t_hoff: 9,
            data: vec![1, 2, 3, 4, 5],
        };
        let encoded = item.encode();
        let decoded = HeapTupleHeaderData::decode(&encoded).unwrap();
        assert_eq!(item, decoded);
    }

    #[test]
    fn test_item_pointer() {
        let item_pointer = ItemPointerData {
            ip_blkid: BlockIdData { bi_hi: 1, bi_lo: 2 },
            ip_posid: 2,
        };
        let encoded = item_pointer.encode();
        let decoded = ItemPointerData::decode(&encoded).unwrap();
        assert_eq!(item_pointer, decoded);
    }

    #[test]
    fn test_offset_length_pair() {
        let mut offset_length_pair = ItemIdData::default();
        offset_length_pair.set_lp_flags(1);
        offset_length_pair.set_lp_len(2);
        offset_length_pair.set_lp_off(3);
        let encoded = offset_length_pair.encode();
        let decoded = ItemIdData::decode(&encoded).unwrap();
        assert_eq!(offset_length_pair, decoded);
    }

    #[test]
    fn test_page_header() {
        let page_header = PageHeaderData {
            pd_lsn: PageXLogRecPtr {
                xlogid: 1,
                xrecoff: 2,
            },
            pd_checksum: 2,
            pd_flags: 3,
            pd_lower: 4,
            pd_upper: 5,
            pd_special: 6,
            pd_pagesize_version: 7,
            pd_prune_xid: 8,
        };
        let encoded = page_header.encode();
        let decoded = PageHeaderData::decode(&encoded).unwrap();
        assert_eq!(page_header, decoded);
    }

    #[test]
    fn test_page_header_encode_into_writer() {
        let page_header = PageHeaderData {
            pd_lsn: PageXLogRecPtr {
                xlogid: 1,
                xrecoff: 2,
            },
            pd_checksum: 2,
            pd_flags: 3,
            pd_lower: 4,
            pd_upper: 5,
            pd_special: 6,
            pd_pagesize_version: 7,
            pd_prune_xid: 8,
        };
        let mut buf = Vec::new();
        page_header.encode_into_writer(&mut buf).unwrap();
        let decoded = PageHeaderData::decode(&buf).unwrap();
        assert_eq!(page_header, decoded);
    }

    #[test]
    fn test_page_header_decode_from_reader() {
        let page_header = PageHeaderData {
            pd_lsn: PageXLogRecPtr {
                xlogid: 1,
                xrecoff: 2,
            },
            pd_checksum: 2,
            pd_flags: 3,
            pd_lower: 4,
            pd_upper: 5,
            pd_special: 6,
            pd_pagesize_version: 7,
            pd_prune_xid: 8,
        };
        let encoded = page_header.encode();
        let mut reader = std::io::Cursor::new(encoded);
        let decoded = PageHeaderData::decode_from_reader(&mut reader).unwrap();
        assert_eq!(page_header, decoded);
    }

    #[test]
    fn test_page_header_decode_from_reader_with_extra_bytes() {
        let page_header = PageHeaderData {
            pd_lsn: PageXLogRecPtr {
                xlogid: 1,
                xrecoff: 2,
            },
            pd_checksum: 2,
            pd_flags: 3,
            pd_lower: 4,
            pd_upper: 5,
            pd_special: 6,
            pd_pagesize_version: 7,
            pd_prune_xid: 8,
        };
        let encoded = page_header.encode();
        let mut reader = std::io::Cursor::new(encoded);
        let decoded = PageHeaderData::decode_from_reader(&mut reader).unwrap();
        assert_eq!(page_header, decoded);
    }

    #[test]
    fn test_page_header_decode_from_reader_with_not_enough_bytes() {
        let page_header = PageHeaderData {
            pd_lsn: PageXLogRecPtr {
                xlogid: 1,
                xrecoff: 2,
            },
            pd_checksum: 2,
            pd_flags: 3,
            pd_lower: 4,
            pd_upper: 5,
            pd_special: 6,
            pd_pagesize_version: 7,
            pd_prune_xid: 8,
        };
        let encoded = page_header.encode();
        let mut reader = std::io::Cursor::new(&encoded[..10]);
        let decoded = PageHeaderData::decode_from_reader(&mut reader);
        assert!(decoded.is_err());
    }
}
