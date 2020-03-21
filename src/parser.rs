use std::fs::File;
use std::io;
use std::io::Read;
use std::result::Result;

pub fn unpack_sect0(f: &mut File) -> Result<usize, String> {
    let mut buf = [0; 16];
    try!(f.read_exact(&mut buf[..]).map_err(clarify_err));

    if buf[0] != b'G' || buf[1] != b'R' || buf[2] != b'I' || buf[3] != b'B' {
        return Err("not a GRIB file".to_string());
    }
    if buf[7] != 2 {
        return Err("not GRIB version 2".to_string());
    }

    let mut fsize = 0u64;
    for i in 8..16 {
        //fsize |= (buf[i] as u64) << ((15 - i) * 0b1000);
        fsize <<= 0b1000;
        fsize |= buf[i] as u64;
    }

    Ok(fsize as usize)
}

pub fn unpack_sect1_body(f: &mut File, sect_size: usize) -> Result<(), String> {
    let mut buf = [0; 16]; // octet 6-21
    try!(f.read_exact(&mut buf[..]).map_err(clarify_err));

    let master_table_version = buf[4];
    println!(
        "GRIB Master Tables Version Number: {}",
        master_table_version
    );

    let local_table_version = buf[5];
    println!("GRIB Local Tables Version Number: {}", local_table_version);

    // octet 13-19
    let year = concat_bytes_as_u16(&mut buf, 7, 2);
    println!(
        "reference time of data: {:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, buf[9], buf[10], buf[11], buf[12], buf[13]
    );

    let len_extra = sect_size - 5 - buf.len(); // 5 is header size
    if len_extra > 0 {
        // placeholder
        println!("reading extra {} bytes", len_extra);
        let mut buf = vec![0; len_extra];
        try!(f.read_exact(&mut buf[..]).map_err(clarify_err));
    }

    Ok(())
}

pub fn unpack_sect2_body(f: &mut File, sect_size: usize) -> Result<(), String> {
    let len_extra = sect_size - 5; // 5 is header size
    if len_extra > 0 {
        // placeholder
        println!("reading extra {} bytes", len_extra);
        let mut buf = vec![0; len_extra];
        try!(f.read_exact(&mut buf[..]).map_err(clarify_err));
    }

    Ok(())
}

pub fn unpack_sect3_body(f: &mut File, sect_size: usize) -> Result<(), String> {
    let mut buf = [0; 9]; // octet 6-14
    try!(f.read_exact(&mut buf[..]).map_err(clarify_err));

    // octet 7-10
    let npoints = concat_bytes_as_u32(&mut buf, 1, 4);
    println!("number of data points: {}", npoints);

    // octet 13-14
    let grid_tmpl_code = concat_bytes_as_u16(&mut buf, 7, 2);
    println!("grid definition template number: {}", grid_tmpl_code);

    let len_extra = sect_size - 5 - buf.len(); // 5 is header size
    if len_extra > 0 {
        // placeholder
        println!("reading extra {} bytes", len_extra);
        let mut buf = vec![0; len_extra];
        try!(f.read_exact(&mut buf[..]).map_err(clarify_err));
    }

    Ok(())
}

pub fn unpack_sect4_body(f: &mut File, sect_size: usize) -> Result<(), String> {
    let mut buf = [0; 4]; // octet 6-9
    try!(f.read_exact(&mut buf[..]).map_err(clarify_err));

    // octet 6-7
    let ncoordinates = concat_bytes_as_u16(&mut buf, 0, 2);
    println!(
        "number of coordinate values after template: {}",
        ncoordinates
    );

    // octet 8-9
    let prod_tmpl_code = concat_bytes_as_u16(&mut buf, 2, 2);
    println!("product definition template number: {}", prod_tmpl_code);

    let len_extra = sect_size - 5 - buf.len(); // 5 is header size
    if len_extra > 0 {
        // placeholder
        println!("reading extra {} bytes", len_extra);
        let mut buf = vec![0; len_extra];
        try!(f.read_exact(&mut buf[..]).map_err(clarify_err));
    }

    Ok(())
}

pub fn unpack_sect5_body(f: &mut File, sect_size: usize) -> Result<(), String> {
    let mut buf = [0; 6]; // octet 6-11
    try!(f.read_exact(&mut buf[..]).map_err(clarify_err));

    // octet 6-9
    let npoints7 = concat_bytes_as_u32(&mut buf, 0, 4);
    println!(
        "number of data points where one or more values are specified in Section 7: {}",
        npoints7
    );

    // octet 10-11
    let represent_tmpl_code = concat_bytes_as_u16(&mut buf, 4, 2);
    println!(
        "data representation template number: {}",
        represent_tmpl_code
    );

    let len_extra = sect_size - 5 - buf.len(); // 5 is header size
    if len_extra > 0 {
        // placeholder
        println!("reading extra {} bytes", len_extra);
        let mut buf = vec![0; len_extra];
        try!(f.read_exact(&mut buf[..]).map_err(clarify_err));
    }

    Ok(())
}

pub fn unpack_sect6_body(f: &mut File, sect_size: usize) -> Result<(), String> {
    let mut buf = [0; 1]; // octet 6
    try!(f.read_exact(&mut buf[..]).map_err(clarify_err));

    let bitmap_indicator = buf[0];
    println!("bit-map indicator: {}", bitmap_indicator);

    let len_extra = sect_size - 5 - buf.len(); // 5 is header size
    if len_extra > 0 {
        // placeholder
        println!("reading extra {} bytes", len_extra);
        let mut buf = vec![0; len_extra];
        try!(f.read_exact(&mut buf[..]).map_err(clarify_err));
    }

    Ok(())
}

pub fn unpack_sect7_body(f: &mut File, sect_size: usize) -> Result<(), String> {
    let len_extra = sect_size - 5; // 5 is header size
    if len_extra > 0 {
        // placeholder
        println!("reading extra {} bytes", len_extra);
        let mut buf = vec![0; len_extra]; // octet 6-21
        try!(f.read_exact(&mut buf[..]).map_err(clarify_err));
    }

    Ok(())
}

pub fn unpack_sect8(f: &mut File) -> Result<(), String> {
    let mut buf = [0; 4];
    try!(f.read_exact(&mut buf[..]).map_err(clarify_err));

    if buf[0] != b'7' || buf[1] != b'7' || buf[2] != b'7' || buf[3] != b'7' {
        return Err("Section 8 (End Section) mismatch".to_string());
    }

    Ok(())
}

/// Reads a common header for sections 1-7 and returns the section
/// number and size.
pub fn unpack_sect_header(f: &mut File) -> Result<(u8, usize), String> {
    let mut buf = [0; 5];
    try!(f.read_exact(&mut buf[..]).map_err(clarify_err));

    let sect_size = concat_bytes_as_u32(&mut buf, 0, 4) as usize;
    let sect_num = buf[4];
    Ok((sect_num, sect_size))
}

// To convert
// io::Result<usize> (= Result<usize, io::Error) -> Result<usize, String>
fn clarify_err(e: io::Error) -> String {
    format!("read error: {}", e.to_string())
}

// It seems possible to write a numeric generic function with num crate...

fn concat_bytes_as_u32(buf: &mut [u8], start: usize, len: usize) -> u32 {
    let mut ret = 0u32;
    for i in start..(start + len) {
        ret <<= 0b1000;
        ret |= buf[i] as u32;
    }

    ret
}

fn concat_bytes_as_u16(buf: &mut [u8], start: usize, len: usize) -> u16 {
    let mut ret = 0u16;
    for i in start..(start + len) {
        ret <<= 0b1000;
        ret |= buf[i] as u16;
    }

    ret
}
