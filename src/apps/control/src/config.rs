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
    let Some(object) = value.as_object() else {
        return Err(format!(
            "{}: configuration must be a JSON object, got {value}",
            path.display()
        ));
    };
    let Some(setting) = object.get("secret_backend") else {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMPS: AtomicU64 = AtomicU64::new(0);

    fn root_with(contents: Option<&str>) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "hctl2-config-{}-{}",
            std::process::id(),
            TEMPS.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).unwrap();
        if let Some(contents) = contents {
            std::fs::write(root.join(CONFIG_FILE), contents).unwrap();
        }
        root
    }

    #[test]
    fn non_object_configuration_is_rejected_instead_of_ignored() {
        // Only objects can carry the key, and the writer only writes objects: reading a
        // non-object as "no setting" silently ran the detected default instead.
        for contents in ["[1, 2, 3]", "\"user-file\"", "42", "null", "true"] {
            let root = root_with(Some(contents));
            let error = secret_backend(&root).unwrap_err();
            assert!(
                error.contains("configuration must be a JSON object"),
                "{contents}: {error}"
            );
            let _ = std::fs::remove_dir_all(root);
        }
    }

    #[test]
    fn absent_file_or_absent_key_keeps_the_detected_default() {
        let missing = root_with(None);
        assert_eq!(secret_backend(&missing).unwrap(), None);
        let _ = std::fs::remove_dir_all(missing);
        let empty = root_with(Some("{\"other\": 1}"));
        assert_eq!(secret_backend(&empty).unwrap(), None);
        let _ = std::fs::remove_dir_all(empty);
    }

    #[test]
    fn unknown_spelling_or_non_string_value_is_rejected() {
        let unknown = root_with(Some("{\"secret_backend\": \"keychain\"}"));
        assert!(
            secret_backend(&unknown)
                .unwrap_err()
                .contains("unknown secret_backend")
        );
        let _ = std::fs::remove_dir_all(unknown);
        let wrong_type = root_with(Some("{\"secret_backend\": 7}"));
        assert!(
            secret_backend(&wrong_type)
                .unwrap_err()
                .contains("secret_backend must be a string")
        );
        let _ = std::fs::remove_dir_all(wrong_type);
    }

    #[test]
    fn a_known_backend_survives_unrelated_keys() {
        let root = root_with(Some("{\"secret_backend\": \"user-file\", \"future\": 1}"));
        assert_eq!(
            secret_backend(&root).unwrap(),
            Some(SecretBackend::UserFile)
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
