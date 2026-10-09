//! What the orchestration needs from a code platform to integrate one review request. An
//! adapter reads the target and the request, and asks for one merge; it decides nothing —
//! the same rules run over every platform in `integrate_platform`.
use repo::integration::{ProtectionSnapshot, Strategy};
use repo::{Result, reject};
use serde_json::Value;

/// A target branch as the platform reports it.
#[derive(Debug)]
pub(crate) struct Target {
    pub head: Option<String>,
    pub protection: ProtectionSnapshot,
}

/// What the platform knows about one review request before a merge attempt or on readback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReviewRequest {
    pub index: u64,
    pub state: String,
    pub merged: bool,
    pub merge_commit_sha: Option<String>,
    pub head_sha: Option<String>,
    pub base_branch: Option<String>,
    pub raw: Value,
}

pub(crate) trait PlatformTarget {
    /// The target branch's head and the protection in force for it.
    fn observe(&self, full_name: &str, target_ref: &str) -> Result<Target>;
    /// One review request; `None` when the platform has no such request.
    fn review_request(&self, full_name: &str, index: u64) -> Result<Option<ReviewRequest>>;
    /// Refuse a strategy the platform cannot carry out on the exact candidate.
    fn check_strategy(&self, strategy: Strategy) -> Result<()>;
    /// Ask the platform to merge the request with the source head pinned to `head`. A refusal
    /// before any write comes back as `NATIVE_REJECTED` / `NATIVE_CONFLICT`; any other error
    /// may have reached the platform.
    fn request_merge(
        &self,
        full_name: &str,
        index: u64,
        strategy: Strategy,
        head: &str,
        message: &str,
    ) -> Result<()>;
}

/// A branch name from a fully qualified ref.
pub(crate) fn branch(target_ref: &str) -> Result<&str> {
    target_ref.strip_prefix("refs/heads/").ok_or_else(|| {
        reject(
            "INVALID_INPUT",
            "platform targets are branches (refs/heads/...)",
            "correct_input",
        )
    })
}
