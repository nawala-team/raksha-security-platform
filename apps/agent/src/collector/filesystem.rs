use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::warn;
use walkdir::WalkDir;

/// File integrity monitoring results.
#[derive(Debug, Clone, Serialize)]
pub struct FileIntegrityMetrics {
    pub monitored_paths: usize,
    pub files_scanned: usize,
    pub changes_detected: Vec<FileChange>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileChange {
    pub path: String,
    pub change_type: ChangeType,
    pub old_hash: Option<String>,
    pub new_hash: Option<String>,
    pub modified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
}

/// Persistent hash database for file integrity monitoring.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HashDatabase {
    pub entries: HashMap<String, FileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub hash: String,
    pub size: u64,
    pub modified: String,
}

impl HashDatabase {
    pub fn load(path: &Path) -> Self {
        if path.exists() {
            std::fs::read_to_string(path)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Paths to monitor by default (platform-specific).
pub fn default_monitored_paths() -> Vec<PathBuf> {
    if cfg!(windows) {
        vec![
            PathBuf::from(r"C:\Windows\System32\drivers\etc"),
            PathBuf::from(r"C:\Windows\System32\config"),
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            PathBuf::from("/etc"),
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/Library/LaunchDaemons"),
        ]
    } else {
        vec![
            PathBuf::from("/etc"),
            PathBuf::from("/usr/bin"),
            PathBuf::from("/usr/sbin"),
            PathBuf::from("/boot"),
        ]
    }
}

/// Compute SHA-256 hash of a file.
fn hash_file(path: &Path) -> Option<String> {
    let data = std::fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Some(hex::encode(hasher.finalize()))
}

/// Run file integrity check against the hash database.
pub fn check_integrity(db: &mut HashDatabase, paths: &[PathBuf]) -> FileIntegrityMetrics {
    let mut changes = Vec::new();
    let mut files_scanned: usize = 0;
    let mut current_files: HashMap<String, FileEntry> = HashMap::new();

    for base_path in paths {
        if !base_path.exists() {
            continue;
        }

        let walker = WalkDir::new(base_path)
            .max_depth(3)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok());

        for entry in walker {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            // Skip large files (>50MB)
            let metadata = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            if metadata.len() > 50 * 1024 * 1024 {
                continue;
            }

            files_scanned += 1;
            let path_str = path.to_string_lossy().to_string();

            let hash = match hash_file(path) {
                Some(h) => h,
                None => continue,
            };

            let modified = metadata
                .modified()
                .ok()
                .map(|t| {
                    let dt: chrono::DateTime<chrono::Utc> = t.into();
                    dt.to_rfc3339()
                })
                .unwrap_or_default();

            let new_entry = FileEntry {
                hash: hash.clone(),
                size: metadata.len(),
                modified: modified.clone(),
            };

            // Compare with existing database
            if let Some(existing) = db.entries.get(&path_str) {
                if existing.hash != hash {
                    changes.push(FileChange {
                        path: path_str.clone(),
                        change_type: ChangeType::Modified,
                        old_hash: Some(existing.hash.clone()),
                        new_hash: Some(hash),
                        modified_at: Some(modified),
                    });
                }
            } else if !db.entries.is_empty() {
                // Only report as new if db already existed
                changes.push(FileChange {
                    path: path_str.clone(),
                    change_type: ChangeType::Created,
                    old_hash: None,
                    new_hash: Some(hash),
                    modified_at: Some(modified),
                });
            }

            current_files.insert(path_str, new_entry);
        }
    }

    // Detect deletions
    for old_path in db.entries.keys() {
        if !current_files.contains_key(old_path) {
            changes.push(FileChange {
                path: old_path.clone(),
                change_type: ChangeType::Deleted,
                old_hash: Some(db.entries[old_path].hash.clone()),
                new_hash: None,
                modified_at: None,
            });
        }
    }

    // Update the database
    db.entries = current_files;

    if !changes.is_empty() {
        warn!("FIM detected {} changes", changes.len());
    }

    FileIntegrityMetrics {
        monitored_paths: paths.len(),
        files_scanned,
        changes_detected: changes,
    }
}
