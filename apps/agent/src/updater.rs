use serde::Deserialize;
use tracing::{error, info, warn};

#[derive(Debug, Deserialize)]
struct UpdateInfo {
    pub version: String,
    pub download_url: String,
    pub checksum_sha256: String,
}

/// Check for and apply agent updates.
pub struct Updater {
    update_url: String,
    client: reqwest::Client,
}

impl Updater {
    pub fn new(update_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to create updater HTTP client");

        Self { update_url, client }
    }

    /// Check if a new version is available.
    pub async fn check_update(&self) -> Option<UpdateInfo> {
        if self.update_url.is_empty() {
            return None;
        }

        let current_version = env!("CARGO_PKG_VERSION");
        let url = format!(
            "{}/updates/check?current={}&platform={}&arch={}",
            self.update_url,
            current_version,
            std::env::consts::OS,
            std::env::consts::ARCH,
        );

        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<UpdateInfo>().await {
                    Ok(info) => {
                        if info.version != current_version {
                            info!("Update available: {} -> {}", current_version, info.version);
                            Some(info)
                        } else {
                            None
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse update info: {e}");
                        None
                    }
                }
            }
            Ok(resp) => {
                warn!("Update check returned status: {}", resp.status());
                None
            }
            Err(e) => {
                warn!("Update check failed: {e}");
                None
            }
        }
    }

    /// Download and apply an update.
    pub async fn apply_update(&self, info: &UpdateInfo) -> Result<(), Box<dyn std::error::Error>> {
        info!("Downloading update v{}...", info.version);

        let resp = self.client.get(&info.download_url).send().await?;
        if !resp.status().is_success() {
            return Err(format!("Download failed: {}", resp.status()).into());
        }

        let bytes = resp.bytes().await?;

        // Verify checksum
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let computed = hex::encode(hasher.finalize());

        if computed != info.checksum_sha256 {
            return Err("Checksum mismatch - update rejected".into());
        }

        // Get current executable path
        let current_exe = std::env::current_exe()?;
        let backup_path = current_exe.with_extension("bak");
        let new_path = current_exe.with_extension("new");

        // Write new binary
        std::fs::write(&new_path, &bytes)?;

        // Platform-specific replacement
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&new_path, std::fs::Permissions::from_mode(0o755))?;
        }

        // Rename current -> backup, new -> current
        std::fs::rename(&current_exe, &backup_path)?;
        if let Err(e) = std::fs::rename(&new_path, &current_exe) {
            // Rollback
            error!("Failed to install update, rolling back: {e}");
            let _ = std::fs::rename(&backup_path, &current_exe);
            return Err(e.into());
        }

        // Clean up backup
        let _ = std::fs::remove_file(&backup_path);

        info!("Update applied successfully. Restart required.");
        Ok(())
    }
}
