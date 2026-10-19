//! Agency-owned model logins. Never import the operator's native credential cache.
use agency_proto::{PortError, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
};

const TOKEN_LIMIT: u64 = 16 * 1024;
const TOKEN_FILE_LIMIT: u64 = TOKEN_LIMIT + 512;

/// Native state is readable by the corresponding harness; bearer tokens stored
/// inside `agency_root` remain behind the existing Agency credential boundary.
pub struct Homes {
    pub agency_root: PathBuf,
    pub root: PathBuf,
    pub home: PathBuf,
    pub claude: PathBuf,
    pub codex: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct ClaudeToken {
    token: String,
    expires_at_ms: u64,
}

impl Homes {
    pub fn prepare(agency_root: &Path) -> Result<Self> {
        crate::storage::private_dir(agency_root)?;
        let agency_root = agency_root.canonicalize()?;
        let root = match std::env::var_os("HCTL2_AGENCY_HARNESS_HOME") {
            Some(path) => PathBuf::from(path),
            None => {
                let mut name = agency_root
                    .file_name()
                    .ok_or_else(|| PortError::invalid("Agency root must have a directory name"))?
                    .to_os_string();
                name.push(".harness-login");
                agency_root.with_file_name(name)
            }
        };
        Self::at(&agency_root, &root)
    }

    fn at(agency_root: &Path, root: &Path) -> Result<Self> {
        let agency_root = agency_root.canonicalize()?;
        let root = crate::confine::intended_path(root)?;
        if crate::confine::allowed_tree_contains_credential(&root, &agency_root)
            || crate::confine::sensitive_paths()
                .iter()
                .any(|path| crate::confine::allowed_tree_contains_credential(&root, path))
        {
            return Err(PortError::new(
                "MANAGED_LOGIN_ROOT_UNSAFE",
                "managed harness home overlaps Agency credentials or personal credentials",
                "choose_managed_harness_home",
            ));
        }
        crate::storage::private_dir(&root)?;
        let homes = Self {
            agency_root: agency_root.to_path_buf(),
            home: root.join("home"),
            claude: root.join("claude"),
            codex: root.join("codex"),
            root,
        };
        for dir in [&homes.home, &homes.claude, &homes.codex] {
            crate::storage::private_dir(dir)?;
        }
        Ok(homes)
    }

    fn token_path(&self) -> PathBuf {
        self.agency_root
            .join("managed-credentials/claude-code.json")
    }

    /// Check only the managed files used by startup. No harness, keychain,
    /// network login or remote token validation is performed.
    pub fn check(&self, harness: &str) -> Result<()> {
        match harness {
            "claude-code" => self.claude_token().map(drop),
            "codex-cli" => self.require_codex_login(),
            _ => Err(PortError::invalid("expected claude-code or codex-cli")),
        }
    }

    /// Human-authorized input only. The CLI refuses a terminal on stdin so a
    /// pasted token cannot be echoed; use a private file or a secret-manager pipe.
    pub fn import_claude_token(&self, reader: impl Read, expires_at_ms: u64) -> Result<()> {
        let mut token = String::new();
        reader
            .take(TOKEN_LIMIT + 1)
            .read_to_string(&mut token)
            .map_err(|_| PortError::invalid("token input must be UTF-8"))?;
        if token.len() as u64 > TOKEN_LIMIT {
            return Err(PortError::invalid("token input exceeds 16 KiB"));
        }
        let token = token.trim();
        if !token.starts_with("sk-ant-oat") || !token.bytes().all(|b| b.is_ascii_graphic()) {
            return Err(PortError::invalid("expected one Claude setup-token value"));
        }
        if expires_at_ms <= crate::storage::now_ms() {
            return Err(self.login_error("MANAGED_LOGIN_EXPIRED"));
        }
        let path = self.token_path();
        let parent = path.parent().expect("token parent");
        crate::storage::private_dir(parent)?;
        // tempfile supplies the private, same-directory temporary file and rename.
        let mut file = tempfile::NamedTempFile::new_in(parent)?;
        serde_json::to_writer(
            &mut file,
            &ClaudeToken {
                token: token.to_owned(),
                expires_at_ms,
            },
        )?;
        file.as_file_mut().sync_all()?;
        file.persist(&path).map_err(|e| e.error)?;
        fs::File::open(parent)?.sync_all()?;
        Ok(())
    }

    pub(crate) fn claude_token(&self) -> Result<String> {
        let path = self.token_path();
        let metadata =
            fs::symlink_metadata(&path).map_err(|_| self.login_error("MANAGED_LOGIN_REQUIRED"))?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.permissions().mode() & 0o077 != 0
            || metadata.len() > TOKEN_FILE_LIMIT
        {
            return Err(self.login_error("MANAGED_LOGIN_INVALID"));
        }
        let value: ClaudeToken = serde_json::from_slice(&fs::read(path)?)
            .map_err(|_| self.login_error("MANAGED_LOGIN_INVALID"))?;
        if value.expires_at_ms <= crate::storage::now_ms() {
            return Err(self.login_error("MANAGED_LOGIN_EXPIRED"));
        }
        if !value.token.starts_with("sk-ant-oat")
            || !value.token.bytes().all(|b| b.is_ascii_graphic())
        {
            return Err(self.login_error("MANAGED_LOGIN_INVALID"));
        }
        Ok(value.token)
    }

    pub(crate) fn require_codex_login(&self) -> Result<()> {
        let path = self.codex.join("auth.json");
        let metadata =
            fs::symlink_metadata(&path).map_err(|_| self.codex_error("MANAGED_LOGIN_REQUIRED"))?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.permissions().mode() & 0o077 != 0
            || metadata.len() > 1024 * 1024
        {
            return Err(self.codex_error("MANAGED_LOGIN_INVALID"));
        }
        let value: serde_json::Value = serde_json::from_slice(&fs::read(path)?)
            .map_err(|_| self.codex_error("MANAGED_LOGIN_INVALID"))?;
        let present = value["OPENAI_API_KEY"]
            .as_str()
            .is_some_and(|s| !s.is_empty())
            || value
                .pointer("/tokens/access_token")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|s| !s.is_empty());
        if !present {
            return Err(self.codex_error("MANAGED_LOGIN_REQUIRED"));
        }
        // Refresh and server-side validity are Codex's responsibility, not a new
        // OAuth implementation here. Every process pins the native file backend.
        Ok(())
    }

    fn login_error(&self, code: &str) -> PortError {
        PortError::new(
            code,
            format!(
                "Claude managed login is missing, expired or unreadable. Generate a fresh token with claude setup-token; import using agency --root {} login claude-token --expires-at <unix-seconds> < <private-token-file>. Do not put the token in arguments. Restart Agency after replacing a login.",
                crate::launch::sh_quote(&self.agency_root)
            ),
            "agency login claude-token",
        )
    }

    fn codex_error(&self, code: &str) -> PortError {
        PortError::new(
            code,
            format!(
                "Sign in to the Agency-owned Codex home: CODEX_HOME={} codex -c 'cli_auth_credentials_store=\"file\"' login. Restart Agency after replacing a login.",
                crate::launch::sh_quote(&self.codex)
            ),
            "agency login prepare",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    fn homes() -> (tempfile::TempDir, Homes) {
        let dir = tempfile::tempdir().unwrap();
        let agency = dir.path().join("agency");
        crate::storage::private_dir(&agency).unwrap();
        let homes = Homes::at(&agency, &dir.path().join("native")).unwrap();
        (dir, homes)
    }
    #[test]
    fn missing_expired_corrupt_or_public_tokens_fail_without_echoing_a_secret() {
        let (_dir, homes) = homes();
        assert_eq!(
            homes.claude_token().unwrap_err().code,
            "MANAGED_LOGIN_REQUIRED"
        );
        let marker = "sk-ant-oat01-fixture-only";
        homes
            .import_claude_token(marker.as_bytes(), u64::MAX)
            .unwrap();
        assert_eq!(homes.claude_token().unwrap(), marker);
        let path = homes.token_path();
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        let error = homes.claude_token().unwrap_err();
        assert_eq!(error.code, "MANAGED_LOGIN_INVALID");
        assert!(!error.message.contains(marker));
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(
            &path,
            serde_json::to_vec(&ClaudeToken {
                token: marker.into(),
                expires_at_ms: 1,
            })
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            homes.claude_token().unwrap_err().code,
            "MANAGED_LOGIN_EXPIRED"
        );
        fs::write(&path, marker).unwrap();
        assert_eq!(
            homes.claude_token().unwrap_err().code,
            "MANAGED_LOGIN_INVALID"
        );
    }
    #[test]
    fn imports_reject_empty_multiple_or_expired_values_and_replace_atomically() {
        let (_dir, homes) = homes();
        for value in ["", "sk-ant-oat01-a\nsk-ant-oat01-b", "not-a-token"] {
            assert!(
                homes
                    .import_claude_token(value.as_bytes(), u64::MAX)
                    .is_err()
            );
        }
        assert!(
            homes
                .import_claude_token(b"sk-ant-oat01-old".as_slice(), 1)
                .is_err()
        );
        homes
            .import_claude_token(b"sk-ant-oat01-first".as_slice(), u64::MAX)
            .unwrap();
        homes
            .import_claude_token(b"sk-ant-oat01-next\n".as_slice(), u64::MAX)
            .unwrap();
        assert_eq!(homes.claude_token().unwrap(), "sk-ant-oat01-next");
        let maximum = format!("sk-ant-oat{}", "a".repeat(TOKEN_LIMIT as usize - 10));
        assert_eq!(maximum.len() as u64, TOKEN_LIMIT);
        homes
            .import_claude_token(maximum.as_bytes(), u64::MAX)
            .unwrap();
        assert_eq!(homes.claude_token().unwrap(), maximum);
        let oversized = format!("{maximum}a");
        assert!(
            homes
                .import_claude_token(oversized.as_bytes(), u64::MAX)
                .is_err()
        );
        assert_eq!(homes.claude_token().unwrap(), maximum);
    }
    #[test]
    fn native_homes_never_grant_the_agency_credential_tree() {
        let (_dir, homes) = homes();
        assert!(!homes.root.starts_with(&homes.agency_root));
        assert!(!homes.agency_root.starts_with(&homes.root));
        assert!(Homes::at(&homes.agency_root, &homes.agency_root.join("bad")).is_err());
        assert!(Homes::at(&homes.agency_root, homes.agency_root.parent().unwrap()).is_err());
        for path in [&homes.root, &homes.home, &homes.claude, &homes.codex] {
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
    }
    #[test]
    fn codex_requires_its_own_private_native_login() {
        let (_dir, homes) = homes();
        assert_eq!(
            homes.require_codex_login().unwrap_err().code,
            "MANAGED_LOGIN_REQUIRED"
        );
        let path = homes.codex.join("auth.json");
        let mut file = crate::storage::private_file(&path).unwrap();
        file.write_all(br#"{"tokens":{"access_token":"fixture-only"}}"#)
            .unwrap();
        homes.require_codex_login().unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            homes.require_codex_login().unwrap_err().code,
            "MANAGED_LOGIN_INVALID"
        );
    }

    #[test]
    fn managed_home_rejects_personal_credentials_and_symlink_aliases() {
        let (dir, homes) = homes();
        // Only resolve names. Never open or chmod anything in the operator's home.
        let personal = PathBuf::from(std::env::var_os("HOME").unwrap()).join(".claude");
        assert_eq!(
            Homes::at(&homes.agency_root, &personal).err().unwrap().code,
            "MANAGED_LOGIN_ROOT_UNSAFE"
        );
        let alias = dir.path().join("alias");
        std::os::unix::fs::symlink(&homes.agency_root, &alias).unwrap();
        assert_eq!(
            Homes::at(&homes.agency_root, &alias).err().unwrap().code,
            "MANAGED_LOGIN_ROOT_UNSAFE"
        );
    }

    #[test]
    fn a_token_symlink_is_not_a_managed_login() {
        let (_dir, homes) = homes();
        homes
            .import_claude_token(b"sk-ant-oat01-fixture-only".as_slice(), u64::MAX)
            .unwrap();
        let path = homes.token_path();
        let other = homes.agency_root.join("other.json");
        fs::rename(&path, &other).unwrap();
        std::os::unix::fs::symlink(other, path).unwrap();
        assert_eq!(
            homes.claude_token().unwrap_err().code,
            "MANAGED_LOGIN_INVALID"
        );
    }
}
