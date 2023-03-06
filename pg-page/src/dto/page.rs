use crate::util::{ByteEncodeResult, ByteEncoded, GetByteSliceExt};

use super::{
    *
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Page {
    pub header_data: PageHeaderData,
    pub item_id_data: Vec<ItemIdData>,
    // Free space goes here
    pub items: Vec<HeapTupleHeaderData>,
    pub special: Option<()>,
}

impl Page {
    pub fn from_reader(reader: &mut impl std::io::Read) -> ByteEncodeResult<Self> {
        let header_size = PageHeaderData::byte_size() as usize;
        let mut bytes = vec![0; header_size];
        reader.read_exact(&mut bytes)?;
        let header_data = PageHeaderData::decode(&bytes)?;
        let page_size = header_data.page_size();
        let mut bytes = vec![0; page_size - header_size];
        reader.read_exact(&mut bytes)?;
        let item_id_data_bytes =
            bytes.get_byte_slice(0, header_data.pd_lower as usize - header_size)?;
        let item_id_data: Vec<ItemIdData> = Vec::decode(item_id_data_bytes)?;
        let mut items = Vec::with_capacity(item_id_data.len());
        for item_id in &item_id_data {
            if !item_id.is_normal() {
                continue;
            }

            let item_bytes = bytes.get_byte_slice(
                item_id.lp_off() as usize - header_size,
                item_id.lp_off() as usize - header_size + item_id.lp_len() as usize,
            )?;
            items.push(HeapTupleHeaderData::decode(item_bytes)?);
        }
        Ok(Page {
            header_data,
            item_id_data,
            items,
            special: None,
        })
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

            self.item_id_data.push(item_id);

            self.header_data.pd_lower += ItemPointerData::byte_size();
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