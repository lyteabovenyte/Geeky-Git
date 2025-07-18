use crate::objects::{Kind, Object};
use anyhow::Context;
use std::{ffi::CStr, io::BufRead};

pub(crate) fn invoke(name_only: bool, tree_hash: &str) -> anyhow::Result<()> {
    anyhow::ensure!(name_only, "mode or -p must be given.");

    let mut object = Object::read(tree_hash).context("parse out tree file")?;
    // TODO: support shortest-unique object hashes
    match object.kind {
        Kind::Tree => {
            let mut buf = Vec::new();
            loop {
                buf.clear();
                let n = object
                    .reader
                    .read_until(0, &mut buf)
                    .context("read next tree object entry")?;
                if n == 0 {
                    break;
                }
                let mode_and_name =
                    CStr::from_bytes_with_nul(&buf).context("invalid tree entry")?;
                let mut bits = mode_and_name.to_bytes().splitn(2, |&b| b == b' ');
                let mode = bits.next().expect("split always yield once.");
                let name = bits
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("tree entry has no filename"))?;
            }
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            let n = std::io::copy(&mut object.reader, &mut stdout)
                .context("write .git/objects file to the stdout")?;
            anyhow::ensure!(
                n == object.expected_size,
                ".git/objects file was not the expected size: (expected: {}, actual: {n})",
                object.expected_size
            );
        }
        _ => anyhow::bail!("don't yet know how to ls '{}'", object.kind),
    }
    Ok(())
}
