//! Installed payload verification and catalog re-pairing through the real CLI.
use super::*;
use std::time::Instant;

struct AgencyProcess {
    child: std::process::Child,
    root: PathBuf,
    log: PathBuf,
}
impl AgencyProcess {
    fn start(root: &Path, install: Option<&Path>, script: Option<&Path>) -> Self {
        std::fs::create_dir_all(root).unwrap();
        let log = root.join("test-serve.err");
        let mut command = Command::new(std::env::var_os("CARGO_BIN_EXE_agency").unwrap());
        command
            .arg("--root")
            .arg(root)
            .arg("serve")
            .env_remove("HCTL2_INSTALL_ROOT")
            .env(
                "HCTL2_CLAUDE",
                std::env::var_os("HCTL2_STANDBY_FIXTURE").unwrap(),
            )
            .env("HCTL2_CODEX", root.join("absent-codex"))
            .stdout(Stdio::null())
            .stderr(File::create(&log).unwrap());
        if let Some(install) = install {
            command.env("HCTL2_INSTALL_ROOT", install);
        }
        if let Some(script) = script {
            command.arg("--script-config").arg(script);
        }
        let mut process = Self {
            child: command.spawn().unwrap(),
            root: root.into(),
            log,
        };
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            if process.status().is_some() {
                return process;
            }
            assert!(
                process.child.try_wait().unwrap().is_none(),
                "{}",
                process.log()
            );
            assert!(Instant::now() < deadline, "{}", process.log());
            std::thread::sleep(Duration::from_millis(50));
        }
    }
    fn status(&self) -> Option<Value> {
        let output = Command::new(std::env::var_os("CARGO_BIN_EXE_agency").unwrap())
            .arg("--root")
            .arg(&self.root)
            .arg("status")
            .output()
            .unwrap();
        output
            .status
            .success()
            .then(|| serde_json::from_slice(&output.stdout).unwrap())
    }
    fn log(&self) -> String {
        std::fs::read_to_string(&self.log).unwrap()
    }
}
impl Drop for AgencyProcess {
    fn drop(&mut self) {
        let _ = Command::new(std::env::var_os("CARGO_BIN_EXE_agency").unwrap())
            .arg("--root")
            .arg(&self.root)
            .arg("stop")
            .output();
        let deadline = Instant::now() + Duration::from_secs(5);
        while self.child.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(25));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn installed_payload_catalogs_claude_and_rejects_tampered_herdr() {
    let mut f = Fixture::unpacked("agency-install");
    let package = f.payload.parent().unwrap();
    let prefix = f.root.join("prefix");
    let output = Command::new(package.join("install.sh"))
        .arg("--prefix")
        .arg(&prefix)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let id = std::fs::read_to_string(f.payload.join("share/hctl2/package-id")).unwrap();
    f.payload = prefix.join("lib/hctl2").join(id.trim());
    assert!(f.payload.join("share/hctl2/PAYLOAD.sha256").is_file());
    f.isolate_ports();
    let (ok, start) = f.run(&["start", "--secret-backend", "user-file"]);
    assert!(ok, "{start}");
    let root = f.root.join("agency");
    let process = AgencyProcess::start(&root, Some(&f.payload), None);
    let (ok, paired) = f.run(&[
        "agency",
        "pair",
        "--binding-id",
        "installed",
        "--agency-root",
        root.to_str().unwrap(),
        "--key",
        "pair-installed",
    ]);
    assert!(ok, "{paired}");
    let (ok, catalog) = f.run(&["agency", "catalog", "installed"]);
    assert!(ok, "{catalog}");
    assert!(
        catalog["professions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["reference"]["id"] == "claude-code"),
        "catalog={catalog}; stderr={}",
        process.log()
    );
    println!(
        "installed payload: {}; catalog: {catalog}",
        f.payload.display()
    );
    drop(process);
    let herdr = f.payload.join("libexec/hctl2/herdr");
    let mut bytes = std::fs::read(&herdr).unwrap();
    bytes[0] ^= 0xff;
    std::fs::write(&herdr, bytes).unwrap();
    let process = AgencyProcess::start(&root, Some(&f.payload), None);
    let (ok, catalog) = f.run(&["agency", "catalog", "installed"]);
    assert!(ok, "{catalog}");
    assert!(catalog["professions"].as_array().unwrap().is_empty());
    assert!(
        process.log().contains("HERDR_DIGEST_MISMATCH"),
        "{}",
        process.log()
    );
}

#[test]
fn empty_catalog_can_be_repaired_with_a_new_binding_from_real_cli() {
    let (f, _) = Fixture::packaged("agency-repair");
    let (ok, start) = f.run(&["start", "--secret-backend", "user-file"]);
    assert!(ok, "{start}");
    let root = f.root.join("agency");
    let process = AgencyProcess::start(&root, None, None);
    let pair = |id: &str, key: &str| {
        f.run(&[
            "agency",
            "pair",
            "--binding-id",
            id,
            "--agency-root",
            root.to_str().unwrap(),
            "--key",
            key,
        ])
    };
    let (ok, old) = pair("empty", "pair-empty");
    assert!(ok, "{old}");
    assert!(
        old["binding"]["data"]["value"]["catalog"]["professions"]
            .as_array()
            .unwrap()
            .is_empty(),
        "{old}"
    );
    drop(process);
    let config = f.root.join("script.json");
    std::fs::write(
        &config,
        json!({"program":"/bin/sh","arguments":["-c","printf '%s\\n' ready"]}).to_string(),
    )
    .unwrap();
    let _process = AgencyProcess::start(&root, None, Some(&config));
    let (ok, catalog) = f.run(&["agency", "catalog", "empty"]);
    assert!(ok, "{catalog}");
    let profession = &catalog["professions"][0]["reference"];
    let reference = f.root.join("profession.json");
    std::fs::write(&reference, profession.to_string()).unwrap();
    let accept = |id: &str, key: &str| {
        f.run(&[
            "agency",
            "accept",
            "--binding-id",
            id,
            "--reference",
            reference.to_str().unwrap(),
            "--key",
            key,
        ])
    };
    let (ok, rejected) = accept("empty", "accept-stale");
    assert!(!ok, "{rejected}");
    assert_eq!(
        rejected["error"]["code"], "PROFESSION_CHANGED",
        "{rejected}"
    );
    assert_eq!(rejected["error"]["recovery_action"], "hctl2 agency pair");
    let (ok, immutable) = pair("empty", "pair-changed");
    assert!(!ok, "{immutable}");
    assert_eq!(immutable["error"]["code"], "BINDING_IMMUTABLE");
    let (ok, refreshed) = pair("refreshed", "pair-refreshed");
    assert!(ok, "{refreshed}");
    assert_eq!(refreshed["binding"]["data"]["value"]["catalog"], catalog);
    assert_eq!(pair("refreshed", "pair-refreshed"), (true, refreshed));
    let (ok, accepted) = accept("refreshed", "accept-refreshed");
    assert!(ok, "{accepted}");
    assert_eq!(
        accepted["profession"]["sources"][0]["key"]["id"],
        "refreshed"
    );
    assert_eq!(accept("refreshed", "accept-refreshed"), (true, accepted));
    let (ok, bindings) = f.run(&["agency", "bindings"]);
    assert!(ok, "{bindings}");
    assert_eq!(bindings["records"].as_array().unwrap().len(), 2);
    assert!(
        bindings["records"]
            .as_array()
            .unwrap()
            .contains(&old["binding"]),
        "{bindings}"
    );
    println!("empty catalog -> new binding -> accept succeeded; old binding retained");
}
