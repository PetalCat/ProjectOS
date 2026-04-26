"""ProjectOS MCP server.

Exposes the local ProjectOS SQLite database via structured MCP tools so
Claude Code (or any MCP-aware client) can list, read, create, and update
issues without shelling out to sqlite3.

DB location resolution, in order:
    1. $PROJECTOS_DB_PATH if set (matches the desktop app's override).
    2. platformdirs.user_data_dir("com.projectos.app") / "projectos.db"
       (resolves correctly on macOS, Linux, and Windows).
"""

from __future__ import annotations

import os
import sqlite3
import time
import uuid
from pathlib import Path
from typing import Annotated, Any, Optional

from fastmcp import FastMCP
from platformdirs import user_data_dir
from pydantic import Field


def _resolve_db_path() -> Path:
    """Return the ProjectOS DB path, honoring $PROJECTOS_DB_PATH first."""
    env = os.environ.get("PROJECTOS_DB_PATH")
    if env:
        return Path(env)
    return Path(user_data_dir("com.projectos.app")) / "projectos.db"


DB_PATH = _resolve_db_path()

mcp = FastMCP("projectos")


# ─────────────────────────────────────────────────────────────────────────────
# DB helpers
# ─────────────────────────────────────────────────────────────────────────────


def _conn() -> sqlite3.Connection:
    if not DB_PATH.exists():
        raise FileNotFoundError(
            f"ProjectOS DB not found at {DB_PATH}. "
            "Launch the ProjectOS desktop app once to create it, "
            "or set PROJECTOS_DB_PATH to point at an existing DB."
        )
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    # WAL is the database-level mode set by the desktop app on first open;
    # asking again is cheap and harmless, and keeps the MCP safe if it's
    # ever started before the app has run.
    conn.execute("PRAGMA journal_mode = WAL")
    conn.execute("PRAGMA foreign_keys = ON")
    return conn


def _now_ms() -> int:
    return int(time.time() * 1000)


def _gen_id() -> str:
    return str(uuid.uuid4())


def _project_id_by_name(conn: sqlite3.Connection, name: str) -> Optional[str]:
    row = conn.execute("SELECT id FROM projects WHERE name = ?", (name,)).fetchone()
    return row["id"] if row else None


def _issue_id(conn: sqlite3.Connection, project_name: str, number: int) -> Optional[str]:
    project_id = _project_id_by_name(conn, project_name)
    if not project_id:
        return None
    row = conn.execute(
        "SELECT id FROM issues WHERE project_id = ? AND number = ?",
        (project_id, number),
    ).fetchone()
    return row["id"] if row else None


def _issue_dict(row: sqlite3.Row) -> dict[str, Any]:
    return {
        "id": row["id"],
        "number": row["number"],
        "title": row["title"],
        "body": row["body"],
        "state": row["state"],
        "status": row["status"],
        "sort_order": row["sort_order"],
        "context": row["context"],
        "locked": bool(row["locked"]),
        "pinned": bool(row["pinned"]),
        "created_at": row["created_at"],
        "updated_at": row["updated_at"],
        "closed_at": row["closed_at"],
        "external_source": row["external_source"],
        "external_id": row["external_id"],
        "external_url": row["external_url"],
    }


# ─────────────────────────────────────────────────────────────────────────────
# Tools
# ─────────────────────────────────────────────────────────────────────────────


@mcp.tool
def list_projects() -> list[dict[str, Any]]:
    """List every project in ProjectOS with id, name, github_repo, and open issue count.

    Use this to discover project names before calling any project-scoped tool,
    or when the user asks what projects they have tracked.
    """
    with _conn() as conn:
        rows = conn.execute(
            """
            SELECT p.id, p.name, p.github_repo, p.description,
                   (SELECT COUNT(*) FROM issues i
                    WHERE i.project_id = p.id AND i.state = 'open') AS open_count,
                   (SELECT COUNT(*) FROM issues i
                    WHERE i.project_id = p.id AND i.state = 'open' AND i.status = 'next') AS has_next
            FROM projects p
            ORDER BY p.name
            """
        ).fetchall()
    return [dict(r) for r in rows]


@mcp.tool
def create_project(
    name: Annotated[str, Field(description="Project name")],
    description: Annotated[
        Optional[str],
        Field(description="Optional description (often a folder path)"),
    ] = None,
    github_repo: Annotated[
        Optional[str],
        Field(description="Optional 'owner/repo' GitHub slug"),
    ] = None,
    notes: Annotated[Optional[str], Field(description="Optional freeform notes")] = None,
) -> dict[str, Any]:
    """Create a new project in ProjectOS.

    Fails if a project with the same name already exists. The created row has
    no folder backing by default — if you want one, pass it as `description`
    (the desktop UI treats `description` as the folder path when present).
    """
    if not name or "/" in name or name.startswith("."):
        raise ValueError(f"Invalid project name: {name!r}")

    with _conn() as conn:
        existing = _project_id_by_name(conn, name)
        if existing:
            raise ValueError(f"Project already exists: {name}")

        project_id = _gen_id()
        now = _now_ms()
        conn.execute(
            """
            INSERT INTO projects (id, name, description, notes, created_at, updated_at, github_repo)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            """,
            (project_id, name, description, notes, now, now, github_repo),
        )
        conn.commit()
        row = conn.execute("SELECT * FROM projects WHERE id = ?", (project_id,)).fetchone()

    return {
        "id": row["id"],
        "name": row["name"],
        "description": row["description"],
        "notes": row["notes"],
        "github_repo": row["github_repo"],
        "created_at": row["created_at"],
        "updated_at": row["updated_at"],
    }


@mcp.tool
def list_issues(
    project_name: Annotated[str, Field(description="Project name (case-sensitive)")],
    state: Annotated[str, Field(description="'open' or 'closed'")] = "open",
    status: Annotated[
        Optional[str],
        Field(description="Optional filter: 'next', 'ready', 'blocked', 'idea'"),
    ] = None,
    limit: Annotated[int, Field(ge=1, le=500, description="Max rows to return")] = 50,
) -> list[dict[str, Any]]:
    """List issues for a project with optional status filter.

    Results are ordered by status priority (next → ready → blocked → idea)
    then sort_order, then number. Prefer this over get_next_across_all()
    when the user asks about a specific project's backlog.
    """
    with _conn() as conn:
        project_id = _project_id_by_name(conn, project_name)
        if not project_id:
            return []
        params: list[Any] = [project_id, state]
        query = """
            SELECT * FROM issues
            WHERE project_id = ? AND state = ?
        """
        if status:
            query += " AND status = ?"
            params.append(status)
        query += """
            ORDER BY
                CASE status
                    WHEN 'next' THEN 0
                    WHEN 'ready' THEN 1
                    WHEN 'blocked' THEN 2
                    WHEN 'idea' THEN 3
                    ELSE 4
                END,
                sort_order ASC,
                number ASC
            LIMIT ?
        """
        params.append(limit)
        rows = conn.execute(query, params).fetchall()
    return [_issue_dict(r) for r in rows]


@mcp.tool
def get_issue(
    project_name: Annotated[str, Field(description="Project name")],
    number: Annotated[int, Field(description="Issue number within the project")],
) -> Optional[dict[str, Any]]:
    """Fetch full detail for a single issue, including comments and reactions.

    Use when you need the complete context (title, body, status, all comments,
    emoji reactions) before acting on an issue. Returns None if not found.
    """
    with _conn() as conn:
        issue_id = _issue_id(conn, project_name, number)
        if not issue_id:
            return None
        issue_row = conn.execute("SELECT * FROM issues WHERE id = ?", (issue_id,)).fetchone()
        comments = conn.execute(
            "SELECT id, body, created_at, updated_at FROM issue_comments WHERE issue_id = ? ORDER BY created_at",
            (issue_id,),
        ).fetchall()
        reactions = conn.execute(
            "SELECT emoji, created_at FROM issue_reactions WHERE issue_id = ?",
            (issue_id,),
        ).fetchall()

    result = _issue_dict(issue_row)
    result["comments"] = [dict(c) for c in comments]
    result["reactions"] = [dict(r) for r in reactions]
    return result


@mcp.tool
def get_next_across_all() -> list[dict[str, Any]]:
    """Return every project's current 'next' issue in one call.

    Ideal for status overviews. Projects without a 'next' issue are omitted.
    Returns rows of {project_name, issue_number, title}.
    """
    with _conn() as conn:
        rows = conn.execute(
            """
            SELECT p.name AS project_name, i.number AS issue_number, i.title
            FROM issues i
            JOIN projects p ON i.project_id = p.id
            WHERE i.state = 'open' AND i.status = 'next'
            ORDER BY p.name
            """
        ).fetchall()
    return [dict(r) for r in rows]


@mcp.tool
def create_issue(
    project_name: Annotated[str, Field(description="Project name")],
    title: Annotated[str, Field(description="Short issue title")],
    body: Annotated[str, Field(description="Optional markdown body")] = "",
    status: Annotated[
        str,
        Field(description="'next', 'ready', 'blocked', or 'idea'"),
    ] = "ready",
) -> dict[str, Any]:
    """Create a new open issue in the given project.

    The issue number is auto-assigned as max(existing)+1 within the project.
    If status='next', does NOT automatically unset an existing 'next' — call
    set_next() for that atomic behavior. Returns the created issue row.
    """
    if status not in ("next", "ready", "blocked", "idea"):
        raise ValueError(f"Invalid status: {status}")

    with _conn() as conn:
        project_id = _project_id_by_name(conn, project_name)
        if not project_id:
            raise ValueError(f"Project not found: {project_name}")

        next_number_row = conn.execute(
            "SELECT COALESCE(MAX(number), 0) + 1 AS n FROM issues WHERE project_id = ?",
            (project_id,),
        ).fetchone()
        next_number = next_number_row["n"]

        next_sort_row = conn.execute(
            "SELECT COALESCE(MAX(sort_order), 0) + 1.0 AS s FROM issues WHERE project_id = ? AND state = 'open'",
            (project_id,),
        ).fetchone()
        next_sort = next_sort_row["s"]

        issue_id = _gen_id()
        now = _now_ms()
        conn.execute(
            """
            INSERT INTO issues
                (id, project_id, number, title, body, state, status, sort_order,
                 locked, pinned, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, 'open', ?, ?, 0, 0, ?, ?)
            """,
            (issue_id, project_id, next_number, title, body, status, next_sort, now, now),
        )
        conn.execute(
            """
            INSERT INTO activity_log (issue_id, project_id, action, detail, created_at)
            VALUES (?, ?, 'created', ?, ?)
            """,
            (issue_id, project_id, f'{{"title":"{title.replace(chr(34), chr(92)+chr(34))}"}}', now),
        )
        conn.commit()

        row = conn.execute("SELECT * FROM issues WHERE id = ?", (issue_id,)).fetchone()
    return _issue_dict(row)


@mcp.tool
def update_issue(
    project_name: Annotated[str, Field(description="Project name")],
    number: Annotated[int, Field(description="Issue number")],
    title: Annotated[Optional[str], Field(description="New title")] = None,
    body: Annotated[Optional[str], Field(description="New body")] = None,
    status: Annotated[
        Optional[str],
        Field(description="'next', 'ready', 'blocked', 'idea', or null to clear"),
    ] = None,
    context: Annotated[
        Optional[str],
        Field(description="Where-I-left-off context note"),
    ] = None,
    pinned: Annotated[Optional[bool], Field(description="Pin/unpin")] = None,
    locked: Annotated[Optional[bool], Field(description="Lock/unlock")] = None,
) -> Optional[dict[str, Any]]:
    """Update mutable fields on an issue. Omit a parameter to leave it unchanged.

    The 'context' field is a "where I left off" note — write it when work on
    an issue pauses so the next session knows where to resume.
    """
    with _conn() as conn:
        issue_id = _issue_id(conn, project_name, number)
        if not issue_id:
            return None

        fields: list[str] = []
        params: list[Any] = []
        if title is not None:
            fields.append("title = ?")
            params.append(title)
        if body is not None:
            fields.append("body = ?")
            params.append(body)
        if status is not None:
            if status not in ("next", "ready", "blocked", "idea"):
                raise ValueError(f"Invalid status: {status}")
            fields.append("status = ?")
            params.append(status)
        if context is not None:
            fields.append("context = ?")
            params.append(context)
        if pinned is not None:
            fields.append("pinned = ?")
            params.append(1 if pinned else 0)
        if locked is not None:
            fields.append("locked = ?")
            params.append(1 if locked else 0)

        if not fields:
            row = conn.execute("SELECT * FROM issues WHERE id = ?", (issue_id,)).fetchone()
            return _issue_dict(row)

        fields.append("updated_at = ?")
        params.append(_now_ms())
        params.append(issue_id)

        conn.execute(f"UPDATE issues SET {', '.join(fields)} WHERE id = ?", params)
        conn.commit()

        row = conn.execute("SELECT * FROM issues WHERE id = ?", (issue_id,)).fetchone()
    return _issue_dict(row)


@mcp.tool
def close_issue(
    project_name: Annotated[str, Field(description="Project name")],
    number: Annotated[int, Field(description="Issue number")],
) -> Optional[dict[str, Any]]:
    """Mark an issue closed. Clears status, sets closed_at timestamp."""
    with _conn() as conn:
        issue_id = _issue_id(conn, project_name, number)
        if not issue_id:
            return None
        now = _now_ms()
        conn.execute(
            """
            UPDATE issues
            SET state = 'closed', status = NULL, closed_at = ?, updated_at = ?
            WHERE id = ?
            """,
            (now, now, issue_id),
        )
        project_id = conn.execute(
            "SELECT project_id FROM issues WHERE id = ?", (issue_id,)
        ).fetchone()["project_id"]
        conn.execute(
            """
            INSERT INTO activity_log (issue_id, project_id, action, detail, created_at)
            VALUES (?, ?, 'closed', '{}', ?)
            """,
            (issue_id, project_id, now),
        )
        conn.commit()

        row = conn.execute("SELECT * FROM issues WHERE id = ?", (issue_id,)).fetchone()
    return _issue_dict(row)


@mcp.tool
def add_comment(
    project_name: Annotated[str, Field(description="Project name")],
    number: Annotated[int, Field(description="Issue number")],
    body: Annotated[str, Field(description="Comment markdown body")],
) -> Optional[dict[str, Any]]:
    """Add a comment to an issue. Returns the created comment row or None if issue not found."""
    with _conn() as conn:
        issue_id = _issue_id(conn, project_name, number)
        if not issue_id:
            return None

        comment_id = _gen_id()
        now = _now_ms()
        conn.execute(
            """
            INSERT INTO issue_comments (id, issue_id, body, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            """,
            (comment_id, issue_id, body, now, now),
        )
        conn.commit()

        row = conn.execute(
            "SELECT id, body, created_at, updated_at FROM issue_comments WHERE id = ?",
            (comment_id,),
        ).fetchone()
    return dict(row)


@mcp.tool
def set_next(
    project_name: Annotated[str, Field(description="Project name")],
    number: Annotated[int, Field(description="Issue number to set as next")],
) -> Optional[dict[str, Any]]:
    """Atomically make the given issue the project's 'next', clearing any
    previous 'next' back to 'ready'. Use this instead of update_issue(status='next')
    when you care about the 'only one next' invariant.
    """
    with _conn() as conn:
        project_id = _project_id_by_name(conn, project_name)
        if not project_id:
            return None
        issue_id = _issue_id(conn, project_name, number)
        if not issue_id:
            return None

        now = _now_ms()
        conn.execute(
            """
            UPDATE issues
            SET status = 'ready', updated_at = ?
            WHERE project_id = ? AND state = 'open' AND status = 'next' AND id != ?
            """,
            (now, project_id, issue_id),
        )
        conn.execute(
            "UPDATE issues SET status = 'next', updated_at = ? WHERE id = ?",
            (now, issue_id),
        )
        conn.commit()

        row = conn.execute("SELECT * FROM issues WHERE id = ?", (issue_id,)).fetchone()
    return _issue_dict(row)


def main() -> None:
    """Entry point for the `projectos-mcp` console script."""
    mcp.run()


if __name__ == "__main__":
    main()
