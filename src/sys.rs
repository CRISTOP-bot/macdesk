//! System helpers: shell commands, downloads, gsettings, dconf, backups.
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug)]
pub struct Ctx {
    pub dry_run: bool,
    pub home: PathBuf,
    pub state_dir: PathBuf,
}

impl Ctx {
    pub fn new(dry_run: bool) -> Self {
        let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
        let state_dir = home.join(".local/state/macdesk");
        Ctx { dry_run, home, state_dir }
    }

    /// Run a shell command (or print it with --dry-run).
    pub fn run(&self, cmd: &str, args: &[&str]) -> Result<(), String> {
        if self.dry_run {
            println!("  [dry] {} {}", cmd, args.join(" "));
            return Ok(());
        }
        let out = Command::new(cmd)
            .args(args)
            .output()
            .map_err(|e| format!("{cmd}: {e}"))?;
        if out.status.success() { Ok(()) } else {
            Err(format!("{cmd} {} failed: {}{}",
                args.join(" "),
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)))
        }
    }

    pub fn gsettings_set(&self, schema: &str, key: &str, value: &str) -> Result<(), String> {
        self.run("gsettings", &["set", schema, key, value])
    }

    pub fn gsettings_get(&self, schema: &str, key: &str) -> Option<String> {
        let out = Command::new("gsettings").args(["get", schema, key]).output().ok()?;
        if out.status.success() {
            Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
        } else { None }
    }

    pub fn dconf_write(&self, key: &str, value: &str) -> Result<(), String> {
        self.run("dconf", &["write", key, value])
    }

    /// Download with curl (system-provided).
    pub fn download(&self, url: &str, dest: &Path) -> Result<(), String> {
        if self.dry_run {
            println!("  [dry] curl -fSL {url} -o {}", dest.display());
            return Ok(());
        }
        if let Some(dir) = dest.parent() { fs::create_dir_all(dir).map_err(|e| e.to_string())?; }
        let out = Command::new("curl").args(["-fSL", url, "-o"]).arg(dest).output()
            .map_err(|e| format!("curl: {e}"))?;
        if out.status.success() { Ok(()) } else {
            Err(format!("download failed: {}", String::from_utf8_lossy(&out.stderr)))
        }
    }

    /// git clone into temp dir? We clone into .state/tmp and copy contents.
    pub fn git_clone(&self, url: &str, dest: &Path, shallow: bool, filter_blob_none: bool) -> Result<(), String> {
        let mut args = vec!["clone".to_string()];
        if shallow { args.push("--depth".into()); args.push("1".into()); }
        if filter_blob_none { args.push("--filter=blob:none".into()); }
        args.push(url.to_string());
        args.push(dest.to_string_lossy().to_string());
        let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
        self.run("git", &args_ref)
    }

    pub fn copy_dir(&self, src: &Path, dst: &Path) -> Result<(), String> {
        if self.dry_run {
            println!("  [dry] cp -a {} {}", src.display(), dst.display());
            return Ok(());
        }
        let parent = dst.parent().unwrap_or(Path::new(".")).to_path_buf();
        fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
        let out = Command::new("cp").args(["-aT",
            src.to_string_lossy().as_ref(),
            dst.to_string_lossy().as_ref(),
        ]).output().map_err(|e| format!("cp: {e}"))?;
        if out.status.success() { Ok(()) } else { Err(format!("cp {src:?} -> {dst:?}: {}", String::from_utf8_lossy(&out.stderr))) }
    }

    pub fn remove(&self, p: &Path) -> Result<(), String> {
        if self.dry_run {
            println!("  [dry] rm -rf {}", p.display());
            return Ok(());
        }
        if p.exists() { fs::remove_dir_all(p).map_err(|e| e.to_string())?; }
        Ok(())
    }

    /// Save a backup copy of a path into ~/.local/state/macdesk/backup/<ts>/
    pub fn backup_path(&self, p: &Path, tag: &str) -> Result<(), String> {
        if !p.exists() { return Ok(()); }
        let ts = chrono_stamp();
        let dst = self.state_dir.join("backup").join(&ts).join(tag);
        fs::create_dir_all(&dst.parent().unwrap()).map_err(|e| e.to_string())?;
        let out = Command::new("cp").args(["-aT",
            p.to_string_lossy().as_ref(),
            dst.to_string_lossy().as_ref(),
        ]).output().map_err(|e| format!("cp: {e}"))?;
        let _ = out.status;
        let _ = fs::set_permissions(&dst, fs::Permissions::from_mode(0o700));
        Ok(())
    }

    pub fn backup_gsettings(&self, schema: &str, tag: &str) -> Result<(), String> {
        let out = Command::new("gsettings").args(["list-recursively", schema]).output()
            .map_err(|e| format!("{e}"))?;
        if !out.status.success() { return Ok(()); }
        let ts = chrono_stamp();
        let file = self.state_dir.join("backup").join(&ts).join(tag);
        if let Some(parent) = file.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
        fs::write(&file, &out.stdout).map_err(|e| e.to_string())?;
        Ok(())
    }


    pub fn sudo_cached(&self) -> bool {
        Command::new("sudo").args(["-n", "true"]).status().map(|s| s.success()).unwrap_or(false)
    }

    pub fn sudo_refresh(&self) -> Result<(), String> {
        self.run("sudo", &["-v"])
    }
}

fn chrono_stamp() -> String {
    // %Y%m%d-%H%M%S from unix epoch via system `date`
    let out = Command::new("date").args(["+%Y%m%d-%H%M%S"]).output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => "0".to_string(),
    }
}
