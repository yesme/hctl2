//! Conditional reads and rate-limit feedback around the pinned native gh client.
use super::provider::Client;
use serde_json::Value;
use std::{collections::BTreeMap, path::Path, process::Command};
use task::{Result, reject};

#[derive(Default)]
pub(super) struct Reads {
    entries: BTreeMap<String, (String, Value)>,
    retry_at: u64,
    rate_failures: u32,
}

impl Reads {
    pub(super) fn ready(&self) -> Result<()> {
        if task::now() < self.retry_at {
            Err(reject(
                "PROVIDER_RATE_LIMITED",
                format!("GitHub retry after {}", self.retry_at),
                "retry_after_rate_reset",
            ))
        } else {
            Ok(())
        }
    }
    pub(super) fn api(
        &mut self,
        gh: &Path,
        host: &str,
        method: &str,
        path: &str,
        input: Option<Value>,
    ) -> Result<Option<Value>> {
        self.ready()?;
        let cached = if method == "GET" {
            self.entries.get(path).cloned()
        } else {
            None
        };
        let mut cmd = Command::new(gh);
        cmd.env("GH_PROMPT_DISABLED", "1")
            .env("NO_COLOR", "1")
            .env_remove("GH_DEBUG")
            .args(["api", "--include", "--hostname", host, "--method", method]);
        if let Some((etag, _)) = &cached {
            cmd.args(["--header", &format!("If-None-Match: {etag}")]);
        }
        if input.is_some() {
            cmd.args(["--input", "-"]);
        }
        cmd.arg(path);
        let out = repo::git::run(&mut cmd, input.map(|v| v.to_string().into_bytes()))?;
        let text = String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n");
        let (headers, body) = text.split_once("\n\n").ok_or_else(|| {
            reject(
                "PROVIDER_RESPONSE",
                "missing GitHub HTTP headers",
                "retry_read",
            )
        })?;
        let status = headers
            .lines()
            .next()
            .and_then(|s| s.split_whitespace().nth(1))
            .and_then(|s| s.parse::<u16>().ok());
        let header = |name: &str| {
            headers
                .lines()
                .filter_map(|l| l.split_once(':'))
                .find(|(k, _)| k.eq_ignore_ascii_case(name))
                .map(|(_, v)| v.trim())
        };
        let limited = header("x-ratelimit-remaining") == Some("0");
        let retry = header("retry-after").and_then(|s| s.parse::<u64>().ok());
        let secondary = status == Some(403) && body.to_ascii_lowercase().contains("rate limit");
        let rate_limited = limited || retry.is_some() || status == Some(429) || secondary;
        if rate_limited {
            self.rate_failures = self.rate_failures.saturating_add(1);
            let backoff = 60_u64 << self.rate_failures.saturating_sub(1).min(6);
            self.retry_at = task::now().saturating_add(retry.unwrap_or(backoff)).max(
                header("x-ratelimit-reset")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
            );
        } else if status.is_some_and(|s| (200..400).contains(&s)) {
            self.rate_failures = 0;
        }
        if status == Some(304) {
            return cached.map(|(_, body)| Some(body)).ok_or_else(|| {
                reject(
                    "PROVIDER_RESPONSE",
                    "304 without a cached representation",
                    "retry_read",
                )
            });
        }
        if status == Some(404) && method == "GET" {
            self.entries.remove(path);
            return Ok(None);
        }
        if !out.status.success() || !status.is_some_and(|s| (200..300).contains(&s)) {
            return Err(reject(
                if rate_limited {
                    "PROVIDER_RATE_LIMITED"
                } else if method != "GET" && matches!(status, Some(401 | 403 | 404)) {
                    "NATIVE_REJECTED"
                } else {
                    "PROVIDER_UNAVAILABLE"
                },
                format!("GitHub {method} failed (HTTP {status:?})"),
                "read_back_original_intent",
            ));
        }
        let body_size = body.len();
        let body: Value = if body.trim().is_empty() {
            Value::Null
        } else {
            serde_json::from_str(body)?
        };
        if body
            .get("errors")
            .is_some_and(|v| v.as_array().is_some_and(|v| !v.is_empty()))
        {
            return Err(reject(
                "PROVIDER_RESPONSE",
                "GitHub GraphQL returned errors; read back the original intent",
                "resume_effect",
            ));
        }
        if method == "GET" {
            if let Some(etag) = header("etag").filter(|_| body_size <= 32 * 1024) {
                // Cache is per source binding, in memory only, and bounded. A cache miss
                // just performs an ordinary current GET, never accepts an old local body.
                if self.entries.len() >= 256 {
                    self.entries.clear();
                }
                self.entries
                    .insert(path.to_owned(), (etag.to_owned(), body.clone()));
            } else {
                self.entries.remove(path);
            }
        } else {
            self.entries.clear();
        }
        Ok(Some(body))
    }
}

impl Client {
    pub(super) fn github(gh: std::path::PathBuf, host: String) -> Self {
        Self::Github {
            gh,
            host,
            reads: Default::default(),
        }
    }
}
