use super::*;

#[derive(Subcommand)]
pub(super) enum ReviewCommand {
    /// Let a publish intent that waits for a human go: preview what it will do, then submit
    /// with the preview token. Publishing itself runs in the background and reads back.
    Publish {
        repo_id: String,
        intent_id: String,
        /// Confirm the previewed release; without it only the preview is printed.
        #[arg(long)]
        preview_token: Option<String>,
    },
    /// The publish intent, its two stages (push, review request) and the mappings written.
    Show {
        repo_id: String,
        intent_id: String,
    },
    List {
        repo_id: String,
    },
}

pub(super) async fn dispatch(
    command: ReviewCommand,
    root: &Path,
    as_json: bool,
) -> Result<(), String> {
    let result = execute(command, root, as_json).await;
    if let Err(error) = &result {
        print_out(
            as_json,
            json!({"error":{"code":"REVIEW_COMMAND_FAILED","message":error,"recovery_action":"inspect_error_and_retry"}}),
        );
    }
    result
}

async fn execute(command: ReviewCommand, root: &Path, as_json: bool) -> Result<(), String> {
    let (repo_id, intent_id, token) = match command {
        ReviewCommand::Publish {
            repo_id,
            intent_id,
            preview_token,
        } => (repo_id, intent_id, preview_token),
        ReviewCommand::Show { repo_id, intent_id } => {
            return task::query_task(
                root,
                as_json,
                "review.show",
                json!({"repo_id":repo_id,"intent_id":intent_id}),
            )
            .await;
        }
        ReviewCommand::List { repo_id } => {
            return task::query_task(root, as_json, "review.list", json!({"repo_id":repo_id}))
                .await;
        }
    };
    let mut client = client(root).await?;
    // The command identity is per publish round: the same round replays, the next round
    // is a new authorization. The current round comes from the intent itself.
    let shown = client
        .query(QueryRequest {
            protocol: Some(Protocol {
                version: PROTOCOL.into(),
            }),
            kind: "review.show".into(),
            payload: serde_json::to_vec(&json!({"repo_id": repo_id, "intent_id": intent_id}))
                .map_err(|e| e.to_string())?,
        })
        .await
        .map_err(|e| e.to_string())?
        .into_inner();
    if let Some(error) = shown.error {
        task::present(Some(error), as_json);
        return Err("review intent could not be read".into());
    }
    let round = bytes_json(&shown.payload)?["intent"]["round"]
        .as_u64()
        .ok_or("review intent has no round")?;
    let payload =
        serde_json::to_vec(&json!({"repo_id": repo_id, "intent_id": intent_id, "round": round}))
            .map_err(|e| e.to_string())?;
    let command_id = format!("review-publish:{intent_id}:{round}");
    if let Some(preview_token) = token {
        let result = client
            .submit(SubmitRequest {
                protocol: Some(Protocol {
                    version: PROTOCOL.into(),
                }),
                operation: "review.publish".into(),
                payload,
                command_id: command_id.clone(),
                idempotency_key: command_id,
                preview_token,
            })
            .await
            .map_err(|e| e.to_string())?
            .into_inner();
        task::present(result.error, as_json);
        print_out(as_json, bytes_json(&result.result)?);
    } else {
        let result = client
            .preview(PreviewRequest {
                protocol: Some(Protocol {
                    version: PROTOCOL.into(),
                }),
                operation: "review.publish".into(),
                payload,
                command_id,
            })
            .await
            .map_err(|e| e.to_string())?
            .into_inner();
        task::present(result.error, as_json);
        print_out(
            as_json,
            json!({"preview_token":result.preview_token,"effect_summary":bytes_json(&result.effect_summary)?}),
        );
    }
    Ok(())
}
