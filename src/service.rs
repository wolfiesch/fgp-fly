//! FGP service implementation for Fly.io.

use anyhow::Result;
use fgp_daemon::service::{HealthStatus, MethodInfo, ParamInfo};
use fgp_daemon::FgpService;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::api::FlyClient;

/// FGP service for Fly.io operations.
pub struct FlyService {
    client: Arc<FlyClient>,
    runtime: Runtime,
}

impl FlyService {
    /// Create a new FlyService with the given API token.
    pub fn new(token: String) -> Result<Self> {
        let client = FlyClient::new(token)?;
        let runtime = Runtime::new()?;

        Ok(Self {
            client: Arc::new(client),
            runtime,
        })
    }

    /// Helper to get a u32 parameter with default.
    fn get_param_i32(params: &HashMap<String, Value>, key: &str, default: i32) -> i32 {
        params
            .get(key)
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
            .unwrap_or(default)
    }

    /// Helper to get a string parameter.
    fn get_param_str<'a>(params: &'a HashMap<String, Value>, key: &str) -> Option<&'a str> {
        params.get(key).and_then(|v| v.as_str())
    }

    /// Health check implementation.
    fn health(&self) -> Result<Value> {
        let client = self.client.clone();
        let ok = self.runtime.block_on(async move { client.ping().await })?;

        Ok(serde_json::json!({
            "status": if ok { "healthy" } else { "unhealthy" },
            "api_connected": ok,
            "version": env!("CARGO_PKG_VERSION"),
        }))
    }

    /// List apps implementation.
    fn list_apps(&self, params: HashMap<String, Value>) -> Result<Value> {
        let limit = Self::get_param_i32(&params, "limit", 25);
        let client = self.client.clone();

        let apps = self.runtime.block_on(async move {
            client.list_apps(Some(limit)).await
        })?;

        Ok(serde_json::json!({
            "apps": apps,
            "count": apps.len(),
        }))
    }

    /// Get app status implementation.
    fn app_status(&self, params: HashMap<String, Value>) -> Result<Value> {
        let app_name = Self::get_param_str(&params, "app")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: app"))?
            .to_string();

        let client = self.client.clone();

        let status = self.runtime.block_on(async move {
            client.get_app_status(&app_name).await
        })?;

        Ok(status)
    }

    /// List machines implementation.
    fn list_machines(&self, params: HashMap<String, Value>) -> Result<Value> {
        let app_name = Self::get_param_str(&params, "app")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: app"))?
            .to_string();

        let client = self.client.clone();

        let machines = self.runtime.block_on(async move {
            client.list_machines(&app_name).await
        })?;

        Ok(serde_json::json!({
            "machines": machines,
            "count": machines.len(),
        }))
    }

    /// Get user info implementation.
    fn get_user(&self) -> Result<Value> {
        let client = self.client.clone();

        let user = self.runtime.block_on(async move {
            client.get_user().await
        })?;

        Ok(user)
    }

    /// List regions implementation.
    fn list_regions(&self) -> Result<Value> {
        let client = self.client.clone();

        let regions = self.runtime.block_on(async move {
            client.list_regions().await
        })?;

        Ok(regions)
    }

    /// Secrets implementation (list/set/delete).
    fn handle_secrets(&self, params: HashMap<String, Value>) -> Result<Value> {
        let app_name = Self::get_param_str(&params, "app")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: app"))?
            .to_string();

        let action = Self::get_param_str(&params, "action").unwrap_or("list");

        let client = self.client.clone();

        match action {
            "list" => {
                let result = self.runtime.block_on(async move {
                    client.list_secrets(&app_name).await
                })?;
                Ok(result)
            }
            "set" => {
                let key = Self::get_param_str(&params, "key")
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: key for action=set"))?
                    .to_string();
                let value = Self::get_param_str(&params, "value")
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: value for action=set"))?
                    .to_string();

                let result = self.runtime.block_on(async move {
                    client.set_secret(&app_name, &key, &value).await
                })?;
                Ok(serde_json::json!({
                    "set": true,
                    "result": result
                }))
            }
            "delete" => {
                let key = Self::get_param_str(&params, "key")
                    .ok_or_else(|| anyhow::anyhow!("Missing required parameter: key for action=delete"))?
                    .to_string();

                let result = self.runtime.block_on(async move {
                    client.delete_secret(&app_name, &key).await
                })?;
                Ok(serde_json::json!({
                    "deleted": true,
                    "result": result
                }))
            }
            _ => anyhow::bail!("Unknown action: {}. Valid actions are: list, set, delete", action),
        }
    }

    /// Restart app implementation.
    fn restart_app(&self, params: HashMap<String, Value>) -> Result<Value> {
        let app_name = Self::get_param_str(&params, "app")
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: app"))?
            .to_string();

        let client = self.client.clone();

        let result = self.runtime.block_on(async move {
            client.restart_app(&app_name).await
        })?;

        Ok(serde_json::json!({
            "restarted": true,
            "result": result
        }))
    }
}

impl FgpService for FlyService {
    fn name(&self) -> &str {
        "fly"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn dispatch(&self, method: &str, params: HashMap<String, Value>) -> Result<Value> {
        match method {
            "health" => self.health(),
            "apps" | "fly.apps" => self.list_apps(params),
            "status" | "fly.status" => self.app_status(params),
            "machines" | "fly.machines" => self.list_machines(params),
            "user" | "fly.user" => self.get_user(),
            "regions" | "fly.regions" => self.list_regions(),
            "secrets" | "fly.secrets" => self.handle_secrets(params),
            "restart" | "fly.restart" => self.restart_app(params),
            _ => anyhow::bail!("Unknown method: {}", method),
        }
    }

    fn method_list(&self) -> Vec<MethodInfo> {
        vec![
            MethodInfo {
                name: "fly.apps".into(),
                description: "List all Fly.io apps".into(),
                params: vec![ParamInfo {
                    name: "limit".into(),
                    param_type: "integer".into(),
                    required: false,
                    default: Some(serde_json::json!(25)),
                }],
            },
            MethodInfo {
                name: "fly.status".into(),
                description: "Get status for a specific app".into(),
                params: vec![ParamInfo {
                    name: "app".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                }],
            },
            MethodInfo {
                name: "fly.machines".into(),
                description: "List machines for an app".into(),
                params: vec![ParamInfo {
                    name: "app".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                }],
            },
            MethodInfo {
                name: "fly.user".into(),
                description: "Get current user info".into(),
                params: vec![],
            },
            MethodInfo {
                name: "fly.regions".into(),
                description: "List all Fly.io regions".into(),
                params: vec![],
            },
            MethodInfo {
                name: "fly.secrets".into(),
                description: "Manage secrets for an app".into(),
                params: vec![
                    ParamInfo {
                        name: "app".into(),
                        param_type: "string".into(),
                        required: true,
                        default: None,
                    },
                    ParamInfo {
                        name: "action".into(),
                        param_type: "string".into(),
                        required: false,
                        default: Some(serde_json::json!("list")),
                    },
                    ParamInfo {
                        name: "key".into(),
                        param_type: "string".into(),
                        required: false,
                        default: None,
                    },
                    ParamInfo {
                        name: "value".into(),
                        param_type: "string".into(),
                        required: false,
                        default: None,
                    },
                ],
            },
            MethodInfo {
                name: "fly.restart".into(),
                description: "Restart all machines for an app".into(),
                params: vec![ParamInfo {
                    name: "app".into(),
                    param_type: "string".into(),
                    required: true,
                    default: None,
                }],
            },
        ]
    }

    fn on_start(&self) -> Result<()> {
        tracing::info!("FlyService starting, verifying API connection...");
        let client = self.client.clone();
        self.runtime.block_on(async move {
            match client.ping().await {
                Ok(true) => {
                    tracing::info!("Fly.io API connection verified");
                    Ok(())
                }
                Ok(false) => {
                    tracing::warn!("Fly.io API returned empty viewer ID");
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("Failed to connect to Fly.io API: {}", e);
                    Err(e)
                }
            }
        })
    }

    fn health_check(&self) -> HashMap<String, HealthStatus> {
        let mut checks = HashMap::new();

        let client = self.client.clone();
        let start = std::time::Instant::now();
        let result = self.runtime.block_on(async move { client.ping().await });

        let latency = start.elapsed().as_secs_f64() * 1000.0;

        match result {
            Ok(true) => {
                checks.insert("fly_api".into(), HealthStatus::healthy_with_latency(latency));
            }
            Ok(false) => {
                checks.insert("fly_api".into(), HealthStatus::unhealthy("Empty viewer ID"));
            }
            Err(e) => {
                checks.insert("fly_api".into(), HealthStatus::unhealthy(e.to_string()));
            }
        }

        checks
    }
}
