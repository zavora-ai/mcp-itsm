# ITSM MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-itsm.svg)](https://crates.io/crates/mcp-itsm)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)
[![Registry Ready](https://img.shields.io/badge/ADK_Registry-Ready-green.svg)](https://www.zavora.ai)

Intelligent IT Service Management for [ADK-Rust Enterprise](https://enterprise.adk-rust.com) agents. 25 MCP tools with agentic workflows — auto-classifies tickets, resolves from knowledge base, detects duplicates, and routes intelligently. No hardcoded rules — everything is configurable at runtime.

## What It Does

Your AI agent becomes an L1/L2 support operator. It can resolve common issues from the knowledge base without creating tickets, detect duplicate incidents, auto-classify and route, and escalate when needed — all learning from the KB and rules you feed it.

## Architecture

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-itsm/main/docs/architecture.svg" alt="ITSM MCP Architecture" width="750"/>
</p>

## Key Principles

- **No hardcoded logic** — classification rules and KB articles are added at runtime via tools.
- **KB-first resolution** — searches knowledge base before creating tickets. Grows smarter over time.
- **TF-IDF scoring** — relevance-ranked KB search (pluggable for `adk-rag` semantic search).
- **Duplicate detection** — 30% word-overlap threshold against all non-closed tickets.
- **Priority escalation** — urgency keywords ("outage", "all users", "critical") auto-escalate regardless of rule defaults.
- **Full audit trace** — every agentic decision is traced and returned.

## Verified Output

```
Setup: 3 classification rules + 2 KB articles added

1. "How do I reset my password?"
   → resolved_with_kb ✓ (KB: "How to reset your password", score: 13.3)

2. "Help VPN connection timeout"
   → resolved_with_kb ✓ (KB: "VPN troubleshooting", score: 13.3)

3. "VPN is down for all users in Nairobi office"
   → ticket_created | INC-1001 | critical | Network Team ✓

4. "VPN down for all users here too"
   → linked_to_existing ✓ (linked to INC-1001, no duplicate created)

5. "My laptop screen is flickering"
   → ticket_created | INC-1002 | Hardware Support ✓
```

## Tools (25)

### Agentic Workflows (3)

| Tool | What It Does | When To Use |
|------|-------------|-------------|
| `handle_support_request` | End-to-end: classify → deduplicate → KB search → resolve or create ticket | "Handle this user's issue" |
| `auto_triage` | Reclassify, check SLA risk, find related incidents | "Triage this ticket" |
| `diagnose_ticket` | Search KB, find patterns, recommend next action | "What should we do about this?" |

### Knowledge Base (5)

| Tool | What It Does |
|------|-------------|
| `create_kb_article` | Add article (title, body, category, tags) |
| `update_kb_article` | Update existing article |
| `delete_kb_article` | Remove article |
| `list_kb_articles` | List all, optionally by category |
| `search_knowledge_articles` | TF-IDF scored search |

### Classification (1)

| Tool | What It Does |
|------|-------------|
| `add_classification_rule` | Add keywords → category/queue/priority mapping |

### Ticket Lifecycle (11)

| Tool | What It Does |
|------|-------------|
| `create_ticket` | Create incident/request/task |
| `get_ticket` | Full details with history |
| `search_tickets` | Search by keyword/status/assignee/queue |
| `update_ticket_fields` | Change priority/category/service |
| `transition_ticket_status` | Move through workflow states |
| `assign_ticket` | Assign to user/team |
| `recommend_ticket_route` | Suggest queue (advisory) |
| `route_ticket` | Move to queue |
| `add_ticket_note` | Internal or customer-facing note |
| `close_ticket` | Close with resolution |
| `get_ticket_sla_status` | SLA deadlines and breach risk |

### Change & Catalog (5)

| Tool | What It Does |
|------|-------------|
| `create_change_request` | Open CHG with risk/impact/plans |
| `get_change_request` | Read CHG details |
| `get_service_catalog_item` | Look up catalog offering |
| `create_service_request` | Submit catalog request |
| `link_knowledge_article` | Attach KB to ticket |

## Installation

### 1. Build

```bash
git clone https://github.com/zavora-ai/mcp-itsm
cd mcp-itsm
cargo build --release
```

### 2. Add to your MCP client

**Claude Desktop / Kiro / Cursor / Windsurf:**
```json
{
  "mcpServers": {
    "itsm": {
      "command": "/path/to/mcp-itsm"
    }
  }
}
```

### 3. Configure (via tools)

```
> add_classification_rule(keywords: ["vpn","network","dns"], category: "Network", queue: "Network Team", default_priority: "high")
> add_classification_rule(keywords: ["password","login","access"], category: "Access", queue: "Identity Team", default_priority: "medium")
> create_kb_article(title: "VPN troubleshooting", body: "1. Restart client 2. Try alternate server", category: "Network", tags: ["vpn","timeout"])
```

### 4. Use it

```
> handle_support_request(message: "I can't connect to VPN", requester: "james")
```

## How It Gets Smarter

1. **Add KB articles** → more issues resolved without tickets
2. **Add classification rules** → better routing and priority
3. **Close tickets with resolution notes** → `diagnose_ticket` learns from past resolutions
4. **Duplicate detection** → fewer redundant tickets as the system sees more patterns

## Pluggable for adk-rag

The TF-IDF search is designed to be swapped for semantic search:

```rust
// Current: TF-IDF (built-in, no dependencies)
let results = store.search_articles("vpn timeout");

// Future: adk-rag with Gemini embeddings
let pipeline = RagPipeline::builder()
    .embedding_provider(Arc::new(GeminiEmbeddingProvider::new(key)))
    .vector_store(Arc::new(InMemoryVectorStore::new()))
    .build()?;
let results = pipeline.query("kb", "vpn timeout").await?;
```

## MCP Server Manifest

```toml
server_id = "mcp_itsm"
display_name = "ITSM MCP"
version = "1.0.0"
domain = "it_operations"
risk_level = "medium"
writes_allowed = "gated"
transports = ["stdio"]
governance_gates = ["change_requires_cab", "closure_requires_resolution"]
```

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START -->
| [<img src="https://github.com/jkmaina.png" width="80px;" alt=""/><br /><sub><b>James Karanja Maina</b></sub>](https://github.com/jkmaina) |
|:---:|
<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Apache-2.0 — see [LICENSE](LICENSE) for details.

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

## Registry Compliance

This server implements the [ADK MCP SDK](https://crates.io/crates/adk-mcp-sdk) contract:

- **HealthCheck** — async health probe for registry monitoring
- **mcp-server.toml** — manifest declaring tools, risk classes, and credentials
- **Structured tracing** — `RUST_LOG` env-filter for observability

