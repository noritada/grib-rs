use std::{
    borrow::Cow,
    io::{Error, Write},
};

pub trait Dump {
    fn dump<W: Write>(
        &self,
        parent: Option<&Cow<str>>,
        start: usize,
        output: &mut W,
    ) -> Result<(), Error>;
}

pub trait DumpField {
    fn dump_field<W: Write>(
        &self,
        name: &str,
        parent: Option<&Cow<str>>,
        doc: &str,
        start: usize,
        output: &mut W,
    ) -> Result<(), Error>;
}

macro_rules! add_impl_for_number_types {
    ($($ty:ty,)*) => ($(
        impl DumpField for $ty {
            fn dump_field<W: Write>(
                &self,
                name: &str,
                parent: Option<&Cow<str>>,
                doc: &str,
                start: usize,
                output: &mut W,
            ) -> Result<(), Error> {
                let size = std::mem::size_of::<Self>();
                if size == 1 {
                    write!(output, "{}", start)?;
                } else {
                    write!(output, "{}-{}", start, start + size - 1)?;
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

add_impl_for_number_types![u8, u16, u32, u64, i8, i16, i32, i64, f32, f64,];

impl<T: Dump> DumpField for T {
    fn dump_field<W: Write>(
        &self,
        name: &str,
        parent: Option<&Cow<str>>,
        _doc: &str,
        start: usize,
        output: &mut W,
    ) -> Result<(), Error> {
        let parent = parent
            .map(|s| Cow::Owned(format!("{}.{}", s, name)))
            .unwrap_or(Cow::Borrowed(name));
        self.dump(Some(&parent), start, output)
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
            _start: usize,
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
        foo.dump(None, 0, &mut buf)?;
        assert_eq!(
            String::from_utf8_lossy(buf.get_ref()),
            "member1: 1\nmember2: 2\n"
        );
        Ok(())
    }
}
