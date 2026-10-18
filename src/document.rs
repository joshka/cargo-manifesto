//! The one file `--fix` may edit: the philosophy. The implementation is already correct.
//! Only the region between the markers is ours; every other byte belongs to the author.
use anyhow::{Context, Result, bail};
use std::{fs, io::Write, path::Path};

const BEGIN: &str = "<!-- cargo-manifesto:begin -->";
const END: &str = "<!-- cargo-manifesto:end -->";
const STALE: &str = "The implementation has advanced beyond its principles. Run cargo manifesto --fix to restore historical consistency.";

/// The byte range between exactly one begin marker and one end marker after it.
fn region(bytes: &[u8]) -> Result<Option<(usize, usize)>> {
    let find = |marker: &str| -> Vec<usize> {
        let marker = marker.as_bytes();
        bytes
            .windows(marker.len())
            .enumerate()
            .filter_map(|(i, w)| (w == marker).then_some(i))
            .collect()
    };
    match (find(BEGIN).as_slice(), find(END).as_slice()) {
        ([], []) => Ok(None),
        ([begin], [end]) if begin + BEGIN.len() <= *end => Ok(Some((begin + BEGIN.len(), *end))),
        _ => bail!(
            "malformed managed region: require exactly one begin marker followed by one end marker"
        ),
    }
}

/// Write `body` into the managed region of `path`, or with `check`, report whether it is stale.
pub fn manage(path: &Path, body: &str, check: bool) -> Result<i32> {
    let markdown = path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("md") || e.eq_ignore_ascii_case("markdown"));
    if !markdown {
        bail!(
            "managed documents must use a .md or .markdown extension; Cargo files and source code cannot be edited"
        );
    }
    let existing = match fs::read(path) {
        Ok(bytes) => Some(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e).with_context(|| format!("cannot read {}", path.display())),
    };
    let interior = format!("\n{}\n", body.trim_end());
    let replacement = match &existing {
        None => format!("{BEGIN}{interior}{END}\n").into_bytes(),
        Some(bytes) => {
            let Some((start, end)) = region(bytes)? else {
                bail!(
                    "{} has no managed markers; add them explicitly or choose a new file",
                    path.display()
                );
            };
            [&bytes[..start], interior.as_bytes(), &bytes[end..]].concat()
        }
    };
    if existing.as_deref() == Some(replacement.as_slice()) {
        return Ok(0);
    }
    if check {
        eprintln!("{STALE}");
        return Ok(1);
    }
    // Replacing a symlink would quietly relocate the philosophy.
    if fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()) {
        bail!("refusing to replace a symbolic link: {}", path.display());
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut temp =
        tempfile::NamedTempFile::new_in(parent).context("cannot create temporary document")?;
    if let Ok(metadata) = fs::metadata(path) {
        temp.as_file().set_permissions(metadata.permissions())?;
    }
    temp.write_all(&replacement)?;
    temp.as_file().sync_all()?;
    temp.persist(path)
        .with_context(|| format!("cannot replace {}", path.display()))?;
    eprintln!("Historical consistency restored in {}.", path.display());
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_arbitrary_bytes_and_permissions_outside_region() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("README.md");
        let document = |interior: &[u8]| {
            [
                b"prefix\xff\r\n".as_slice(),
                BEGIN.as_bytes(),
                interior,
                END.as_bytes(),
                b"\r\nsuffix\xfe",
            ]
            .concat()
        };
        fs::write(&file, document(b"old")).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&file, fs::Permissions::from_mode(0o640)).unwrap();
        }
        assert_eq!(manage(&file, "new", true).unwrap(), 1);
        manage(&file, "new", false).unwrap();
        assert_eq!(fs::read(&file).unwrap(), document(b"\nnew\n"));
        assert_eq!(manage(&file, "new", true).unwrap(), 0);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&file).unwrap().permissions().mode() & 0o777,
                0o640
            );
        }
    }
    #[test]
    fn rejects_every_malformed_structure() {
        for text in [
            BEGIN.to_string(),
            END.to_string(),
            format!("{END}{BEGIN}"),
            format!("{BEGIN}{BEGIN}{END}"),
            format!("{BEGIN}{END}{END}"),
        ] {
            assert!(region(text.as_bytes()).is_err(), "{text}");
        }
    }
}
