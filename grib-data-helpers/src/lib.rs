use std::io::{Error, Write};

pub use buf::{read_number, FromSlice};

pub trait Dump {
    fn dump<W: Write>(&self, output: &mut W) -> Result<(), Error>;
}

mod buf;

#[cfg(test)]
mod tests {
    use super::*;

    struct Foo {
        member1: u8,
        member2: u16,
    }

    impl Dump for Foo {
        fn dump<W: Write>(&self, output: &mut W) -> Result<(), Error> {
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
        foo.dump(&mut buf)?;
        assert_eq!(
            String::from_utf8_lossy(buf.get_ref()),
            "member1: 1\nmember2: 2\n"
        );
        Ok(())
    }
}
