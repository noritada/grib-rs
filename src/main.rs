use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

mod parser;
use crate::parser::GribReader;

fn main() {
    if env::args().len() != 2 {
        return;
    }

    let args: Vec<String> = env::args().collect();
    let path = Path::new(&args[1]);
    let display = path.display();

    let f = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why.to_string()),
        Ok(f) => f,
    };
    let f = BufReader::new(f);

    let grib2 = parser::Grib2FileReader::new(f).unwrap();
    println!("GRIB2: {:#?}", grib2.list_submessages());
}
