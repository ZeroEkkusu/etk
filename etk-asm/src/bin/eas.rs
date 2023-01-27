use etk_cli::errors::WithSources;
use etk_cli::io::HexWrite;

use etk_asm::ingest::{Error, Ingest};

use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use clap::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "eas")]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    #[structopt(parse(from_os_str))]
    out: Option<PathBuf>,
}

fn create(path: PathBuf) -> File {
    match create_dir_all(path.parent().unwrap()) {
        Ok(_) => (),
        Err(why) => panic!("couldn't create parent directories: {}", why),
    }

    match File::create(&path) {
        Err(why) => panic!("couldn't create `{}`: {}", path.display(), why),
        Ok(file) => file,
    }
}

fn main() {
    let err = match run() {
        Ok(_) => return,
        Err(e) => e,
    };

    eprintln!("{}", WithSources(err));
    std::process::exit(1);
}

fn run() -> Result<(), Error> {
    // Parse the command line arguments.
    let opt: Opt = clap::Parser::parse();

    // Check if the output file already exists.
    let mut out_file_exists = false;
    let mut out_file_content = vec![];
    let mut non_existing_directories = vec![];
    let o = opt.out.as_ref();
    if let Some(o) = o {
        if Path::new(o).exists() {
            out_file_exists = true;
            match std::fs::read(o) {
                Ok(content) => out_file_content = content,
                Err(e) => panic!("couldn't backup existing file: {}", e),
            }            
        } else {
            let mut path = Path::new(o);
            while !path.exists() {
                non_existing_directories.push(path);
                path = path.parent().unwrap();
            }
        }
    }

    // Create an output file handle to write the data to. If the user
    // did not specify an output file, use standard output.
    let mut out: Box<dyn Write> = match o {
        Some(o) => Box::new(create(o)),
        None => Box::new(std::io::stdout()),
    };

    // Create a wrapper around the output file handle that will write
    // hexadecimal data to it.
    let hex_out = HexWrite::new(&mut out);

    // Create an Ingest object that will read the data from the input
    // file and write it to the output file.
    let mut ingest = Ingest::new(hex_out);

    // Read the data from the input file and write it to the output file.
    if let Err(e) = ingest.ingest_file(opt.input) {
        if out_file_exists {
            match std::fs::write(o.unwrap(), &out_file_content) {
                Ok(_) => (),
                Err(e) => panic!("couldn't restore existing file: {}", e),
            };
        } else if let Some(o) = o {
            match std::fs::remove_file(o) {
                Ok(_) => (),
                Err(e) => panic!("couldn't remove artifacts: {}", e),
            }            
            for non_existing_dir in non_existing_directories {
                match std::fs::remove_dir(non_existing_dir) {
                    Ok(()) => (),
                    Err(e) => panic!("couldn't remove artifacts: {}", e),
                }                
            }
        }
        return Err(e);
    }

    // Write a newline to the output file.
    out.write_all(b"\n").unwrap();

    // Exit the program successfully.
    Ok(())
}