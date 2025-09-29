use std::{
    borrow::Cow,
    io::{Error, Write},
};

pub trait Dump {
    fn dump<W: Write>(
        &self,
        parent: Option<&Cow<str>>,
        pos: &mut usize,
        output: &mut W,
    ) -> Result<(), Error>;
}

pub trait DumpField: OctetSize {
    fn dump_field<W: Write>(
        &self,
        name: &str,
        parent: Option<&Cow<str>>,
        doc: &str,
        pos: &mut usize,
        output: &mut W,
    ) -> Result<(), Error>;
}

macro_rules! add_impl_of_dump_field_for_number_types {
    ($($ty:ty,)*) => ($(
        impl DumpField for $ty {
            fn dump_field<W: Write>(
                &self,
                name: &str,
                parent: Option<&Cow<str>>,
                doc: &str,
                pos: &mut usize,
                output: &mut W,
            ) -> Result<(), Error> {
                let size = self.octet_size();
                write_position_column(output, pos, size)?;
                if let Some(parent) = parent {
                    write!(output, "{}.", parent)?;
                }
                writeln!(output, "{} = {:?}{}",
                    name,
                    self,
                    doc,
                )
            }
        }
    )*);
}

add_impl_of_dump_field_for_number_types![
    u8,
    u16,
    u32,
    u64,
    i8,
    i16,
    i32,
    i64,
    f32,
    f64,
    Vec<u8>,
    Vec<u16>,
    Vec<u32>,
    Vec<u64>,
    Vec<i8>,
    Vec<i16>,
    Vec<i32>,
    Vec<i64>,
    Vec<f32>,
    Vec<f64>,
];

impl<T: Dump> DumpField for T {
    fn dump_field<W: Write>(
        &self,
        name: &str,
        parent: Option<&Cow<str>>,
        _doc: &str,
        pos: &mut usize,
        output: &mut W,
    ) -> Result<(), Error> {
        let parent = parent
            .map(|s| Cow::Owned(format!("{}.{}", s, name)))
            .unwrap_or(Cow::Borrowed(name));
        self.dump(Some(&parent), pos, output)
    }
}

pub trait OctetSize {
    fn octet_size(&self) -> usize;
}

macro_rules! add_impl_of_octet_size_for_number_types {
    ($($ty:ty,)*) => ($(
        impl OctetSize for $ty {
            fn octet_size(&self) -> usize {
                std::mem::size_of::<Self>()
            }
        }
    )*);
}

add_impl_of_octet_size_for_number_types![u8, u16, u32, u64, i8, i16, i32, i64, f32, f64,];

macro_rules! add_impl_of_octet_size_for_number_vectors {
    ($($ty:ty,)*) => ($(
        impl OctetSize for Vec<$ty> {
            fn octet_size(&self) -> usize {
                std::mem::size_of::<$ty>() * self.len()
            }
        }
    )*);
}

add_impl_of_octet_size_for_number_vectors![u8, u16, u32, u64, i8, i16, i32, i64, f32, f64,];

impl<T: Dump> OctetSize for T {
    fn octet_size(&self) -> usize {
        0
    }
}

fn write_position_column<W: Write>(
    output: &mut W,
    pos: &mut usize,
    size: usize,
) -> Result<(), Error> {
    let str_len = |i: usize| -> usize {
        let mut i = i;
        let mut len = 1;
        while i >= 10 {
            i /= 10;
            len += 1;
        }
        len
    };

    const COLUMN_WIDTH: usize = 10;
    let mut pad_width = COLUMN_WIDTH;
    let pad = |size| " ".repeat(size);

    let start = *pos;
    *pos += size;
    pad_width -= str_len(start);
    if size == 1 {
        write!(output, "{}{}", start, pad(pad_width))?;
    } else {
        let end = *pos - 1;
        pad_width -= str_len(end) + 1;
        write!(output, "{}-{}{}", start, end, pad(pad_width))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Foo {
        member1: u8,
        member2: u16,
    }

    impl Dump for Foo {
        fn dump<W: Write>(
            &self,
            _parent: Option<&Cow<str>>,
            _pos: &mut usize,
            output: &mut W,
        ) -> Result<(), Error> {
            writeln!(output, "member1: {}", self.member1)?;
            writeln!(output, "member2: {}", self.member2)
        }
    }

    #[test]
    fn dumping_to_stream() -> Result<(), Box<dyn std::error::Error>> {
        let foo = Foo {
            member1: 1,
            member2: 2,
        };
        let mut buf = std::io::Cursor::new(Vec::with_capacity(1024));
        let mut pos = 0;
        foo.dump(None, &mut pos, &mut buf)?;
        assert_eq!(
            String::from_utf8_lossy(buf.get_ref()),
            "member1: 1\nmember2: 2\n"
        );
        Ok(())
    }

    fn write_position_column_with_newline<W: Write>(
        output: &mut W,
        pos: &mut usize,
        size: usize,
    ) -> Result<(), Error> {
        write_position_column(output, pos, size)?;
        write!(output, "$\n")
    }

    #[test]
    fn writing_position_columns() -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = std::io::Cursor::new(Vec::with_capacity(1024));
        let mut pos = 1;
        write_position_column_with_newline(&mut buf, &mut pos, 1)?;
        write_position_column_with_newline(&mut buf, &mut pos, 8)?;
        pos = 3;
        write_position_column_with_newline(&mut buf, &mut pos, 8)?;
        write_position_column_with_newline(&mut buf, &mut pos, 1)?;
        write_position_column_with_newline(&mut buf, &mut pos, 8)?;
        pos = 92;
        write_position_column_with_newline(&mut buf, &mut pos, 8)?;
        pos = 93;
        write_position_column_with_newline(&mut buf, &mut pos, 8)?;
        pos = 100;
        write_position_column_with_newline(&mut buf, &mut pos, 1)?;
        write_position_column_with_newline(&mut buf, &mut pos, 8)?;

        let expected = "\
1         $
2-9       $
3-10      $
11        $
12-19     $
92-99     $
93-100    $
100       $
101-108   $
";
        assert_eq!(String::from_utf8_lossy(buf.get_ref()), expected);
        Ok(())
    }
}
