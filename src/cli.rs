use crate::block::{BlockDecodeError, BlockHeader};
use crate::checksum::{Checksum, Crc32, Crc64};
use crate::error::{DecodeError, DecodeResult, EncodeResult};
use crate::lzma2::decode_lzma2;
use crate::stream::{BlockIndex, StreamFooter, StreamHeader};
use crate::util::{CheckedReader, Decode, InputRead};
use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io::{self, Seek};
use std::io::{BufRead, BufReader, Read};
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

/*
#[derive(Parser, Debug)]
#[command(name = "xz")]
#[command(about = "Compress or decompress FILEs in the .xz format.", long_about = None)]
pub struct XzArgs {
    /// Files to process
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Force compression
    #[arg(short = 'z', long = "compress")]
    pub compress: bool,

    /// Force decompression
    #[arg(short = 'd', long = "decompress")]
    pub decompress: bool,

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

    /// Compression preset
    #[arg(short = 'p', value_name = "LEVEL", default_value = "6")]
    pub compression_level: u8,

    /// Try to improve compression ratio by using more CPU time
    #[arg(short = 'e', long = "extreme")]
    pub extreme: bool,

    /// Use at most NUM threads; 0 for autodetect
    #[arg(short = 'T', long = "threads", value_name = "NUM", default_value = "0")]
    pub threads: usize,

    /// Suppress warnings; specify twice to suppress errors too
    #[arg(short = 'q', long = "quiet", action = ArgAction::Count)]
    pub quiet: u8,

    /// Be verbose; specify twice for even more verbose
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    pub verbose: u8,
}
 */

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

pub fn compress_files(files: &[PathBuf], options: &Options) -> EncodeResult<()> {
    todo!()
}

pub fn decompress_files(files: &[PathBuf], options: &Options) -> DecodeResult<()> {
    for file in files {
        let mut input = BufReader::new(File::open(file)?);

        // The first part of the file is the stream header.
        // TODO: do CRC32/CRC64/SHA256 verification based on the header.
        let stream_header = StreamHeader::decode(&mut input).map_err(|e| {
            eprintln!("Error: {e}");
            io::Error::from(io::ErrorKind::InvalidData)
        })?;

        // Then, the file is made up of a series of blocks.
        // We don't know how many there are up front,
        // but a block header starts with a non-zero size byte
        // and the stream index (first thing after the last block)
        // starts with a 0 byte.
        while input.fill_buf()?[0] != 0x0 {
            // Decode the block header.
            // TODO: use this to determine what & how to decode.
            let block_header = BlockHeader::decode(&mut input)?;

            let mut output = Vec::new();
            decode_lzma2(&mut input, &mut output)?;

            let padding = (4 - (input.stream_position()? % 4)) % 4;
            let mut padding_bytes = vec![0; padding as usize];
            input.read_exact(&mut padding_bytes)?;

            match stream_header.flags {
                crate::stream::StreamFlags::None => {}
                crate::stream::StreamFlags::Crc32 => {
                    let mut crc32 = Crc32::new();
                    crc32.process_bytes(&output);
                    let actual_crc32 = crc32.result();
                    let expected_crc32 = input.read_le_u32()?;
                    if actual_crc32 != expected_crc32 {
                        return Err(DecodeError::BlockDecodeError(
                            BlockDecodeError::ChecksumMismatch,
                        ));
                    }
                }
                crate::stream::StreamFlags::Crc64 => {
                    let mut crc64 = Crc64::new();
                    crc64.process_bytes(&output);
                    let actual_crc64 = crc64.result();
                    let expected_crc64 = input.read_le_u64()?;
                    if actual_crc64 != expected_crc64 {
                        return Err(DecodeError::BlockDecodeError(
                            BlockDecodeError::ChecksumMismatch,
                        ));
                    }
                }
                crate::stream::StreamFlags::Sha256 => todo!(),
            }

            println!(
                "decoded string:\n---------------\n{}",
                String::from_utf8(output.clone()).unwrap_or("[utf8 error]".into())
            );
        }

        // Decode the stream index.
        // TODO: use this to verify the validity of the file.
        let _index = BlockIndex::decode(&mut input).map_err(|e| {
            eprintln!("Error: {e}");
            io::Error::from(io::ErrorKind::InvalidData)
        })?;

        // Decode the stream footer.
        let _footer = StreamFooter::decode(&mut input).map_err(|e| {
            eprintln!("Error: {e}");
            io::Error::from(io::ErrorKind::InvalidData)
        })?;

        println!("file decoded successfully");
    }

    Ok(())
}

pub fn test_files(files: &[PathBuf]) -> std::io::Result<()> {
    todo!()
}

pub fn list_files(files: &[PathBuf]) -> DecodeResult<()> {
    todo!()
}
