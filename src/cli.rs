use crate::block::{BlockDecodeError, BlockHeader};
use crate::checksum::{Crc32, Crc64};
use crate::error::{DecodeError, DecodeResult, EncodeResult};
use crate::lzma2::decode_lzma2;
use crate::stream::{BlockIndex, StreamDecodeError, StreamFlags, StreamFooter, StreamHeader};
use crate::util::{CheckedWriter, Decode, InputRead};
use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{stdout, BufRead, BufReader, Read, Seek, Write};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(about = "Compress or decompress FILEs in the .xz format")]
pub struct XzArgs {
    /// Files to process
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Force compression
    #[arg(short = 'z', long = "compress")]
    pub compress: Option<bool>,

    /// Force decompression
    #[arg(short = 'd', long = "decompress")]
    pub decompress: Option<bool>,

    /// Test compressed file integrity
    #[arg(short = 't', long = "test")]
    pub test: bool,

    /// List information about .xz files
    #[arg(short = 'l', long = "list")]
    pub list: bool,

    /// Keep (don't delete) input files
    #[arg(short = 'k', long = "keep")]
    pub keep: bool,

    /// Force overwrite of output file and (de)compress links
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Write to standard output and don't delete input files
    #[arg(short = 'c', long = "stdout")]
    pub stdout: bool,
}

pub enum Action {
    Compress,
    Decompress,
    Test,
    List,
}

pub struct Options {
    pub keep: bool,
    pub force: bool,
    pub stdout: bool,
}

pub fn do_action(
    action: &Action,
    options: &Options,
    files: &[PathBuf],
) -> Result<(), Box<dyn Error>> {
    match action {
        Action::Compress => Ok(compress_files(files, options)?),
        Action::Decompress => Ok(decompress_files(files, options)?),
        Action::Test => Ok(test_files(files)?),
        Action::List => Ok(list_files(files)?),
    }
}

pub fn compress_files(_files: &[PathBuf], _options: &Options) -> EncodeResult<()> {
    todo!()
}

pub fn decompress_files(files: &[PathBuf], options: &Options) -> DecodeResult<()> {
    for in_filename in files {
        let out_filename = if in_filename.extension().is_some_and(|ext| ext == "xz") {
            in_filename.with_extension("")
        } else {
            eprintln!(
                "{}: Filename has an unknown suffix, skipping",
                in_filename.to_string_lossy()
            );
            continue;
        };

        let mut input = BufReader::new(File::open(in_filename)?);
        let mut output: Box<dyn Write> = if options.stdout {
            Box::new(stdout())
        } else {
            Box::new(File::create(&out_filename)?)
        };

        // The first part of the file is the stream header.
        let stream_header = StreamHeader::decode(&mut input)?;

        // Then, the file is made up of a series of blocks.
        // We don't know how many there are up front,
        // but a block header starts with a non-zero size byte
        // and the stream index (first thing after the last block)
        // starts with a 0 byte.
        while input.fill_buf()?[0] != 0x0 {
            // Decode the block header.
            // TODO: use this to determine what & how to decode.
            let _block_header = BlockHeader::decode(&mut input)?;

            match stream_header.flags {
                StreamFlags::None => {
                    decode_lzma2(&mut input, &mut output)?;
                    let padding = (4 - (input.stream_position()? % 4)) % 4;
                    let mut padding_bytes = vec![0; padding as usize];
                    input.read_exact(&mut padding_bytes)?;
                }
                StreamFlags::Crc32 => {
                    let mut output = CheckedWriter::new(&mut output, Crc32::new());
                    decode_lzma2(&mut input, &mut output)?;
                    let padding = (4 - (input.stream_position()? % 4)) % 4;
                    let mut padding_bytes = vec![0; padding as usize];
                    input.read_exact(&mut padding_bytes)?;
                    let expected_crc32 = input.read_le_u32()?;
                    if output.checksum() != expected_crc32 {
                        return Err(DecodeError::BlockDecodeError(
                            BlockDecodeError::ChecksumMismatch,
                        ));
                    }
                }
                StreamFlags::Crc64 => {
                    let mut output = CheckedWriter::new(&mut output, Crc64::new());
                    decode_lzma2(&mut input, &mut output)?;
                    let padding = (4 - (input.stream_position()? % 4)) % 4;
                    let mut padding_bytes = vec![0; padding as usize];
                    input.read_exact(&mut padding_bytes)?;
                    let expected_crc64 = input.read_le_u64()?;
                    if output.checksum() != expected_crc64 {
                        return Err(DecodeError::BlockDecodeError(
                            BlockDecodeError::ChecksumMismatch,
                        ));
                    }
                }
                StreamFlags::Sha256 => todo!(),
            }
        }

        // Decode the stream index.
        // TODO: use this to verify the validity of the file.
        let _index = BlockIndex::decode(&mut input)?;

        // Decode the stream footer.
        let stream_footer = StreamFooter::decode(&mut input)?;
        if stream_header.flags != stream_footer.flags {
            return Err(DecodeError::StreamDecodeError(
                StreamDecodeError::HeaderFooterMismatch,
            ));
        }

        if !options.keep {
            std::fs::remove_file(in_filename)?;
        }
    }

    Ok(())
}

pub fn test_files(_files: &[PathBuf]) -> std::io::Result<()> {
    todo!()
}

pub fn list_files(_files: &[PathBuf]) -> DecodeResult<()> {
    todo!()
}
