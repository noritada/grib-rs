use std::env;
use std::error::Error;
use std::fs::File;
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
        Err(why) => panic!("couldn't open {}: {}", display, Error::description(&why)),
        Ok(f) => f,
    };

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

        let (sect_num, sect_size) = match parser::unpack_sect_header(&mut f) {
            Err(why) => panic!(why),
            Ok(tuple) => tuple,
        };
        println!("reading section {} with size {}...", sect_num, sect_size);

        let _ = match sect_num {
            1 => parser::unpack_sect1_body(&mut f, sect_size),
            2 => parser::unpack_sect2_body(&mut f, sect_size),
            // actually, section 2 is optional
            3 => parser::unpack_sect3_body(&mut f, sect_size),
            4 => parser::unpack_sect4_body(&mut f, sect_size),
            5 => parser::unpack_sect5_body(&mut f, sect_size),
            6 => parser::unpack_sect6_body(&mut f, sect_size),
            7 => parser::unpack_sect7_body(&mut f, sect_size),
            _ => panic!("unknown section number: {}", sect_num),
        };
        rest_size -= sect_size;
    }

    println!("GRIB2 with size {}", whole_size);
}
