use crate::metadata::{self, TrashEntry};
use chrono::Utc;
use std::path::{Path, PathBuf};
use std::{fs, io};
use uuid::Uuid;

const PROTECTED_SYSTEM: &[&str] = &[
    // Commun
    "/",
    "/dev",
    "/etc",
    "/tmp",
    "/var",
    "/opt",
    // Linux
    "/home",
    "/usr",
    "/bin",
    "/sbin",
    "/lib",
    "/lib64",
    "/root",
    "/proc",
    "/sys",
    // macOS
    "/System",
    "/Library",
    "/Applications",
    "/Users",
    "/Volumes",
    "/private",
    "/cores",
];

const PROTECTED_HOME: &[&str] = &[
    "Desktop",
    "Documents",
    "Downloads",
    "Pictures",
    "Music",
    "Videos",
    "Movies",
    "Library",
];

pub fn trash_files(paths: &[PathBuf], force: bool, verbose: bool) -> io::Result<Vec<TrashEntry>> {
    metadata::init_dirs()?;
    let mut entries = Vec::new();
    for path in paths {
        match trash_single(path) {
            Ok(entry) => {
                if verbose {
                    eprintln!(
                        "{} -> {}",
                        entry.original_path.display(),
                        entry.trash_path.display()
                    );
                }
                entries.push(entry);
            }
            Err(e) => {
                if !force {
                    eprintln!("zut: {}: {e}", path.display());
                }
            }
        }
    }
    Ok(entries)
}

fn trash_single(path: &Path) -> io::Result<TrashEntry> {
    let sym_meta = path.symlink_metadata()?;

    let canonical = if sym_meta.file_type().is_symlink() {
        path.to_path_buf()
    } else {
        path.canonicalize()?
    };

    check_protected(&canonical)?;

    let is_symlink = sym_meta.file_type().is_symlink();
    let is_dir = sym_meta.file_type().is_dir();

    let size_bytes = if is_dir {
        dir_size(path)
    } else {
        sym_meta.len()
    };

    let id = Uuid::new_v4();
    let trash_subdir = metadata::trash_dir().join(id.to_string());
    fs::create_dir_all(&trash_subdir)?;

    let file_name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "chemin sans nom de fichier"))?;
    let trash_path = trash_subdir.join(file_name);

    move_to_trash(path, &trash_path, is_dir, is_symlink)?;

    let entry = TrashEntry {
        id,
        original_path: canonical,
        trash_path,
        deleted_at: Utc::now(),
        size_bytes,
        is_dir,
        is_symlink,
    };

    metadata::append_entry(&entry)?;
    Ok(entry)
}

fn check_protected(path: &Path) -> io::Result<()> {
    let err = || io::Error::new(io::ErrorKind::PermissionDenied, "Non. Juste non.");

    if PROTECTED_SYSTEM.iter().any(|p| path == Path::new(p)) {
        return Err(err());
    }

    if let Some(home) = dirs::home_dir() {
        if path == home {
            return Err(err());
        }
        if PROTECTED_HOME.iter().any(|sub| path == home.join(sub)) {
            return Err(err());
        }
    }

    Ok(())
}

fn move_to_trash(from: &Path, to: &Path, is_dir: bool, is_symlink: bool) -> io::Result<()> {
    if let Ok(()) = fs::rename(from, to) {
        return Ok(());
    }

    if is_symlink {
        let target = fs::read_link(from)?;
        std::os::unix::fs::symlink(&target, to)?;
        fs::remove_file(from)?;
    } else if is_dir {
        copy_dir_recursive(from, to)?;
        fs::remove_dir_all(from)?;
    } else {
        fs::copy(from, to)?;
        fs::remove_file(from)?;
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let meta = entry.metadata()?;
        if meta.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else if meta.file_type().is_symlink() {
            let target = fs::read_link(&src_path)?;
            std::os::unix::fs::symlink(&target, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn dir_size(path: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    entries
        .filter_map(|e| e.ok())
        .map(|e| {
            let p = e.path();
            match e.metadata() {
                Ok(m) if m.is_dir() => dir_size(&p),
                Ok(m) => m.len(),
                Err(_) => 0,
            }
        })
        .sum()
}

pub fn undo_last() -> io::Result<TrashEntry> {
    let entries = metadata::load_entries()?;
    let entry = entries
        .iter()
        .max_by_key(|e| e.deleted_at)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "La corbeille est vide"))?
        .clone();
    restore_entry(&entry)?;
    Ok(entry)
}

pub fn undo_by_name(name: &str) -> io::Result<TrashEntry> {
    let entries = metadata::load_entries()?;
    let entry = entries
        .iter()
        .find(|e| {
            e.original_path
                .file_name()
                .is_some_and(|n| n.to_string_lossy() == name)
        })
        .or_else(|| entries.iter().find(|e| e.id.to_string().starts_with(name)))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("'{name}' introuvable dans la corbeille"),
            )
        })?
        .clone();
    restore_entry(&entry)?;
    Ok(entry)
}

pub struct PurgeStats {
    pub count: usize,
    pub total_size: u64,
}

pub fn purge_all() -> io::Result<PurgeStats> {
    let entries = metadata::load_entries()?;
    let stats = PurgeStats {
        count: entries.len(),
        total_size: entries.iter().map(|e| e.size_bytes).sum(),
    };
    for entry in &entries {
        let _ = fs::remove_dir_all(entry.trash_path.parent().unwrap_or(&entry.trash_path));
    }
    metadata::save_entries(&[])?;
    Ok(stats)
}

pub fn purge_older_than(duration: std::time::Duration) -> io::Result<PurgeStats> {
    let entries = metadata::load_entries()?;
    let cutoff =
        Utc::now() - chrono::Duration::from_std(duration).unwrap_or(chrono::Duration::zero());
    let (to_purge, to_keep): (Vec<_>, Vec<_>) =
        entries.into_iter().partition(|e| e.deleted_at < cutoff);

    let stats = PurgeStats {
        count: to_purge.len(),
        total_size: to_purge.iter().map(|e| e.size_bytes).sum(),
    };

    for entry in &to_purge {
        let _ = fs::remove_dir_all(entry.trash_path.parent().unwrap_or(&entry.trash_path));
    }

    metadata::save_entries(&to_keep)?;
    Ok(stats)
}

pub fn parse_duration(s: &str) -> io::Result<std::time::Duration> {
    let s = s.trim();
    let (num, unit) = s.split_at(s.len().saturating_sub(1));
    let num: u64 = num.parse().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Durée invalide : '{s}'"),
        )
    })?;
    let secs = match unit {
        "m" => num * 60,
        "h" => num * 3600,
        "d" => num * 86400,
        "w" => num * 604800,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unité inconnue : '{unit}' (utilise m/h/d/w)"),
            ));
        }
    };
    Ok(std::time::Duration::from_secs(secs))
}

fn restore_entry(entry: &TrashEntry) -> io::Result<()> {
    if entry.original_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Le fichier existe déjà : {}", entry.original_path.display()),
        ));
    }

    if let Some(parent) = entry.original_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::rename(&entry.trash_path, &entry.original_path)?;

    if let Some(uuid_dir) = entry.trash_path.parent() {
        let _ = fs::remove_dir(uuid_dir);
    }

    metadata::remove_entry(&entry.id)?;
    Ok(())
}
