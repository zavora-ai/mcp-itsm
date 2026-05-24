# Changelog

## [1.0.0] - 2026-05-24

### Added
- 25 MCP tools: 17 CRUD + 3 agentic + 5 KB management
- `handle_support_request` — end-to-end intelligent workflow (classify → deduplicate → KB → resolve/create)
- `auto_triage` — reclassify, check SLA, find related incidents
- `diagnose_ticket` — KB search, pattern detection, resolution recommendations
- `create_kb_article` / `update_kb_article` / `delete_kb_article` / `list_kb_articles` — full KB lifecycle
- `add_classification_rule` — configurable keyword → category/queue/priority mapping
- TF-IDF scored knowledge base search (pluggable for adk-rag semantic search)
- Duplicate detection with 30% word-overlap threshold
- Priority auto-escalation from urgency keywords
- Full decision trace on every agentic tool call
- Service catalog with fulfillment groups
- Change management with CAB requirements
- SLA tracking with breach detection
