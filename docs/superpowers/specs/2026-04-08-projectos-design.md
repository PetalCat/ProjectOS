# ProjectOS — Design Spec

**Local-first issue system for everything you think, build, and manage.**

A Tauri v2 desktop app (Svelte frontend, Rust backend) that tracks software projects, ideas, experiments, and real-life tasks through a unified issue system modeled after GitHub Issues.

---

## Core Philosophy

> You shouldn't manage tasks — you should always know what to do next.

- Everything is an issue
- Issues are ordered, not prioritized
- One issue per project is always clearly "Next"
- Ideas can evolve into projects or tasks over time

---

## Architecture

### Approach: Hybrid (Rust data core, Svelte reactivity layer)

- **Rust** owns the database (`rusqlite` + SQLite), exposes Tauri commands, and pushes state changes via Tauri events
- **Svelte** renders the UI, calls Tauri commands for mutations, and subscribes to Tauri events for reactive updates
- No frontend ORM — all data logic lives in Rust
- SQLite FTS5 for full-text search

### Communication Pattern

**Tauri Commands** (frontend → Rust, request/response):
- `create_project`, `update_project`, `delete_project`, `list_projects`
- `create_issue`, `update_issue`, `close_issue`, `reopen_issue`, `delete_issue`
- `reorder_issue`, `transfer_issue` (move to another project)
- `promote_idea` (idea → new project, closes source issue with reference)
- `create_comment`, `update_comment`, `delete_comment`
- `add_label`, `remove_label`, `create_label`
- `set_milestone`, `create_milestone`, `close_milestone`
- `add_reaction`, `remove_reaction`
- `add_dependency`, `remove_dependency`, `add_relation`, `remove_relation`
- `assign_issue`, `unassign_issue`
- `create_machine`, `update_machine`, `delete_machine`
- `create_machine_doc`, `update_machine_doc`, `delete_machine_doc`
- `get_activity_log` (filtered by project, date range, etc.)
- `search_issues` (full-text across all projects via FTS5)
- `get_dashboard` (returns all "next" issues + recent activity in one call)

**Tauri Events** (Rust → frontend, push updates):
- `issues-changed` — emitted after any issue mutation, carries affected project ID
- `projects-changed` — after project create/update/delete
- `activity` — new activity log entry, for live feed updates

**Flow:**
1. User action → Svelte calls Tauri command
2. Rust updates DB, logs activity, returns result
3. Rust emits relevant events
4. Svelte stores pick up events, re-fetch affected data
5. Optimistic updates on frontend, confirmed by Rust events

---

## Data Model

### `projects`
| Column | Type | Notes |
|--------|------|-------|
| `id` | TEXT (UUID) | Primary key |
| `name` | TEXT | Required |
| `description` | TEXT | Nullable |
| `notes` | TEXT | Nullable, markdown, project-level wiki |
| `created_at` | INTEGER | Unix ms |
| `updated_at` | INTEGER | Unix ms |

### `issues`
| Column | Type | Notes |
|--------|------|-------|
| `id` | TEXT (UUID) | Primary key |
| `project_id` | TEXT FK → projects | Nullable (unassigned ideas) |
| `number` | INTEGER | Sequential per-project, display as `#1`, `#2` |
| `title` | TEXT | Required |
| `body` | TEXT | Nullable, markdown |
| `state` | TEXT | `open` or `closed` |
| `status` | TEXT | `next`, `ready`, `blocked`, `idea`, or NULL. Only applies when open |
| `sort_order` | REAL | Fractional indexing for drag-to-reorder |
| `context` | TEXT | Nullable, "where I left off" bookmark |
| `machine_id` | TEXT FK → machines | Nullable |
| `milestone_id` | TEXT FK → milestones | Nullable |
| `locked` | BOOLEAN | Default false, prevents further comments |
| `pinned` | BOOLEAN | Default false, max 3 per project (enforced in Rust) |
| `created_at` | INTEGER | Unix ms |
| `updated_at` | INTEGER | Unix ms |
| `closed_at` | INTEGER | Nullable, unix ms |

**Constraints:**
- Only one issue per project can have `status = 'next'` (enforced in Rust)
- `number` is auto-incremented per `project_id`
- Issues with `project_id = NULL` get no number (they're unassigned ideas)

### `issue_comments`
| Column | Type | Notes |
|--------|------|-------|
| `id` | TEXT (UUID) | Primary key |
| `issue_id` | TEXT FK → issues | Required |
| `body` | TEXT | Markdown |
| `created_at` | INTEGER | Unix ms |
| `updated_at` | INTEGER | Unix ms |

### `issue_reactions`
| Column | Type | Notes |
|--------|------|-------|
| `id` | TEXT (UUID) | Primary key |
| `issue_id` | TEXT FK → issues | Nullable (reaction on issue body) |
| `comment_id` | TEXT FK → issue_comments | Nullable (reaction on comment) |
| `emoji` | TEXT | e.g. "👍", "🔥" |
| `created_at` | INTEGER | Unix ms |

One of `issue_id` or `comment_id` must be set.

### `issue_deps` (blocking relationships)
| Column | Type | Notes |
|--------|------|-------|
| `blocker_id` | TEXT FK → issues | The issue that blocks |
| `blocked_id` | TEXT FK → issues | The issue that is blocked |

Compound primary key on `(blocker_id, blocked_id)`.

### `issue_relations` (non-blocking links)
| Column | Type | Notes |
|--------|------|-------|
| `issue_a_id` | TEXT FK → issues | |
| `issue_b_id` | TEXT FK → issues | |

Compound primary key on `(issue_a_id, issue_b_id)`. Bidirectional — order doesn't matter.

### `labels`
| Column | Type | Notes |
|--------|------|-------|
| `id` | TEXT (UUID) | Primary key |
| `name` | TEXT | e.g. "bug", "quick-win" |
| `color` | TEXT | Hex color, e.g. "#a78bfa" |
| `project_id` | TEXT FK → projects | Nullable — NULL = global label |

### `issue_labels`
| Column | Type | Notes |
|--------|------|-------|
| `issue_id` | TEXT FK → issues | |
| `label_id` | TEXT FK → labels | |

Compound primary key.

### `milestones`
| Column | Type | Notes |
|--------|------|-------|
| `id` | TEXT (UUID) | Primary key |
| `project_id` | TEXT FK → projects | Required |
| `title` | TEXT | e.g. "v1 Launch" |
| `description` | TEXT | Nullable |
| `due_date` | INTEGER | Nullable, unix ms |
| `state` | TEXT | `open` or `closed` |
| `created_at` | INTEGER | Unix ms |
| `updated_at` | INTEGER | Unix ms |

### `assignees`
| Column | Type | Notes |
|--------|------|-------|
| `id` | TEXT (UUID) | Primary key |
| `name` | TEXT | e.g. "parker", "claude", machine name |

### `issue_assignees`
| Column | Type | Notes |
|--------|------|-------|
| `issue_id` | TEXT FK → issues | |
| `assignee_id` | TEXT FK → assignees | |

Compound primary key.

### `machines`
| Column | Type | Notes |
|--------|------|-------|
| `id` | TEXT (UUID) | Primary key |
| `name` | TEXT | e.g. "homelab", "MacBook" |
| `hostname` | TEXT | Nullable, auto-detected or manual |
| `ip` | TEXT | Nullable, e.g. "192.168.1.50" |
| `user` | TEXT | Nullable, e.g. "root", "parker" |
| `os` | TEXT | Nullable, e.g. "Ubuntu 24.04" |
| `notes` | TEXT | Nullable, freeform markdown |
| `created_at` | INTEGER | Unix ms |
| `updated_at` | INTEGER | Unix ms |

### `machine_docs`
| Column | Type | Notes |
|--------|------|-------|
| `id` | TEXT (UUID) | Primary key |
| `machine_id` | TEXT FK → machines | Required |
| `title` | TEXT | e.g. "SSH config", "Grafana dashboard" |
| `content` | TEXT | Nullable, inline markdown |
| `url` | TEXT | Nullable, external link |
| `created_at` | INTEGER | Unix ms |

### `issue_templates`
| Column | Type | Notes |
|--------|------|-------|
| `id` | TEXT (UUID) | Primary key |
| `project_id` | TEXT FK → projects | Nullable — NULL = global template |
| `name` | TEXT | e.g. "Bug Report", "Feature Request" |
| `body` | TEXT | Markdown template |
| `labels` | TEXT | JSON array of label IDs to auto-apply |

### `activity_log`
| Column | Type | Notes |
|--------|------|-------|
| `id` | INTEGER | Autoincrement |
| `issue_id` | TEXT FK → issues | Nullable |
| `project_id` | TEXT FK → projects | Nullable |
| `action` | TEXT | e.g. "created", "closed", "labeled", "moved", "reordered" |
| `detail` | TEXT | Nullable, JSON with change specifics |
| `created_at` | INTEGER | Unix ms |

---

## Visual Design

### Style Direction
- **Font:** Inter (400–800 weights)
- **Palette:** Warm dark base (#0a0a0a, #141410, #1a1a16) with earthy undertones — not cold grays
- **Accents:** Each project gets a distinct color (amber #e8a040, teal #60b8e0, lime #b8e060, rose #e06080, etc.) used for dots, card top-edge gradients, labels
- **Status colors:** Green/lime (#b8e060) for Next, amber (#e8a040) for Blocked, muted gray (#6a6a5a) for Ideas
- **Typography:** Bold, confident — 800 weight for headings, 600–700 for labels
- **Feel:** Warm, alive, opinionated — inspired by Beeper's energy but with its own identity. Not sterile, not soulless.
- **Mockups:** See `.superpowers/brainstorm/` for HTML mockups

### Assignees & Avatars
- Assignees and commenters display with colored avatar dots (colored circles with initials in final)
- Claude gets a distinct accent color (teal #60b8e0)

## UI Layout

### Hybrid: Sidebar + Dashboard Home + Project Drill-down

**Sidebar (always visible):**
- Project list with colored dots and open issue counts
- Machine list with active indicator (green diamond for current machine)
- Home button at top
- Keyboard shortcut hints at bottom (⌘K search, ⌘N capture)
- Current user + machine name in header

**Home view (default on launch):**
- "What's Next" header with date
- 2-column card grid — one card per project showing the `next` issue
- Each card: project name, issue title, machine indicator, milestone tag
- Cards have colored top-edge gradient matching project color
- Activity feed below, grouped by day ("Today", "Yesterday")
- Activity entries show action icons (✓ done, + created, → moved), issue refs, relative time

**Project view (click a project):**
- Header: project colored dot, name (28px 800-weight), open/closed count, "+ New Issue" button
- Milestone bar at top (always visible when milestone exists): title, detail, progress bar, percentage
- Open/Closed tab toggle
- Issues grouped by status in order:
  1. **📌 Pinned** — persistent reference issues, highlighted border
  2. **▶ Next** — the one current task, green filled circle indicator
  3. **Ready** — actionable queue, open circle indicators
  4. **⚠ Blocked** — dimmed rows, orange circle with !, "blocked by #N" in subtitle
  5. **💡 Ideas** — muted styling, dashed circle indicator
- Each group has a labeled divider with count
- Issue rows: drag handle (appears on hover), state circle, title, subtitle (opened time, machine), labels, status badge, issue number
- Drag-to-reorder within groups

**Issue detail view (click an issue):**
- Back nav ("← ProjectName")
- Header: Open/Closed badge (pill), status badge, title (26px 800-weight), #number · opened time · author
- Body: markdown rendered in a bordered card, supports code blocks and cross-references
- "Where I left off" context block: dashed border, editable, 📍 icon
- Comment thread: left-bordered comments, author name (colored for claude), timestamp, reactions (emoji chips with counts)
- Comment input at bottom
- **Action buttons:** Close issue, Close with comment, Reopen, Edit title/body — prominent and clear
- Right metadata sidebar (280px):
  - Status (colored text)
  - Assignees (colored dots + names, avatars in final)
  - Labels (colored pills)
  - Milestone (name + mini progress bar + percentage)
  - Machine
  - Blocks / Related issue links (clickable, colored)
  - Actions: Close, Transfer, Promote to project, Lock, Delete (danger red)

**Machine view (click a machine):**
- Machine info: name, hostname, IP, user, OS, notes (editable markdown)
- Attached docs list with title, content/url
- Issues tagged to this machine

### Issue State Indicators (Circle System)
- **Next:** Filled green circle with ▶ arrow
- **Ready:** Open circle with gray border
- **Blocked:** Open circle with orange border, ! inside
- **Idea:** Dashed circle with muted border
- **Closed:** Not shown in open list (separate tab)

---

## Interactions

### Global Search (Cmd+K)
- Searches across all issues, projects, machines
- Fuzzy matching on title, body, comments, labels
- Results grouped by type
- Powered by SQLite FTS5

### Fast Idea Capture (Cmd+N)
- Minimal popup input, available from anywhere
- Type title, hit Enter → creates issue with `state: open`, `status: idea`, no project
- No forms, no required fields

### Keyboard Shortcuts
| Key | Action |
|-----|--------|
| `Cmd+K` | Search |
| `Cmd+N` | New idea/issue |
| `J` / `K` | Navigate issue list |
| `Enter` | Open selected issue |
| `Esc` | Back / close |
| `X` | Close / reopen issue |
| `L` | Add label |
| `M` | Set milestone |

### Drag-to-Reorder
- Drag issues in list to reorder
- Fractional indexing — instant DB write, no re-sorting of other rows

### Cross-Project References
- `#12` in markdown auto-links to issue 12 in the same project
- `ProjectOS#12` references an issue in another project

---

## Key Flows

### Idea → Project Promotion
1. User clicks "Promote to project" on an idea issue
2. Rust creates a new `projects` row using the idea's title/body
3. Original issue is closed with activity log: "Promoted to project: ProjectOS"
4. New project appears in sidebar

### Issue Transfer
- Move an issue from one project to another
- Issue gets a new `number` in the destination project (old number in source project is retired, never reused)
- Activity log records the transfer with source and destination

### Blocking
- Add a dependency: issue A blocks issue B
- Issue B automatically gets `status: blocked` (enforced in Rust)
- When issue A is closed, issue B's blocked status is re-evaluated
- If no remaining blockers, issue B reverts to `ready`

---

## Tech Stack

- **Tauri v2** — desktop app framework
- **Svelte** (latest) — frontend UI
- **Rust** — backend, all data logic
- **rusqlite** — SQLite access from Rust
- **SQLite FTS5** — full-text search
- Fractional indexing for sort order
- UUID primary keys (except activity_log which uses autoincrement)

---

## Non-Goals (v1)

- Team collaboration / multi-user
- Cloud sync / remote database
- Deadlines and scheduling
- Priority matrices
- Git integration per-project
- Claude Code "Work on This" integration (deferred to v2)
- Complex automation / actions
