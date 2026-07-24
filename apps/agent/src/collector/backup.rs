use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

// ---------------------------------------------------------------------------
// Configuration types
// ---------------------------------------------------------------------------

/// Backup policy loaded from backup-policies.yml.
#[derive(Debug, Clone, Deserialize)]
pub struct BackupPolicy {
    pub name: String,
    pub paths: Vec<PathBuf>,
    pub expected_frequency_hours: u64,
    pub max_age_hours: u64,
    pub min_retention_count: usize,
    pub rpo_hours: u64,
    pub backup_type: BackupType,
    /// Optional glob pattern to match backup filenames.
    pub file_pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BackupType {
    FileSystem,
    PgDump,
    Mysqldump,
    Mongodump,
    Custom,
}

/// Configuration for the backup collector.
#[derive(Debug, Clone, Deserialize)]
pub struct BackupCollectorConfig {
    pub policies: Vec<BackupPolicy>,
    /// Additional search paths beyond policy-defined paths.
    pub extra_search_paths: Vec<PathBuf>,
    /// Whether to perform checksum verification (can be expensive).
    pub verify_checksums: bool,
}

impl Default for BackupCollectorConfig {
    fn default() -> Self {
        Self {
            policies: Vec::new(),
            extra_search_paths: Vec::new(),
            verify_checksums: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Output types
// ---------------------------------------------------------------------------

/// A discovered backup file with metadata.
#[derive(Debug, Clone, Serialize)]
pub struct BackupFile {
    pub path: String,
    pub size_bytes: u64,
    pub modified_at: DateTime<Utc>,
    pub age_hours: f64,
    pub backup_type: BackupType,
    pub checksum_sha256: Option<String>,
    pub integrity_status: IntegrityStatus,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IntegrityStatus {
    Verified,
    Unverified,
    Failed,
    Skipped,
}

/// RPO compliance result for a single policy.
#[derive(Debug, Clone, Serialize)]
pub struct RpoCompliance {
    pub policy_name: String,
    pub rpo_hours: u64,
    pub newest_backup_age_hours: Option<f64>,
    pub compliant: bool,
    pub backup_count: usize,
    pub retention_met: bool,
}

/// Full backup inventory report.
#[derive(Debug, Clone, Serialize)]
pub struct BackupInventory {
    pub collected_at: DateTime<Utc>,
    pub total_backups_found: usize,
    pub total_size_bytes: u64,
    pub backups: Vec<BackupFile>,
    pub rpo_compliance: Vec<RpoCompliance>,
    pub stale_backups: Vec<String>,
    pub alerts: Vec<BackupAlert>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackupAlert {
    pub severity: BackupAlertSeverity,
    pub policy_name: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupAlertSeverity {
    Warning,
    Critical,
}

// ---------------------------------------------------------------------------
// Backup collector implementation
// ---------------------------------------------------------------------------

/// BackupCollector discovers and validates backup files on the filesystem.
pub struct BackupCollector {
    config: BackupCollectorConfig,
}

impl BackupCollector {
    pub fn new(config: BackupCollectorConfig) -> Self {
        Self { config }
    }

    /// Default search paths for common backup locations.
    fn default_search_paths() -> Vec<PathBuf> {
        if cfg!(windows) {
            vec![
                PathBuf::from(r"C:\Backups"),
                PathBuf::from(r"C:\ProgramData\Backups"),
            ]
        } else {
            vec![
                PathBuf::from("/var/backups"),
                PathBuf::from("/backup"),
                PathBuf::from("/opt/backups"),
                PathBuf::from("/srv/backups"),
            ]
        }
    }

    /// Collect a full backup inventory report.
    pub fn collect(&self) -> BackupInventory {
        let now = Utc::now();
        let mut all_backups: Vec<BackupFile> = Vec::new();
        let mut rpo_compliance: Vec<RpoCompliance> = Vec::new();
        let mut alerts: Vec<BackupAlert> = Vec::new();

        for policy in &self.config.policies {
            let backups = self.discover_backups_for_policy(policy, now);

            let newest_age = backups
                .iter()
                .map(|b| b.age_hours)
                .fold(f64::MAX, f64::min);
            let newest_age_opt = if backups.is_empty() {
                None
            } else {
                Some(newest_age)
            };

            let compliant = newest_age_opt
                .map(|age| age <= policy.rpo_hours as f64)
                .unwrap_or(false);

            let retention_met = backups.len() >= policy.min_retention_count;

            if !compliant {
                let severity = if newest_age_opt.is_none()
                    || newest_age_opt.unwrap_or(0.0) > (policy.rpo_hours * 2) as f64
                {
                    BackupAlertSeverity::Critical
                } else {
                    BackupAlertSeverity::Warning
                };
                alerts.push(BackupAlert {
                    severity,
                    policy_name: policy.name.clone(),
                    message: format!(
                        "RPO violation: newest backup is {:.1}h old (target: {}h)",
                        newest_age_opt.unwrap_or(f64::INFINITY),
                        policy.rpo_hours
                    ),
                });
            }

            if !retention_met {
                alerts.push(BackupAlert {
                    severity: BackupAlertSeverity::Warning,
                    policy_name: policy.name.clone(),
                    message: format!(
                        "Retention not met: found {} backups, need {}",
                        backups.len(),
                        policy.min_retention_count
                    ),
                });
            }

            for backup in &backups {
                if backup.integrity_status == IntegrityStatus::Failed {
                    alerts.push(BackupAlert {
                        severity: BackupAlertSeverity::Critical,
                        policy_name: policy.name.clone(),
                        message: format!("Integrity FAILED: {}", backup.path),
                    });
                }
            }

            rpo_compliance.push(RpoCompliance {
                policy_name: policy.name.clone(),
                rpo_hours: policy.rpo_hours,
                newest_backup_age_hours: newest_age_opt,
                compliant,
                backup_count: backups.len(),
                retention_met,
            });

            all_backups.extend(backups);
        }

        // Discover in extra/default paths
        let extra_paths: Vec<PathBuf> = Self::default_search_paths()
            .into_iter()
            .chain(self.config.extra_search_paths.iter().cloned())
            .collect();

        for path in &extra_paths {
            if !path.exists() {
                continue;
            }
            let discovered = self.scan_directory(path, None, now);
            for backup in discovered {
                if !all_backups.iter().any(|b| b.path == backup.path) {
                    all_backups.push(backup);
                }
            }
        }

        let stale_backups: Vec<String> = all_backups
            .iter()
            .filter(|b| b.age_hours > 168.0)
            .map(|b| b.path.clone())
            .collect();

        let total_size: u64 = all_backups.iter().map(|b| b.size_bytes).sum();

        info!(
            "Backup inventory: {} files, {:.2} GB, {} alerts",
            all_backups.len(),
            total_size as f64 / (1024.0 * 1024.0 * 1024.0),
            alerts.len()
        );

        BackupInventory {
            collected_at: now,
            total_backups_found: all_backups.len(),
            total_size_bytes: total_size,
            backups: all_backups,
            rpo_compliance,
            stale_backups,
            alerts,
        }
    }

    /// Discover backup files matching a specific policy.
    fn discover_backups_for_policy(
        &self,
        policy: &BackupPolicy,
        now: DateTime<Utc>,
    ) -> Vec<BackupFile> {
        let mut results = Vec::new();
        for search_path in &policy.paths {
            if !search_path.exists() {
                debug!("Policy '{}': path missing: {:?}", policy.name, search_path);
                continue;
            }
            let discovered = self.scan_directory(search_path, Some(policy), now);
            results.extend(discovered);
        }
        results
    }

    /// Scan a directory for backup files.
    fn scan_directory(
        &self,
        dir: &Path,
        policy: Option<&BackupPolicy>,
        now: DateTime<Utc>,
    ) -> Vec<BackupFile> {
        let mut results = Vec::new();

        let walker = walkdir::WalkDir::new(dir)
            .max_depth(3)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok());

        for entry in walker {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            // Match file pattern if policy specifies one
            if let Some(pol) = policy {
                if let Some(ref pattern) = pol.file_pattern {
                    let fname = path.file_name().unwrap_or_default().to_string_lossy();
                    if !Self::matches_glob(pattern, &fname) {
                        continue;
                    }
                }
            }

            let backup_type = policy
                .map(|p| p.backup_type.clone())
                .unwrap_or_else(|| Self::detect_backup_type(path));

            // Skip unrecognized files in unscoped scans
            if policy.is_none() && backup_type == BackupType::Custom {
                if !Self::is_likely_backup(path) {
                    continue;
                }
            }

            let metadata = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            // Skip files > 100GB
            if metadata.len() > 100 * 1024 * 1024 * 1024 {
                continue;
            }

            let modified: DateTime<Utc> = metadata
                .modified()
                .map(|t| t.into())
                .unwrap_or(now);

            let age_hours = now
                .signed_duration_since(modified)
                .num_minutes() as f64
                / 60.0;

            let (checksum, integrity_status) = if self.config.verify_checksums {
                self.verify_integrity(path)
            } else {
                (None, IntegrityStatus::Skipped)
            };

            results.push(BackupFile {
                path: path.to_string_lossy().to_string(),
                size_bytes: metadata.len(),
                modified_at: modified,
                age_hours,
                backup_type,
                checksum_sha256: checksum,
                integrity_status,
            });
        }

        results
    }

    /// Verify backup integrity by checking for a .sha256 sidecar file.
    fn verify_integrity(&self, path: &Path) -> (Option<String>, IntegrityStatus) {
        let actual_hash = match Self::compute_sha256(path) {
            Some(h) => h,
            None => return (None, IntegrityStatus::Unverified),
        };

        let sidecar = PathBuf::from(format!("{}.sha256", path.to_string_lossy()));
        let expected_hash = Self::read_sidecar_hash(&sidecar);

        match expected_hash {
            Some(expected) => {
                if expected.to_lowercase() == actual_hash.to_lowercase() {
                    (Some(actual_hash), IntegrityStatus::Verified)
                } else {
                    warn!(
                        "Backup integrity FAILED: {} (expected: {}, got: {})",
                        path.display(), expected, actual_hash
                    );
                    (Some(actual_hash), IntegrityStatus::Failed)
                }
            }
            None => (Some(actual_hash), IntegrityStatus::Unverified),
        }
    }

    /// Read expected hash from a sidecar file.
    fn read_sidecar_hash(path: &Path) -> Option<String> {
        let content = std::fs::read_to_string(path).ok()?;
        let line = content.lines().next()?.trim();
        let hash = line.split_whitespace().next()?;
        if hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
            Some(hash.to_string())
        } else {
            None
        }
    }

    /// Compute SHA-256 streaming to avoid loading huge files in memory.
    fn compute_sha256(path: &Path) -> Option<String> {
        let file = std::fs::File::open(path).ok()?;
        let mut reader = std::io::BufReader::with_capacity(1024 * 1024, file);
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 65536];
        loop {
            let n = reader.read(&mut buffer).ok()?;
            if n == 0 { break; }
            hasher.update(&buffer[..n]);
        }
        Some(hex::encode(hasher.finalize()))
    }

    /// Detect backup type from file name.
    fn detect_backup_type(path: &Path) -> BackupType {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();

        if name.contains("pg_dump") || name.contains("pgdump") {
            return BackupType::PgDump;
        }
        if name.contains("mysqldump") || name.contains("mysql_dump") {
            return BackupType::Mysqldump;
        }
        if name.contains("mongodump") || name.contains("mongo_dump") {
            return BackupType::Mongodump;
        }

        let exts = [
            ".sql.gz", ".sql.bz2", ".sql.xz", ".sql", ".dump",
            ".bak", ".tar.gz", ".tar.bz2", ".tar.xz", ".tgz", ".zip",
        ];
        for ext in &exts {
            if name.ends_with(ext) {
                return BackupType::FileSystem;
            }
        }
        BackupType::Custom
    }

    /// Check if a file is likely a backup based on naming conventions.
    fn is_likely_backup(path: &Path) -> bool {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();

        let indicators = ["backup", "bak", "dump", "snapshot", "export"];
        let exts = [
            ".sql", ".sql.gz", ".sql.bz2", ".dump", ".bak",
            ".tar.gz", ".tar.bz2", ".tar.xz", ".tgz", ".zip", ".7z",
        ];

        if indicators.iter().any(|i| name.contains(i)) {
            return true;
        }
        exts.iter().any(|e| name.ends_with(e))
    }

    /// Simple glob matching supporting * and ? wildcards.
    fn matches_glob(pattern: &str, text: &str) -> bool {
        let p = pattern.to_lowercase();
        let t = text.to_lowercase();
        Self::glob_match(p.as_bytes(), t.as_bytes())
    }

    fn glob_match(pattern: &[u8], text: &[u8]) -> bool {
        let mut p = 0;
        let mut t = 0;
        let mut star_p = usize::MAX;
        let mut star_t = 0;

        while t < text.len() {
            if p < pattern.len() && (pattern[p] == b'?' || pattern[p] == text[t]) {
                p += 1;
                t += 1;
            } else if p < pattern.len() && pattern[p] == b'*' {
                star_p = p;
                star_t = t;
                p += 1;
            } else if star_p != usize::MAX {
                p = star_p + 1;
                star_t += 1;
                t = star_t;
            } else {
                return false;
            }
        }

        while p < pattern.len() && pattern[p] == b'*' {
            p += 1;
        }
        p == pattern.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_matching() {
        assert!(BackupCollector::matches_glob("*.sql.gz", "backup_2024.sql.gz"));
        assert!(BackupCollector::matches_glob("db_*_dump.*", "db_prod_dump.sql"));
        assert!(!BackupCollector::matches_glob("*.sql.gz", "backup.tar.gz"));
    }

    #[test]
    fn test_detect_backup_type() {
        assert_eq!(
            BackupCollector::detect_backup_type(Path::new("/backups/pg_dump_prod.sql.gz")),
            BackupType::PgDump
        );
        assert_eq!(
            BackupCollector::detect_backup_type(Path::new("/backup/mysqldump_db.sql.gz")),
            BackupType::Mysqldump
        );
        assert_eq!(
            BackupCollector::detect_backup_type(Path::new("/backup/site.tar.gz")),
            BackupType::FileSystem
        );
    }

    #[test]
    fn test_is_likely_backup() {
        assert!(BackupCollector::is_likely_backup(Path::new("daily_backup.tar.gz")));
        assert!(BackupCollector::is_likely_backup(Path::new("database.dump")));
        assert!(!BackupCollector::is_likely_backup(Path::new("readme.txt")));
    }
}
