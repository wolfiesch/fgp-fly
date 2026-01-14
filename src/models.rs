//! Data models for Fly.io API responses.

use serde::{Deserialize, Serialize};

/// Fly.io application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub organization: Option<Organization>,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub deployed: bool,
    #[serde(default)]
    pub current_release: Option<Release>,
}

/// Fly.io organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub slug: String,
}

/// Fly.io machine (VM instance).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Machine {
    pub id: String,
    pub name: String,
    pub state: String,
    pub region: String,
    #[serde(default)]
    pub instance_id: Option<String>,
    #[serde(default)]
    pub private_ip: Option<String>,
    #[serde(default)]
    pub config: Option<MachineConfig>,
}

/// Machine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfig {
    #[serde(default)]
    pub size: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
}

/// Fly.io release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub id: String,
    pub version: i32,
    pub status: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// Application status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStatus {
    pub app: App,
    #[serde(default)]
    pub machines: Vec<Machine>,
    #[serde(default)]
    pub allocations: Vec<Allocation>,
}

/// VM allocation (legacy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allocation {
    pub id: String,
    pub status: String,
    pub region: String,
    #[serde(default)]
    pub version: Option<i32>,
}

/// Log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub message: String,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub instance: Option<String>,
}

/// GraphQL response wrapper.
#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    #[serde(default)]
    pub errors: Option<Vec<GraphQLError>>,
}

/// GraphQL error.
#[derive(Debug, Deserialize)]
pub struct GraphQLError {
    pub message: String,
    #[serde(default)]
    pub path: Option<Vec<serde_json::Value>>,  // Path can be strings or integers
}
