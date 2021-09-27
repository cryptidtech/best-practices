extern crate structopt;
use best_practices::{
    error::Error,
    cli::io::*,
    Result
};
use clap::{
    crate_authors,
    crate_description,
    crate_name,
    crate_version
};
use log::*;
use std::io;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = crate_name!(),
    version = crate_version!(),
    author = crate_authors!("\n"),
    about = crate_description!(),
)]
struct Opt {
    
    /// Silence all output
    #[structopt(short = "q", long = "quiet")]
    quiet: bool,

    /// Verbose mode (-v, -vv, -vvv, etc)
    #[structopt(long = "verbose", short = "v", parse(from_occurrences))]
    verbosity: usize,

    /// Subcommand
    #[structopt(subcommand)]
    cmd: Command
}

#[derive(Debug, StructOpt)]
enum Command {

    #[structopt(name = "echo")]
    /// Echo input to output
    Echo {
        /// Output file, otherwise stdout
        #[structopt(short = "o", parse(from_os_str))]
        output: Option<PathBuf>,

        /// Input file, otherwise stdin
        #[structopt(parse(from_os_str))]
        input: Option<PathBuf>,
    }
}

fn main() -> Result<()> {

    // parse the command line flags
    let opt = Opt::from_args();

    // set up the logger
    match stderrlog::new().module(module_path!()).quiet(opt.quiet).verbosity(opt.verbosity).init() {
        Err(e) => {
            return Err(Error::LogError(e.to_string()));
        }
        _ => {}
    }

    match opt.cmd {
        Command::Echo { output, input } => {
            debug!("echoing {} to {}",
                 reader_name(&input)?.to_string_lossy(),
                 writer_name(&output)?.to_string_lossy());

            // copy all input to the output
            io::copy(&mut reader(&input)?, &mut writer(&output)?)?;
        }
    }
    Ok(())
}
