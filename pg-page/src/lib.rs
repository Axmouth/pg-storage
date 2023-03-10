use thiserror::Error;

pub mod compile_constants;
pub mod util;
pub mod dto;
pub mod page_reader;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error with byte decoding: {0}")]
    ByteEncoding(#[from] util::ByteEncodeError),
    #[error("Invalid byte encoding: {0}")]
    InvalidByteEncoding(String),
    #[error("Invalid page size: {0}")]
    InvalidPageSize(u16),
    #[error("Invalid page header size: {0}")]
    InvalidPageHeaderSize(u16),
    #[error("Invalid page header lower bound: {0}")]
    InvalidPageHeaderLowerBound(u16),
    #[error("Invalid page header upper bound: {0}")]
    InvalidPageHeaderUpperBound(u16),
    #[error("Invalid page header special bound: {0}")]
    InvalidPageHeaderSpecialBound(u16),
    #[error("Invalid page header special size: {0}")]
    InvalidPageHeaderSpecialSize(u16),
    #[error("Invalid page header special offset: {0}")]
    InvalidPageHeaderSpecialOffset(u16),
}