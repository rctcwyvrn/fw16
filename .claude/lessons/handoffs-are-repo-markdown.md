# Handoff documents go in the repo as `.claude/handoff-<path>.md`, never as a Claude Docs artifact

Asked to "summarize current work into a handoff document", I started a Claude Docs doc; the user stopped it:
handoffs are local markdown under `.claude/` so the next session (and the other path sessions) read them from
the checkout. Follow the existing shape (`.claude/handoff-path2.md`, `-path3.md`): read-first list, decisions
already made, state, next steps, judgment calls to surface. Commit and push it with the path's files.
