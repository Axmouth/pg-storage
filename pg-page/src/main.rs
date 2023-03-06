use pg_page::dto::HeapTupleHeaderData;
use pg_page::page_reader::PageReader;
use pg_page::{
    dto::{Page, PageLazy},
    util::{ByteEncodeError, ByteEncodeResult},
};
use std::io::{Read, Write, Seek};
use std::time::Instant;
use std::{fs::File, io::BufReader};

fn main() {
    let table_file_name = std::env::args().nth(1).unwrap();

    let (pages, elapsed) = bench_func(|| {
        let mut table_file = File::open(&table_file_name).unwrap();
        let mut reader = BufReader::new(&mut table_file);
        read_pages_lazy(&mut reader).unwrap()
    });
    eprintln!(
        "Read pages lazy: {}ns or {}ms",
        elapsed.as_nanos(),
        elapsed.as_millis()
    );

    let (pages, elapsed) = bench_func(|| {
        let mut table_file = File::open(&table_file_name).unwrap();
        let mut reader = BufReader::new(&mut table_file);
        read_pages(&mut reader).unwrap()
    });
    eprintln!(
        "Read pages: {}ns or {}ms",
        elapsed.as_nanos(),
        elapsed.as_millis()
    );

    std::fs::write("pages.txt", format!("{:#?}", pages)).unwrap();
    // eprintln!("Pages: {}", pages.len());
    // for page in pages {
    //     eprintln!("Page data len: {:#?}", page.items.len());

    //     for item in page.items {
    //         // let (_, item) = res.unwrap();
    //         println!("{:#?}", String::from_utf8_lossy(&item.data));
    //     }
    // }
}

fn read_pages_lazy(reader: &mut (impl std::io::Read + Seek)) -> ByteEncodeResult<Vec<PageLazy>> {
    let mut pages = Vec::new();
    for page in PageReader::new(reader).into_iter() {
        let page = page?;
        let tuples = page.iter_tuples().map(Result::unwrap).collect::<Vec<_>>();
        pages.push(page);
    }

    Ok(pages)
}

fn read_pages(reader: &mut impl std::io::Read) -> ByteEncodeResult<Vec<Page>> {
    let mut pages = Vec::new();

    loop {
        let page = match Page::from_reader(reader) {
            Ok(page) => page,
            Err(err) => {
                if matches!(err, ByteEncodeError::IoError(_)) {
                    break;
                } else {
                    return Err(err);
                }
            }
        };
        pages.push(page);
    }

    Ok(pages)
}

fn bench_func<T>(func: impl Fn() -> T) -> (T, std::time::Duration) {
    let now = Instant::now();
    let mut res = func();
    for _ in 0..100 {
        res = func();
    }
    let elapsed = now.elapsed();
    (res, elapsed / 100)
}
