# ProjectOS

**Local issue tracking for agent-driven development. With a UI.**

A SQLite-backed issue tracker that runs entirely on your machine. Two
halves of one workflow: a polished desktop app for humans, and an MCP
server for the agents you work with. Both read and write the same
local database, so the human view and the agent view always agree.

**Fully local, no telemetry, no account.** Issues stay in a SQLite
file on your disk. Nothing leaves your machine unless you opt into
GitHub Issues sync per project.

<!-- TODO: add screenshots/dashboard.png and screenshots/issue-detail.png -->
<!-- ![ProjectOS dashboard](./screenshots/dashboard.png) -->


## The async idea problem

You're heads-down with your agent on issue #5 — the auth refactor.
Mid-task, you spot a race in the rate limiter. Or your agent notices
the migration script is fragile. Or you remember you wanted to redo
the empty-state copy.

Without ProjectOS, you have bad options:

- Mention it in chat → it gets lost when the session ends.
- Stop and file a GitHub Issue → context-switch tax, breaks flow.
- Write it on a sticky note → fine until the sticky disappears.

With ProjectOS, either of you files it as an `idea` — two seconds,
no context switch — and you keep working. Later, you open the app's
ideas pile and triage: promote to actionable, spin off into its own
project, or close as won't-pursue. **Async idea capture is the core
of the system; everything else is plumbing around it.**

> **Hey, AI agent reading this?** Skip the rest of this README and go
> read [`AGENTS.md`](./AGENTS.md). It has install steps, the workflow
> protocol, and how the ideas system works — written for you. The user
> wants you to set ProjectOS up; that file is the briefing.

---

## What you get

- **Desktop app** (Tauri + SvelteKit): Up Next dashboard, per-project
  backlog with drag-to-reorder, full-text search, comments,
  reactions, milestones, machine notes, optional GitHub Issues sync.
- **MCP server** (`integrations/claude-mcp/`): structured tools so any
  Claude session — Code, Desktop, or any other MCP-aware client — can
  list, file, update, and close issues.
- **Claude Code skill** (`integrations/claude-skill/`): a smaller
  shell-based fallback for agents that don't have the MCP connected.

## Install (humans)

Download the prebuilt installer for your OS from the
[Releases](../../releases) page:

- **Windows** — `.msi` installer
- **macOS** — `.dmg` (Apple Silicon + Intel)
- **Linux** — `.AppImage` or `.deb`

Open the app once. It creates the local database at:

| OS | Path |
|---|---|
| macOS | `~/Library/Application Support/com.projectos.app/projectos.db` |
| Linux | `~/.local/share/com.projectos.app/projectos.db` |
| Windows | `%APPDATA%\com.projectos.app\projectos.db` |

Set `PROJECTOS_DB_PATH` in your environment to override the location.

## Connect your agent

The MCP server is the recommended path. From a clone of this repo:

```bash
pipx install ./integrations/claude-mcp
claude mcp add projectos -- projectos-mcp
```

For Claude Desktop, add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "projectos": {
      "command": "projectos-mcp"
    }
  }
}
```

Full instructions, including the optional Claude Code skill and the
agent-driven setup walkthrough, live in
[`AGENTS.md`](./AGENTS.md).

## How it works

```
       ┌────────────────┐                  ┌────────────────┐
       │   Desktop UI   │                  │  Your agent    │
       │  (you triage)  │                  │  (files work)  │
       └───────┬────────┘                  └───────┬────────┘
               │                                   │
               │       ┌──────────────────┐        │
               └──────►│  projectos.db    │◄───────┘
                       │  (SQLite, local) │
                       └──────────────────┘
```

- The agent notices follow-up work mid-task and files it as a
  `ready` issue (or an `idea` if it's not yet actionable).
- You open the desktop app, see what's queued, drag the most
  important one to `next`.
- Work happens. The agent closes issues as they ship. Loop.

Issue statuses: `next` (the one thing to do now — only one per
project), `ready` (actionable, waiting), `blocked`, `idea`. Open or
closed on top of those.

The `idea` status is the system's brain dump — file freely, promote
later. AGENTS.md explains the full ideas workflow.

## Optional: GitHub Issues sync

Per project, you can attach a GitHub `owner/repo` slug. The desktop
app will then mirror local issues to GitHub Issues (requires the
[`gh` CLI](https://cli.github.com) installed and authenticated). The
sync is one-way at a time: local edits push to GitHub; pulls happen
on demand from the project view.

## Build from source

Requires Rust (stable), Node 20+, and pnpm 9+.

```bash
pnpm install
pnpm tauri dev      # full hot-reload dev loop
pnpm check          # svelte-check + tsc
pnpm test           # vitest
cargo check --manifest-path src-tauri/Cargo.toml
pnpm tauri build    # platform installer in src-tauri/target/release/bundle/
```

## Report a bug or contribute

Bug reports and feature requests go in the
[Issues tab](../../issues). For code contributions, file an issue
first to discuss the change — this is a focused project and I'd
rather steer something before you write it than reject a finished
PR. Pre-existing license terms apply to anything merged.

## License

ProjectOS is © 2026 Parker. All rights reserved. The official builds
are free to use; redistribution and source modifications require
written permission. See [`LICENSE`](./LICENSE).
