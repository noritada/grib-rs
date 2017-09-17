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
        Err(why) => panic!("couldn't open {}: {}",
                           display, Error::description(&why)),
        Ok(f) => f,
    };

    let size = parser::read_sect0(&mut f);

    println!("GRIB2 with size {}", size);
}
