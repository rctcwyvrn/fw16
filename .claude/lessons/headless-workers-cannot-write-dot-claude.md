# Headless `claude -p --permission-mode acceptEdits` workers are denied every write under `.claude/`

Even with Edit/Write in `--allowedTools`, the Path 3 worker (2026-10-29) could not append session logs or write
lessons in `.claude/`. Workers should put log entries and lessons in their `_build/agents/<lane>.report.md`, and
the orchestrator copies them into `.claude/chat/` and `.claude/lessons/` and commits them.
Also denied or approval-gated headless: `curl`, compound `sed -i`, `rm`/`mkdir`/`cp` into `/tmp`, multi-line
`python3 -c` with loops. One simple `python3 - <<'EOF'` heredoc per edit, inside the repo, works.
