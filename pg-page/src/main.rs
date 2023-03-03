use std::{fs::File, io::BufReader};

use pg_page::{dto::PageLazy, util::ByteEncodeError};

fn main() {
    let mut table_file = std::env::args().nth(1).map(File::open).unwrap().unwrap();
    let mut reader = BufReader::new(&mut table_file);

    let mut pages = Vec::new();

    loop {
        let page = match PageLazy::from_reader(&mut reader) {
            Ok(page) => page,
            Err(err) => {
                if matches!(err, ByteEncodeError::IoError(_)) {
                    break;
                } else {
                    panic!("Error: {:#?}", err);
                }
            }
        };
        pages.push(page);
    }

    std::fs::write("pages.txt", format!("{:#?}", pages)).unwrap();
    eprintln!("Pages: {}", pages.len());
    for page in pages {
        eprintln!("Page data len: {:#?}", page.data.len());

        for res in page.iter_tuples() {
            let (_, item) = res.unwrap();
            println!("{:#?}", String::from_utf8_lossy(&item.data));
        }
    }
}
