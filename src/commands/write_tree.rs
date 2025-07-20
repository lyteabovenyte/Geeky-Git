use anyhow::Context;
use std::fs;
use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::objects::{Kind, Object};

pub(crate) fn write_tree_for(path: &Path) -> anyhow::Result<Option<[u8; 20]>> {
    let mut dir =
        fs::read_dir(path).with_context(|| format!("open directory {}", path.display()))?;
    let mut tree_object = Vec::new();
    while let Some(entry) = dir.next() {
        let entry = entry.with_context(|| format!("bad directory {}", path.display()))?;
        let file_name = entry.file_name();
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
        let temp = "temporary";
        let hash = Object {
            kind: Kind::Tree,
            expected_size: tree_object.len() as u64,
            reader: Cursor::new(tree_object),
        }
        .write(std::fs::File::create(temp).context("construct temporary file for blob.")?)
        .context("stream file into blob")?;
        let hash_hex = hex::encode(hash);
        std::fs::create_dir_all(format!(".git/objects/{}/", &hash_hex[..2]))
            .context("create subdir of .git/objects")?;

        std::fs::rename(
            temp,
            format!(".git/objects/{}/{}", &hash_hex[..2], &hash_hex[..2]),
        )
        .context("moved blob file into .git/objects")?;
        Ok(Some(hash))
    }
}
