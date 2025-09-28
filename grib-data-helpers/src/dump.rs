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
                let start = *pos;
                *pos += size;
                if size == 1 {
                    write!(output, "{}", start)?;
                } else {
                    write!(output, "{}-{}", start, *pos - 1)?;
                }
                if let Some(parent) = parent {
                    write!(output, ": {}.", parent)?;
                } else {
                    write!(output, ": ")?;
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
}
