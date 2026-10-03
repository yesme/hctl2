//! Control's on-disk configuration: `<root>/control.json`.
//!
//! The file is optional and small; `hctl2 init` and `hctl2 start` write it, the
//! daemon only reads it. Unknown keys are preserved by the writer so the shape can
//! grow without clobbering another setting.

use std::path::Path;

use foundation::{SecretBackend, SecretStore};

/// File name of the control configuration inside the control root.
pub const CONFIG_FILE: &str = "control.json";

/// The configured secret backend, or `None` for the detected default.
///
/// # Errors
///
/// Returns an error when the file exists but is unreadable, not a JSON object, or
/// names a backend this build does not know.
pub fn secret_backend(root: &Path) -> Result<Option<SecretBackend>, String> {
    let path = root.join(CONFIG_FILE);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))?;
    let Some(setting) = value.get("secret_backend") else {
        return Ok(None);
    };
    let Some(text) = setting.as_str() else {
        return Err(format!(
            "{}: secret_backend must be a string, got {setting}",
            path.display()
        ));
    };
    SecretBackend::parse(text)
        .map(Some)
        .ok_or_else(|| format!("{}: unknown secret_backend '{text}'", path.display()))
}

/// The secret store for this control root, resolved from the configuration.
///
/// # Errors
///
/// Returns an error when the configuration is invalid or an explicitly configured
/// `system-keyring` is unavailable on this machine.
pub fn secret_store(root: &Path) -> Result<SecretStore, String> {
    let requested = secret_backend(root)?;
    SecretStore::select("hctl2", root.join("secrets"), requested)
        .map_err(|error| format!("{}: {error}", root.join(CONFIG_FILE).display()))
}
