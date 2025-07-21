use crate::objects::{Kind, Object};
use anyhow::Context;
use std::cmp::Ordering;
use std::fmt::Write;
use std::fs;
use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub(crate) fn invoke(
    message: String,
    tree_hash: String,
    parent_hash: Option<String>,
) -> anyhow::Result<()> {
    let mut commit = String::new();
    writeln!(commit, "tree {tree_hash}")?;
    if let Some(parent_hash) = parent_hash {
        writeln!(commit, "parent {parent_hash}")?;
    }
    writeln!(
        commit,
        "author lyteabovenyte <lyteabovenyte@gmail.com> 1753053612 +0330"
    )?;
    writeln!(
        commit,
        "committer lyteabovenyte <lyteabovenyte@gmail.com> 1753053612 +0330"
    )?;
    writeln!(commit, "")?;
    writeln!(commit, "{message}")?;
    let hash = Object {
        kind: Kind::Commit,
        expected_size: commit.len() as u64,
        reader: Cursor::new(commit),
    }
    .write_to_objects()
    .context("write commit object")?;
    println!("{}", hex::encode(hash));
    Ok(())
}
