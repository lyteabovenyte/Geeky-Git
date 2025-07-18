use anyhow::Context;
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

enum Kind {
    Blob,
}

pub(crate) fn invoke(pretty_print: bool, object_hash: &str) -> anyhow::Result<()> {
    anyhow::ensure!(pretty_print, "mode or -p must be given.");
    // TODO: support shortest-unique object hashes
    let f = File::open(format!(
        ".git/objects/{}/{}",
        &object_hash[..2],
        &object_hash[2..]
    ))
    .context("open in .git/objects")?;
    let zl = ZlibDecoder::new(f);
    let mut z = BufReader::new(zl);
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
        .parse::<u64>()
        .context(".git/objects file header has invalid size: {size}.")?;
    let mut z = z.take(size);
    match kind {
        Kind::Blob => {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            let n = std::io::copy(&mut z, &mut stdout)
                .context("write .git/objects file to the stdout")?;
            anyhow::ensure!(
                n == size,
                ".git/objects file was not the expected size: (expected: {size}, actual: {n})"
            );
        }
    }
    Ok(())
}
