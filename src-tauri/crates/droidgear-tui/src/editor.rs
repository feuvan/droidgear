use anyhow::Context;
use std::path::Path;
use std::process::Command;

pub fn editor_command() -> String {
    std::env::var("VISUAL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| std::env::var("EDITOR").ok().filter(|s| !s.trim().is_empty()))
        .unwrap_or_else(|| "vi".to_string())
}

pub fn open_in_editor(path: &Path) -> anyhow::Result<()> {
    let editor = editor_command();
    let mut parts = editor.split_whitespace();
    let bin = parts.next().context("Empty editor command")?;

    let mut cmd = Command::new(bin);
    for p in parts {
        cmd.arg(p);
    }
    cmd.arg(path);

    let status = cmd.status().with_context(|| format!("Failed to run editor: {editor}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Editor exited with status: {status:?}"))
    }
}

pub fn pager_command() -> String {
    std::env::var("PAGER")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "less -R".to_string())
}

pub fn open_in_pager(path: &Path) -> anyhow::Result<()> {
    let pager = pager_command();
    let mut parts = pager.split_whitespace();
    let bin = parts.next().context("Empty pager command")?;

    let mut cmd = Command::new(bin);
    for p in parts {
        cmd.arg(p);
    }
    cmd.arg(path);

    let status = cmd.status().with_context(|| format!("Failed to run pager: {pager}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Pager exited with status: {status:?}"))
    }
}
