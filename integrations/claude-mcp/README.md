# ProjectOS — MCP server

A FastMCP server that wraps the ProjectOS SQLite database in structured
tools so any MCP-aware client (Claude Code, Claude Desktop, Codex, etc.)
can list / read / create / update issues without shelling out to sqlite3.

This is the preferred integration. The Claude Code skill in
`../claude-skill/` is a fallback for shell-based access.

## Install

We recommend `pipx` so the server lands on your PATH as `projectos-mcp`
without polluting any project's venv:

```bash
# from the cloned ProjectOS repo
pipx install ./integrations/claude-mcp
```

Or with `uv`:

```bash
uv tool install ./integrations/claude-mcp
```

Verify:

```bash
projectos-mcp --help  # should print FastMCP usage
```

## Register with Claude Code

```bash
claude mcp add projectos -- projectos-mcp
```

Confirm:

```bash
claude mcp list | grep projectos
```

## Register with Claude Desktop

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`
(macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows) and
add:

```json
{
  "mcpServers": {
    "projectos": {
      "command": "projectos-mcp"
    }
  }
}
```

Restart Claude Desktop.

## Custom DB location

The server resolves the DB in this order:

1. `$PROJECTOS_DB_PATH` if set.
2. The OS's per-user app-data directory:
   - macOS: `~/Library/Application Support/com.projectos.app/projectos.db`
   - Linux: `~/.local/share/com.projectos.app/projectos.db`
   - Windows: `%APPDATA%\com.projectos.app\projectos.db`

If you've moved the database (or you're testing against a fixture),
export `PROJECTOS_DB_PATH` in the shell that launches the agent.

## Tools exposed

| Tool | Purpose |
|------|---------|
| `list_projects` | Discover project names + open counts |
| `create_project` | Create a new project row |
| `list_issues` | List open/closed issues for a project (status filter) |
| `get_issue` | Full detail for one issue, with comments + reactions |
| `get_next_across_all` | Each project's `next` issue in one call |
| `create_issue` | New open issue (auto-numbered) |
| `update_issue` | Mutate title/body/status/context/pinned/locked |
| `close_issue` | Mark closed, set `closed_at` |
| `add_comment` | Append a comment |
| `set_next` | Atomically promote one issue to `next`, demote the previous |

## Develop

```bash
uv venv
source .venv/bin/activate
uv pip install -e .
projectos-mcp            # run the server
```
