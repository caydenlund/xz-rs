use crate::block::BlockHeader;
use crate::error::{DecodeResult, EncodeResult};
use crate::lzma2::decode_lzma2;
use crate::stream::{BlockIndex, StreamFooter, StreamHeader};
use crate::util::Decode;
use clap::{ArgAction, Parser};
use std::fs::File;
use std::io::{self, Seek};
use std::io::{BufRead, BufReader, Cursor, Read};
use std::path::PathBuf;

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

pub fn compress_files(_files: &[PathBuf]) -> EncodeResult<()> {
    todo!()
}

pub fn decompress_files(files: &[PathBuf]) -> DecodeResult<()> {
    fn print_err<E: std::error::Error>(e: E) -> E {
        eprintln!("Error: {e}");
        e
    }

    for file in files {
        let mut file = BufReader::new(File::open(file).map_err(print_err)?);

        // The first part of the file is the stream header.
        // TODO: do CRC32/CRC64/SHA256 verification based on the header.
        let _header = StreamHeader::decode(&mut file).map_err(|e| {
            eprintln!("Error: {e}");
            io::Error::from(io::ErrorKind::InvalidData)
        })?;

        // Then, the file is made up of a series of blocks.
        // We don't know how many there are up front,
        // but a block header starts with a non-zero size byte
        // and the stream index (first thing after the last block)
        // starts with a 0 byte.
        while file.fill_buf()?[0] != 0x0 {
            // Decode the block header.
            // TODO: use this to determine what & how to decode.
            let _header = BlockHeader::decode(&mut file)?;

            // Decode the block body.
            let mut decoded = Vec::new();
            decode_lzma2(&mut file, &mut decoded)?;
            println!(
                "decoded string: `{}`",
                String::from_utf8(decoded)
                    .unwrap_or("[error]".into())
                    .replace("\n", "\\n")
            );

            // Read padding bytes for 4-byte alignment.
            {
                let padding = (4 - (file.stream_position()? % 4)) % 4;
                let mut bytes = vec![0; padding as usize];
                file.read_exact(&mut bytes)?;
            }
        }

        // Decode the stream index.
        // TODO: use this to verify the validity of the file.
        let _index = BlockIndex::decode(&mut file).map_err(|e| {
            eprintln!("Error: {e}");
            io::Error::from(io::ErrorKind::InvalidData)
        })?;

        // Decode the stream footer.
        let _footer = StreamFooter::decode(&mut file).map_err(|e| {
            eprintln!("Error: {e}");
            io::Error::from(io::ErrorKind::InvalidData)
        })?;

        println!("file decoded successfully");
    }

    Ok(())
}
