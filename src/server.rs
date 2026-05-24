use crate::store::ItsmStore;
use crate::types::*;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateTicketInput { pub ticket_type: TicketType, pub title: String, pub description: String, pub priority: Priority, pub category: String, pub requester: String, #[serde(default = "default_queue")] pub queue: String }
fn default_queue() -> String { "L1 Support".into() }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetTicketInput { pub id: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchTicketsInput { pub query: Option<String>, pub status: Option<String>, pub assignee: Option<String>, pub queue: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateTicketFieldsInput { pub id: String, pub priority: Option<Priority>, pub category: Option<String>, pub service: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TransitionStatusInput { pub id: String, pub status: TicketStatus }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AssignTicketInput { pub id: String, pub assignee: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RecommendRouteInput { pub id: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RouteTicketInput { pub id: String, pub queue: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddNoteInput { pub id: String, pub body: String, #[serde(default = "default_internal")] pub visibility: NoteVisibility, pub author: String }
fn default_internal() -> NoteVisibility { NoteVisibility::Internal }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CloseTicketInput { pub id: String, pub resolution_code: String, pub resolution_notes: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetSlaStatusInput { pub id: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateChangeInput { pub title: String, pub description: String, pub change_type: ChangeType, pub risk: Priority, pub impact: Impact, pub implementation_plan: String, pub rollback_plan: String, pub requester: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetChangeInput { pub id: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetCatalogItemInput { pub id: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateServiceRequestInput { pub catalog_item_id: String, pub requester: String, pub variables: Option<serde_json::Value> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchKbInput { pub query: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LinkArticleInput { pub ticket_id: String, pub article_id: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateKbArticleInput { pub title: String, pub body: String, pub category: String, #[serde(default)] pub tags: Vec<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateKbArticleInput { pub id: String, pub title: Option<String>, pub body: Option<String>, pub tags: Option<Vec<String>> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DeleteKbArticleInput { pub id: String }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListKbArticlesInput { pub category: Option<String> }

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddClassificationRuleInput { pub keywords: Vec<String>, pub category: String, pub queue: String, pub default_priority: String }

// --- Agentic tools ---

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct HandleSupportRequestInput {
    /// The user's issue description in natural language
    pub message: String,
    /// Who is reporting the issue
    pub requester: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AutoTriageInput {
    /// Ticket ID to auto-triage (classify, prioritize, route)
    pub ticket_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DiagnoseTicketInput {
    /// Ticket ID to diagnose (search KB, find related incidents, recommend action)
    pub ticket_id: String,
}

#[derive(Clone)]
pub struct ItsmServer { pub store: Arc<ItsmStore> }

#[tool_router(server_handler)]
impl ItsmServer {
    #[tool(description = "Create an incident, service request, or task ticket")]
    fn create_ticket(&self, Parameters(i): Parameters<CreateTicketInput>) -> String {
        let t = self.store.create_ticket(i.ticket_type, i.title, i.description, i.priority, i.category, i.requester, i.queue);
        serde_json::to_string_pretty(&serde_json::json!({"id": t.id, "status": t.status, "priority": t.priority, "queue": t.queue, "sla_policy": t.sla.policy})).unwrap()
    }

    #[tool(description = "Get full ticket details including status, assignee, SLA, notes, and related records")]
    fn get_ticket(&self, Parameters(i): Parameters<GetTicketInput>) -> String {
        match self.store.get_ticket(&i.id) {
            Some(t) => serde_json::to_string_pretty(&t).unwrap(),
            None => format!("Ticket not found: {}", i.id),
        }
    }

    #[tool(description = "Search tickets by keyword, status, assignee, or queue")]
    fn search_tickets(&self, Parameters(i): Parameters<SearchTicketsInput>) -> String {
        let results = self.store.search_tickets(i.query.as_deref(), i.status.as_deref(), i.assignee.as_deref(), i.queue.as_deref());
        let summary: Vec<serde_json::Value> = results.iter().map(|t| serde_json::json!({"id": t.id, "title": t.title, "status": t.status, "priority": t.priority, "assignee": t.assignee, "queue": t.queue})).collect();
        serde_json::to_string_pretty(&serde_json::json!({"count": summary.len(), "tickets": summary})).unwrap()
    }

    #[tool(description = "Update ticket priority, category, or service fields")]
    fn update_ticket_fields(&self, Parameters(i): Parameters<UpdateTicketFieldsInput>) -> String {
        match self.store.update_ticket_fields(&i.id, i.priority, i.category, i.service) {
            Ok(t) => serde_json::to_string_pretty(&serde_json::json!({"id": t.id, "priority": t.priority, "category": t.category, "updated": true})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Move ticket through valid workflow states (New → Open → InProgress → Pending → Resolved → Closed)")]
    fn transition_ticket_status(&self, Parameters(i): Parameters<TransitionStatusInput>) -> String {
        match self.store.transition_status(&i.id, i.status) {
            Ok(t) => serde_json::to_string_pretty(&serde_json::json!({"id": t.id, "status": t.status, "updated_at": t.updated_at})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Assign ticket to a user or team")]
    fn assign_ticket(&self, Parameters(i): Parameters<AssignTicketInput>) -> String {
        match self.store.assign_ticket(&i.id, &i.assignee) {
            Ok(t) => serde_json::to_string_pretty(&serde_json::json!({"id": t.id, "assignee": t.assignee, "status": t.status})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Recommend best queue/team for a ticket based on classification rules (advisory, does not move ticket)")]
    fn recommend_ticket_route(&self, Parameters(i): Parameters<RecommendRouteInput>) -> String {
        let ticket = match self.store.get_ticket(&i.id) {
            Some(t) => t,
            None => return format!("Ticket not found: {}", i.id),
        };
        let text = format!("{} {}", ticket.title, ticket.description);
        let (category, queue, priority, confidence) = self.store.classify(&text);
        serde_json::to_string_pretty(&serde_json::json!({
            "ticket_id": ticket.id,
            "recommended_queue": queue,
            "suggested_category": category,
            "suggested_priority": priority,
            "confidence": confidence,
            "current_queue": ticket.queue,
        })).unwrap()
    }

    #[tool(description = "Move ticket to a different queue/team")]
    fn route_ticket(&self, Parameters(i): Parameters<RouteTicketInput>) -> String {
        match self.store.route_ticket(&i.id, &i.queue) {
            Ok(t) => serde_json::to_string_pretty(&serde_json::json!({"id": t.id, "queue": t.queue, "routed": true})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Add an internal or customer-facing note to a ticket")]
    fn add_ticket_note(&self, Parameters(i): Parameters<AddNoteInput>) -> String {
        match self.store.add_note(&i.id, &i.body, i.visibility, &i.author) {
            Ok(n) => serde_json::to_string_pretty(&serde_json::json!({"note_id": n.id, "visibility": n.visibility, "added": true})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Close ticket with resolution code, notes, and evidence")]
    fn close_ticket(&self, Parameters(i): Parameters<CloseTicketInput>) -> String {
        match self.store.close_ticket(&i.id, &i.resolution_code, &i.resolution_notes) {
            Ok(t) => serde_json::to_string_pretty(&serde_json::json!({"id": t.id, "status": t.status, "resolution_code": t.resolution_code, "resolved_at": t.resolved_at})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Get SLA deadlines, breach state, and time remaining for a ticket")]
    fn get_ticket_sla_status(&self, Parameters(i): Parameters<GetSlaStatusInput>) -> String {
        match self.store.get_sla_status(&i.id) {
            Ok(v) => serde_json::to_string_pretty(&v).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Open a change request with risk, impact, implementation plan, and rollback plan")]
    fn create_change_request(&self, Parameters(i): Parameters<CreateChangeInput>) -> String {
        let cr = self.store.create_change_request(i.title, i.description, i.change_type, i.risk, i.impact, i.implementation_plan, i.rollback_plan, i.requester);
        serde_json::to_string_pretty(&serde_json::json!({"id": cr.id, "status": cr.status, "cab_required": cr.cab_required, "change_type": cr.change_type})).unwrap()
    }

    #[tool(description = "Get change request details including approvals, schedule, and linked incidents")]
    fn get_change_request(&self, Parameters(i): Parameters<GetChangeInput>) -> String {
        match self.store.get_change_request(&i.id) {
            Some(cr) => serde_json::to_string_pretty(&cr).unwrap(),
            None => format!("Change request not found: {}", i.id),
        }
    }

    #[tool(description = "Look up a service catalog offering and its required variables")]
    fn get_service_catalog_item(&self, Parameters(i): Parameters<GetCatalogItemInput>) -> String {
        match self.store.get_catalog_item(&i.id) {
            Some(item) => serde_json::to_string_pretty(&item).unwrap(),
            None => format!("Catalog item not found: {}", i.id),
        }
    }

    #[tool(description = "Submit a service request from a catalog item with requester and variables")]
    fn create_service_request(&self, Parameters(i): Parameters<CreateServiceRequestInput>) -> String {
        let item = self.store.get_catalog_item(&i.catalog_item_id);
        match item {
            Some(cat) => {
                let t = self.store.create_ticket(TicketType::ServiceRequest, format!("Service Request: {}", cat.name), cat.description.clone(), Priority::Medium, cat.category.clone(), i.requester, cat.fulfillment_group.clone());
                serde_json::to_string_pretty(&serde_json::json!({"ticket_id": t.id, "catalog_item": cat.name, "fulfillment_group": cat.fulfillment_group, "approval_required": cat.approval_required})).unwrap()
            }
            None => format!("Catalog item not found: {}", i.catalog_item_id),
        }
    }

    #[tool(description = "Search knowledge base articles using TF-IDF scoring. Returns ranked results with relevance scores.")]
    fn search_knowledge_articles(&self, Parameters(i): Parameters<SearchKbInput>) -> String {
        let results = self.store.search_articles(&i.query);
        serde_json::to_string_pretty(&serde_json::json!({"count": results.len(), "articles": results.iter().map(|(a, s)| serde_json::json!({"id": a.id, "title": a.title, "category": a.category, "score": s})).collect::<Vec<_>>()})).unwrap()
    }

    #[tool(description = "Attach a knowledge article to a ticket or resolution")]
    fn link_knowledge_article(&self, Parameters(i): Parameters<LinkArticleInput>) -> String {
        match self.store.link_article(&i.ticket_id, &i.article_id) {
            Ok(()) => serde_json::to_string_pretty(&serde_json::json!({"linked": true, "ticket_id": i.ticket_id, "article_id": i.article_id})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Create a knowledge base article. The KB grows over time and improves auto-resolution.")]
    fn create_kb_article(&self, Parameters(i): Parameters<CreateKbArticleInput>) -> String {
        let article = self.store.create_article(i.title, i.body, i.category, i.tags);
        serde_json::to_string_pretty(&serde_json::json!({"id": article.id, "title": article.title, "category": article.category, "created": true})).unwrap()
    }

    #[tool(description = "Update an existing knowledge base article")]
    fn update_kb_article(&self, Parameters(i): Parameters<UpdateKbArticleInput>) -> String {
        match self.store.update_article(&i.id, i.title, i.body, i.tags) {
            Ok(a) => serde_json::to_string_pretty(&serde_json::json!({"id": a.id, "title": a.title, "updated": true})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Delete a knowledge base article")]
    fn delete_kb_article(&self, Parameters(i): Parameters<DeleteKbArticleInput>) -> String {
        match self.store.delete_article(&i.id) {
            Ok(()) => serde_json::to_string_pretty(&serde_json::json!({"deleted": true, "id": i.id})).unwrap(),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "List all knowledge base articles, optionally filtered by category")]
    fn list_kb_articles(&self, Parameters(i): Parameters<ListKbArticlesInput>) -> String {
        let articles = self.store.list_articles(i.category.as_deref());
        serde_json::to_string_pretty(&serde_json::json!({"count": articles.len(), "articles": articles})).unwrap()
    }

    #[tool(description = "Add a classification rule. Rules map keywords to category/queue/priority. The more rules, the smarter the auto-classification.")]
    fn add_classification_rule(&self, Parameters(i): Parameters<AddClassificationRuleInput>) -> String {
        use crate::store::ClassificationRule;
        self.store.add_classification_rule(ClassificationRule { keywords: i.keywords.clone(), category: i.category.clone(), queue: i.queue.clone(), default_priority: i.default_priority.clone() });
        serde_json::to_string_pretty(&serde_json::json!({"added": true, "keywords": i.keywords, "category": i.category, "queue": i.queue})).unwrap()
    }

    // ═══════════════════════════════════════════════════════════════════
    // AGENTIC TOOLS — intelligent workflows that compose the above tools
    // ═══════════════════════════════════════════════════════════════════

    #[tool(description = "Handle a support request end-to-end: classify → deduplicate → search KB (TF-IDF scored) → auto-resolve or create ticket → route. Uses configurable classification rules and knowledge base.")]
    fn handle_support_request(&self, Parameters(i): Parameters<HandleSupportRequestInput>) -> String {
        let mut trace = Vec::new();

        // Step 1: Classify using configurable rules
        let (category, queue, priority, confidence) = self.store.classify(&i.message);
        trace.push(serde_json::json!({"step": "classify", "category": &category, "queue": &queue, "priority": &priority, "confidence": confidence}));

        // Step 2: Deduplicate — search non-closed tickets for similar issues
        let existing = self.store.search_tickets(None, None, None, None);
        let existing: Vec<_> = existing.into_iter().filter(|t| t.status != TicketStatus::Closed).collect();
        let msg_lower = i.message.to_lowercase();
        let msg_words: Vec<&str> = msg_lower.split_whitespace().filter(|w| w.len() > 3).collect();
        let duplicate = existing.iter().find(|t| {
            let title_lower = t.title.to_lowercase();
            let desc_lower = t.description.to_lowercase();
            let doc = format!("{} {}", title_lower, desc_lower);
            let matches = msg_words.iter().filter(|w| doc.contains(*w)).count();
            // Require at least 30% of significant words to match
            msg_words.len() > 0 && (matches as f64 / msg_words.len() as f64) >= 0.3
        });

        if let Some(dup) = duplicate {
            let _ = self.store.add_note(&dup.id, &format!("User {} also affected: {}", i.requester, i.message), NoteVisibility::Internal, "itsm-agent");
            trace.push(serde_json::json!({"step": "deduplicate", "found": dup.id}));
            return serde_json::to_string_pretty(&serde_json::json!({
                "outcome": "linked_to_existing",
                "existing_ticket": dup.id,
                "message": format!("There's an existing incident {} for this issue. You've been linked.", dup.id),
                "trace": trace,
            })).unwrap();
        }
        trace.push(serde_json::json!({"step": "deduplicate", "result": "no_duplicate"}));

        // Step 3: Search KB with TF-IDF scoring
        let kb_results = self.store.search_articles(&i.message);
        if let Some((article, score)) = kb_results.first() {
            trace.push(serde_json::json!({"step": "kb_search", "found": article.id, "title": &article.title, "score": score}));

            // If it's a help/how-to question and KB score is high, resolve with KB
            let msg_lower = i.message.to_lowercase();
            if *score > 3.0 && (msg_lower.contains("how") || msg_lower.contains("help") || msg_lower.contains("reset") || msg_lower.contains("where")) {
                return serde_json::to_string_pretty(&serde_json::json!({
                    "outcome": "resolved_with_kb",
                    "article": {"id": &article.id, "title": &article.title, "body": &article.body, "relevance_score": score},
                    "message": format!("I found a solution: {}\n\n{}", article.title, article.body),
                    "trace": trace,
                })).unwrap();
            }
        } else {
            trace.push(serde_json::json!({"step": "kb_search", "result": "no_match"}));
        }

        // Step 4: Create ticket
        let prio = match priority.as_str() {
            "critical" => Priority::Critical, "high" => Priority::High, "low" => Priority::Low, _ => Priority::Medium,
        };
        let ticket_type = if i.message.to_lowercase().contains("request") || i.message.to_lowercase().contains("access") || i.message.to_lowercase().contains("new") {
            TicketType::ServiceRequest
        } else { TicketType::Incident };
        let ticket = self.store.create_ticket(ticket_type, i.message.clone(), i.message.clone(), prio, category.clone(), i.requester.clone(), queue.clone());
        trace.push(serde_json::json!({"step": "create_ticket", "id": &ticket.id, "type": &ticket.ticket_type}));

        // Step 5: Attach KB if found
        let kb_attached = if let Some((article, _)) = kb_results.first() {
            let _ = self.store.link_article(&ticket.id, &article.id);
            let _ = self.store.add_note(&ticket.id, &format!("Related KB: {} - {}", article.id, article.title), NoteVisibility::Internal, "itsm-agent");
            true
        } else { false };

        serde_json::to_string_pretty(&serde_json::json!({
            "outcome": "ticket_created",
            "ticket_id": ticket.id,
            "priority": ticket.priority,
            "category": category,
            "queue": queue,
            "classification_confidence": confidence,
            "kb_attached": kb_attached,
            "message": format!("Created {} — priority {:?}, routed to {}.", ticket.id, ticket.priority, queue),
            "trace": trace,
        })).unwrap()
    }

    #[tool(description = "Auto-triage a ticket: reclassify using rules, check SLA risk, find related incidents, recommend actions.")]
    fn auto_triage(&self, Parameters(i): Parameters<AutoTriageInput>) -> String {
        let ticket = match self.store.get_ticket(&i.ticket_id) {
            Some(t) => t,
            None => return format!("Ticket not found: {}", i.ticket_id),
        };

        let text = format!("{} {}", ticket.title, ticket.description);
        let (suggested_category, suggested_queue, suggested_priority, confidence) = self.store.classify(&text);
        let sla = self.store.get_sla_status(&i.ticket_id).unwrap_or(serde_json::json!({}));
        let related = self.store.search_tickets(Some(&ticket.category), Some("open"), None, None);
        let related_count = related.iter().filter(|t| t.id != ticket.id).count();

        serde_json::to_string_pretty(&serde_json::json!({
            "ticket_id": ticket.id,
            "current": {"priority": ticket.priority, "category": ticket.category, "queue": ticket.queue},
            "suggested": {"priority": suggested_priority, "category": suggested_category, "queue": suggested_queue, "confidence": confidence},
            "sla": sla,
            "related_open_incidents": related_count,
            "recommendations": {
                "reprioritize": suggested_priority != format!("{:?}", ticket.priority).to_lowercase(),
                "reroute": suggested_queue != ticket.queue,
                "link_related": related_count > 0,
            },
        })).unwrap()
    }

    #[tool(description = "Diagnose a ticket: search KB (TF-IDF), find related/resolved incidents, detect patterns, recommend next action.")]
    fn diagnose_ticket(&self, Parameters(i): Parameters<DiagnoseTicketInput>) -> String {
        let ticket = match self.store.get_ticket(&i.ticket_id) {
            Some(t) => t,
            None => return format!("Ticket not found: {}", i.ticket_id),
        };

        let kb_results = self.store.search_articles(&format!("{} {}", ticket.title, ticket.description));
        let related = self.store.search_tickets(Some(&ticket.category), None, None, None);
        let related_open: Vec<_> = related.iter().filter(|t| t.id != ticket.id && t.status != TicketStatus::Closed).collect();
        let related_resolved: Vec<_> = related.iter().filter(|t| t.id != ticket.id && t.status == TicketStatus::Closed && t.resolution_notes.is_some()).collect();

        let recommendation = if !kb_results.is_empty() {
            format!("KB match: {} (score: {:.1}) — try this solution first", kb_results[0].0.title, kb_results[0].1)
        } else if !related_resolved.is_empty() {
            format!("Similar ticket {} resolved with: {}", related_resolved[0].id, related_resolved[0].resolution_notes.as_deref().unwrap_or("?"))
        } else if related_open.len() >= 3 {
            "Multiple similar incidents open — possible systemic issue.".into()
        } else {
            "No KB match or pattern. Escalate to specialist.".into()
        };

        serde_json::to_string_pretty(&serde_json::json!({
            "ticket_id": ticket.id,
            "diagnosis": {
                "kb_matches": kb_results.iter().take(3).map(|(a, s)| serde_json::json!({"id": a.id, "title": a.title, "score": s})).collect::<Vec<_>>(),
                "related_open": related_open.len(),
                "related_resolved": related_resolved.len(),
                "past_resolutions": related_resolved.iter().take(2).map(|t| serde_json::json!({"id": t.id, "resolution": t.resolution_notes})).collect::<Vec<_>>(),
                "pattern_detected": related_open.len() >= 3,
            },
            "recommendation": recommendation,
        })).unwrap()
    }
}

