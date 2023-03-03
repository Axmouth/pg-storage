use crate::{util::{ByteEncodeResult, ByteEncoded, GetByteSliceExt}, Error};

use super::{
    *
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct PageLazy {
    pub header_data: PageHeaderData,
    pub data: Vec<u8>,
}

impl PageLazy {
    pub fn from_reader(reader: &mut impl std::io::Read) -> ByteEncodeResult<Self> {
        let header_size = PageHeaderData::byte_size() as usize;
        let mut bytes = vec![0; header_size];
        reader.read_exact(&mut bytes)?;
        let header_data = PageHeaderData::decode(&bytes)?;
        let page_size = header_data.page_size();
        
        let mut data = vec![0; page_size - header_size];
        reader.read_exact(&mut data)?;

        Ok(PageLazy {
            header_data,
            data,
        })
    }

    pub fn iter_tuples(&self) -> PageLazyTuplesIter {
        PageLazyTuplesIter {
            page: self,
            cursor: 0,
        }
    }

    pub fn reserve_tuple(&mut self, data_size: u16) -> Option<ItemIdData> {
        let tuple_size = HeapTupleHeaderData::byte_size() + data_size;
        // TODO: add logic for alignment and null bitmap
        if self.header_data.pd_upper - self.header_data.pd_lower < tuple_size + ItemPointerData::byte_size() {
            None
        } else {
            let mut item_id = ItemIdData::default();
            item_id.set_lp_off(self.header_data.pd_upper - tuple_size);
            item_id.set_lp_len(tuple_size);
            let item_id_bytes = item_id.encode();

            let new_pd_lower = self.header_data.pd_lower + ItemPointerData::byte_size();

            // TODO: Handle error differently?
            self.data.get_byte_slice_mut(self.header_data.pd_lower as usize, new_pd_lower as usize).ok()?.copy_from_slice(&item_id_bytes);

            self.header_data.pd_lower = new_pd_lower;
            self.header_data.pd_upper -= tuple_size;
            
            assert!(self.header_data.pd_upper >= self.header_data.pd_lower);

            Some(item_id)
        }
    }

    pub fn vacuum(&mut self) {
        // let mut new_item_id_data = Vec::new();
        // let mut new_items = Vec::new();
        // for (item_id, item) in self.item_id_data.iter().zip(self.items.iter()) {
        //     if !item.is_dead() {
        //         new_item_id_data.push(*item_id);
        //         new_items.push(*item);
        //     }
        // }
        // self.item_id_data = new_item_id_data;
        // self.items = new_items;
    }
}


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct PageLazyTuplesIter<'a> {
    page: &'a PageLazy,
    cursor: u16,
}

impl Iterator for PageLazyTuplesIter<'_> {
    type Item = Result<(ItemIdData, HeapTupleHeaderData), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= (self.page.header_data.pd_lower - PageHeaderData::byte_size()) {
            None
        } else {
            // TODO: Handle errors(return Result?)
            let item_id = match self.page.data.get_byte_slice(self.cursor as usize, (self.cursor + ItemIdData::byte_size()) as usize) {
                Ok(item_id) => item_id,
                Err(err) => return Some(Err(err.into())),
            };
            let item_id = match ItemIdData::decode(item_id) {
                Ok(item_id) => item_id,
                Err(err) => return Some(Err(err.into())),
            };
            let real_offset = item_id.lp_off() - PageHeaderData::byte_size();
            let item = match self.page.data.get_byte_slice(real_offset as usize, (real_offset + item_id.lp_len()) as usize) {
                Ok(item) => item,
                Err(err) => return Some(Err(err.into())),
            };
            let item = match HeapTupleHeaderData::decode(item) {
                Ok(item) => item,
                Err(err) => return Some(Err(err.into())),
            };
            self.cursor += ItemIdData::byte_size();
            Some(Ok((item_id, item)))
        }
    }
}