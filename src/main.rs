use anyhow::Context;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use std::env;
use std::ffi::CStr;
use std::fs;
use std::fs::write;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,
        object_hash: String,
    },
}

enum Kind {
    Blob,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory")
        }
        Command::CatFile {
            pretty_print,
            object_hash,
        } => {
            anyhow::ensure!(pretty_print, "mode or -p must be given.");
            // TODO: support shortest-unique object hashes
            let f = File::open(format!(
                ".git/objects/{}/{}",
                &object_hash[..2],
                &object_hash[2..]
            ))
            .context("open in .git/objects")?;
            let z = ZlibDecoder::new(f);
            let mut z = BufReader::new(z);
            let mut buf = Vec::new();
            z.read_until(0, &mut buf)
                .context("read header from .git/objects")?;
            let header = CStr::from_bytes_with_nul(&buf)
                .expect("know there is exactly one nul, and it's at the end.");
            let header = header
                .to_str()
                .context(".git/objects file header isn't a valid UTF-8")?;
            let Some((kind, size)) = header.split_once(' ') else {
                anyhow::bail!(".git/objects file did not start with a known type");
            };
            let kind = match kind {
                "blob" => Kind::Blob,
                _ => anyhow::bail!("we do not yet know how to print a '{kind}'"),
            };
            let size = size
                .parse::<usize>()
                .context(".git/objects file header has invalid size: {size}.")?;
            buf.clear();
            buf.resize(size, 0);
            z.read_exact(&mut buf[..])
                .context("read true contents from .git/objects")?;
            let n = z
                .read(&mut [0])
                .context("validate EOF in .git/objects file")?;
            anyhow::ensure!(n == 0, ".git/objects file had {n} trailing bytes");
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();

            match kind {
                Kind::Blob => stdout
                    .write_all(&buf)
                    .context("write content objects into stdout.")?,
            }
        }
    }
    Ok(())
}
