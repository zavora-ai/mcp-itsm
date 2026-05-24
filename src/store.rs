use crate::types::*;
use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

pub struct ItsmStore {
    tickets: Mutex<HashMap<String, Ticket>>,
    changes: Mutex<HashMap<String, ChangeRequest>>,
    catalog: Mutex<Vec<ServiceCatalogItem>>,
    articles: Mutex<Vec<KnowledgeArticle>>,
    classification_rules: Mutex<Vec<ClassificationRule>>,
    next_inc: Mutex<u64>,
    next_chg: Mutex<u64>,
}

/// Configurable classification rule (replaces hardcoded if/else)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClassificationRule {
    pub keywords: Vec<String>,
    pub category: String,
    pub queue: String,
    pub default_priority: String,
}

impl ItsmStore {
    pub fn new() -> Self {
        Self {
            tickets: Mutex::new(HashMap::new()),
            changes: Mutex::new(HashMap::new()),
            catalog: Mutex::new(Vec::new()),
            articles: Mutex::new(Vec::new()),
            classification_rules: Mutex::new(Vec::new()),
            next_inc: Mutex::new(1000),
            next_chg: Mutex::new(100),
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // KB Management
    // ═══════════════════════════════════════════════════════════════

    pub fn create_article(&self, title: String, body: String, category: String, tags: Vec<String>) -> KnowledgeArticle {
        let article = KnowledgeArticle {
            id: format!("KB-{}", Uuid::new_v4().simple().to_string()[..6].to_uppercase()),
            title, body, category, tags, created_at: Utc::now(),
        };
        self.articles.lock().unwrap().push(article.clone());
        article
    }

    pub fn update_article(&self, id: &str, title: Option<String>, body: Option<String>, tags: Option<Vec<String>>) -> Result<KnowledgeArticle, String> {
        let mut articles = self.articles.lock().unwrap();
        let a = articles.iter_mut().find(|a| a.id == id).ok_or_else(|| format!("Article not found: {}", id))?;
        if let Some(t) = title { a.title = t; }
        if let Some(b) = body { a.body = b; }
        if let Some(t) = tags { a.tags = t; }
        Ok(a.clone())
    }

    pub fn delete_article(&self, id: &str) -> Result<(), String> {
        let mut articles = self.articles.lock().unwrap();
        let len_before = articles.len();
        articles.retain(|a| a.id != id);
        if articles.len() == len_before { Err(format!("Article not found: {}", id)) } else { Ok(()) }
    }

    pub fn list_articles(&self, category: Option<&str>) -> Vec<KnowledgeArticle> {
        self.articles.lock().unwrap().iter()
            .filter(|a| category.map_or(true, |c| a.category.to_lowercase() == c.to_lowercase()))
            .cloned().collect()
    }

    /// TF-IDF inspired scoring: score = sum of (1/doc_frequency) for each matching term
    pub fn search_articles(&self, query: &str) -> Vec<(KnowledgeArticle, f64)> {
        let terms: Vec<String> = query.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2 && !STOP_WORDS.contains(w))
            .map(|s| s.to_string())
            .collect();

        if terms.is_empty() { return Vec::new(); }

        let articles = self.articles.lock().unwrap();
        let total_docs = articles.len().max(1) as f64;

        let mut scored: Vec<(KnowledgeArticle, f64)> = articles.iter().map(|article| {
            let doc_text = format!("{} {} {} {}", article.title, article.body, article.category, article.tags.join(" ")).to_lowercase();
            let mut score = 0.0;
            for term in &terms {
                if doc_text.contains(term.as_str()) {
                    // IDF-like: rarer terms score higher
                    let doc_freq = articles.iter().filter(|a| {
                        let t = format!("{} {} {}", a.title, a.body, a.tags.join(" ")).to_lowercase();
                        t.contains(term.as_str())
                    }).count() as f64;
                    score += (total_docs / doc_freq.max(1.0)).ln() + 1.0;
                }
                // Boost for title/tag match
                if article.title.to_lowercase().contains(term.as_str()) { score += 2.0; }
                if article.tags.iter().any(|t| t.to_lowercase().contains(term.as_str())) { score += 1.5; }
            }
            (article.clone(), score)
        }).filter(|(_, score)| *score > 0.0).collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(5);
        scored
    }

    // ═══════════════════════════════════════════════════════════════
    // Classification Rules (configurable, not hardcoded)
    // ═══════════════════════════════════════════════════════════════

    pub fn add_classification_rule(&self, rule: ClassificationRule) {
        self.classification_rules.lock().unwrap().push(rule);
    }

    pub fn list_classification_rules(&self) -> Vec<ClassificationRule> {
        self.classification_rules.lock().unwrap().clone()
    }

    /// Classify using configurable rules. Returns (category, queue, priority, confidence)
    pub fn classify(&self, text: &str) -> (String, String, String, f64) {
        let lower = text.to_lowercase();
        let rules = self.classification_rules.lock().unwrap();

        let mut best_match: Option<(&ClassificationRule, usize)> = None;
        for rule in rules.iter() {
            let hits = rule.keywords.iter().filter(|kw| lower.contains(kw.as_str())).count();
            if hits > 0 {
                if best_match.map_or(true, |(_, best)| hits > best) {
                    best_match = Some((rule, hits));
                }
            }
        }

        if let Some((rule, hits)) = best_match {
            let confidence = (hits as f64 / rule.keywords.len() as f64).min(1.0);
            // Escalate priority if urgency keywords present
            let priority = if lower.contains("urgent") || lower.contains("critical") || lower.contains("outage") || lower.contains("all users") || lower.contains("production") {
                "critical".to_string()
            } else if lower.contains("down") || lower.contains("broken") || lower.contains("cannot") || lower.contains("blocked") {
                "high".to_string()
            } else {
                rule.default_priority.clone()
            };
            return (rule.category.clone(), rule.queue.clone(), priority, confidence);
        }

        // Fallback: priority from urgency keywords
        let priority = if lower.contains("urgent") || lower.contains("critical") || lower.contains("outage") || lower.contains("all users") {
            "critical"
        } else if lower.contains("down") || lower.contains("broken") || lower.contains("cannot") || lower.contains("blocked") {
            "high"
        } else { "medium" };

        ("General".into(), "L1 Support".into(), priority.into(), 0.3)
    }

    // ═══════════════════════════════════════════════════════════════
    // Tickets
    // ═══════════════════════════════════════════════════════════════

    pub fn create_ticket(&self, ticket_type: TicketType, title: String, description: String, priority: Priority, category: String, requester: String, queue: String) -> Ticket {
        let mut n = self.next_inc.lock().unwrap();
        *n += 1;
        let prefix = match ticket_type { TicketType::Incident => "INC", TicketType::ServiceRequest => "REQ", TicketType::Task => "TASK" };
        let id = format!("{}-{}", prefix, n);
        let now = Utc::now();
        let ticket = Ticket {
            id: id.clone(), ticket_type, title, description, status: TicketStatus::New,
            priority: priority.clone(), impact: Impact::Medium, category, subcategory: None, service: None,
            assignee: None, queue, requester,
            sla: SlaStatus {
                policy: match &priority { Priority::Critical => "P1 - 1hr response, 4hr resolve", Priority::High => "P2 - 4hr response, 8hr resolve", _ => "P3 - 8hr response, 24hr resolve" }.into(),
                response_due: now + Duration::hours(match &priority { Priority::Critical => 1, Priority::High => 4, _ => 8 }),
                resolution_due: now + Duration::hours(match &priority { Priority::Critical => 4, Priority::High => 8, _ => 24 }),
                response_breached: false, resolution_breached: false,
            },
            notes: Vec::new(), related_change_ids: Vec::new(), resolution_code: None, resolution_notes: None,
            knowledge_articles: Vec::new(), created_at: now, updated_at: now, resolved_at: None,
        };
        self.tickets.lock().unwrap().insert(id, ticket.clone());
        ticket
    }

    pub fn get_ticket(&self, id: &str) -> Option<Ticket> {
        self.tickets.lock().unwrap().get(id).cloned()
    }

    pub fn search_tickets(&self, query: Option<&str>, status: Option<&str>, assignee: Option<&str>, queue: Option<&str>) -> Vec<Ticket> {
        self.tickets.lock().unwrap().values()
            .filter(|t| query.map_or(true, |q| { let ql = q.to_lowercase(); t.title.to_lowercase().contains(&ql) || t.description.to_lowercase().contains(&ql) }))
            .filter(|t| status.map_or(true, |s| format!("{:?}", t.status).to_lowercase() == s.to_lowercase()))
            .filter(|t| assignee.map_or(true, |a| t.assignee.as_deref() == Some(a)))
            .filter(|t| queue.map_or(true, |q| t.queue == q))
            .cloned().collect()
    }

    pub fn update_ticket_fields(&self, id: &str, priority: Option<Priority>, category: Option<String>, service: Option<String>) -> Result<Ticket, String> {
        let mut tickets = self.tickets.lock().unwrap();
        let t = tickets.get_mut(id).ok_or_else(|| format!("Ticket not found: {}", id))?;
        if let Some(p) = priority { t.priority = p; }
        if let Some(c) = category { t.category = c; }
        if let Some(s) = service { t.service = Some(s); }
        t.updated_at = Utc::now();
        Ok(t.clone())
    }

    pub fn transition_status(&self, id: &str, new_status: TicketStatus) -> Result<Ticket, String> {
        let mut tickets = self.tickets.lock().unwrap();
        let t = tickets.get_mut(id).ok_or_else(|| format!("Ticket not found: {}", id))?;
        t.status = new_status;
        t.updated_at = Utc::now();
        Ok(t.clone())
    }

    pub fn assign_ticket(&self, id: &str, assignee: &str) -> Result<Ticket, String> {
        let mut tickets = self.tickets.lock().unwrap();
        let t = tickets.get_mut(id).ok_or_else(|| format!("Ticket not found: {}", id))?;
        t.assignee = Some(assignee.to_string());
        if t.status == TicketStatus::New { t.status = TicketStatus::Open; }
        t.updated_at = Utc::now();
        Ok(t.clone())
    }

    pub fn route_ticket(&self, id: &str, queue: &str) -> Result<Ticket, String> {
        let mut tickets = self.tickets.lock().unwrap();
        let t = tickets.get_mut(id).ok_or_else(|| format!("Ticket not found: {}", id))?;
        t.queue = queue.to_string();
        t.updated_at = Utc::now();
        Ok(t.clone())
    }

    pub fn add_note(&self, id: &str, body: &str, visibility: NoteVisibility, author: &str) -> Result<Note, String> {
        let mut tickets = self.tickets.lock().unwrap();
        let t = tickets.get_mut(id).ok_or_else(|| format!("Ticket not found: {}", id))?;
        let note = Note { id: format!("note_{}", Uuid::new_v4().simple()), visibility, author: author.to_string(), body: body.to_string(), created_at: Utc::now() };
        t.notes.push(note.clone());
        t.updated_at = Utc::now();
        Ok(note)
    }

    pub fn close_ticket(&self, id: &str, resolution_code: &str, resolution_notes: &str) -> Result<Ticket, String> {
        let mut tickets = self.tickets.lock().unwrap();
        let t = tickets.get_mut(id).ok_or_else(|| format!("Ticket not found: {}", id))?;
        t.status = TicketStatus::Closed;
        t.resolution_code = Some(resolution_code.to_string());
        t.resolution_notes = Some(resolution_notes.to_string());
        t.resolved_at = Some(Utc::now());
        t.updated_at = Utc::now();
        Ok(t.clone())
    }

    pub fn get_sla_status(&self, id: &str) -> Result<serde_json::Value, String> {
        let tickets = self.tickets.lock().unwrap();
        let t = tickets.get(id).ok_or_else(|| format!("Ticket not found: {}", id))?;
        let now = Utc::now();
        Ok(serde_json::json!({
            "ticket_id": t.id, "policy": t.sla.policy,
            "response_due": t.sla.response_due, "resolution_due": t.sla.resolution_due,
            "response_breached": now > t.sla.response_due && t.status == TicketStatus::New,
            "resolution_breached": now > t.sla.resolution_due && t.status != TicketStatus::Closed,
            "time_to_response_breach_min": (t.sla.response_due - now).num_minutes(),
            "time_to_resolution_breach_min": (t.sla.resolution_due - now).num_minutes(),
        }))
    }

    pub fn link_article(&self, ticket_id: &str, article_id: &str) -> Result<(), String> {
        let mut tickets = self.tickets.lock().unwrap();
        let t = tickets.get_mut(ticket_id).ok_or_else(|| format!("Ticket not found: {}", ticket_id))?;
        t.knowledge_articles.push(article_id.to_string());
        t.updated_at = Utc::now();
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════
    // Change Management
    // ═══════════════════════════════════════════════════════════════

    pub fn create_change_request(&self, title: String, description: String, change_type: ChangeType, risk: Priority, impact: Impact, implementation_plan: String, rollback_plan: String, requester: String) -> ChangeRequest {
        let mut n = self.next_chg.lock().unwrap();
        *n += 1;
        let id = format!("CHG-{:04}", n);
        let cab_required = matches!((&risk, &change_type), (Priority::High | Priority::Critical, _) | (_, ChangeType::Emergency));
        let cr = ChangeRequest {
            id: id.clone(), title, description, change_type, risk, impact,
            status: ChangeStatus::Draft, requester, approvers: Vec::new(),
            implementation_plan, rollback_plan, test_plan: None,
            cab_required, linked_incident_ids: Vec::new(), scheduled_at: None, created_at: Utc::now(),
        };
        self.changes.lock().unwrap().insert(id, cr.clone());
        cr
    }

    pub fn get_change_request(&self, id: &str) -> Option<ChangeRequest> {
        self.changes.lock().unwrap().get(id).cloned()
    }

    // ═══════════════════════════════════════════════════════════════
    // Service Catalog
    // ═══════════════════════════════════════════════════════════════

    pub fn add_catalog_item(&self, item: ServiceCatalogItem) {
        self.catalog.lock().unwrap().push(item);
    }

    pub fn get_catalog_item(&self, id: &str) -> Option<ServiceCatalogItem> {
        self.catalog.lock().unwrap().iter().find(|c| c.id == id || c.name.to_lowercase().contains(&id.to_lowercase())).cloned()
    }

    pub fn list_catalog(&self) -> Vec<ServiceCatalogItem> {
        self.catalog.lock().unwrap().clone()
    }
}

const STOP_WORDS: &[&str] = &[
    "the", "and", "for", "are", "but", "not", "you", "all", "can", "had", "her",
    "was", "one", "our", "out", "has", "have", "been", "from", "with", "they",
    "this", "that", "what", "when", "how", "who", "will", "each", "make", "like",
    "just", "over", "such", "take", "than", "them", "very", "some", "could",
    "into", "year", "then", "also", "back", "after", "use", "two", "way",
    "about", "many", "need", "help", "does",
];
