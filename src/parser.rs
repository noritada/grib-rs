use crate::context::SectionInfo;
use crate::error::*;
use std::iter::Peekable;

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

pub struct Grib2SubmessageStream<I>
where
    I: Iterator,
{
    iter: Peekable<I>,
    message_count: usize,
    submessage_count: usize,
    sect0: SectionInfo,
    sect1: SectionInfo,
    sect2: Option<SectionInfo>,
    sect3: SectionInfo,
}

impl<I> Grib2SubmessageStream<I>
where
    I: Iterator,
{
    pub fn new(iter: I) -> Self {
        Self {
            iter: iter.peekable(),
            message_count: 0,
            submessage_count: 0,
            sect0: Default::default(),
            sect1: Default::default(),
            sect2: Default::default(),
            sect3: Default::default(),
        }
    }

    pub fn initialize_submessage_cache(&mut self) {
        self.submessage_count = 0;
        self.sect0 = Default::default();
        self.sect1 = Default::default();
        self.sect2 = Default::default();
        self.sect3 = Default::default();
    }

    fn new_unexpected_end_of_data_err(
        &self,
    ) -> Option<Result<(usize, usize, Submessage), ParseError>> {
        return Some(Err(ParseError::UnexpectedEndOfData(
            self.message_count,
            self.submessage_count,
        )));
    }

    fn new_invalid_section_order_err(
        &self,
    ) -> Option<Result<(usize, usize, Submessage), ParseError>> {
        Some(Err(ParseError::InvalidSectionOrder(
            self.message_count,
            self.submessage_count,
        )))
    }

    fn new_no_grid_definition_err(&self) -> Option<Result<(usize, usize, Submessage), ParseError>> {
        return Some(Err(ParseError::NoGridDefinition(
            self.message_count,
            self.submessage_count,
        )));
    }
}

impl<I> Iterator for Grib2SubmessageStream<I>
where
    I: Iterator<Item = Result<SectionInfo, ParseError>>,
{
    type Item = Result<(usize, usize, Submessage), ParseError>;

    fn next(&mut self) -> Option<Result<(usize, usize, Submessage), ParseError>> {
        macro_rules! check_next_sect {
            ($num:expr) => {{
                let sect = self.iter.next();
                if sect.is_none() {
                    return self.new_unexpected_end_of_data_err();
                }

                let sect = sect.unwrap();
                if let Err(e) = sect {
                    return Some(Err(e));
                }

                let sect = sect.unwrap();
                if sect.num != $num {
                    return self.new_invalid_section_order_err();
                }

                sect
            }};
        }

        if self.submessage_count == 0 {
            let sect0 = self.iter.next()?;
            if let Err(e) = sect0 {
                return Some(Err(e));
            }

            self.sect0 = sect0.unwrap();
            if self.sect0.num != 0 {
                return self.new_invalid_section_order_err();
            }

            self.sect1 = check_next_sect!(1);
        }

        let sect = self.iter.next();
        if sect.is_none() {
            return self.new_unexpected_end_of_data_err();
        }
        let sect = sect.unwrap();
        if let Err(e) = sect {
            return Some(Err(e));
        }

        let sect = sect.unwrap();
        let sect4 = match sect.num {
            2 => {
                self.sect2 = Some(sect);
                self.sect3 = check_next_sect!(3);
                check_next_sect!(4)
            }
            3 => {
                self.sect3 = sect;
                check_next_sect!(4)
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

        let sect5 = check_next_sect!(5);
        let sect6 = check_next_sect!(6);
        let sect7 = check_next_sect!(7);

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
        if sect8.is_none() {
            return self.new_unexpected_end_of_data_err();
        }

        let sect8 = sect8.unwrap();
        match sect8 {
            Err(_) => {
                if let Some(Err(e)) = self.iter.next() {
                    return Some(Err(e));
                } else {
                    unreachable!()
                }
            }
            Ok(SectionInfo { num: 8, .. }) => {
                self.iter.next();
                self.message_count += 1;
                self.initialize_submessage_cache();
            }
            _ => {
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
            vec![Err(ParseError::UnexpectedEndOfData(0, 0)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect1() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(0, 0)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect2() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(0, 0)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect3() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(0, 0)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect3_without_sect2() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 3]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(0, 0)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect4() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(0, 0)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect4_without_sect2() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 3, 4]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(0, 0)),],
        );
    }

    #[test]
    fn submessage_stream_ending_with_sect7() {
        let sects = new_sect_vec_with_dummy_offset(vec![0, 1, 2, 3, 4, 5, 6, 7]);

        assert_eq!(
            Grib2SubmessageStream::new(sects.into_iter())
                .map(|result| result.map(|i| digest_submessage_iter_item(i)))
                .collect::<Vec<_>>(),
            vec![Err(ParseError::UnexpectedEndOfData(0, 0)),],
        );
    }
}
