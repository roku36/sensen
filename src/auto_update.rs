//! Auto-update system using GitHub Releases.
//!
//! Checks for new versions on Title screen entry, shows a notification banner,
//! and allows one-click update + restart.
//! Downloads the full zip (binary + assets) and replaces everything.

#[cfg(not(target_family = "wasm"))]
mod inner {
    use bevy::prelude::*;
    use bevy::tasks::{IoTaskPool, Task, block_on, poll_once};
    use std::path::Path;

    use crate::screens::Screen;

    const REPO_OWNER: &str = "roku36";
    const REPO_NAME: &str = "sensen";
    const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

    /// Current status of the auto-update system.
    #[derive(Resource, Default)]
    pub enum UpdateStatus {
        #[default]
        Idle,
        Checking,
        UpToDate,
        Available {
            latest_version: String,
        },
        Downloading,
        RestartRequired,
        Error(String),
    }

    /// Trigger resource: insert to start downloading the update.
    #[derive(Resource)]
    pub struct TriggerUpdateDownload;

    #[derive(Resource)]
    struct UpdateCheckTask(Task<CheckResult>);

    #[derive(Resource)]
    struct UpdateDownloadTask(Task<Result<String, String>>);

    enum CheckResult {
        UpToDate,
        Available { version: String },
        Error(String),
    }

    fn platform_target() -> &'static str {
        if cfg!(target_os = "macos") {
            "macos"
        } else if cfg!(target_os = "windows") {
            "windows"
        } else {
            "linux"
        }
    }

    fn trigger_update_check(mut commands: Commands) {
        info!("Checking for updates...");
        let task = IoTaskPool::get().spawn(async move {
            match check_for_update() {
                Ok(r) => r,
                Err(e) => CheckResult::Error(format!("{e}")),
            }
        });
        commands.insert_resource(UpdateCheckTask(task));
        commands.insert_resource(UpdateStatus::Checking);
    }

    fn check_for_update() -> Result<CheckResult, Box<dyn std::error::Error + Send + Sync>> {
        let releases = self_update::backends::github::ReleaseList::configure()
            .repo_owner(REPO_OWNER)
            .repo_name(REPO_NAME)
            .build()?
            .fetch()?;

        let latest = match releases.first() {
            Some(r) => r,
            None => return Ok(CheckResult::UpToDate),
        };

        let latest_ver = latest.version.trim_start_matches('v');
        let current_ver = CURRENT_VERSION.trim_start_matches('v');

        if self_update::version::bump_is_greater(current_ver, latest_ver)
            .unwrap_or(false)
        {
            Ok(CheckResult::Available {
                version: latest_ver.to_string(),
            })
        } else {
            Ok(CheckResult::UpToDate)
        }
    }

    fn poll_update_check(mut commands: Commands, task: Option<ResMut<UpdateCheckTask>>) {
        let Some(mut task) = task else { return };
        if !task.0.is_finished() {
            return;
        }
        let result = block_on(poll_once(&mut task.0));
        commands.remove_resource::<UpdateCheckTask>();

        if let Some(result) = result {
            match result {
                CheckResult::UpToDate => {
                    info!("App is up to date (v{CURRENT_VERSION})");
                    commands.insert_resource(UpdateStatus::UpToDate);
                }
                CheckResult::Available { version } => {
                    info!("Update available: v{CURRENT_VERSION} → v{version}");
                    commands.insert_resource(UpdateStatus::Available {
                        latest_version: version,
                    });
                }
                CheckResult::Error(e) => {
                    warn!("Update check failed: {e}");
                    commands.insert_resource(UpdateStatus::Error(e));
                }
            }
        }
    }

    fn start_download_on_trigger(
        mut commands: Commands,
        trigger: Option<Res<TriggerUpdateDownload>>,
        existing: Option<Res<UpdateDownloadTask>>,
    ) {
        if trigger.is_none() || existing.is_some() {
            return;
        }
        commands.remove_resource::<TriggerUpdateDownload>();
        info!("Starting update download...");

        let task = IoTaskPool::get().spawn(async move {
            match perform_update() {
                Ok(v) => Ok(v),
                Err(e) => Err(format!("{e}")),
            }
        });
        commands.insert_resource(UpdateDownloadTask(task));
        commands.insert_resource(UpdateStatus::Downloading);
    }

    /// Recursively copy a directory.
    fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let dst_path = dst.join(entry.file_name());
            if entry.file_type()?.is_dir() {
                copy_dir_all(&entry.path(), &dst_path)?;
            } else {
                std::fs::copy(entry.path(), &dst_path)?;
            }
        }
        Ok(())
    }

    fn perform_update() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let target = platform_target();
        let bin_name = env!("CARGO_PKG_NAME");

        // 1. Find the latest release and matching asset.
        let releases = self_update::backends::github::ReleaseList::configure()
            .repo_owner(REPO_OWNER)
            .repo_name(REPO_NAME)
            .build()?
            .fetch()?;

        let latest = releases.first().ok_or("No releases found")?;
        let asset = latest
            .assets
            .iter()
            .find(|a| a.name.contains(target) && a.name.ends_with(".zip"))
            .ok_or_else(|| format!("No asset found for platform '{target}'"))?;

        info!("Downloading: {}", asset.name);

        // 2. Download the full zip via ureq (streaming to file).
        let tmp_dir = std::env::temp_dir().join("sensen_update");
        if tmp_dir.exists() {
            std::fs::remove_dir_all(&tmp_dir)?;
        }
        std::fs::create_dir_all(&tmp_dir)?;

        let zip_path = tmp_dir.join(&asset.name);
        let response = ureq::get(&asset.download_url).call()?;
        {
            let mut reader = response.into_reader();
            let mut file = std::fs::File::create(&zip_path)?;
            std::io::copy(&mut reader, &mut file)?;
        }

        // 3. Extract the full zip.
        let extract_dir = tmp_dir.join("extract");
        std::fs::create_dir_all(&extract_dir)?;
        {
            let file = std::fs::File::open(&zip_path)?;
            let mut archive = zip::ZipArchive::new(file)?;
            archive.extract(&extract_dir)?;
        }

        // 4. Locate the new binary and assets in the extracted files.
        //    Zip structure:
        //      macOS:   sensen.app/Contents/MacOS/sensen  + .../assets/
        //      Linux:   sensen/sensen                     + sensen/assets/
        //      Windows: sensen/sensen.exe                 + sensen/assets/
        let bin_filename = if cfg!(target_os = "windows") {
            format!("{bin_name}.exe")
        } else {
            bin_name.to_string()
        };

        let new_binary = if target == "macos" {
            extract_dir.join(format!("{bin_name}.app/Contents/MacOS/{bin_filename}"))
        } else {
            extract_dir.join(format!("{bin_name}/{bin_filename}"))
        };

        if !new_binary.exists() {
            return Err(format!("Binary not found at: {}", new_binary.display()).into());
        }

        let new_assets_dir = new_binary.parent().unwrap().join("assets");

        // 5. Replace assets alongside the current executable.
        let current_exe = std::env::current_exe()?;
        let install_dir = current_exe
            .parent()
            .ok_or("Cannot determine install directory")?;
        let current_assets = install_dir.join("assets");

        if new_assets_dir.is_dir() {
            info!("Updating assets...");
            if current_assets.exists() {
                std::fs::remove_dir_all(&current_assets)?;
            }
            copy_dir_all(&new_assets_dir, &current_assets)?;
        }

        // 6. Atomically replace the running binary.
        info!("Replacing binary...");
        self_replace::self_replace(&new_binary)?;

        // 7. Ensure executable permission (Unix).
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(
                &current_exe,
                std::fs::Permissions::from_mode(0o755),
            );
        }

        // 8. Cleanup temp files.
        let _ = std::fs::remove_dir_all(&tmp_dir);

        let version = latest.version.trim_start_matches('v').to_string();
        info!("Update to v{version} complete!");
        Ok(version)
    }

    fn poll_update_download(mut commands: Commands, task: Option<ResMut<UpdateDownloadTask>>) {
        let Some(mut task) = task else { return };
        if !task.0.is_finished() {
            return;
        }
        let result = block_on(poll_once(&mut task.0));
        commands.remove_resource::<UpdateDownloadTask>();

        if let Some(result) = result {
            match result {
                Ok(version) => {
                    info!("Update to v{version} complete! Restart required.");
                    commands.insert_resource(UpdateStatus::RestartRequired);
                }
                Err(e) => {
                    error!("Update download failed: {e}");
                    commands.insert_resource(UpdateStatus::Error(e));
                }
            }
        }
    }

    pub fn plugin(app: &mut App) {
        app.init_resource::<UpdateStatus>();
        app.add_systems(OnEnter(Screen::Title), trigger_update_check);
        app.add_systems(
            Update,
            (
                poll_update_check,
                start_download_on_trigger,
                poll_update_download,
            )
                .run_if(in_state(Screen::Title)),
        );
    }
}

#[cfg(target_family = "wasm")]
mod inner {
    use bevy::prelude::*;

    pub fn plugin(_app: &mut App) {}
}

pub use inner::plugin;
#[cfg(not(target_family = "wasm"))]
pub use inner::{TriggerUpdateDownload, UpdateStatus};
