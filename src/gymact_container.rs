//! Bounded real GymAct implementation for an owned local container runtime.
//!
//! The concrete adapter is intentionally private. External callers cannot obtain
//! it and therefore cannot call `GymActAdapter::execute` as an alternate DO path.
//! The only public entry point delegates to `execute_powl_with_gym_act`, preserving
//! the existing admission → permit → OCEL → receipt boundary.

use std::collections::BTreeMap;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::castle::{
    execute_powl_with_gym_act, ActuationPermit, DoAuthorizationContext, GymActAdapter,
    GymActResult, GymActStatus, OcelObject, PowlActivity, PowlProcess, ReceiptedOcelLog,
    TestEnvelope, WorldState,
};

const DEFAULT_TRANSITION: &str = "observe-alpine-container";
const DEFAULT_IMAGE: &str = "alpine:3.20";

struct ContainerGymActAdapter {
    docker_bin: String,
    now_ms: fn() -> i64,
    allowed_images: BTreeMap<String, String>,
}

impl ContainerGymActAdapter {
    fn default_owned_observation(docker_bin: impl Into<String>) -> Self {
        let mut allowed_images = BTreeMap::new();
        allowed_images.insert(DEFAULT_TRANSITION.to_string(), DEFAULT_IMAGE.to_string());
        Self {
            docker_bin: docker_bin.into(),
            now_ms: real_now_ms,
            allowed_images,
        }
    }

    fn refused(transition_id: &str) -> GymActResult {
        GymActResult {
            transition_id: transition_id.to_string(),
            status: GymActStatus::Refused,
            objects: vec![OcelObject {
                id: format!("object:{transition_id}"),
                kind: "RefusedObservation".to_string(),
            }],
            attributes: Default::default(),
        }
    }

    fn remove_owned_container(&self, name: &str) {
        match Command::new(&self.docker_bin)
            .args(["rm", "-f", name])
            .output()
        {
            Ok(output) if output.status.success() => {
                tracing::info!(container_name = name, "removed owned CASTLE GymAct container");
            }
            Ok(output) => {
                tracing::warn!(
                    container_name = name,
                    stderr = %String::from_utf8_lossy(&output.stderr),
                    "failed to remove owned CASTLE GymAct container"
                );
            }
            Err(error) => {
                tracing::warn!(container_name = name, %error, "failed to spawn docker rm");
            }
        }
    }
}

#[async_trait]
impl GymActAdapter for ContainerGymActAdapter {
    async fn execute(
        &self,
        activity: &PowlActivity,
        state: &WorldState,
        permit: &ActuationPermit,
    ) -> GymActResult {
        let Some(image) = self.allowed_images.get(&activity.transition_id) else {
            return Self::refused(&activity.transition_id);
        };
        if permit.transition_id != activity.transition_id || permit.subject != state.system_id {
            return Self::refused(&activity.transition_id);
        }
        if (self.now_ms)() > permit.expires_at_epoch_ms {
            return Self::refused(&activity.transition_id);
        }

        let name = format!(
            "castle-gymact-{}-{}",
            activity.transition_id,
            nonce_suffix()
        );
        let run = Command::new(&self.docker_bin)
            .args([
                "run",
                "-d",
                "--network=none",
                "--read-only",
                "--cap-drop=ALL",
                "--security-opt",
                "no-new-privileges",
                "--pids-limit",
                "32",
                "--memory",
                "64m",
                "--cpus",
                "0.25",
                "--name",
                &name,
                image,
                "sleep",
                "5",
            ])
            .output();
        let run = match run {
            Ok(output) if output.status.success() => output,
            Ok(output) => {
                tracing::warn!(stderr = %String::from_utf8_lossy(&output.stderr), "docker run refused");
                return Self::refused(&activity.transition_id);
            }
            Err(error) => {
                tracing::warn!(%error, "docker run unavailable");
                return Self::refused(&activity.transition_id);
            }
        };

        let inspect = Command::new(&self.docker_bin)
            .args(["inspect", &name])
            .output();
        self.remove_owned_container(&name);

        let inspect = match inspect {
            Ok(output) if output.status.success() => output,
            Ok(output) => {
                tracing::warn!(stderr = %String::from_utf8_lossy(&output.stderr), "docker inspect refused");
                return Self::refused(&activity.transition_id);
            }
            Err(error) => {
                tracing::warn!(%error, "docker inspect unavailable");
                return Self::refused(&activity.transition_id);
            }
        };

        let parsed: Value = match serde_json::from_slice(&inspect.stdout) {
            Ok(value) => value,
            Err(error) => {
                tracing::warn!(%error, "docker inspect returned invalid JSON");
                return Self::refused(&activity.transition_id);
            }
        };
        let container_id = parsed
            .get(0)
            .and_then(|value| value.get("Id"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let state_status = parsed
            .get(0)
            .and_then(|value| value.pointer("/State/Status"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let inspect_digest = blake3::hash(&inspect.stdout).to_hex().to_string();
        let run_digest = blake3::hash(&run.stdout).to_hex().to_string();

        let mut attributes = BTreeMap::new();
        attributes.insert("image".to_string(), json!(image));
        attributes.insert("container_name".to_string(), json!(name));
        attributes.insert("container_state_status".to_string(), json!(state_status));
        attributes.insert("inspect_stdout_blake3".to_string(), json!(inspect_digest));
        attributes.insert("run_stdout_blake3".to_string(), json!(run_digest));
        attributes.insert("permit_authority".to_string(), json!(permit.authority));

        GymActResult {
            transition_id: activity.transition_id.clone(),
            status: GymActStatus::Observed,
            objects: vec![OcelObject {
                id: format!("docker:{name}:{container_id}"),
                kind: "Container".to_string(),
            }],
            attributes,
        }
    }
}

/// Run the fixed, network-isolated Alpine observation through CASTLE's existing
/// admitted DO path. The image/command are code-fixed; the caller chooses no
/// arbitrary container command and receives only the normal receipted OCEL result.
pub async fn execute_default_container_observation(
    process: &PowlProcess,
    state: &WorldState,
    envelope: &TestEnvelope,
    authorization: DoAuthorizationContext<'_>,
    docker_bin: impl Into<String>,
) -> Result<ReceiptedOcelLog, String> {
    let adapter = ContainerGymActAdapter::default_owned_observation(docker_bin);
    execute_powl_with_gym_act(process, state, envelope, &adapter, authorization).await
}

fn real_now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

fn nonce_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| format!("{:x}", duration.as_nanos()))
        .unwrap_or_else(|_| "0".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn activity(id: &str) -> PowlActivity {
        PowlActivity {
            id: format!("activity:{id}"),
            transition_id: id.to_string(),
            predecessors: Vec::new(),
        }
    }

    fn state() -> WorldState {
        WorldState {
            system_id: "system:local-docker".to_string(),
            facts: BTreeSet::new(),
        }
    }

    fn permit(id: &str, expires_at_epoch_ms: i64) -> ActuationPermit {
        ActuationPermit {
            construct_digest: "0".repeat(64),
            process_digest: "0".repeat(64),
            subject: "system:local-docker".to_string(),
            authority: "defensive-test".to_string(),
            transition_id: id.to_string(),
            expires_at_epoch_ms,
        }
    }

    #[tokio::test]
    async fn unknown_transition_refuses_before_process_spawn() {
        let adapter = ContainerGymActAdapter::default_owned_observation("/definitely/missing/docker");
        let result = adapter
            .execute(&activity("arbitrary-image"), &state(), &permit("arbitrary-image", i64::MAX))
            .await;
        assert_eq!(result.status, GymActStatus::Refused);
    }

    #[tokio::test]
    async fn expired_permit_refuses_before_process_spawn() {
        let mut adapter = ContainerGymActAdapter::default_owned_observation("/definitely/missing/docker");
        adapter.now_ms = || 100;
        let result = adapter
            .execute(&activity(DEFAULT_TRANSITION), &state(), &permit(DEFAULT_TRANSITION, 99))
            .await;
        assert_eq!(result.status, GymActStatus::Refused);
    }

    #[tokio::test]
    async fn live_docker_observation_when_prepared() {
        let docker = std::env::var("DOCKER_BIN").unwrap_or_else(|_| "docker".to_string());
        let prepared = Command::new(&docker)
            .args(["image", "inspect", DEFAULT_IMAGE])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);
        if !prepared {
            eprintln!("SKIPPED: {DEFAULT_IMAGE} is not prepared in the local Docker daemon");
            return;
        }

        let adapter = ContainerGymActAdapter::default_owned_observation(&docker);
        let result = adapter
            .execute(&activity(DEFAULT_TRANSITION), &state(), &permit(DEFAULT_TRANSITION, i64::MAX))
            .await;
        assert_eq!(result.status, GymActStatus::Observed);
        assert_eq!(
            result
                .attributes
                .get("inspect_stdout_blake3")
                .and_then(Value::as_str)
                .map(str::len),
            Some(64)
        );

        let remaining = Command::new(&docker)
            .args([
                "ps",
                "-a",
                "--filter",
                "name=castle-gymact-observe-alpine-container-",
                "--format",
                "{{.Names}}",
            ])
            .output()
            .expect("prepared Docker daemon must remain reachable");
        assert!(String::from_utf8_lossy(&remaining.stdout).trim().is_empty());
    }
}
