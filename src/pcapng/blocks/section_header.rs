use std::io::{Read, BufRead};
use crate::errors::PcapError;
use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};
use crate::Endianness;
use crate::pcapng::blocks::section_header::SectionHeaderOption::Comment;
use crate::peek_reader::PeekReader;
use crate::pcapng::blocks::common::opts_from_slice;
use std::borrow::Cow;

///Section Header Block: it defines the most important characteristics of the capture file.
#[derive(Clone, Debug)]
pub struct SectionHeaderBlock<'a> {

    /// Magic number, whose value is 0x1A2B3C4D.
    /// This number can be used to distinguish sections that have been saved
    /// on little-endian machines from the ones saved on big-endian machines.
    pub magic: u32,

    /// Major version of the format.
    /// Current value is 1.
    pub major_version: u16,

    /// Minor version of the format.
    /// Current value is 0.
    pub minor_version: u16,

    /// Length in bytes of the following section excluding this block.
    ///
    /// This block can be used to skip the section for faster navigation in
    /// large files. Length of -1i64 means that the length is unspecified.
    pub section_length: i64,

    pub options: Vec<SectionHeaderOption<'a>>
}


impl<'a> SectionHeaderBlock<'a> {

    pub fn from_slice(mut slice: &'a [u8]) -> Result<(Self, &'a [u8]), PcapError> {

        let magic = slice.read_u32::<BigEndian>()?;

        let (major_version, minor_version, section_length, options, slice) = match magic {

            0x1A2B3C4D => parse_inner::<BigEndian>(slice)?,
            0x4D3C2B1A => parse_inner::<LittleEndian>(slice)?,

            _ => return Err(PcapError::InvalidField("SectionHeaderBlock invalid magic number"))
        };

        let block = SectionHeaderBlock {
            magic,
            major_version,
            minor_version,
            section_length,
            options
        };

        return Ok((block, slice));

        fn parse_inner<B: ByteOrder>(mut slice: &[u8]) -> Result<(u16, u16, i64, Vec<SectionHeaderOption<'_>>, &[u8]), PcapError> {

            let maj_ver = slice.read_u16::<B>()?;
            let min_ver = slice.read_u16::<B>()?;
            let sec_len = slice.read_i64::<B>()?;
            let (opts, slice) = SectionHeaderOption::from_slice::<B>(slice)?;

            Ok((maj_ver, min_ver, sec_len, opts, slice))
        }
    }

    pub fn endianness(&self) -> Endianness {

        match self.magic {

            0x1A2B3C4D => Endianness::Big,
            0x4D3C2B1A => Endianness::Little,
            _ => unreachable!()

        }
    }
}

#[derive(Clone, Debug)]
pub enum SectionHeaderOption<'a> {

    /// Comment associated with the current block
    Comment(&'a str),

    /// Description of the hardware used to create this section
    Hardware(&'a str),

    /// Name of the operating system used to create this section
    OS(&'a str),

    /// Name of the application used to create this section
    UserApplication(&'a str)
}


impl<'a> SectionHeaderOption<'a> {

    fn from_slice<B:ByteOrder>(slice: &'a [u8]) -> Result<(Vec<Self>, &'a [u8]), PcapError> {

        opts_from_slice::<B, _, _>(slice, |slice, type_, len| {

            let mut string = std::str::from_utf8(slice)?;

            let opt = match type_ {

                1 => SectionHeaderOption::Comment(string),
                2 => SectionHeaderOption::Hardware(string),
                3 => SectionHeaderOption::OS(string),
                4 => SectionHeaderOption::UserApplication(string),

                _ => return Err(PcapError::InvalidField("SectionHeaderOption type invalid"))
            };

            Ok(opt)
        })
    }
}


