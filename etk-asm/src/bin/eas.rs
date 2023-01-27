use etk_cli::errors::WithSources;
use etk_cli::io::HexWrite;

use etk_asm::ingest::{Error, Ingest};

use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::path::PathBuf;

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


/*
// This function creates a file at the given path, and returns the file
// object. If the file cannot be created, the function panics.
fn create(path: PathBuf) -> File {
    match File::create(&path) {
        Err(why) => panic!("couldn't create `{}`: {}", path.display(), why),
        Ok(file) => file,
    }
}
*/

// This is the main function of the program. It calls run() and
// prints the error message returned by run() if run() returns
// an error. Otherwise, it exits the program.
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

    // Create an Ingest object that will read the data from the input
    // file and write it to the output file.
    let mut ingest = Ingest::new(HexWrite::new(&mut vec![]));

    // Read the data from the input file and write it to the vec
    let res = ingest.ingest_file(opt.input)?;

    // Create an output file handle to write the data to. If the user
    // did not specify an output file, use standard output.
    let mut out: Box<dyn Write> = match opt.out {
        Some(o) => Box::new(create(o)),
        None => Box::new(std::io::stdout()),
    };

    // Write the data to the output file
    out.write_all(&res)?;
    // Write a newline to the output file.
    out.write_all(b"\n")?;

    // Exit the program successfully.
    Ok(())
}

/*
fn run() -> Result<(), Error> {
    // Parse the command line arguments.
    let opt: Opt = clap::Parser::parse();

    // Create an output file handle to write the data to. If the user
    // did not specify an output file, use standard output.
    let mut out: Box<dyn Write> = match opt.out {
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
    ingest.ingest_file(opt.input)?;

    // Write a newline to the output file.
    out.write_all(b"\n").unwrap();

    // Exit the program successfully.
    Ok(())
}
*/
