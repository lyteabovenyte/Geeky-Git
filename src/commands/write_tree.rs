use anyhow::Context;
use hex::encode;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::{fs, io::Cursor};

use crate::objects::{Kind, Object};

pub(crate) fn write_tree_for(path: &Path) -> anyhow::Result<Option<[u8; 20]>> {
    let mut dir =
        fs::read_dir(path).with_context(|| format!("open directory {}", path.display()))?;

    let mut entries = Vec::new();
    while let Some(entry) = dir.next() {
        let entry = entry.with_context(|| format!("bad directory {}", path.display()))?;
        entries.push(entry);
    }
    entries.sort_unstable_by(|a, b| {
        let afn = a.file_name();
        let bfn = b.file_name();
        let mut afn = afn.into_encoded_bytes();
        let mut bfn = bfn.into_encoded_bytes();
        afn.push(0xff);
        bfn.push(0xff);
        afn.cmp(&bfn)
    });

    let mut tree_object = Vec::new();
    for entry in entries {
        let file_name = entry.file_name();
        if file_name == ".git" {
            // skip the .git directory
            continue;
        }
        let meta = entry.metadata().context("metadata for directory entry")?;
        let mode = if meta.is_dir() {
            "40000"
        } else if meta.is_symlink() {
            "120000"
        } else if (meta.permissions().mode() & 0o111) != 0 {
            // has at least one executable bit set
            "100755"
        } else {
            "100644"
        };

        let path = entry.path();
        let hash = if meta.is_dir() {
            let Some(hash) = write_tree_for(&path)? else {
                // don't include in parent
                continue;
            };
            hash
        } else {
            let temp = "temporary";
            let hash = Object::blob_from_file(&path)
                .context("open blob input file")?
                .write(std::fs::File::create(temp).context("construct temporary file for tree.")?)
                .context("stream tree object into tree object file")?;
            let hash_hex = hex::encode(hash);
            std::fs::create_dir_all(format!(".git/objects/{}/", &hash_hex[..2]))
                .context("create subdir of .git/objects")?;

            std::fs::rename(
                temp,
                format!(".git/objects/{}/{}", &hash_hex[..2], &hash_hex[..2]),
            )
            .context("moved tree file into .git/objects")?;
            hash
        };

        tree_object.extend(mode.as_bytes());
        tree_object.extend(file_name.as_encoded_bytes());
        tree_object.push(0);
        tree_object.extend(hash);
    }
    if tree_object.is_empty() {
        Ok(None)
    } else {
        Ok(Some(
            Object {
                kind: Kind::Tree,
                expected_size: tree_object.len() as u64,
                reader: Cursor::new(tree_object),
            }
            .write_to_objects()
            .context("write tree object")?,
        ))
    }
}

pub(crate) fn invoke() -> anyhow::Result<()> {
    let Some(hash) = write_tree_for(Path::new(".")).context("construct root tree object")? else {
        anyhow::bail!("asked to make tree object for empty tree.");
    };
    println!("{}", encode(hash));
    Ok(())
}
