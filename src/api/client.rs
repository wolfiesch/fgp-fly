//! Fly.io GraphQL API client with connection pooling.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::{App, GraphQLResponse, Machine};

const GRAPHQL_ENDPOINT: &str = "https://api.fly.io/graphql";

/// Fly.io GraphQL client with persistent connection.
pub struct FlyClient {
    client: Client,
    token: String,
}

impl FlyClient {
    /// Create a new Fly.io client.
    pub fn new(token: String) -> Result<Self> {
        let client = Client::builder()
            .pool_max_idle_per_host(5)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client, token })
    }

    /// Execute a GraphQL query.
    async fn query<T: for<'de> Deserialize<'de>>(
        &self,
        query: &str,
        variables: Option<Value>,
    ) -> Result<T> {
        let body = GraphQLRequest {
            query: query.to_string(),
            variables,
        };

        let response = self
            .client
            .post(GRAPHQL_ENDPOINT)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send GraphQL request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("GraphQL request failed: {} - {}", status, text);
        }

        // Get raw text for debugging
        let text = response.text().await.context("Failed to read response")?;

        let result: GraphQLResponse<T> = serde_json::from_str(&text)
            .map_err(|e| anyhow::anyhow!("JSON parse error: {} | Raw: {}", e, &text[..text.len().min(300)]))?;

        // Only fail on GraphQL errors if there's no data at all
        // GraphQL allows partial results with field-level errors
        if result.data.is_none() {
            if let Some(errors) = result.errors {
                if !errors.is_empty() {
                    let messages: Vec<_> = errors.iter().map(|e| e.message.as_str()).collect();
                    anyhow::bail!("GraphQL errors: {}", messages.join(", "));
                }
            }
        }

        result
            .data
            .context("GraphQL response missing data field")
    }

    /// Check if the client can connect to Fly.io API.
    pub async fn ping(&self) -> Result<bool> {
        let query = r#"
            query {
                viewer {
                    id
                }
            }
        "#;

        #[derive(Deserialize)]
        struct ViewerResponse {
            viewer: Viewer,
        }

        #[derive(Deserialize)]
        struct Viewer {
            id: String,
        }

        let result: ViewerResponse = self.query(query, None).await?;
        Ok(!result.viewer.id.is_empty())
    }

    /// List all apps for the authenticated user.
    pub async fn list_apps(&self, limit: Option<i32>) -> Result<Vec<App>> {
        let limit = limit.unwrap_or(25);

        let query = r#"
            query($first: Int) {
                apps(first: $first) {
                    nodes {
                        id
                        name
                        status
                        deployed
                        hostname
                        organization {
                            id
                            name
                            slug
                        }
                        currentRelease {
                            id
                            version
                            status
                            description
                            createdAt
                        }
                    }
                }
            }
        "#;

        #[derive(Deserialize)]
        struct AppsResponse {
            apps: AppsNodes,
        }

        #[derive(Deserialize)]
        struct AppsNodes {
            // Some apps may return null due to authorization errors
            nodes: Vec<Option<AppNode>>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct AppNode {
            id: String,
            name: String,
            #[serde(default)]
            status: String,
            #[serde(default)]
            deployed: bool,
            #[serde(default)]
            hostname: Option<String>,
            #[serde(default)]
            organization: Option<OrgNode>,
            #[serde(default)]
            current_release: Option<ReleaseNode>,
        }

        #[derive(Deserialize)]
        struct OrgNode {
            id: String,
            name: String,
            slug: String,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ReleaseNode {
            id: String,
            version: i32,
            status: String,
            #[serde(default)]
            description: Option<String>,
            #[serde(default)]
            created_at: Option<String>,
        }

        let variables = serde_json::json!({ "first": limit });
        let result: AppsResponse = self.query(query, Some(variables)).await?;

        // Filter out unauthorized apps (null values)
        let apps = result
            .apps
            .nodes
            .into_iter()
            .flatten() // Skip None values
            .map(|n| App {
                id: n.id,
                name: n.name,
                status: n.status,
                deployed: n.deployed,
                hostname: n.hostname,
                organization: n.organization.map(|o| crate::models::Organization {
                    id: o.id,
                    name: o.name,
                    slug: o.slug,
                }),
                current_release: n.current_release.map(|r| crate::models::Release {
                    id: r.id,
                    version: r.version,
                    status: r.status,
                    description: r.description,
                    created_at: r.created_at,
                }),
            })
            .collect();

        Ok(apps)
    }

    /// Get status for a specific app.
    pub async fn get_app_status(&self, app_name: &str) -> Result<Value> {
        let query = r#"
            query($name: String!) {
                app(name: $name) {
                    id
                    name
                    status
                    deployed
                    hostname
                    organization {
                        id
                        name
                        slug
                    }
                    currentRelease {
                        id
                        version
                        status
                        description
                        createdAt
                    }
                    machines {
                        nodes {
                            id
                            name
                            state
                            region
                        }
                    }
                    allocations {
                        id
                        status
                        region
                        version
                    }
                }
            }
        "#;

        let variables = serde_json::json!({ "name": app_name });
        let result: Value = self.query(query, Some(variables)).await?;

        Ok(result)
    }

    /// List machines for an app.
    pub async fn list_machines(&self, app_name: &str) -> Result<Vec<Machine>> {
        let query = r#"
            query($name: String!) {
                app(name: $name) {
                    machines {
                        nodes {
                            id
                            name
                            state
                            region
                        }
                    }
                }
            }
        "#;

        #[derive(Deserialize)]
        struct AppResponse {
            app: AppMachines,
        }

        #[derive(Deserialize)]
        struct AppMachines {
            machines: MachinesNodes,
        }

        #[derive(Deserialize)]
        struct MachinesNodes {
            nodes: Vec<MachineNode>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct MachineNode {
            id: String,
            name: String,
            state: String,
            region: String,
        }

        let variables = serde_json::json!({ "name": app_name });
        let result: AppResponse = self.query(query, Some(variables)).await?;

        let machines = result
            .app
            .machines
            .nodes
            .into_iter()
            .map(|n| Machine {
                id: n.id,
                name: n.name,
                state: n.state,
                region: n.region,
                instance_id: None,
                private_ip: None,
                config: None,
            })
            .collect();

        Ok(machines)
    }

    /// Get current user info.
    pub async fn get_user(&self) -> Result<Value> {
        let query = r#"
            query {
                viewer {
                    id
                    email
                    name
                    organizations {
                        nodes {
                            id
                            name
                            slug
                        }
                    }
                }
            }
        "#;

        let result: Value = self.query(query, None).await?;
        Ok(result)
    }
}

#[derive(Serialize)]
struct GraphQLRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<Value>,
}
