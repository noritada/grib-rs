use std::fs::File;
use std::io::Read;

pub fn read_sect0(f: &mut File) -> u64 {
    let mut buf = [0; 16];
    match f.read(&mut buf[..]) {
        Err(_) => panic!("read failed"),
        Ok(nbytes) => {
            if nbytes < buf.len() {
                panic!("not a GRIB2 file");
            }
        },
    };

    if buf[0] != b'G' || buf[1] != b'R' || buf[2] != b'I' || buf[3] != b'B' {
        panic!("not a GRIB file");
    }
    if buf[7] != 2 {
        panic!("not GRIB version 2");
    }

    let mut fsize = 0u64;
    for i in 8..16 {
        //fsize |= (buf[i] as u64) << ((15 - i) * 0b1000);
        fsize <<= 0b1000;
        fsize |= buf[i] as u64;
    }

    fsize
}
