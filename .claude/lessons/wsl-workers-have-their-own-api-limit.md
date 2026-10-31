# The WSL `claude -p` workers bill against a different usage limit than the Windows orchestrator session

2026-10-29 15:45: all four WSL headless workers died within a minute of each other with
`API Error: 400 You have reached your specified API usage limits. You will regain access on 2026-11-01 at 00:00 UTC.`
(systemd unit `status=1/FAILURE`, result in the stream-json tail of `_build/agents/<lane>.log`), while the Windows
session kept working. Check `tail -c 600 _build/agents/<lane>.log` for `api_error_status` before assuming a worker
crashed, and size parallel worker rounds against that account's limit.
