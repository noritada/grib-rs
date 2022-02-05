use crate::context::SectionInfo;
use crate::error::*;
use std::iter::{Enumerate, Peekable};

pub struct Submessage(
    pub SectionInfo,
    pub SectionInfo,
    pub Option<SectionInfo>,
    pub SectionInfo,
    pub SectionInfo,
    pub SectionInfo,
    pub SectionInfo,
    pub SectionInfo,
    pub SectionInfo,
);

///
/// # Example
/// In every `Submessage`, the `offset` of all Section 8 is always set to 0
/// since this parser only reads required submessages from the beginning of the
/// file.
///
/// ```
/// use grib::context::{SectionBody, SectionInfo};
/// use grib::datatypes::Indicator;
/// use grib::parser::Grib2SubmessageStream;
/// use grib::reader::{Grib2SectionStream, SeekableGrib2Reader};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let f = std::fs::File::open(
///         "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2",
///     )?;
///     let f = std::io::BufReader::new(f);
///     let grib2_reader = SeekableGrib2Reader::new(f);
///
///     let sect_stream = Grib2SectionStream::new(grib2_reader);
///     let mut parser = Grib2SubmessageStream::new(sect_stream);
///
///     let first = parser.next();
///     assert!(first.is_some());
///
///     let first = first.unwrap();
///     assert!(first.is_ok());
///
///     let first = first.unwrap();
///     let (message_index, submessage_index, submessage) = first;
///     assert_eq!(message_index, 0);
///     assert_eq!(submessage_index, 0);
///     assert_eq!(
///         (
///             submessage.0.offset,
///             submessage.1.offset,
///             submessage.2.map(|s| s.offset),
///             submessage.3.offset,
///             submessage.4.offset,
///             submessage.5.offset,
///             submessage.6.offset,
///             submessage.7.offset,
///             submessage.8.offset
///         ),
///         (0, 16, Some(37), 64, 99, 157, 178, 184, 0)
///     );
///
///     let second = parser.next();
///     assert!(second.is_none());
///     Ok(())
/// }
/// ```
pub struct Grib2SubmessageStream<I>
where
    I: Iterator,
{
    iter: Peekable<Enumerate<I>>,
    pos: usize,
    message_count: usize,
    submessage_count: usize,
    sect0: SectionInfo,
    sect1: SectionInfo,
    sect2: Option<SectionInfo>,
    sect3: SectionInfo,
    terminated: bool,
}

impl<I> Grib2SubmessageStream<I>
where
    I: Iterator,
{
    /// # Example
    /// ```
    /// use grib::parser::Grib2SubmessageStream;
    /// use grib::reader::{Grib2SectionStream, SeekableGrib2Reader};
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let f = std::fs::File::open(
    ///         "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2",
    ///     )?;
    ///     let mut f = std::io::BufReader::new(f);
    ///     let grib2_reader = SeekableGrib2Reader::new(f);
    ///     let sect_stream = Grib2SectionStream::new(grib2_reader);
    ///     let _parser = Grib2SubmessageStream::new(sect_stream);
    ///     Ok(())
    /// }
    /// ```
    pub fn new(iter: I) -> Self {
        Self {
            iter: iter.enumerate().peekable(),
            pos: 0,
            message_count: 0,
            submessage_count: 0,
            sect0: Default::default(),
            sect1: Default::default(),
            sect2: Default::default(),
            sect3: Default::default(),
            terminated: false,
        }
    }

    fn clear_submessage_cache(&mut self) {
        self.submessage_count = 0;
        self.sect0 = Default::default();
        self.sect1 = Default::default();
        self.sect2 = Default::default();
        self.sect3 = Default::default();
    }

    fn new_unexpected_end_of_data_err(
        &mut self,
    ) -> Option<Result<(usize, usize, Submessage), ParseError>> {
        self.terminated = true;
        Some(Err(ParseError::UnexpectedEndOfData(self.pos)))
    }

    fn new_invalid_section_order_err(
        &mut self,
    ) -> Option<Result<(usize, usize, Submessage), ParseError>> {
        self.terminated = true;
        Some(Err(ParseError::InvalidSectionOrder(self.pos)))
    }

    fn new_no_grid_definition_err(
        &mut self,
    ) -> Option<Result<(usize, usize, Submessage), ParseError>> {
        self.terminated = true;
        Some(Err(ParseError::NoGridDefinition(self.pos)))
    }
}

impl<I> Iterator for Grib2SubmessageStream<I>
where
    I: Iterator<Item = Result<SectionInfo, ParseError>>,
{
    type Item = Result<(usize, usize, Submessage), ParseError>;

    fn next(&mut self) -> Option<Result<(usize, usize, Submessage), ParseError>> {
        macro_rules! ensure_next_sect_num {
            ($num:expr) => {{
                match self.iter.next() {
                    Some((pos, Ok(s))) => {
                        self.pos = pos;
                        if s.num == $num {
                            s
                        } else {
                            return self.new_invalid_section_order_err();
                        }
                    }
                    Some((_, Err(e))) => {
                        return Some(Err(e));
                    }
                    None => {
                        return self.new_unexpected_end_of_data_err();
                    }
                }
            }};
        }

        if self.terminated {
            return None;
        }

        if self.submessage_count == 0 {
            match self.iter.next()? {
                (pos, Ok(s)) => {
                    self.pos = pos;
                    if s.num == 0 {
                        self.sect0 = s
                    } else {
                        return self.new_invalid_section_order_err();
                    }
                }
                (_, Err(e)) => {
                    return Some(Err(e));
                }
            }

            self.sect1 = ensure_next_sect_num!(1);
        }

        let sect = match self.iter.next() {
            Some((pos, Ok(s))) => {
                self.pos = pos;
                s
            }
            Some((_, Err(e))) => {
                return Some(Err(e));
            }
            _ => {
                return self.new_unexpected_end_of_data_err();
            }
        };

        let sect4 = match sect.num {
            2 => {
                self.sect2 = Some(sect);
                self.sect3 = ensure_next_sect_num!(3);
                ensure_next_sect_num!(4)
            }
            3 => {
                self.sect3 = sect;
                ensure_next_sect_num!(4)
            }
            4 => {
                if self.sect3.num == 0 {
                    return self.new_no_grid_definition_err();
                }
                sect
            }
            _ => {
                return self.new_invalid_section_order_err();
            }
        };

        let sect5 = ensure_next_sect_num!(5);
        let sect6 = ensure_next_sect_num!(6);
        let sect7 = ensure_next_sect_num!(7);

        let next_value = Some(Ok((
            self.message_count,
            self.submessage_count,
            Submessage(
                self.sect0.clone(),
                self.sect1.clone(),
                self.sect2.clone(),
                self.sect3.clone(),
                sect4,
                sect5,
                sect6,
                sect7,
                SectionInfo::new_8(0),
            ),
        )));

        let sect8 = self.iter.peek();
        match sect8 {
            None => {
                return self.new_unexpected_end_of_data_err();
            }
            Some((_, Err(_))) => {
                if let Some((_, Err(e))) = self.iter.next() {
                    return Some(Err(e));
                } else {
                    unreachable!()
                }
            }
            Some((_, Ok(SectionInfo { num: 8, .. }))) => {
                self.iter.next();
                self.message_count += 1;
                self.clear_submessage_cache();
            }
            Some((_, Ok(_))) => {
                self.submessage_count += 1;
            }
        }

        next_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_sect_vec_with_dummy_offset(vec: Vec<u8>) -> Vec<Result<SectionInfo, ParseError>> {
        vec.iter()
            .enumerate()
            .map(|(index, num)| {
                Ok(SectionInfo {
                    num: *num,
                    offset: index,
                    ..Default::default()
                })
            })
            .collect::<Vec<_>>()
    }

    fn digest_submessage_iter_item(
        item: (usize, usize, Submessage),
    ) -> (
        usize,
        usize,
        usize,
        usize,
        Option<usize>,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
    ) {
        let (i1, i2, submessage) = item;
        (
            i1,
            i2,
            submessage.0.offset,
            submessage.1.offset,
            submessage.2.map(|r| r.offset),
            submessage.3.offset,
            submessage.4.offset,
            submessage.5.offset,
            submessage.6.offset,
            submessage.7.offset,
            submessage.8.offset,
        )
    }

    #[test]
    fn submessage_stream_from_1_message() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Ok((0, 0, 0, 1, Some(2), 3, 4, 5, 6, 7, 0))],
        );
    }

    #[test]
    fn submessage_stream_from_multiple_messages() {
        let sects = new_sect_vec_with_dummy_offset(vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8,
        ]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, Some(2), 3, 4, 5, 6, 7, 0)),
                Ok((1, 0, 9, 10, Some(11), 12, 13, 14, 15, 16, 0))
            ],
        );
    }

    #[test]
    fn submessage_stream_empty() {
        let sects = new_sect_vec_with_dummy_offset(vec![]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            Vec::new(),
        );
    }

    #[test]
    fn submessage_stream_from_multiple_messages_without_sect2() {
        let sects =
            new_sect_vec_with_dummy_offset(vec![0, 1, 3, 4, 5, 6, 7, 8, 0, 1, 3, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, None, 2, 3, 4, 5, 6, 0)),
                Ok((1, 0, 8, 9, None, 10, 11, 12, 13, 14, 0))
            ],
        );
    }

    #[test]
    fn submessage_stream_from_1_message_with_multiple_submessages_from_sect2() {
        let sects =
            new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 5, 6, 7, 2, 3, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, Some(2), 3, 4, 5, 6, 7, 0)),
                Ok((0, 1, 0, 1, Some(8), 9, 10, 11, 12, 13, 0))
            ],
        );
    }

    #[test]
    fn submessage_stream_from_1_message_with_multiple_submessages_from_sect2_with_sect2_toggled() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 3, 4, 5, 6, 7, 2, 3, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, None, 2, 3, 4, 5, 6, 0)),
                Ok((0, 1, 0, 1, Some(7), 8, 9, 10, 11, 12, 0))
            ],
        );
    }

    #[test]
    fn submessage_stream_from_1_message_with_multiple_submessages_from_sect3() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 5, 6, 7, 3, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, Some(2), 3, 4, 5, 6, 7, 0)),
                Ok((0, 1, 0, 1, Some(2), 8, 9, 10, 11, 12, 0))
            ],
        );
    }

    #[test]
    fn submessage_stream_from_1_message_with_multiple_submessages_from_sect3_without_sect2() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 3, 4, 5, 6, 7, 3, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, None, 2, 3, 4, 5, 6, 0)),
                Ok((0, 1, 0, 1, None, 7, 8, 9, 10, 11, 0))
            ],
        );
    }
    #[test]
    fn submessage_stream_from_1_message_with_multiple_submessages_from_sect4() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 5, 6, 7, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, Some(2), 3, 4, 5, 6, 7, 0)),
                Ok((0, 1, 0, 1, Some(2), 3, 8, 9, 10, 11, 0))
            ],
        );
    }

    #[test]
    fn submessage_stream_from_1_message_with_multiple_submessages_from_sect4_without_sect2() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 3, 4, 5, 6, 7, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, None, 2, 3, 4, 5, 6, 0)),
                Ok((0, 1, 0, 1, None, 2, 7, 8, 9, 10, 0))
            ],
        );
    }

    #[test]
    fn submessage_stream_from_multiple_messages_and_submessages_with_sect2_toggled() {
        // testing cache of submessage_count and sect2
        let sects = new_sect_vec_with_dummy_offset(vec![
            0, 1, 2, 3, 4, 5, 6, 7, 2, 3, 4, 5, 6, 7, 8, 0, 1, 3, 4, 5, 6, 7, 8,
        ]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, Some(2), 3, 4, 5, 6, 7, 0)),
                Ok((0, 1, 0, 1, Some(8), 9, 10, 11, 12, 13, 0)),
                Ok((1, 0, 15, 16, None, 17, 18, 19, 20, 21, 0))
            ],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect0() {
        let sects = new_sect_vec_with_dummy_offset(vec![0]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(0)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect1() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(1)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect2() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(2)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect3() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(3)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect3_without_sect2() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 3]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(2)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect4() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(4)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect4_without_sect2() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 3, 4]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(3)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect7() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 5, 6, 7]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(7)),],
        );
    }

    #[test]
    fn submessage_stream_from_submessage_without_sect3() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::NoGridDefinition(2)),],
        );
    }

    #[test]
    fn submessage_stream_wrongly_ordered_from_the_beginning() {
        let sects = new_sect_vec_with_dummy_offset(vec![1, 0, 2, 3, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::InvalidSectionOrder(0)),],
        );
    }

    #[test]
    fn submessage_stream_wrongly_ordered_after_sect0() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 2, 1, 3, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::InvalidSectionOrder(1)),],
        );
    }

    #[test]
    fn submessage_stream_wrongly_ordered_after_sect1() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 5, 2, 3, 4, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::InvalidSectionOrder(2)),],
        );
    }

    #[test]
    fn submessage_stream_wrongly_ordered_after_sect2() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 4, 3, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::InvalidSectionOrder(3)),],
        );
    }

    #[test]
    fn submessage_stream_wrongly_ordered_after_sect3() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 5, 4, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::InvalidSectionOrder(4)),],
        );
    }

    #[test]
    fn submessage_stream_wrongly_ordered_after_sect3_without_sect2() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 3, 5, 4, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::InvalidSectionOrder(3)),],
        );
    }

    #[test]
    fn submessage_stream_wrongly_ordered_after_sect4() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 6, 5, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::InvalidSectionOrder(5)),],
        );
    }

    #[test]
    fn submessage_stream_wrongly_ordered_after_sect4_without_sect2() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 3, 4, 6, 5, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::InvalidSectionOrder(4)),],
        );
    }

    #[test]
    fn submessage_stream_wrongly_ordered_after_sect5() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 5, 7, 6, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::InvalidSectionOrder(6)),],
        );
    }

    #[test]
    fn submessage_stream_wrongly_ordered_after_sect6() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 5, 6, 8, 7]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::InvalidSectionOrder(7)),],
        );
    }

    #[test]
    fn submessage_stream_with_2nd_submessage_wrongly_ordered_from_the_beginning() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 5, 6, 7, 0, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, Some(2), 3, 4, 5, 6, 7, 0)),
                Err(ParseError::InvalidSectionOrder(8)),
            ],
        );
    }

    #[test]
    fn submessage_stream_with_2nd_submessage_wrongly_ordered_after_sect2() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 5, 6, 7, 2, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, Some(2), 3, 4, 5, 6, 7, 0)),
                Err(ParseError::InvalidSectionOrder(9)),
            ],
        );
    }

    #[test]
    fn submessage_stream_with_2nd_submessage_wrongly_ordered_after_sect4_without_sect2_and_sect3() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 5, 6, 7, 4, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, Some(2), 3, 4, 5, 6, 7, 0)),
                Err(ParseError::InvalidSectionOrder(9)),
            ],
        );
    }

    fn new_sect_vec_with_dummy_offset_and_errors(
        vec: Vec<Result<u8, ParseError>>,
    ) -> Vec<Result<SectionInfo, ParseError>> {
        vec.iter()
            .enumerate()
            .map(|(index, result)| {
                result.clone().map(|num| SectionInfo {
                    num: num,
                    offset: index,
                    ..Default::default()
                })
            })
            .collect::<Vec<_>>()
    }

    #[test]
    fn submessage_stream_with_sect_streem_error_at_sect0_in_1st_submessage() {
        let sects = new_sect_vec_with_dummy_offset_and_errors(vec![Err(ParseError::ReadError(
            "failed to fill whole buffer".to_owned(),
        ))]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned()
            )),],
        );
    }

    #[test]
    fn submessage_stream_with_sect_streem_error_at_sect1_in_1st_submessage() {
        let sects = new_sect_vec_with_dummy_offset_and_errors(vec![
            Ok(0),
            Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned(),
            )),
        ]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned()
            )),],
        );
    }

    #[test]
    fn submessage_stream_with_sect_streem_error_at_sect2_in_1st_submessage() {
        let sects = new_sect_vec_with_dummy_offset_and_errors(vec![
            Ok(0),
            Ok(1),
            Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned(),
            )),
        ]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned()
            )),],
        );
    }

    #[test]
    fn submessage_stream_with_sect_streem_error_at_sect3_in_1st_submessage() {
        let sects = new_sect_vec_with_dummy_offset_and_errors(vec![
            Ok(0),
            Ok(1),
            Ok(2),
            Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned(),
            )),
        ]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned()
            )),],
        );
    }

    #[test]
    fn submessage_stream_with_sect_streem_error_at_sect4_in_1st_submessage() {
        let sects = new_sect_vec_with_dummy_offset_and_errors(vec![
            Ok(0),
            Ok(1),
            Ok(2),
            Ok(3),
            Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned(),
            )),
        ]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned()
            )),],
        );
    }

    #[test]
    fn submessage_stream_with_sect_streem_error_at_sect5_in_1st_submessage() {
        let sects = new_sect_vec_with_dummy_offset_and_errors(vec![
            Ok(0),
            Ok(1),
            Ok(2),
            Ok(3),
            Ok(4),
            Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned(),
            )),
        ]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned()
            )),],
        );
    }

    #[test]
    fn submessage_stream_with_sect_streem_error_at_sect6_in_1st_submessage() {
        let sects = new_sect_vec_with_dummy_offset_and_errors(vec![
            Ok(0),
            Ok(1),
            Ok(2),
            Ok(3),
            Ok(4),
            Ok(5),
            Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned(),
            )),
        ]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned()
            )),],
        );
    }

    #[test]
    fn submessage_stream_with_sect_streem_error_at_sect7_in_1st_submessage() {
        let sects = new_sect_vec_with_dummy_offset_and_errors(vec![
            Ok(0),
            Ok(1),
            Ok(2),
            Ok(3),
            Ok(4),
            Ok(5),
            Ok(6),
            Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned(),
            )),
        ]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned()
            )),],
        );
    }

    #[test]
    fn submessage_stream_with_sect_streem_error_at_sect8_in_1st_submessage() {
        let sects = new_sect_vec_with_dummy_offset_and_errors(vec![
            Ok(0),
            Ok(1),
            Ok(2),
            Ok(3),
            Ok(4),
            Ok(5),
            Ok(6),
            Ok(7),
            Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned(),
            )),
        ]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned()
            )),],
        );
    }
}
