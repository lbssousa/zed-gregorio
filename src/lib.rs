use std::fs;

use zed_extension_api::{
    self as zed, settings::LspSettings, Architecture, DownloadedFileType, GithubReleaseOptions,
    LanguageServerId, LanguageServerInstallationStatus, Os, Result,
};

/// Name of the language server, as declared in `extension.toml` and used as
/// the key of `lsp.<name>` in Zed's `settings.json`.
const SERVER_NAME: &str = "gregorio-lsp";

/// GitHub repository whose releases carry the prebuilt server binaries.
const GITHUB_REPO: &str = "AISCGre-BR/gregorio-lsp";

/// Prefix of the per-version download directories inside the extension's
/// working directory (`gregorio-lsp-<version>/`).
const DOWNLOAD_DIR_PREFIX: &str = "gregorio-lsp-";

const INSTALL_HINT: &str = "Install it manually with:\n\
    \n\
    cargo install --git https://github.com/AISCGre-BR/gregorio-lsp \\\n    \
    --tag v0.13.0 gregorio-server --bin gregorio-lsp\n\
    \n\
    and make sure ~/.cargo/bin is in your PATH, or set lsp.gregorio-lsp.binary.path \
    in Zed's settings.json.";

struct GregorioExtension {
    cached_binary_path: Option<String>,
}

/// Returns the Rust target triple used in the release asset names for the
/// current platform, together with the archive extension and type.
fn release_target() -> Result<(&'static str, &'static str, DownloadedFileType)> {
    let (os, arch) = zed::current_platform();
    let target = match (os, arch) {
        (Os::Linux, Architecture::X8664) => "x86_64-unknown-linux-musl",
        (Os::Linux, Architecture::Aarch64) => "aarch64-unknown-linux-musl",
        (Os::Mac, Architecture::X8664) => "x86_64-apple-darwin",
        (Os::Mac, Architecture::Aarch64) => "aarch64-apple-darwin",
        (Os::Windows, Architecture::X8664) => "x86_64-pc-windows-msvc",
        (Os::Windows, Architecture::Aarch64) => "aarch64-pc-windows-msvc",
        (os, arch) => {
            return Err(format!(
                "no prebuilt gregorio-lsp binary for {os:?}/{arch:?}. {INSTALL_HINT}"
            ))
        }
    };
    Ok(match os {
        Os::Windows => (target, "zip", DownloadedFileType::Zip),
        _ => (target, "tar.gz", DownloadedFileType::GzipTar),
    })
}

fn binary_name() -> &'static str {
    match zed::current_platform().0 {
        Os::Windows => "gregorio-lsp.exe",
        _ => "gregorio-lsp",
    }
}

fn is_file(path: &str) -> bool {
    fs::metadata(path).is_ok_and(|m| m.is_file())
}

/// Finds a binary left by a previous download, used when GitHub is
/// unreachable (e.g. offline).
fn previously_downloaded_binary() -> Option<String> {
    fs::read_dir(".")
        .ok()?
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with(DOWNLOAD_DIR_PREFIX))
        .map(|dir| format!("{dir}/{}", binary_name()))
        .filter(|path| is_file(path))
        .max()
}

impl GregorioExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        // 1. A binary on PATH (cargo install, Nix, system package) always wins.
        if let Some(path) = worktree.which(SERVER_NAME) {
            return Ok(path);
        }

        // 2. A binary downloaded earlier in this session.
        if let Some(path) = &self.cached_binary_path {
            if is_file(path) {
                return Ok(path.clone());
            }
        }

        // 3. Download the latest release for this platform.
        match self.download_latest_release(language_server_id) {
            Ok(path) => {
                self.cached_binary_path = Some(path.clone());
                Ok(path)
            }
            Err(err) => {
                if let Some(path) = previously_downloaded_binary() {
                    zed::set_language_server_installation_status(
                        language_server_id,
                        &LanguageServerInstallationStatus::None,
                    );
                    self.cached_binary_path = Some(path.clone());
                    return Ok(path);
                }
                zed::set_language_server_installation_status(
                    language_server_id,
                    &LanguageServerInstallationStatus::Failed(err.clone()),
                );
                Err(err)
            }
        }
    }

    fn download_latest_release(&self, language_server_id: &LanguageServerId) -> Result<String> {
        zed::set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            GITHUB_REPO,
            GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )
        .map_err(|err| format!("could not fetch gregorio-lsp releases: {err}. {INSTALL_HINT}"))?;

        let (target, extension, file_type) = release_target()?;
        let asset_name = format!("gregorio-lsp-{target}.{extension}");
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| {
                format!(
                    "gregorio-lsp {} has no asset named {asset_name}. {INSTALL_HINT}",
                    release.version
                )
            })?;

        let version_dir = format!("{DOWNLOAD_DIR_PREFIX}{}", release.version);
        let binary_path = format!("{version_dir}/{}", binary_name());

        if !is_file(&binary_path) {
            zed::set_language_server_installation_status(
                language_server_id,
                &LanguageServerInstallationStatus::Downloading,
            );
            zed::download_file(&asset.download_url, &version_dir, file_type)
                .map_err(|err| format!("failed to download {asset_name}: {err}"))?;
            if !matches!(zed::current_platform().0, Os::Windows) {
                zed::make_file_executable(&binary_path)?;
            }
            if !is_file(&binary_path) {
                return Err(format!(
                    "{asset_name} does not contain {}. {INSTALL_HINT}",
                    binary_name()
                ));
            }

            // Remove binaries of older versions.
            if let Ok(entries) = fs::read_dir(".") {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    if name.starts_with(DOWNLOAD_DIR_PREFIX) && name != version_dir {
                        fs::remove_dir_all(entry.path()).ok();
                    }
                }
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::None,
        );
        Ok(binary_path)
    }
}

impl zed::Extension for GregorioExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // An explicit `lsp.gregorio-lsp.binary` setting overrides everything.
        let binary_settings = LspSettings::for_worktree(SERVER_NAME, worktree)
            .ok()
            .and_then(|settings| settings.binary);

        let command = match binary_settings.as_ref().and_then(|b| b.path.clone()) {
            Some(path) => path,
            None => self.language_server_binary_path(language_server_id, worktree)?,
        };
        let args = binary_settings
            .as_ref()
            .and_then(|b| b.arguments.clone())
            .unwrap_or_else(|| vec!["--stdio".to_string()]);
        let env = binary_settings
            .and_then(|b| b.env)
            .map(|env| env.into_iter().collect())
            .unwrap_or_default();

        Ok(zed::Command { command, args, env })
    }
}

zed::register_extension!(GregorioExtension);
