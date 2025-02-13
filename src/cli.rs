use crate::block::BlockHeader;
use crate::error::{DecodeResult, EncodeResult};
use crate::stream::{BlockIndex, StreamFooter, StreamHeader};
use crate::util::Decode;
use clap::{ArgAction, Parser};
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::{fs::File, io};

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

fn read_file(file: &PathBuf) -> io::Result<Vec<u8>> {
    if !file.try_exists()? {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File not found: `{}`", file.to_string_lossy()),
        ));
    }

    std::fs::read(file)
}

pub fn compress_files(files: &[PathBuf]) -> EncodeResult<()> {
    for file in files {
        let _contents = read_file(file)?;

        todo!();
    }

    Ok(())
}

pub fn decompress_files(files: &[PathBuf]) -> DecodeResult<()> {
    fn print_err<E: std::error::Error>(e: E) -> E {
        eprintln!("Error: {e}");
        e
    }

    for file in files {
        let mut file = BufReader::new(File::open(file).map_err(print_err)?);

        println!("\n==========| Stream Header |==========");
        let header = StreamHeader::decode(&mut file).map_err(|e| {
            eprintln!("Error: {e}");
            io::Error::from(io::ErrorKind::InvalidData)
        })?;

        println!("Valid header: {:?}", header);

        while file.fill_buf()?[0] != 0x0 {
            println!("\n==========| Block |==========");

            let header = BlockHeader::decode(&mut file)?;
            println!("Block header:");
            println!("    Filters:");
            for (i, f) in header.filters.iter().enumerate() {
                println!("        {i}:  {:?}", f);
            }

            println!(
                "    Compressed size: {}",
                header
                    .compressed_size
                    .map(|s| s.to_string())
                    .unwrap_or("not specified".into())
            );
            println!(
                "    Uncompressed size: {}",
                header
                    .uncompressed_size
                    .map(|s| s.to_string())
                    .unwrap_or("not specified".into())
            );

            // TODO: Don't assume that the compressed size is specified.
            let mut compressed_size = header.compressed_size.unwrap() as usize;
            compressed_size = (compressed_size + 3) & !3; // pad to multiple of 4 bytes
            let mut block_body = vec![0u8; compressed_size];
            file.read_exact(&mut block_body)?;
            println!("\nBody size: {compressed_size} bytes");
            (0..block_body.len()).for_each(|i| {
                if i % 8 == 0 {
                    if i > 0 {
                        println!();
                    }
                    print!("    [{i:04x}]  ");
                } else if i % 8 == 4 {
                    print!(" ");
                }
                print!("{:02x} ", block_body[i]);
            });
            println!();
        }

        println!("\n==========| Index |==========");

        let index = BlockIndex::decode(&mut file).map_err(|e| {
            eprintln!("Error: {e}");
            io::Error::from(io::ErrorKind::InvalidData)
        })?;

        println!("Valid index: {:?}", index);

        println!("\n==========| Stream Footer |==========");

        let footer = StreamFooter::decode(&mut file).map_err(|e| {
            eprintln!("Error: {e}");
            io::Error::from(io::ErrorKind::InvalidData)
        })?;

        println!("Valid footer: {:?}", footer);
    }

    Ok(())
}
