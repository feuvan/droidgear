use std::path::Path;

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {e}"))?;
        }
    }

    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, bytes).map_err(|e| format!("Failed to write file: {e}"))?;
    std::fs::rename(&temp_path, path).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        format!("Failed to finalize file: {e}")
    })?;
    Ok(())
}
