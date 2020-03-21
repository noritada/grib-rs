use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

mod parser;

fn main() {
    if env::args().len() != 2 {
        return;
    }

    let args: Vec<String> = env::args().collect();
    let path = Path::new(&args[1]);
    let display = path.display();

    let mut f = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why.to_string()),
        Ok(f) => f,
    };
    let mut f = BufReader::new(&mut f);

    let whole_size = match parser::unpack_sect0(&mut f) {
        Err(why) => panic!(why),
        Ok(size) => size,
    };
    let mut rest_size: usize = whole_size - 16; // 16 is Section 0 size

    loop {
        if rest_size == 4 {
            match parser::unpack_sect8(&mut f) {
                Err(why) => panic!(why),
                Ok(_) => {
                    break;
                }
            };
        }

        let sect_info = parser::unpack_sect_header(&mut f).unwrap();
        let sect_body = sect_info.read_body(&mut f).unwrap();
        println!("{:#?},\n{:#?}", sect_info, sect_body);
        rest_size -= sect_info.size;
    }

    println!("GRIB2 with size {}", whole_size);
}
