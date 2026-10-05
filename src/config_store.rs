//! Recoverable replacement on filesystems whose rename cannot overwrite a file.
use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

pub fn recover(path: &Path) -> io::Result<()> {
    let backup = path.with_extension("previous.ini");
    if !path.exists() && backup.exists() {
        fs::rename(backup, path)?;
    }
    Ok(())
}

pub fn save(path: &Path, contents: &str) -> io::Result<()> {
    recover(path)?;
    let temp = path.with_extension("tmp");
    let backup = path.with_extension("previous.ini");
    {
        let mut file = File::create(&temp)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
    } // Close before rename: the Switch filesystem also tracks open handles.
    let had_previous = path.exists();
    if had_previous {
        if backup.exists() {
            fs::remove_file(&backup)?;
        }
        fs::rename(path, &backup)?;
    }
    if let Err(error) = fs::rename(&temp, path) {
        if had_previous {
            // Keep the last good file recoverable even if this rollback fails.
            let _ = fs::rename(&backup, path);
        }
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repeated_saves_keep_previous_and_recover_interrupted_rename() {
        let dir = std::env::temp_dir().join(format!("deluxe-config-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.ini");
        save(&path, "effects=on\n").unwrap();
        save(&path, "effects=off\n").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "effects=off\n");
        assert_eq!(
            fs::read_to_string(path.with_extension("previous.ini")).unwrap(),
            "effects=on\n"
        );
        // Interruption after parking the original but before promoting temp.
        fs::remove_file(&path).unwrap();
        recover(&path).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "effects=on\n");
        save(&path, "effects=off\n").unwrap();
        recover(&path).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "effects=off\n");
        fs::remove_dir_all(dir).unwrap();
    }
}
