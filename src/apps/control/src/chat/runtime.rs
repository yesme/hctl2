use super::*;

/// No independent lifecycle: the daemon owns the listener and reloads restored registration.
pub(crate) async fn serve(root: PathBuf, services: Arc<Supervisor>, shared: Shared) {
    loop {
        let control_id = loop {
            if let Some(store) = shared.lock().await.as_ref() {
                break store.control_id().to_owned();
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        };
        let prepare = Arc::clone(&services);
        let prepared =
            tokio::task::spawn_blocking(move || config::prepare(&prepare, &control_id)).await;
        let (registration, listener) = match prepared {
            Ok(Ok(pair)) => pair,
            Ok(Err(e)) if e.code == "CHAT_NOT_INSTALLED" => {
                let startup = Arc::clone(&services);
                let _ = tokio::task::spawn_blocking(move || startup.ensure_up()).await;
                std::future::pending::<()>().await;
                return;
            }
            _ => {
                services.set_last_error("AppService setup failed; inspect chat configuration");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }
        };
        let path = match config::path(&services) {
            Ok(path) => path,
            Err(_) => continue,
        };
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };
        let digest = foundation::bytes_sha256(&bytes);
        let startup = Arc::clone(&services);
        tokio::task::spawn_blocking(move || {
            if let Err(e) = startup.ensure_chat_up(&digest) {
                startup.set_last_error(e);
            }
        });
        let listener = match tokio::net::TcpListener::from_std(listener) {
            Ok(listener) => listener,
            Err(_) => continue,
        };
        let router = inbox::Inbox {
            root: root.clone(),
            token: registration.hs_token,
        }
        .router();
        // Native service restore replaces the registration (including callback port and token).
        // Drop the old listener first, then load the restored registration on the next iteration.
        let changed = async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                if std::fs::read(&path).ok().as_ref() != Some(&bytes) {
                    break;
                }
            }
        };
        if axum::serve(listener, router)
            .with_graceful_shutdown(changed)
            .await
            .is_err()
        {
            services.set_last_error("AppService listener stopped");
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}

pub(crate) async fn reconcile(
    shared: Shared,
    services: Arc<Supervisor>,
    root: PathBuf,
    operations: Arc<Mutex<()>>,
) {
    use std::os::unix::fs::MetadataExt;
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        interval.tick().await;
        let (shared, services, root, operations) = (
            Arc::clone(&shared),
            Arc::clone(&services),
            root.clone(),
            Arc::clone(&operations),
        );
        let _ = tokio::task::spawn_blocking(move || -> Result<()> {
            let ids = access(&shared, |s| s.pending_effects())?;
            let actor = crate::owner_actor(std::fs::metadata(&root)?.uid());
            for id in ids {
                let _operation = operations.blocking_lock();
                let (effect, _) = access(&shared, |s| s.effect(&id))?;
                if effect.operation.starts_with("chat.") {
                    let _ = drive(&shared, &services, &root, &actor, &id);
                }
            }
            Ok(())
        })
        .await;
    }
}
