use best_practices::{
    error::Error,
    cli::io::*,
    cli::fs::{
        TreeIndexBuilder,
        TreeListBuilder
    },
    Result,
};
use clap::{
    crate_authors,
    crate_description,
    crate_name,
    crate_version
};
use log::*;
use std::collections::HashSet;
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

    #[structopt(name = "list")]
    /// Recursively scan a dir tree and output digest+path pairs
    List {
        /// Use faster file hashing, less precise but mutch faster
        #[structopt(long)]
        fast: bool,

        /// The root directory to index recursively, otherwise current dir
        #[structopt(parse(from_os_str))]
        root: Option<PathBuf>,

        /// The file to save the index to, otherwise stdout.
        #[structopt(parse(from_os_str))]
        output: Option<PathBuf>,
    },

    #[structopt(name = "index")]
    /// Recursively scan a dir tree and output an index with or without dupes
    Index {
        /// Include duplicates? Default is no
        #[structopt(long)]
        dupes: bool,

        /// Use faster file hashing, less precise but mutch faster
        #[structopt(long)]
        fast: bool,

        /// The root directory to index recursively, otherwise current dir
        #[structopt(parse(from_os_str))]
        root: Option<PathBuf>,

        /// The file to save the index to, otherwise stdout.
        #[structopt(parse(from_os_str))]
        output: Option<PathBuf>,
    },

    #[structopt(name = "match")]
    /// Find duplicates of files in the given index file
    Match {
        /// Use faster file hashing, less precise but mutch faster
        #[structopt(long)]
        fast: bool,

        /// The root directory to search for duplicates
        #[structopt(parse(from_os_str))]
        root: Option<PathBuf>,

        /// The index data file, otherwise stdin
        #[structopt(parse(from_os_str))]
        input: Option<PathBuf>,

        /// The file to save the index to, otherwise stdout.
        #[structopt(parse(from_os_str))]
        output: Option<PathBuf>,
    },

    #[structopt(name = "confirm")]
    /// Goes through an index file and uses slow digesting to confirm dupes
    Confirm {
        /// The index data file, otherwise stdin
        #[structopt(parse(from_os_str))]
        input: Option<PathBuf>,

        /// The file to save the index to, otherwise stdout.
        #[structopt(parse(from_os_str))]
        output: Option<PathBuf>,
    },

    #[structopt(name = "zeroes")]
    /// Goes through an index file and removes all items and dupes with 0 length
    Zeroes {
        /// The index data file, otherwise stdin
        #[structopt(parse(from_os_str))]
        input: Option<PathBuf>,

        /// The file to save the index to, otherwise stdout.
        #[structopt(parse(from_os_str))]
        output: Option<PathBuf>,
    },

    #[structopt(name = "dupes")]
    /// Commands for handling duplicate files
    Dupes {
        /// Subcommand
        #[structopt(subcommand)]
        cmd: DupesCommand
    }
}

#[derive(Debug, StructOpt)]
enum DupesCommand {

    #[structopt(name = "find")]
    /// Find duplicates of files from one index in another and producing a third
    Find {

        /// The "needle" index data file, otherwise stdin
        #[structopt(parse(from_os_str))]
        needle: Option<PathBuf>,

        /// The "haystack" index data file
        #[structopt(parse(from_os_str))]
        haystack: Option<PathBuf>,

        /// The file to save the dupe dir list, otherwise stdout.
        #[structopt(parse(from_os_str))]
        output: Option<PathBuf>,
    },

    #[structopt(name = "listdirs")]
    /// Find duplicates of files in the given index file
    ListDirs {

        /// The index data file, otherwise stdin
        #[structopt(parse(from_os_str))]
        input: Option<PathBuf>,

        /// The file to save the dupe dir list, otherwise stdout.
        #[structopt(parse(from_os_str))]
        output: Option<PathBuf>,
    },

    #[structopt(name = "size")]
    /// Sum up the total size of storage space that would be saved by de-duping
    Size {

        /// The index data file, otherwise stdin
        #[structopt(parse(from_os_str))]
        input: Option<PathBuf>,

        /// The file to save the stats to, otherwise stdout.
        #[structopt(parse(from_os_str))]
        output: Option<PathBuf>,
    },

    #[structopt(name = "copy")]
    /// Copy all duplicate files to the specified folder
    CopyFiles {

        /// Dry run flag
        #[structopt(long)]
        dry_run: bool,

        /// The index data file, otherwise stdin
        #[structopt(parse(from_os_str))]
        input: Option<PathBuf>,

        /// The destination directory to move the dupe files to
        #[structopt(parse(from_os_str))]
        dest: Option<PathBuf>,

        /// The file to save the log of actions to
        #[structopt(parse(from_os_str))]
        output: Option<PathBuf>,
    },

    #[structopt(name = "delete")]
    /// Delete all duplicate files in the index
    DeleteFiles {

        /// Dry run flag
        #[structopt(long)]
        dry_run: bool,

        /// The index data file, otherwise stdin
        #[structopt(parse(from_os_str))]
        input: Option<PathBuf>,

        /// The file to save the log of actions to
        #[structopt(parse(from_os_str))]
        output: Option<PathBuf>,
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

        Command::List { fast, root, output } => {
            debug!("listing {} to {}",
                 dir_name(&root)?.to_string_lossy(),
                 writer_name(&output)?.to_string_lossy());

            // create the list from the directory tree
            let tl = TreeListBuilder::new()
                .fast(fast)
                .path(&dir(&root)?)
                .build()?;

            // output the list
            let mut w = writer(&output)?;
            for item in tl.list {
                write!(w, "{}", item)?;
            }
        },

        Command::Index { dupes, fast, root, output } => {
            debug!("indexing {} to {}",
                 dir_name(&root)?.to_string_lossy(),
                 writer_name(&output)?.to_string_lossy());

            // create the index from the directory tree
            let tl = TreeListBuilder::new()
                .fast(fast)
                .path(&dir(&root)?)
                .build()?;
            let ti = TreeIndexBuilder::new()
                .with_dupes(dupes)
                .from_list(&tl)
                .build()?;

            // output the index
            let mut w = writer(&output)?;
            for item in ti.idx.into_values() {
                write!(w, "{}", item)?;
            }
        },

        Command::Match { fast, root, input, output } => {
            debug!("matching {} to {} output to {}",
                 dir_name(&root)?.to_string_lossy(),
                 reader_name(&input)?.to_string_lossy(),
                 writer_name(&output)?.to_string_lossy());

            // read the index from the input source without dupes
            let mut ti = TreeIndexBuilder::new()
                .with_dupes(false)
                .from_reader(&mut reader(&input)?)
                .build()?;

            // get the maximum file size so we don't digest files that can't match
            let max = ti.max();

            // build a list of files in the target tree
            let tl = TreeListBuilder::new()
                .fast(fast)
                .max_size(max)
                .path(&dir(&root)?)
                .build()?;

            // go through the list and add any dupes to the source_index
            for i in tl.list {
                match ti.idx.get_mut(&i.digest) {
                    Some(item) => {
                        item.push(i.path.clone());
                    },
                    _ => {}
                }
            }

            // output the index with dupes
            let mut w = writer(&output)?;
            for item in ti.idx.into_values() {
                write!(w, "{}", item)?;
            }
        },

        Command::Confirm { input, output } => {
            debug!("confirming {}, output to {}",
                 reader_name(&input)?.to_string_lossy(),
                 writer_name(&output)?.to_string_lossy());

            // read the index from the input source with dupes
            let ti = TreeIndexBuilder::new()
                .with_dupes(true)
                .from_reader(&mut reader(&input)?)
                .build()?;

            // create new index by confirming old index
            let cti = TreeIndexBuilder::new()
                .confirm(&ti)
                .build()?;

            // output the index with dupes
            let mut w = writer(&output)?;
            for item in cti.idx.into_values() {
                write!(w, "{}", item)?;
            }
        },

        Command::Zeroes { input, output } => {
            debug!("removing zero length items from {}, output to {}",
                 reader_name(&input)?.to_string_lossy(),
                 writer_name(&output)?.to_string_lossy());

            // read the index from the input source with dupes
            let ti = TreeIndexBuilder::new()
                .with_dupes(true)
                .from_reader(&mut reader(&input)?)
                .build()?;

            // keep anything with a size > 0
            let mut index = TreeIndexBuilder::new().build()?;
            for (digest, item) in ti.idx.iter() {
                if item.item.size > 0 {
                    trace!("{}", item.item.path.to_string_lossy());
                    index.idx.insert(digest.clone(), item.clone());
                }
            }

            // output the index with dupes
            let mut w = writer(&output)?;
            for item in index.idx.into_values() {
                write!(w, "{}", item)?;
            }
        },

        Command::Dupes { cmd } => {
            match cmd {

                DupesCommand::Find { needle, haystack, output } => {
                    debug!("finding needles from {} in hastack {}, output to {}",
                         reader_name(&needle)?.to_string_lossy(),
                         reader_name(&haystack)?.to_string_lossy(),
                         writer_name(&output)?.to_string_lossy());

                    // read the needles from the input source without dupes
                    let needle_ti = TreeIndexBuilder::new()
                        .with_dupes(false)
                        .from_reader(&mut reader(&needle)?)
                        .build()?;
                    trace!("loaded {} items with {} dupes in the needle",
                           needle_ti.idx.len(), needle_ti.count_dupes());

                    // read the haystack from the input source with dupes
                    let haystack_ti = TreeIndexBuilder::new()
                        .with_dupes(true)
                        .from_reader(&mut reader(&haystack)?)
                        .build()?;
                    trace!("loaded {} items with {} dupes in the needle",
                           haystack_ti.idx.len(), haystack_ti.count_dupes());

                    let mut index = TreeIndexBuilder::new().build()?;
                    for (digest, needle_item) in needle_ti.idx.iter() {
                        match haystack_ti.idx.get(digest) {
                            Some(haystack_item) => {
                                if needle_item.item.path != haystack_item.item.path {
                                    trace!("adding {} to {}",
                                           haystack_item.item.path.to_string_lossy(),
                                           needle_item.item.path.to_string_lossy());
                                    let mut item = needle_item.clone();
                                    item.dupes.push(haystack_item.item.path.clone());
                                    for i in haystack_item.dupes.iter() {
                                        if item.item.path != *i {
                                            trace!("adding {} to {}",
                                                   i.to_string_lossy(),
                                                   needle_item.item.path.to_string_lossy());
                                            item.dupes.push(i.clone());
                                        }
                                    }
                                    index.idx.insert(digest.clone(), item);
                                }
                            },
                            None => {}
                        }
                    }

                    // output the index
                    let mut w = writer(&output)?;
                    for item in index.idx.into_values() {
                        write!(w, "{}", item)?;
                    }
                },

                DupesCommand::ListDirs { input, output } => {
                    debug!("listing dupe dirs in {} to {}",
                           reader_name(&input)?.to_string_lossy(),
                           writer_name(&output)?.to_string_lossy());

                    // read the index from the input source with dupes
                    let ti = TreeIndexBuilder::new()
                        .with_dupes(true)
                        .from_reader(&mut reader(&input)?)
                        .build()?;
                    trace!("loaded {} items with {} dupes in the index",
                           ti.idx.len(), ti.count_dupes());

                    // create a list for the dirs
                    let mut set = HashSet::new();
                    for (_, i) in ti.idx {
                        for d in i.dupes {
                            if let Some(p) = d.parent() {
                                let pb = PathBuf::from(p);
                                set.insert(pb.clone());
                            }
                        }
                    }
                    trace!("found {} unique dupe dirs", set.len());

                    // output the list
                    let mut w = writer(&output)?;
                    for d in set.iter() {
                        writeln!(w, "{}", d.to_string_lossy())?;
                    }
                },

                DupesCommand::Size { input, output } => {
                    debug!("summing size of dups in {} to {}",
                           reader_name(&input)?.to_string_lossy(),
                           writer_name(&output)?.to_string_lossy());

                    // read the index from the input source with dupes
                    let ti = TreeIndexBuilder::new()
                        .with_dupes(true)
                        .from_reader(&mut reader(&input)?)
                        .build()?;
                    trace!("loaded {} items with {} dupes in the index",
                           ti.idx.len(), ti.count_dupes());

                    // sum up the size of all of the dupes
                    let mut size = 0u64;
                    for (_, i) in ti.idx {
                        let dupe_size = i.item.size * i.dupes.len() as u64;
                        trace!("{} saved {}", dupe_size, i.item.path.to_string_lossy());
                        size += dupe_size;
                    }

                    // output the list
                    let mut w = writer(&output)?;
                    if size > (1024 * 1024 * 1024) {
                        writeln!(w, "Total saved {} GB", size >> 30)?;
                    } else if size > (1024 * 1024) {
                        writeln!(w, "Total saved {} MB", size >> 20)?;
                    } else if size > (1024) {
                        writeln!(w, "Total saved {} KB", size >> 10)?;
                    } else {
                        writeln!(w, "Total saved {} Bytes", size)?;
                    }
                },

                DupesCommand::CopyFiles { dry_run, input, dest, output } => {
                    debug!("copy dupe files in {} to {}, logging to {}",
                         reader_name(&input)?.to_string_lossy(),
                         dir(&dest)?.to_string_lossy(),
                         writer_name(&output)?.to_string_lossy());

                    // read the index from the input source with dupes
                    let ti = TreeIndexBuilder::new()
                        .with_dupes(true)
                        .from_reader(&mut reader(&input)?)
                        .build()?;
                    trace!("loaded {} items with {} dupes in the index",
                           ti.idx.len(), ti.count_dupes());

                    let destd = dir(&dest)?;
                    trace!("is_dir == {}", destd.is_dir());
                    match destd.file_name() {
                        Some(f) => trace!("filename == {}", f.to_string_lossy()),
                        None => trace!("no file name")
                    }
                    let mut w = writer(&output)?;
                    for (digest, i) in ti.idx {
                        for d in i.dupes {
                            if d.is_file() {
                                let mut destf = destd.clone();
                                destf.push(&digest);
                                let destf = match d.extension() {
                                    Some(ext) => destf.with_extension(ext),
                                    None => destf
                                };
                                writeln!(w, "cp {} {}", d.to_string_lossy(), destf.to_string_lossy())?;
                                if !dry_run {
                                    std::fs::copy(d.as_path(), &destf)?;
                                }
                            }
                        }
                    }
                },

                DupesCommand::DeleteFiles { dry_run, input, output } => {
                    trace!("deleting dupe files in {}, logging to {}",
                         reader_name(&input)?.to_string_lossy(),
                         writer_name(&output)?.to_string_lossy());

                    // read the index from the input source with dupes
                    let ti = TreeIndexBuilder::new()
                        .with_dupes(true)
                        .from_reader(&mut reader(&input)?)
                        .build()?;
                    trace!("loaded {} items with {} dupes in the index",
                           ti.idx.len(), ti.count_dupes());

                    let mut w = writer(&output)?;
                    for (_, i) in ti.idx {
                        for d in i.dupes {
                            if d.is_file() {
                                writeln!(w, "rm {}", d.to_string_lossy())?;
                                if !dry_run {
                                    std::fs::remove_file(d.as_path())?;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
