//! Reject output/input aliases before starting a process or writing artifacts.
use std::{
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

fn resolved(path: &Path) -> Result<PathBuf, String> {
    match std::fs::canonicalize(path) {
        Ok(path) => Ok(path),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            let absolute = if path.is_absolute() {
                path.to_owned()
            } else {
                std::env::current_dir()
                    .map_err(|_| "cannot resolve working directory")?
                    .join(path)
            };
            let parent = absolute.parent().ok_or("cannot resolve output path")?;
            let mut base = resolved(parent)?;
            match absolute.components().next_back().ok_or("empty path")? {
                Component::Normal(name) => base.push(name),
                Component::ParentDir => {
                    base.pop();
                }
                Component::CurDir => {}
                _ => return Err("invalid output path".into()),
            }
            Ok(base)
        }
        Err(_) => Err("cannot resolve artifact path".into()),
    }
}

pub fn ensure_separate(inputs: &[&Path], outputs: &[&Path]) -> Result<(), String> {
    let inputs = inputs
        .iter()
        .map(|p| resolved(p))
        .collect::<Result<Vec<_>, _>>()?;
    let mut seen = Vec::new();
    for output in outputs {
        if output.as_os_str().is_empty() || output.is_dir() {
            return Err("output must be a file path".into());
        }
        let output = resolved(output)?;
        if inputs.contains(&output) || seen.contains(&output) {
            return Err("output paths must differ from input files and from each other".into());
        }
        seen.push(output);
    }
    Ok(())
}
