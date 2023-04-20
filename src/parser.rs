use std::iter::{Enumerate, Peekable};

use crate::{Grib2SubmessageIndex, SectionInfo, *};

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
/// In all but the last `Submessage` in a message, the `offset` of Section 8 is
/// set to 0 since this parser only reads required submessages from the
/// beginning of the file.
///
/// ```
/// use grib::{
///     Grib2SectionStream, Grib2SubmessageStream, Indicator, SectionBody, SectionInfo,
///     SeekableGrib2Reader,
/// };
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
///         (0, 16, Some(37), 64, 99, 157, 178, 184, 189)
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
    iter: Grib2SubmessageValidator<I>,
    sect0: SectionInfo,
    sect1: SectionInfo,
    sect2: Option<SectionInfo>,
    sect3: SectionInfo,
}

impl<I> Grib2SubmessageStream<I>
where
    I: Iterator,
{
    /// # Example
    /// ```
    /// use grib::{Grib2SectionStream, Grib2SubmessageStream, SeekableGrib2Reader};
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
            iter: Grib2SubmessageValidator::new(iter),
            sect0: Default::default(),
            sect1: Default::default(),
            sect2: Default::default(),
            sect3: Default::default(),
        }
    }

    fn clear_message_cache(&mut self) {
        self.sect0 = Default::default();
        self.sect1 = Default::default();
        self.sect2 = Default::default();
        self.sect3 = Default::default();
    }
}

impl<I> Iterator for Grib2SubmessageStream<I>
where
    I: Iterator<Item = Result<SectionInfo, ParseError>>,
{
    type Item = Result<(usize, usize, Submessage), ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut sect4 = Default::default();
        let mut sect5 = Default::default();
        let mut sect6 = Default::default();
        let mut sect7 = Default::default();
        loop {
            match self.iter.next()? {
                Err(e) => return Some(Err(e)),
                Ok((pos, message_count, submessage_count, s)) => match s.num {
                    0 => {
                        self.sect0 = s;
                    }
                    1 => {
                        self.sect1 = s;
                    }
                    2 => {
                        self.sect2 = Some(s);
                    }
                    3 => {
                        self.sect3 = s;
                    }
                    4 => {
                        sect4 = s;
                    }
                    5 => {
                        sect5 = s;
                    }
                    6 => {
                        sect6 = s;
                    }
                    7 => {
                        sect7 = s;
                    }
                    8 => {
                        let ret = Some(Ok((
                            message_count,
                            submessage_count,
                            Submessage(
                                self.sect0.clone(),
                                self.sect1.clone(),
                                self.sect2.clone(),
                                self.sect3.clone(),
                                sect4,
                                sect5,
                                sect6,
                                sect7,
                                s,
                            ),
                        )));

                        // if pos is 0, it is dummy
                        if pos != 0 {
                            self.clear_message_cache();
                        }
                        return ret;
                    }
                    _ => unreachable!(),
                },
            }
        }
    }
}

pub(crate) struct Grib2SubmessageIndexStream<'cacher, I>
where
    I: Iterator,
{
    iter: Grib2SubmessageValidator<I>,
    sect_cacher: Option<&'cacher mut Vec<SectionInfo>>,
    sect0: usize,
    sect1: usize,
    sect2: Option<usize>,
    sect3: usize,
}

impl<'cacher, I> Grib2SubmessageIndexStream<'cacher, I>
where
    I: Iterator,
{
    pub(crate) fn new(iter: I) -> Self {
        Self {
            iter: Grib2SubmessageValidator::new(iter),
            sect_cacher: None,
            sect0: Default::default(),
            sect1: Default::default(),
            sect2: Default::default(),
            sect3: Default::default(),
        }
    }

    pub(crate) fn with_cacher(mut self, cacher: &'cacher mut Vec<SectionInfo>) -> Self {
        self.sect_cacher = Some(cacher);
        self
    }

    fn cache_sect(&mut self, sect: SectionInfo) {
        if let Some(cacher) = self.sect_cacher.as_mut() {
            cacher.push(sect);
        }
    }

    fn clear_message_cache(&mut self) {
        self.sect0 = Default::default();
        self.sect1 = Default::default();
        self.sect2 = Default::default();
        self.sect3 = Default::default();
    }
}

impl<'cacher, I> Iterator for Grib2SubmessageIndexStream<'cacher, I>
where
    I: Iterator<Item = Result<SectionInfo, ParseError>>,
{
    type Item = Result<Grib2SubmessageIndex, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut sect4 = Default::default();
        let mut sect5 = Default::default();
        let mut sect6 = Default::default();
        let mut sect7 = Default::default();
        loop {
            match self.iter.next()? {
                Err(e) => return Some(Err(e)),
                Ok((pos, message_count, submessage_count, s)) => {
                    let num = s.num;
                    match num {
                        0 => {
                            self.cache_sect(s);
                            self.sect0 = pos;
                        }
                        1 => {
                            self.cache_sect(s);
                            self.sect1 = pos;
                        }
                        2 => {
                            self.cache_sect(s);
                            self.sect2 = Some(pos);
                        }
                        3 => {
                            self.cache_sect(s);
                            self.sect3 = pos;
                        }
                        4 => {
                            self.cache_sect(s);
                            sect4 = pos;
                        }
                        5 => {
                            self.cache_sect(s);
                            sect5 = pos;
                        }
                        6 => {
                            self.cache_sect(s);
                            sect6 = pos;
                        }
                        7 => {
                            self.cache_sect(s);
                            sect7 = pos;
                        }
                        8 => {
                            let ret = Some(Ok(Grib2SubmessageIndex::new(
                                (message_count, submessage_count),
                                (
                                    self.sect0, self.sect1, self.sect2, self.sect3, sect4, sect5,
                                    sect6, sect7, pos,
                                ),
                            )));

                            // if pos is 0, it is dummy
                            if pos != 0 {
                                self.cache_sect(s);
                                self.clear_message_cache();
                            }
                            return ret;
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }
    }
}

struct Grib2SubmessageValidator<I>
where
    I: Iterator,
{
    iter: Peekable<Enumerate<I>>,
    pos: usize,
    message_count: usize,
    submessage_count: usize,
    state: Grib2SubmessageValidatorState,
    has_sect3: bool,
}

impl<I> Grib2SubmessageValidator<I>
where
    I: Iterator,
{
    pub fn new(iter: I) -> Self {
        Grib2SubmessageValidator {
            iter: iter.enumerate().peekable(),
            pos: 0,
            message_count: 0,
            submessage_count: 0,
            state: Grib2SubmessageValidatorState::StartOfMessage,
            has_sect3: false,
        }
    }

    #[inline]
    fn reinitialize_message(&mut self) {
        self.submessage_count = 0;
        self.state = Grib2SubmessageValidatorState::StartOfMessage;
        self.has_sect3 = false;
    }

    fn new_unexpected_end_of_data_err(
        &mut self,
    ) -> Option<Result<(usize, usize, usize, SectionInfo), ParseError>> {
        self.state = Grib2SubmessageValidatorState::EndOfStream;
        Some(Err(ParseError::UnexpectedEndOfData(self.pos)))
    }

    fn new_invalid_section_order_err(
        &mut self,
    ) -> Option<Result<(usize, usize, usize, SectionInfo), ParseError>> {
        self.state = Grib2SubmessageValidatorState::EndOfStream;
        Some(Err(ParseError::InvalidSectionOrder(self.pos)))
    }

    fn new_no_grid_definition_err(
        &mut self,
    ) -> Option<Result<(usize, usize, usize, SectionInfo), ParseError>> {
        self.state = Grib2SubmessageValidatorState::EndOfStream;
        Some(Err(ParseError::NoGridDefinition(self.pos)))
    }

    fn wrap_parse_err(
        &mut self,
        e: ParseError,
    ) -> Option<Result<(usize, usize, usize, SectionInfo), ParseError>> {
        self.state = Grib2SubmessageValidatorState::EndOfStream;
        Some(Err(e))
    }
}

impl<I> Grib2SubmessageValidator<I>
where
    I: Iterator<Item = Result<SectionInfo, ParseError>>,
{
    #[inline]
    fn ensure_next_is_sect0(
        &mut self,
    ) -> Option<Result<(usize, usize, usize, SectionInfo), ParseError>> {
        match self.iter.next()? {
            (pos, Ok(s)) => {
                self.pos = pos;
                if s.num == 0 {
                    self.state = Grib2SubmessageValidatorState::EndOfSect(0);
                    Some(Ok((self.pos, self.message_count, self.submessage_count, s)))
                } else {
                    self.new_invalid_section_order_err()
                }
            }
            (_, Err(e)) => self.wrap_parse_err(e),
        }
    }

    #[inline]
    fn ensure_next_is_sect_2_or_3_or_4(
        &mut self,
    ) -> Option<Result<(usize, usize, usize, SectionInfo), ParseError>> {
        match self.iter.next() {
            Some((pos, Ok(s))) => {
                self.pos = pos;
                match s.num {
                    2 => {
                        self.state = Grib2SubmessageValidatorState::EndOfSect(2);
                        Some(Ok((self.pos, self.message_count, self.submessage_count, s)))
                    }
                    3 => {
                        self.state = Grib2SubmessageValidatorState::EndOfSect(3);
                        self.has_sect3 = true;
                        Some(Ok((self.pos, self.message_count, self.submessage_count, s)))
                    }
                    4 => {
                        if self.has_sect3 {
                            self.state = Grib2SubmessageValidatorState::EndOfSect(4);
                            Some(Ok((self.pos, self.message_count, self.submessage_count, s)))
                        } else {
                            self.new_no_grid_definition_err()
                        }
                    }
                    _ => self.new_invalid_section_order_err(),
                }
            }
            Some((_, Err(e))) => self.wrap_parse_err(e),
            None => self.new_unexpected_end_of_data_err(),
        }
    }

    #[inline]
    fn ensure_next_is_sect8(
        &mut self,
    ) -> Option<Result<(usize, usize, usize, SectionInfo), ParseError>> {
        match self.iter.peek() {
            Some((_, Err(_))) => {
                if let Some((_, Err(e))) = self.iter.next() {
                    self.wrap_parse_err(e)
                } else {
                    unreachable!()
                }
            }
            Some((_, Ok(SectionInfo { num: 8, .. }))) => {
                let sect = if let Some((pos, Ok(s))) = self.iter.next() {
                    self.pos = pos;
                    s
                } else {
                    unreachable!()
                };
                let ret = Some(Ok((
                    self.pos,
                    self.message_count,
                    self.submessage_count,
                    sect,
                )));

                self.message_count += 1;
                self.reinitialize_message();
                ret
            }
            Some((_, Ok(_))) => {
                // return dummy Section 8
                let ret = Some(Ok((
                    0,
                    self.message_count,
                    self.submessage_count,
                    SectionInfo::new_8(0),
                )));
                self.submessage_count += 1;
                self.state = Grib2SubmessageValidatorState::StartOfSubmessage;
                ret
            }
            None => self.new_unexpected_end_of_data_err(),
        }
    }

    fn ensure_next_is_sect_num(
        &mut self,
        num: u8,
    ) -> Option<Result<(usize, usize, usize, SectionInfo), ParseError>> {
        match self.iter.next() {
            Some((pos, Ok(s))) => {
                self.state = Grib2SubmessageValidatorState::EndOfSect(num);
                self.pos = pos;
                if s.num == num {
                    Some(Ok((self.pos, self.message_count, self.submessage_count, s)))
                } else {
                    self.new_invalid_section_order_err()
                }
            }
            Some((_, Err(e))) => self.wrap_parse_err(e),
            None => self.new_unexpected_end_of_data_err(),
        }
    }
}

impl<I> Iterator for Grib2SubmessageValidator<I>
where
    I: Iterator<Item = Result<SectionInfo, ParseError>>,
{
    type Item = Result<(usize, usize, usize, SectionInfo), ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            Grib2SubmessageValidatorState::EndOfStream => None,
            Grib2SubmessageValidatorState::StartOfMessage => self.ensure_next_is_sect0(),
            Grib2SubmessageValidatorState::EndOfSect(0) => self.ensure_next_is_sect_num(1),
            Grib2SubmessageValidatorState::EndOfSect(1) => self.ensure_next_is_sect_2_or_3_or_4(),
            Grib2SubmessageValidatorState::StartOfSubmessage => {
                self.ensure_next_is_sect_2_or_3_or_4()
            }
            Grib2SubmessageValidatorState::EndOfSect(2) => {
                self.has_sect3 = true;
                self.ensure_next_is_sect_num(3)
            }
            Grib2SubmessageValidatorState::EndOfSect(3) => self.ensure_next_is_sect_num(4),
            Grib2SubmessageValidatorState::EndOfSect(4) => self.ensure_next_is_sect_num(5),
            Grib2SubmessageValidatorState::EndOfSect(5) => self.ensure_next_is_sect_num(6),
            Grib2SubmessageValidatorState::EndOfSect(6) => self.ensure_next_is_sect_num(7),
            Grib2SubmessageValidatorState::EndOfSect(7) => self.ensure_next_is_sect8(),
            Grib2SubmessageValidatorState::EndOfSect(_) => unreachable!(),
        }
    }
}

enum Grib2SubmessageValidatorState {
    StartOfMessage,
    StartOfSubmessage,
    EndOfSect(u8),
    EndOfStream,
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

    type SubmessageDigest = (
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
    );

    fn digest_submessage_iter_item(item: (usize, usize, Submessage)) -> SubmessageDigest {
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

    fn digest_submessage_index_iter_item(item: Grib2SubmessageIndex) -> SubmessageDigest {
        let (message_index, submessage_index) = item.message_index();
        (
            message_index,
            submessage_index,
            item.0,
            item.1,
            item.2,
            item.3,
            item.4,
            item.5,
            item.6,
            item.7,
            item.8,
        )
    }

    #[test]
    fn submessage_stream_from_1_message() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Ok((0, 0, 0, 1, Some(2), 3, 4, 5, 6, 7, 8))],
        );
    }

    #[test]
    fn submessage_stream_from_multiple_messages() {
        let sects = new_sect_vec_with_dummy_offset(vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 0, 1, 2, 3, 4, 5, 6, 7, 8,
        ]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, Some(2), 3, 4, 5, 6, 7, 8)),
                Ok((1, 0, 9, 10, Some(11), 12, 13, 14, 15, 16, 17))
            ],
        );
    }

    #[test]
    fn submessage_stream_empty() {
        let sects = new_sect_vec_with_dummy_offset(vec![]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
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
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, None, 2, 3, 4, 5, 6, 7)),
                Ok((1, 0, 8, 9, None, 10, 11, 12, 13, 14, 15))
            ],
        );
    }

    #[test]
    fn submessage_stream_from_1_message_with_multiple_submessages_from_sect2() {
        let sects =
            new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 5, 6, 7, 2, 3, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, Some(2), 3, 4, 5, 6, 7, 0)),
                Ok((0, 1, 0, 1, Some(8), 9, 10, 11, 12, 13, 14))
            ],
        );
    }

    #[test]
    fn submessage_stream_from_1_message_with_multiple_submessages_from_sect2_with_sect2_toggled() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 3, 4, 5, 6, 7, 2, 3, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, None, 2, 3, 4, 5, 6, 0)),
                Ok((0, 1, 0, 1, Some(7), 8, 9, 10, 11, 12, 13))
            ],
        );
    }

    #[test]
    fn submessage_stream_from_1_message_with_multiple_submessages_from_sect3() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 5, 6, 7, 3, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, Some(2), 3, 4, 5, 6, 7, 0)),
                Ok((0, 1, 0, 1, Some(2), 8, 9, 10, 11, 12, 13))
            ],
        );
    }

    #[test]
    fn submessage_stream_from_1_message_with_multiple_submessages_from_sect3_without_sect2() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 3, 4, 5, 6, 7, 3, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, None, 2, 3, 4, 5, 6, 0)),
                Ok((0, 1, 0, 1, None, 7, 8, 9, 10, 11, 12))
            ],
        );
    }
    #[test]
    fn submessage_stream_from_1_message_with_multiple_submessages_from_sect4() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 5, 6, 7, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, Some(2), 3, 4, 5, 6, 7, 0)),
                Ok((0, 1, 0, 1, Some(2), 3, 8, 9, 10, 11, 12))
            ],
        );
    }

    #[test]
    fn submessage_stream_from_1_message_with_multiple_submessages_from_sect4_without_sect2() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 3, 4, 5, 6, 7, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, None, 2, 3, 4, 5, 6, 0)),
                Ok((0, 1, 0, 1, None, 2, 7, 8, 9, 10, 11))
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
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, Some(2), 3, 4, 5, 6, 7, 0)),
                Ok((0, 1, 0, 1, Some(8), 9, 10, 11, 12, 13, 14)),
                Ok((1, 0, 15, 16, None, 17, 18, 19, 20, 21, 22))
            ],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect0() {
        let sects = new_sect_vec_with_dummy_offset(vec![0]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(0)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect1() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(1)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect2() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(2)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect3() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(3)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect3_without_sect2() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 3]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(2)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect4() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(4)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect4_without_sect2() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 3, 4]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(3)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect7() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 5, 6, 7]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(7)),],
        );
    }

    #[test]
    fn submessage_stream_from_submessage_without_sect3() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::NoGridDefinition(2)),],
        );
    }

    #[test]
    fn submessage_stream_wrongly_ordered_from_the_beginning() {
        let sects = new_sect_vec_with_dummy_offset(vec![1, 0, 2, 3, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::InvalidSectionOrder(0)),],
        );
    }

    #[test]
    fn submessage_stream_wrongly_ordered_after_sect0() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 2, 1, 3, 4, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::InvalidSectionOrder(1)),],
        );
    }

    #[test]
    fn submessage_stream_wrongly_ordered_after_sect1() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 5, 2, 3, 4, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::InvalidSectionOrder(2)),],
        );
    }

    #[test]
    fn submessage_stream_wrongly_ordered_after_sect2() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 4, 3, 5, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::InvalidSectionOrder(3)),],
        );
    }

    #[test]
    fn submessage_stream_wrongly_ordered_after_sect3() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 5, 4, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::InvalidSectionOrder(4)),],
        );
    }

    #[test]
    fn submessage_stream_wrongly_ordered_after_sect3_without_sect2() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 3, 5, 4, 6, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::InvalidSectionOrder(3)),],
        );
    }

    #[test]
    fn submessage_stream_wrongly_ordered_after_sect4() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 6, 5, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::InvalidSectionOrder(5)),],
        );
    }

    #[test]
    fn submessage_stream_wrongly_ordered_after_sect4_without_sect2() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 3, 4, 6, 5, 7, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::InvalidSectionOrder(4)),],
        );
    }

    #[test]
    fn submessage_stream_wrongly_ordered_after_sect5() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 5, 7, 6, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::InvalidSectionOrder(6)),],
        );
    }

    #[test]
    fn submessage_stream_wrongly_ordered_after_sect6() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 5, 6, 8, 7]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::InvalidSectionOrder(7)),],
        );
    }

    #[test]
    fn submessage_stream_with_2nd_submessage_wrongly_ordered_from_the_beginning() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 5, 6, 7, 0, 8]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
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
                .map(|result| result.map(digest_submessage_iter_item))
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
                .map(|result| result.map(digest_submessage_iter_item))
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
                    num,
                    offset: index,
                    ..Default::default()
                })
            })
            .collect::<Vec<_>>()
    }

    #[test]
    fn submessage_stream_with_sect_stream_error_at_sect0_in_1st_submessage() {
        let sects = new_sect_vec_with_dummy_offset_and_errors(vec![Err(ParseError::ReadError(
            "failed to fill whole buffer".to_owned(),
        ))]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned()
            )),],
        );
    }

    #[test]
    fn submessage_stream_with_sect_stream_error_at_sect1_in_1st_submessage() {
        let sects = new_sect_vec_with_dummy_offset_and_errors(vec![
            Ok(0),
            Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned(),
            )),
        ]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned()
            )),],
        );
    }

    #[test]
    fn submessage_stream_with_sect_stream_error_at_sect2_in_1st_submessage() {
        let sects = new_sect_vec_with_dummy_offset_and_errors(vec![
            Ok(0),
            Ok(1),
            Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned(),
            )),
        ]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned()
            )),],
        );
    }

    #[test]
    fn submessage_stream_with_sect_stream_error_at_sect3_in_1st_submessage() {
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
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned()
            )),],
        );
    }

    #[test]
    fn submessage_stream_with_sect_stream_error_at_sect4_in_1st_submessage() {
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
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned()
            )),],
        );
    }

    #[test]
    fn submessage_stream_with_sect_stream_error_at_sect5_in_1st_submessage() {
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
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned()
            )),],
        );
    }

    #[test]
    fn submessage_stream_with_sect_stream_error_at_sect6_in_1st_submessage() {
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
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned()
            )),],
        );
    }

    #[test]
    fn submessage_stream_with_sect_stream_error_at_sect7_in_1st_submessage() {
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
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned()
            )),],
        );
    }

    #[test]
    fn submessage_stream_with_sect_stream_error_at_sect8_in_1st_submessage() {
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
                .map(|result| result.map(digest_submessage_iter_item))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::ReadError(
                "failed to fill whole buffer".to_owned()
            )),],
        );
    }

    #[test]
    fn submessage_index_stream_from_multiple_messages_and_submessages_with_sect2_toggled() {
        // testing cache of submessage_count and sect2
        let sect_nums = vec![
            0, 1, 2, 3, 4, 5, 6, 7, 2, 3, 4, 5, 6, 7, 8, 0, 1, 3, 4, 5, 6, 7, 8,
        ];
        let sects = new_sect_vec_with_dummy_offset(sect_nums.clone());

        let mut cacher = Vec::new();
        let stream = Grib2SubmessageIndexStream::new(sects.into_iter()).with_cacher(&mut cacher);
        assert_eq!(
            stream
                .map(|result| result.map(digest_submessage_index_iter_item))
                .collect::<Vec<_>>(),
            vec![
                Ok((0, 0, 0, 1, Some(2), 3, 4, 5, 6, 7, 0)),
                Ok((0, 1, 0, 1, Some(8), 9, 10, 11, 12, 13, 14)),
                Ok((1, 0, 15, 16, None, 17, 18, 19, 20, 21, 22))
            ],
        );
        assert_eq!(cacher.iter().map(|s| s.num).collect::<Vec<_>>(), sect_nums,);
    }
}
