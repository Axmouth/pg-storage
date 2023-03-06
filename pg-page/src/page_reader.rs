use std::{io::{BufReader, Read, Seek}};

use crate::{dto::{PageHeaderData, PageLazy}, util::{ByteEncodeResult, ByteEncoded, read_exact_with_eof}};

// TODO: handle locked pages

pub struct PageReader<R: Read + Seek> {
    reader: BufReader<R>,
    cursor: u64,
    ended: bool,
}

impl<R: Read + Seek> PageReader<R> {
    pub fn new(reader: R) -> Self {
        let reader = BufReader::new(reader);
        PageReader {
            reader,
            cursor: 0,
            ended: false,
        }
    }

    pub fn read_page_at(&mut self, offset: u64) -> ByteEncodeResult<Option<PageLazy>> {
        self.reader.seek(std::io::SeekFrom::Start(offset))?;
        self.read_next_page()
    }

    pub fn cursor(&self) -> u64 {
        self.cursor
    }

    pub fn seek(&mut self, offset: u64) -> ByteEncodeResult<()> {
        self.reader.seek(std::io::SeekFrom::Start(offset))?;
        self.cursor = offset;
        Ok(())
    }

    pub fn seek_relative(&mut self, offset: i64) -> ByteEncodeResult<()> {
        self.reader.seek_relative(offset)?;
        self.cursor = (self.cursor as i64 + offset) as u64;
        Ok(())
    }

    pub fn read_next_page(&mut self) -> ByteEncodeResult<Option<PageLazy>> {
        self.read_next_page_filtered(|_| true)
    }

    pub fn read_next_page_filtered(&mut self, filter: impl Fn(&PageHeaderData) -> bool) -> ByteEncodeResult<Option<PageLazy>> {
        if self.ended {
            return Ok(None);
        }

        let header_size = PageHeaderData::byte_size() as usize;
        let mut bytes = vec![0; header_size];
        if read_exact_with_eof(&mut bytes, &mut self.reader)?.is_none() {
            self.ended = true;
            return Ok(None);
        }

        let header_data = PageHeaderData::decode(&bytes)?;
        let page_size = header_data.page_size();

        if !filter(&header_data) {
            self.reader.seek_relative((page_size - header_size) as i64)?;
            self.cursor += page_size as u64;
            return self.read_next_page_filtered(filter);
        }

        let mut data = vec![0; page_size - header_size];
        if read_exact_with_eof(&mut data, &mut self.reader)?.is_none() {
            self.ended = true;
            return Ok(None);
        }
        self.cursor += page_size as u64;

        Ok(Some(PageLazy {
            header_data,
            data,
        }))
    }
}

impl<R: Read + Seek> IntoIterator for PageReader<R> {
    type Item = ByteEncodeResult<PageLazy>;
    type IntoIter = PageReaderIter<R>;

    fn into_iter(self) -> Self::IntoIter {
        PageReaderIter::new(self)
    }
}

pub struct PageReaderIter<R: Read + Seek> {
    reader: PageReader<R>,
    filter: Box<dyn Fn(&PageHeaderData) -> bool>,
    prerun: Box<dyn Fn(u64)>,
}

impl<R: Read + Seek> PageReaderIter<R> {
    pub fn new(reader: PageReader<R>) -> Self {
        PageReaderIter {
            reader,
            filter: Box::new(|_| true),
            prerun: Box::new(|_| {}),
        }
    }
}

impl<R: Read + Seek> Iterator for PageReaderIter<R> {
    type Item = ByteEncodeResult<PageLazy>;

    fn next(&mut self) -> Option<Self::Item> {
        (self.prerun)(self.reader.cursor);
        self.reader.read_next_page_filtered(&self.filter).transpose()
    }
}

impl<R: Read + Seek> PageReaderIter<R> {
    pub fn with_prerun(self, prerun: impl Fn(u64) + 'static) -> Self {
        Self { prerun: Box::new(prerun), ..self }
    }

    pub fn with_filter(self, filter: impl Fn(&PageHeaderData) -> bool + 'static) -> Self {
        Self { filter: Box::new(filter), ..self }
    }
}