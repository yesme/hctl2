use super::{Temp, run};
use agency_proto::{Capabilities, FrozenRef, hash};
use participant::profiles::WorkerProfile;
use serde_json::{Value, json};

struct Child(std::process::Child);
impl Drop for Child {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn ok(root: &std::path::Path, args: &[&str]) -> Value {
    let (ok, out, err) = run(root, args);
    assert!(ok, "{args:?}: {out} {err}");
    if out.trim().is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&out).unwrap()
    }
}

#[test]
fn profile_creation_starts_from_a_real_paired_agency_catalog_and_survives_control_restart() {
    let temp = Temp::new();
    let root = &temp.0;
    let agency_root = root.join("independent-agency");
    let config = root.join("script-config.json");
    std::fs::write(
        &config,
        json!({"program":"/bin/sh","arguments":["-c","read -r initial"]}).to_string(),
    )
    .unwrap();
    let binary = std::env::var_os("CARGO_BIN_EXE_agency").expect("native Agency target");
    let _agency = Child(
        std::process::Command::new(&binary)
            .arg("--root")
            .arg(&agency_root)
            .arg("serve")
            .arg("--script-config")
            .arg(&config)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap(),
    );
    let mut ready = false;
    for _ in 0..100 {
        if std::process::Command::new(&binary)
            .arg("--root")
            .arg(&agency_root)
            .arg("status")
            .output()
            .unwrap()
            .status
            .success()
        {
            ready = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert!(ready, "script Agency became ready");
    ok(root, &["init", "--secret-backend", "user-file"]);
    ok(root, &["start"]);
    let paired = ok(
        root,
        &[
            "agency",
            "pair",
            "--binding-id",
            "local",
            "--agency-root",
            agency_root.to_str().unwrap(),
            "--key",
            "pair",
        ],
    );
    assert_eq!(paired["paired"], true);
    let catalog = ok(root, &["agency", "catalog", "local"]);
    let profession = &catalog["professions"][0];
    let reference = root.join("profession-ref.json");
    std::fs::write(&reference, profession["reference"].to_string()).unwrap();
    let accepted = ok(
        root,
        &[
            "agency",
            "accept",
            "--binding-id",
            "local",
            "--reference",
            reference.to_str().unwrap(),
            "--key",
            "accept",
        ],
    );
    assert_eq!(
        accepted["profession"]["data"]["value"]["reference"],
        profession["reference"]
    );
    let action = json!({"id":"research", "profile":{
        "harness":profession["harness"], "model":profession["model"], "mode":"read_only",
        "permissions":["context.read"], "environment":[], "required_capabilities":Capabilities::default(),
        "max_context_bytes":65536}});
    let path = root.join("profile.json");
    std::fs::write(&path, action.to_string()).unwrap();
    let args = [
        "profile",
        "create",
        "--input",
        path.to_str().unwrap(),
        "--key",
        "create",
    ];
    let preview = ok(root, &args);
    let mut confirm = args.to_vec();
    confirm.extend([
        "--preview-token",
        preview["preview_token"].as_str().unwrap(),
    ]);
    let created = ok(root, &confirm);
    assert_eq!(created["profile_id"], "research");
    assert_eq!(
        created["revision"]["key"]["kind"],
        "worker_profile_revision"
    );
    ok(root, &["stop"]);
    ok(root, &["start"]);
    let preview = ok(root, &args);
    let mut confirm = args.to_vec();
    confirm.extend([
        "--preview-token",
        preview["preview_token"].as_str().unwrap(),
    ]);
    assert_eq!(ok(root, &confirm), created);
    ok(root, &["stop"]);
}

#[test]
fn profile_cli_local_input_errors_are_stdout_json() {
    let temp = Temp::new();
    for contents in [
        None,
        Some("not-json"),
        Some("[]"),
        Some(r#"{"kind":"update"}"#),
    ] {
        let path = temp.0.join("bad-profile.json");
        if let Some(contents) = contents {
            std::fs::write(&path, contents).unwrap();
        }
        let (ok, out, _) = run(
            &temp.0,
            &[
                "profile",
                "create",
                "--input",
                path.to_str().unwrap(),
                "--key",
                "bad",
            ],
        );
        assert!(!ok);
        let failure: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(failure["error"]["code"], "PROFILE_COMMAND_FAILED");
        assert_eq!(
            failure["error"]["recovery_action"],
            "inspect_error_and_retry"
        );
    }
}

#[test]
fn profile_create_cli_requires_confirmation_and_preserves_exact_revision() {
    let temp = Temp::new();
    let root = &temp.0;
    assert!(run(root, &["init", "--secret-backend", "user-file"]).0);
    let (ok, out, err) = run(root, &["start"]);
    assert!(ok, "{out} {err}");
    let path = root.join("profile.json");
    let profile = WorkerProfile {
        harness: FrozenRef {
            id: "script".into(),
            revision: "1".into(),
            digest: hash(b"script"),
        },
        model: "script".into(),
        mode: "read_only".into(),
        permissions: vec!["context.read".into()],
        environment: vec![],
        required_capabilities: Capabilities::default(),
        max_context_bytes: 65536,
    };
    let action = json!({"id":"research","profile":profile});
    std::fs::write(&path, serde_json::to_vec(&action).unwrap()).unwrap();
    let args = [
        "profile",
        "create",
        "--input",
        path.to_str().unwrap(),
        "--key",
        "create-profile",
    ];
    let (ok, out, err) = run(root, &args);
    assert!(ok, "{out} {err}");
    let preview: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        preview["effect_summary"]["input"]["action"]["profile"],
        serde_json::to_value(&profile).unwrap()
    );
    let (ok, out, err) = run(root, &["export"]);
    assert!(ok, "{out} {err}");
    let exported: Value = serde_json::from_str(&out).unwrap();
    assert!(!exported.to_string().contains("worker_profile_revision"));
    let mut invalid = action.clone();
    invalid["profile"]["permissions"] = json!(["command.submit"]);
    std::fs::write(&path, serde_json::to_vec(&invalid).unwrap()).unwrap();
    let mut confirmed = args.to_vec();
    confirmed.extend([
        "--preview-token",
        preview["preview_token"].as_str().unwrap(),
    ]);
    let (ok, out, err) = run(root, &confirmed);
    assert!(!ok, "{out} {err}");
    assert_eq!(
        serde_json::from_str::<Value>(&out).unwrap()["error"]["code"],
        "PREVIEW_REQUIRED"
    );
    let (ok, out, err) = run(root, &args);
    assert!(!ok, "{out} {err}");
    assert_eq!(
        serde_json::from_str::<Value>(&out).unwrap()["error"]["code"],
        "PERMISSION_SCOPE_INVALID"
    );
    std::fs::write(&path, serde_json::to_vec(&action).unwrap()).unwrap();
    let (ok, out, err) = run(root, &confirmed);
    assert!(ok, "{out} {err}");
    let result: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(result["profile_id"], "research");
    assert_eq!(result["version"], 1);
    assert_eq!(result["revision"]["key"]["kind"], "worker_profile_revision");
    let (ok, out, err) = run(root, &confirmed);
    assert!(!ok, "{out} {err}");
    assert_eq!(
        serde_json::from_str::<Value>(&out).unwrap()["error"]["code"],
        "PREVIEW_REQUIRED"
    );
    let (ok, out, err) = run(root, &args);
    assert!(ok, "{out} {err}");
    let replay: Value = serde_json::from_str(&out).unwrap();
    let mut retry = args.to_vec();
    retry.extend(["--preview-token", replay["preview_token"].as_str().unwrap()]);
    let (ok, out, err) = run(root, &retry);
    assert!(ok, "{out} {err}");
    assert_eq!(serde_json::from_str::<Value>(&out).unwrap(), result);
    let (ok, out, err) = run(root, &["stop"]);
    assert!(ok, "{out} {err}");
}
