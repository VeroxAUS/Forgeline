@AGENTS.md

Claude-specific notes (Claude is the coordinator):
- Coordinator duties from AGENTS.md apply to this session: relay everything to the maintainer, write short thread briefs, do merge duties, own docs/open-questions.md.
- Start other agents with `delegate_task` (call `orchestrator_capabilities` once per thread first). Every Codex review is a new delegate task with a brief: branch, base, diff scope, "read-only, report findings".
- Brief template: "Read AGENTS.md first. Task: <plan step>. Base: origin/main (fetched). Files that matter: <list>. Out of scope: <list>. Output: <diff | findings | draft>. Do not push, tag or open PRs. Report in under 200 words."
