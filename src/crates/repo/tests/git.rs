mod common;
use common::*;
use repo::{LocalInput, Origin, Platform, git::Git, prepare};
use std::{path::Path, process::Command};
fn git(path: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().into()
}
fn input(path: &Path) -> LocalInput {
    LocalInput {
        machine: "control".into(),
        path: path.into(),
        in_place: false,
        extra_refs: vec![],
        publish_governance: false,
    }
}

#[test]
fn unreadable_or_wrong_machine_is_not_pure_local_and_empty_is_valid() {
    let temp = Temp::new();
    let g = Git::discover().unwrap();
    assert!(g.inspect(&input(&temp.0)).is_err());
    git(&temp.0, &["init", "-b", "main"]);
    assert!(g.inspect(&input(&temp.0)).unwrap().refs.is_empty());
    let mut i = input(&temp.0);
    i.machine = "other-machine".into();
    assert_eq!(g.inspect(&i).unwrap_err().code, "MACHINE_UNREACHABLE");
    let mut r = request(Platform::Local);
    r.platform = None;
    r.local = Some(input(&temp.0));
    assert_eq!(
        prepare(
            r.clone(),
            Some(g.inspect(r.local.as_ref().unwrap()).unwrap())
        )
        .unwrap()
        .platform,
        Platform::Local
    );
    git(
        &temp.0,
        &["remote", "add", "origin", "https://github.com/a/b.git"],
    );
    assert_eq!(
        prepare(
            r.clone(),
            Some(g.inspect(r.local.as_ref().unwrap()).unwrap())
        )
        .unwrap_err()
        .code,
        "REMOTE_CHOICE_REQUIRED"
    );
    r.origin = Origin::Independent;
    r.platform = Some(Platform::Local);
    assert!(
        prepare(
            r.clone(),
            Some(g.inspect(r.local.as_ref().unwrap()).unwrap())
        )
        .is_ok()
    );
    assert_eq!(
        git(&temp.0, &["remote", "get-url", "origin"]),
        "https://github.com/a/b.git"
    );
}

fn commit(path: &Path, message: &str) -> String {
    git(
        path,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "--allow-empty",
            "-m",
            message,
        ],
    );
    git(path, &["rev-parse", "HEAD"])
}

#[test]
fn default_push_and_independent_copy_preserve_input_and_exclude_other_refs() {
    let source = Temp::new();
    let target = Temp::new();
    let copy = Temp::new();
    git(&source.0, &["init", "-b", "main"]);
    std::fs::write(source.0.join("tracked"), "frozen contents").unwrap();
    git(&source.0, &["add", "tracked"]);
    let sha = commit(&source.0, "one");
    git(&source.0, &["branch", "unpublished"]);
    git(&source.0, &["update-ref", "refs/hctl2/private", &sha]);
    git(
        &source.0,
        &["remote", "add", "origin", "https://github.com/a/b.git"],
    );
    std::fs::write(source.0.join("untracked"), "must survive").unwrap();
    git(&target.0, &["init", "--bare"]);
    let g = Git::discover().unwrap();
    let snapshot = g.inspect(&input(&source.0)).unwrap();
    assert_eq!(snapshot.refs.len(), 1);
    let copied = g.copy(&snapshot, &copy.0).unwrap();
    assert_eq!(
        std::fs::read_to_string(copied.join("tracked")).unwrap(),
        "frozen contents"
    );
    g.deliver(&copied, &snapshot, target.0.to_str().unwrap(), None)
        .unwrap();
    g.deliver(&copied, &snapshot, target.0.to_str().unwrap(), None)
        .unwrap();
    assert_eq!(
        git(&target.0, &["for-each-ref", "--format=%(refname)"]),
        "refs/heads/main"
    );
    assert_eq!(git(&target.0, &["rev-parse", "refs/heads/main"]), sha);
    assert_eq!(
        git(&source.0, &["remote", "get-url", "origin"]),
        "https://github.com/a/b.git"
    );
    assert_eq!(
        std::fs::read_to_string(source.0.join("untracked")).unwrap(),
        "must survive"
    );
    assert_eq!(git(&source.0, &["rev-parse", "HEAD"]), sha);
    assert!(git(&copy.0, &["remote"]).is_empty());
    let next = commit(&source.0, "two");
    let new_snapshot = g.inspect(&input(&source.0)).unwrap();
    assert_ne!(sha, next);
    assert_eq!(
        g.deliver(&source.0, &new_snapshot, target.0.to_str().unwrap(), None)
            .unwrap_err()
            .code,
        "INITIAL_REF_CONFLICT"
    );
    assert_eq!(git(&target.0, &["rev-parse", "refs/heads/main"]), sha);
    assert_eq!(
        g.recheck(&input(&source.0), &snapshot).unwrap_err().code,
        "LOCAL_INPUT_CHANGED"
    );
}

#[test]
fn detached_head_extra_refs_and_private_material_history() {
    let source = Temp::new();
    git(&source.0, &["init", "-b", "feature"]);
    let sha = commit(&source.0, "base");
    git(&source.0, &["checkout", "--detach", &sha]);
    let g = Git::discover().unwrap();
    assert_eq!(
        g.inspect(&input(&source.0)).unwrap().head_branch,
        "refs/heads/main"
    );
    git(&source.0, &["branch", "extra"]);
    let mut i = input(&source.0);
    i.extra_refs.push("refs/heads/extra".into());
    assert_eq!(g.inspect(&i).unwrap().refs.len(), 2);
    i.extra_refs.push("refs/hctl2/private".into());
    assert_eq!(g.inspect(&i).unwrap_err().code, "PRIVATE_REF");
    std::fs::create_dir(source.0.join(".memo")).unwrap();
    std::fs::write(source.0.join(".memo/private.md"), "private").unwrap();
    git(&source.0, &["add", ".memo"]);
    commit(&source.0, "private material");
    git(&source.0, &["rm", ".memo/private.md"]);
    commit(&source.0, "remove material");
    let snapshot = g.inspect(&input(&source.0)).unwrap();
    let mut registration = request(Platform::Local);
    registration.local = Some(input(&source.0));
    assert_eq!(
        prepare(registration.clone(), Some(snapshot.clone()))
            .unwrap_err()
            .code,
        "GOVERNANCE_PUBLICATION_REQUIRED"
    );
    registration.platform = Some(Platform::None);
    registration.platform_path = None;
    assert!(
        prepare(registration, Some(snapshot)).is_ok(),
        "inspection alone does not publish history"
    );
    let mut publish = input(&source.0);
    publish.publish_governance = true;
    assert_eq!(
        g.inspect(&publish).unwrap().governance_paths,
        vec![".memo/private.md"]
    );
}

#[test]
fn explicitly_selected_annotated_tag_preserves_its_object() {
    let source = Temp::new();
    let target = Temp::new();
    git(&source.0, &["init", "-b", "main"]);
    commit(&source.0, "base");
    git(
        &source.0,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "tag",
            "-a",
            "release",
            "-m",
            "annotated",
        ],
    );
    git(&target.0, &["init", "--bare"]);
    let mut selected = input(&source.0);
    selected.extra_refs.push("refs/tags/release".into());
    let g = Git::discover().unwrap();
    let snapshot = g.inspect(&selected).unwrap();
    g.deliver(&source.0, &snapshot, target.0.to_str().unwrap(), None)
        .unwrap();
    assert_eq!(
        git(&target.0, &["cat-file", "-t", "refs/tags/release"]),
        "tag"
    );
    assert_eq!(
        git(&source.0, &["rev-parse", "refs/tags/release"]),
        git(&target.0, &["rev-parse", "refs/tags/release"])
    );
}

#[test]
fn in_place_selection_needs_independent_origin_and_frozen_remote() {
    let source = Temp::new();
    git(&source.0, &["init", "-b", "main"]);
    git(
        &source.0,
        &["remote", "add", "origin", "https://github.com/a/b.git"],
    );
    let mut i = input(&source.0);
    i.in_place = true;
    let g = Git::discover().unwrap();
    let snapshot = g.inspect(&i).unwrap();
    let mut r = request(Platform::Local);
    r.local = Some(i);
    r.origin = Origin::Local;
    assert!(prepare(r.clone(), Some(snapshot.clone())).is_err());
    r.origin = Origin::Independent;
    assert!(prepare(r, Some(snapshot.clone())).is_ok());
    git(
        &source.0,
        &[
            "remote",
            "set-url",
            "origin",
            "https://github.com/a/changed.git",
        ],
    );
    assert_eq!(
        g.switch_remote(&snapshot, "http://127.0.0.1:3000/a/new.git")
            .unwrap_err()
            .code,
        "LOCAL_INPUT_CHANGED"
    );
    assert_eq!(
        git(&source.0, &["remote", "get-url", "origin"]),
        "https://github.com/a/changed.git"
    );
}

#[test]
fn detached_head_does_not_silently_replace_existing_main() {
    let source = Temp::new();
    git(&source.0, &["init", "-b", "main"]);
    let main = commit(&source.0, "main");
    git(&source.0, &["checkout", "-b", "other"]);
    let detached = commit(&source.0, "other");
    git(&source.0, &["checkout", "--detach"]);
    let g = Git::discover().unwrap();
    assert_ne!(main, detached);
    assert_eq!(
        g.inspect(&input(&source.0)).unwrap_err().code,
        "DETACHED_HEAD_AMBIGUOUS"
    );
    let mut selected = input(&source.0);
    selected.extra_refs.push("refs/heads/main".into());
    let snapshot = g.inspect(&selected).unwrap();
    assert_eq!(snapshot.refs["refs/heads/main"], main);
    assert_eq!(git(&source.0, &["rev-parse", "HEAD"]), detached);
    git(&source.0, &["checkout", "-b", "hctl2/private"]);
    assert_eq!(
        g.inspect(&input(&source.0)).unwrap_err().code,
        "PRIVATE_REF"
    );
}

#[test]
fn credentials_do_not_reach_user_hooks_or_configured_tracing() {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(root) = std::env::var("HCTL2_REPO_GIT_ISOLATION_FIXTURE") {
        let root = Path::new(&root);
        let g = Git::discover().unwrap();
        let snapshot = g.inspect(&input(&root.join("source"))).unwrap();
        let copy = g.copy(&snapshot, &root.join("copy")).unwrap();
        let target = root.join("target");
        // Readback must ignore configuration even if its cwd is inside the source repository.
        assert!(
            !g.delivered(
                &root.join("source"),
                &snapshot,
                target.to_str().unwrap(),
                Some(("fixture", "not-a-real-token"))
            )
            .unwrap()
        );
        g.deliver(
            &copy,
            &snapshot,
            target.to_str().unwrap(),
            Some(("fixture", "not-a-real-token")),
        )
        .unwrap();
        assert!(
            g.delivered(&copy, &snapshot, target.to_str().unwrap(), None)
                .unwrap()
        );
        return;
    }
    let temp = Temp::new();
    let source = temp.0.join("source");
    let target = temp.0.join("target");
    let template = temp.0.join("template");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::create_dir(&target).unwrap();
    std::fs::create_dir_all(template.join("hooks")).unwrap();
    git(&source, &["init", "-b", "main"]);
    commit(&source, "frozen");
    git(&target, &["init", "--bare"]);
    let marker = temp.0.join("hook-ran");
    let hook = format!("#!/bin/sh\nprintf hook > '{}'\nexit 1\n", marker.display());
    for path in [
        source.join(".git/hooks/pre-push"),
        template.join("hooks/pre-push"),
    ] {
        std::fs::write(&path, &hook).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    // A URL rewrite in the input copy must not affect privileged readback/push.
    git(
        &source,
        &[
            "config",
            "url./must-not-be-used.insteadOf",
            target.to_str().unwrap(),
        ],
    );
    let global = temp.0.join("global-config");
    std::fs::write(
        &global,
        format!(
            "[core]\n hooksPath = {}\n[url \"/must-not-be-used\"]\n insteadOf = {}\n",
            template.join("hooks").display(),
            target.display()
        ),
    )
    .unwrap();
    let trace = temp.0.join("trace");
    let child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "credentials_do_not_reach_user_hooks_or_configured_tracing",
        ])
        .env("HCTL2_REPO_GIT_ISOLATION_FIXTURE", &temp.0)
        .env("GIT_CONFIG_GLOBAL", global)
        .env("GIT_TEMPLATE_DIR", template)
        .env("GIT_TRACE", &trace)
        .env("GIT_TRACE_CURL", &trace)
        .env("GIT_CURL_VERBOSE", "1")
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "{} {}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
    assert!(!marker.exists());
    assert!(
        !trace.exists(),
        "Git diagnostics must not inherit caller tracing"
    );
    assert_eq!(
        git(&target, &["rev-parse", "main"]),
        git(&source, &["rev-parse", "main"])
    );
}
