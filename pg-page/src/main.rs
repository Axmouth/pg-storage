use std::{fs::File, io::BufReader};

use transactional_store::storage::Page;

fn main() {
    let mut table_file =
        File::open(r#"C:\Program Files\PostgreSQL\14\data\base\16616\2603"#).unwrap();
    let mut reader = BufReader::new(&mut table_file);

    let mut pages = Vec::new();

    loop {
        let page = Page::from_reader(&mut reader).unwrap();
        pages.push(page);
        if reader.buffer().is_empty() {
            break;
        }
    }

    std::fs::write("pages.txt", format!("{:#?}", pages)).unwrap();

    for page in pages {
        for item in page.items {
            println!("{:#?}", String::from_utf8_lossy(&item.data));
        }
    }
}
