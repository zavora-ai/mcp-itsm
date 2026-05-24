use chrono::{DateTime, Utc};
use rmcp::schemars;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TicketType { Incident, ServiceRequest, Task }

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TicketStatus { New, Open, InProgress, Pending, Resolved, Closed }

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Priority { Critical, High, Medium, Low }

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Impact { Low, Medium, High, Enterprise }

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NoteVisibility { Internal, CustomerFacing }

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType { Standard, Normal, Emergency }

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeStatus { Draft, Submitted, Approved, Implementing, Completed, Rejected }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub id: String,
    pub ticket_type: TicketType,
    pub title: String,
    pub description: String,
    pub status: TicketStatus,
    pub priority: Priority,
    pub impact: Impact,
    pub category: String,
    pub subcategory: Option<String>,
    pub service: Option<String>,
    pub assignee: Option<String>,
    pub queue: String,
    pub requester: String,
    pub sla: SlaStatus,
    pub notes: Vec<Note>,
    pub related_change_ids: Vec<String>,
    pub resolution_code: Option<String>,
    pub resolution_notes: Option<String>,
    pub knowledge_articles: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaStatus {
    pub policy: String,
    pub response_due: DateTime<Utc>,
    pub resolution_due: DateTime<Utc>,
    pub response_breached: bool,
    pub resolution_breached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub visibility: NoteVisibility,
    pub author: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRequest {
    pub id: String,
    pub title: String,
    pub description: String,
    pub change_type: ChangeType,
    pub risk: Priority,
    pub impact: Impact,
    pub status: ChangeStatus,
    pub requester: String,
    pub approvers: Vec<String>,
    pub implementation_plan: String,
    pub rollback_plan: String,
    pub test_plan: Option<String>,
    pub cab_required: bool,
    pub linked_incident_ids: Vec<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCatalogItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub fulfillment_group: String,
    pub approval_required: bool,
    pub variables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeArticle {
    pub id: String,
    pub title: String,
    pub body: String,
    pub category: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}
