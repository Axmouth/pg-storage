use std::{fs::File, io::BufReader};

use pg_page::{dto::PageLazy, util::ByteEncodeError};

fn main() {
    let mut table_file =
        File::open(r#"C:\Program Files\PostgreSQL\14\data\base\16616\238525"#).unwrap();
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

        for (_, item) in page.iter_tuples() {
            println!("{:#?}", String::from_utf8_lossy(&item.data));
        }
    }
}
