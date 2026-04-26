---
name: projectos
description: Check and update ProjectOS issues when working in any project. Use when starting work in a tracked project, completing tasks, or when the user asks about their issues/tasks/todos.
---

# ProjectOS

ProjectOS is a local-first issue tracker. Each project the user works in
typically has a row in ProjectOS, and issues for that project are tracked
there.

## Prefer the MCP server

If the `projectos` MCP server is connected, **use its tools** instead of
the bash recipes below — they're structured, atomic, and don't need
sqlite3 on the user's PATH. The bash recipes are a fallback when MCP
isn't available.

## When to activate

- Starting work in a project — check ProjectOS for the `next` issue.
- Completing a task — close or update the relevant issue.
- User asks "what should I work on" — check `get_next_across_all` (MCP)
  or query the dashboard.
- User mentions issues, tasks, or todos in the context of a project.

Be quiet about it. Mention ProjectOS only when relevant; don't interrupt
flow with status updates.

## Finding the database

The desktop app stores its SQLite DB in the OS's per-user app-data dir.
Check `PROJECTOS_DB_PATH` first; if unset, use the OS default:

```bash
if [ -n "$PROJECTOS_DB_PATH" ]; then
  DBPATH="$PROJECTOS_DB_PATH"
else
  case "$(uname -s)" in
    Darwin) DBPATH="$HOME/Library/Application Support/com.projectos.app/projectos.db" ;;
    Linux)  DBPATH="${XDG_DATA_HOME:-$HOME/.local/share}/com.projectos.app/projectos.db" ;;
    *)      DBPATH="$APPDATA/com.projectos.app/projectos.db" ;;  # Windows under MSYS/WSL
  esac
fi
```

If the file doesn't exist, ProjectOS isn't installed or hasn't been
launched yet — say so and stop.

## Bash recipes (fallback when MCP isn't connected)

Match the current project by directory name:

```bash
PROJECT_NAME=$(basename "$PWD")
sqlite3 "$DBPATH" "SELECT id, name, github_repo FROM projects WHERE name = '$PROJECT_NAME';"
```

List open issues for the current project, ordered by status priority:

```bash
sqlite3 "$DBPATH" "
  SELECT number, title, status, context
  FROM issues
  WHERE project_id = '<PROJECT_ID>' AND state = 'open'
  ORDER BY
    CASE status
      WHEN 'next' THEN 0 WHEN 'ready' THEN 1
      WHEN 'blocked' THEN 2 WHEN 'idea' THEN 3
    END,
    sort_order ASC;
"
```

Cross-project "next" overview:

```bash
sqlite3 "$DBPATH" "
  SELECT p.name, i.number, i.title
  FROM issues i JOIN projects p ON i.project_id = p.id
  WHERE i.status = 'next' AND i.state = 'open';
"
```

Update an issue's "where I left off" context:

```bash
sqlite3 "$DBPATH" "
  UPDATE issues
  SET context = 'Implemented X. Still need Y.', updated_at = $(date +%s)000
  WHERE project_id = '<PROJECT_ID>' AND number = <ISSUE_NUMBER>;
"
```

Close an issue:

```bash
sqlite3 "$DBPATH" "
  UPDATE issues
  SET state = 'closed', status = NULL,
      closed_at = $(date +%s)000, updated_at = $(date +%s)000
  WHERE project_id = '<PROJECT_ID>' AND number = <ISSUE_NUMBER>;
"
```

Add a comment:

```bash
COMMENT_ID=$(uuidgen | tr '[:upper:]' '[:lower:]')
ISSUE_ID=$(sqlite3 "$DBPATH" \
  "SELECT id FROM issues WHERE project_id = '<PROJECT_ID>' AND number = <NUM>;")
sqlite3 "$DBPATH" "
  INSERT INTO issue_comments (id, issue_id, body, created_at, updated_at)
  VALUES ('$COMMENT_ID', '$ISSUE_ID', 'Comment body', $(date +%s)000, $(date +%s)000);
"
```

Create a new issue:

```bash
ISSUE_ID=$(uuidgen | tr '[:upper:]' '[:lower:]')
NEXT_NUM=$(sqlite3 "$DBPATH" \
  "SELECT COALESCE(MAX(number), 0) + 1 FROM issues WHERE project_id = '<PROJECT_ID>';")
SORT=$(sqlite3 "$DBPATH" \
  "SELECT COALESCE(MAX(sort_order), 0) + 1.0 FROM issues
   WHERE project_id = '<PROJECT_ID>' AND state = 'open';")
NOW=$(date +%s)000
sqlite3 "$DBPATH" "
  INSERT INTO issues
    (id, project_id, number, title, body, state, status, sort_order,
     locked, pinned, created_at, updated_at)
  VALUES
    ('$ISSUE_ID', '<PROJECT_ID>', $NEXT_NUM, 'Title', 'Body',
     'open', 'ready', $SORT, 0, 0, $NOW, $NOW);
"
```

Log activity (optional but keeps the desktop UI's activity feed honest):

```bash
sqlite3 "$DBPATH" "
  INSERT INTO activity_log (issue_id, project_id, action, detail, created_at)
  VALUES ('<ISSUE_ID>', '<PROJECT_ID>', 'closed', '{\"title\": \"…\"}', $(date +%s)000);
"
```

## Issue states

- `next` — the one thing to do right now (only one per project).
- `ready` — actionable, waiting in line.
- `blocked` — waiting on another issue (check `issue_deps`).
- `idea` — unstructured thought, not actionable yet.

## External (GitHub-synced) issues

Issues with `external_source = 'github'` came from GitHub. They have an
`external_id` like `owner/repo#123` and an `external_url`. You can update
local state freely; pushes to GitHub are handled by the desktop app, so
don't shell out to `gh` from this skill.
