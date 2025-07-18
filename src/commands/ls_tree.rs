use anyhow::Context;
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

enum Kind {
    Blob,
}

pub(crate) fn invoke(name_only: bool) -> anyhow::Result<()> {
    anyhow::ensure!(name_only, "only --name-only is supproted for now");
    todo!();
    Ok(())
}
