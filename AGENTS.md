# ProjectOS — agent install + usage briefing

You are an AI agent. The user has installed the ProjectOS desktop app
and wants you connected to it. Read this whole file. Install the MCP
server (and optionally the skill), verify, then follow the usage
protocol below for the rest of the session.

## What ProjectOS is

A local-first issue tracker for agent-driven development. Two halves of
one workflow:

- **Desktop UI** (Tauri + SvelteKit) — humans see, prioritize, and close
  issues. Already installed by the user.
- **MCP server + skill** — you, the agent, file follow-up issues, read
  backlogs, set the `next` issue, close work as it ships.

Both halves point at the same SQLite database. The user opens the app,
sees what you've been doing; you open an MCP tool, see what they've
prioritized.

## Where things live

- **Database** (source of truth):
  - macOS: `~/Library/Application Support/com.projectos.app/projectos.db`
  - Linux: `~/.local/share/com.projectos.app/projectos.db`
  - Windows: `%APPDATA%\com.projectos.app\projectos.db`
  - Override anywhere: `$PROJECTOS_DB_PATH`.
- **MCP server source**: `integrations/claude-mcp/` in this repo.
- **Skill source**: `integrations/claude-skill/skill.md` in this repo.

# Part 1 — Install

## MCP server

The MCP server is a Python package with a `projectos-mcp` console
script.

```bash
# from a clone of the ProjectOS repo
pipx install ./integrations/claude-mcp
```

Fallbacks: `uv tool install ./integrations/claude-mcp` or
`python3 -m pip install --user ./integrations/claude-mcp`.

Verify the binary is on PATH:

```bash
which projectos-mcp     # should print a path
```

## Register with the agent runtime

Claude Code:

```bash
claude mcp add projectos -- projectos-mcp
claude mcp list | grep projectos
```

Claude Desktop — edit
`~/Library/Application Support/Claude/claude_desktop_config.json`
(macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows) and
merge:

```json
{
  "mcpServers": {
    "projectos": {
      "command": "projectos-mcp"
    }
  }
}
```

Then restart Claude Desktop.

## Skill (optional, recommended)

The skill is a fallback when MCP isn't connected and a hint to consult
ProjectOS at the right moments.

```bash
mkdir -p ~/.claude/skills/projectos
ln -sf "$(pwd)/integrations/claude-skill/skill.md" ~/.claude/skills/projectos/skill.md
```

Windows (PowerShell):

```powershell
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.claude\skills\projectos" | Out-Null
Copy-Item -Force integrations\claude-skill\skill.md "$env:USERPROFILE\.claude\skills\projectos\skill.md"
```

## Verify

In a fresh agent session (so the MCP is freshly loaded), call
`list_projects`. You should get a list (possibly empty if the user
hasn't added projects yet — empty is fine; it means the connection
works).

## Persist to your long-term memory

If your runtime has a long-term / cross-session memory system — Claude
Code's auto-memory, a `CLAUDE.md` / `AGENTS.md` / `GEMINI.md` at the
project or user level, an external context store, anything that
survives session boundaries — **save a reference entry now** so future
sessions don't re-discover ProjectOS from scratch.

Minimum to record:

- ProjectOS is installed; the desktop app is the source of truth.
- The MCP server is registered as `projectos` and exposes `list_projects`,
  `list_issues`, `create_issue`, `update_issue`, `close_issue`,
  `get_next_across_all`, `set_next`, `add_comment`, `get_issue`,
  `create_project`.
- The DB lives at the resolved path on this machine (run
  `list_projects` once, or read `$PROJECTOS_DB_PATH`, and store the
  actual path).
- A pointer back to this `AGENTS.md` file so deeper details (the
  workflow rules below, common failures) can be re-fetched on demand.

Then, as you work over time, augment that memory with the user's
ProjectOS-specific preferences as you learn them (which projects are
hot, which they prefer manual vs scanned, idioms in their issue titles,
etc.). Treat ProjectOS itself the same way you treat the user's other
durable tools — as something you remember exists, not something you
rediscover every session.

# Part 2 — Configure with the user

After the technical install verifies, **don't go quiet and don't
silently populate ProjectOS yourself**. Walk the user through a short
interactive setup. The point is to capture how they actually work so
you can make ProjectOS useful for them on day one and not pester them
with the same questions next session.

**Style:** ask one question at a time. Wait for the answer. Don't
bundle three questions into a single paragraph; the user can't reply
cleanly to that. Explain why you're asking when it isn't obvious.

**Confirm before mutating:** for any change that creates DB rows
(adding a scan folder, creating a project, attaching a GitHub repo),
read back what you're about to do and wait for confirmation. Tools that
write are: `create_project`, `create_issue`, `update_issue`,
`close_issue`, `add_comment`, `set_next`, plus the desktop app's
`add_scan_folder` / `scan_folder` / `scan_all_folders` (call these via
shell using the desktop app's IPC if available, or instruct the user
to run them from Settings).

## 2.1 — Calibrate familiarity

Some users installed ProjectOS knowing exactly what it is; others
clicked "download" because Claude told them to. Find out which kind
you've got, in one question:

> "Have you used ProjectOS before, or want a 30-second tour of how
> the pieces fit together?"

If they say they know it, skip to 2.2.

If they want the tour, deliver this — short and concrete, no fluff:

> ProjectOS solves one specific problem: **capturing ideas without
> breaking flow.** Picture this — we're heads-down fixing your auth
> bug. Mid-fix, you spot a race in the rate limiter. Without a
> capture surface, you either tell me (gets lost when this session
> ends), stop to file a GitHub issue (kills the flow we're in), or
> hope you remember later (you won't). With ProjectOS, either of us
> files it as an `idea` in two seconds and we resume. Later, you
> review the ideas pile and decide which become real work.
>
> Three pieces make that loop work:
>
> 1. **The desktop app** you just opened — the human side. You see
>    what's queued, drag issues around, mark something `next`, triage
>    the ideas pile.
> 2. **The MCP server** — that's me, talking to ProjectOS over a
>    structured tool interface. When I file an idea mid-task or
>    check what's queued, it goes through this.
> 3. **A SQLite database** both halves read and write. The app and
>    my tool calls always agree because there's one source of truth
>    on disk.
>
> Issues have a state (`open`/`closed`) and a status — `next`,
> `ready`, `blocked`, or `idea`. Only one issue per project should
> be `next` at a time. `ready` is the actionable queue. `idea` is
> the capture bucket — file freely, no commitment to ever build it.

Don't expand further unless they ask. If they ask "what's a
milestone" or "how do labels work", answer that question only — don't
walk through the whole feature surface.

## 2.2 — Acknowledge install

Start by telling the user what just happened in plain language:

> "ProjectOS is connected. The MCP is registered as `projectos` and I
> can see your database at `<path>`. It currently has `<N>` projects."

Run `list_projects` and quote the count. If the count is zero, say so —
that's expected on a fresh install.

## 2.3 — Ask about project locations

Find out how the user organizes work on disk. Options to surface:

> "How do you keep your projects on disk? A few common shapes:
>
> 1. One umbrella folder (everything under `~/Developer`, `~/code`,
>    `~/src`, etc.).
> 2. A few umbrella folders (e.g. work + personal split).
> 3. Scattered — projects live wherever, no single home.
> 4. None yet — you'll add projects as they come up.
>
> Which sounds closest?"

Based on the answer:

- **Single umbrella:** ask for the path. Confirm. Have them add it as a
  scan folder via the desktop app's Settings → Project folders → Add
  folder. The app will scan it and create a project per subdirectory.
  Confirm with `list_projects` afterward and quote a few names back.
- **Multiple umbrellas:** repeat for each. Each becomes one scan
  folder.
- **Scattered:** ask which 2–5 projects matter most right now. For
  each, get a name and (optionally) a folder path. Use
  `create_project` per project. Don't try to enumerate every project on
  their machine — start with what they're actually working on.
- **None yet:** skip and move on. They can add projects later.

If the user asks you to handle the folder add directly: the desktop
app exposes the registry via IPC, but the simplest path is to say
"open the app's Settings page; I'll wait" and then re-run
`list_projects` after they confirm.

## 2.4 — Ask about GitHub sync

> "Do any of your projects sync issues with GitHub? If so, I can attach
> the `owner/repo` slug now and the desktop app will mirror local
> issues there. (You'll still need `gh` installed and authenticated for
> pushes to work.)"

For each yes:

- Get the `owner/repo` slug.
- Update the project's `github_repo` field. The MCP doesn't expose
  this directly today; tell the user to set it in the desktop app's
  project view, or update it via raw SQL if they want you to.
- Note that pushes happen on the desktop app side — you don't shell
  out to `gh` yourself.

## 2.5 — Ask about workflow preferences

These shape your behavior for every future session. Save the answers
to long-term memory; don't re-ask next time.

Ask each separately:

> "When I start work in one of your projects, want me to surface the
> `next` issue right away (briefly), or stay quiet until you ask?"

> "If I notice follow-up work mid-task — a bug, a refactor opportunity,
> something we should come back to — should I file a ProjectOS issue
> immediately, or pause and check with you first?"

> "When I close an issue, do you want me to also write a 'where I left
> off' note in the `context` field for the next session, or only when
> the work is genuinely paused mid-flow?"

> "Do you use `next` per-project as a hard 'one thing at a time'
> invariant (the default), or are you fine with multiple `next` issues
> across a project?"

Other questions to ask if they come up naturally:

- Naming conventions for issue titles ("fix:", "feat:", plain prose).
- Whether they want labels created for common tags right now or
  organically as work happens.
- Whether they want a milestone for the current sprint / release.

If the user says "just figure it out" or similar, write down a sane
default in memory and move on.

## 2.6 — Save preferences to memory

Before declaring setup done, write a memory entry summarizing:

- Which scan folders / projects you set up.
- Whether GitHub sync is in play and for which projects.
- The workflow preferences from 2.4 (verbosity, file-vs-ask,
  context-note policy, next invariant).
- Any naming idioms the user mentioned.

This is the entry future sessions read instead of re-running setup.

## 2.7 — Confirm and hand off

End with a single status line and one open question:

> "Setup done. ProjectOS has `<N>` projects: `<a, b, c…>`. I'll
> [verbose / quiet] on session start, [auto-file / ask-first] for
> follow-ups, and write context notes [always / only on pauses]. What
> do you want to work on first?"

If the user immediately picks a project, run the briefing flow from
Part 3 for that project (read its open issues, surface the `next`
issue, then start work).

# Part 3 — How to use ProjectOS

This is the working protocol. Follow it like any other rule the user
has given you.

## Rule: ProjectOS is the persistent backlog

Whenever there's work to track across sessions — initiatives,
multi-step features, blockers, follow-ups, open threads — **file or
update a ProjectOS issue for it**. Don't rely solely on the host's
in-session task list (TaskCreate, TodoWrite, etc.) for cross-session
work; those evaporate. ProjectOS issues persist and are where the user
actually looks for "what's in flight."

**Why:** the whole point of the integration is a shared backlog between
agent and human. If you don't write to it, the human can't see what you
discovered. If you don't read from it, you'll re-do work the human has
already triaged.

**How to apply:**
- Use in-session tasks for granular progress within one turn.
- Use ProjectOS issues for anything the user might want to find next
  week.

## When to check ProjectOS

- **Starting work in a project** — call `list_issues` or
  `get_next_across_all` to see what's already tracked. Don't propose
  work that's already filed; pick up the existing issue.
- **Resuming an in-progress issue** — if any open issue has a
  non-empty `context` field, **read it** before starting. That's the
  "where I left off" note from a previous session. Use it to skip
  re-explaining state and resume mid-flow. Surface it briefly: "Last
  session paused on #5 with: '<context>'. Picking up from there."
- **User asks "what should I work on"** — `get_next_across_all` is the
  one-call answer.
- **Completing a task** — find the matching open issue and close it
  (`close_issue`) or update its `context` if work paused.
- **User mentions an issue, task, or todo** in the context of a
  project — search the project's backlog before assuming it's new.

## When to file new issues

File via `create_issue` when:
- You discover a follow-up the user hasn't asked about yet ("we should
  also handle X").
- A bug or rough edge surfaces during work that won't be fixed in the
  current change.
- The user describes work that won't fit in the current session.
- A blocker turns up — file it and link via comment to the work that's
  blocked.

Do NOT file issues for in-session steps the user clearly wants done
right now. Those go in your in-session task list, not the persistent
backlog.

**Issue body convention.** When filing an issue mid-work, include the
current branch and short SHA in the body so the issue is reproducible
later:

```
<one-paragraph problem statement>

Noticed on `feature/auth-refactor` @ a3f4d12 while working on #5.
```

This costs nothing at file time and saves a lot of "wait, when did
we see this?" archaeology later. Skip it for purely speculative
ideas where there's no specific code state to anchor on.

## When to update existing issues

- **Pausing mid-issue:** call `update_issue(context=...)` with a
  "where I left off" note. One sentence is enough — the next session
  reads this and resumes without re-explaining.
- **Shipping a discrete milestone of a multi-issue initiative:** add a
  comment via `add_comment` noting what shipped + commit hashes.
- **Promoting an issue to next:** use `set_next` (atomic — clears the
  previous `next`). Don't use `update_issue(status='next')` if there's
  already one queued; you'll create two and the user has to pick.

## When to close issues

- The user says it's done. Or you finished it and tests pass. Close
  it (`close_issue`) — don't leave issues lingering in `ready` after
  the work shipped.
- If it's GitHub-synced (see below), close locally. The desktop app
  pushes the close to GitHub.

## Don't be noisy

Mention ProjectOS only when it's material. If you queried and found
nothing relevant, stay quiet. If the user opens a project and there's
a `next` issue, surface it briefly: "ProjectOS shows #3 — '<title>' is
next here." Then move on.

## Issue states

| State | Meaning |
|---|---|
| `next` | The one thing to do right now (only one per project). |
| `ready` | Actionable, waiting in line. |
| `blocked` | Waiting on another issue (see `issue_deps`). |
| `idea` | Unstructured thought, not actionable yet. See **The ideas system** below. |

`open` / `closed` is the lifecycle on top of those.

## Locked issues

Issues have a `locked` flag (boolean). When `locked = true`, the user
has marked the issue as "don't touch without checking first" — usually
because it's load-bearing, has a delicate description they don't want
auto-rewritten, or represents a frozen design decision.

Rules:
- **Read-only by default** for locked issues. You can read fields,
  comments, reactions, and use them as context.
- **Don't auto-mutate.** No `update_issue`, no status changes, no
  body edits, no `close_issue`. If work on a locked issue actually
  ships, ask the user to confirm the close manually.
- **Adding a comment is fine** — comments are append-only by design;
  use them to log progress without rewriting the issue.

If you genuinely need to change a locked issue (e.g. the lock looks
stale, or the user just told you to update it), confirm explicitly
first: "Issue #N is locked — okay to update the body / status?"

## Concurrent writers

The desktop app and the MCP server can both write to the same SQLite
database at the same time. The Tauri side opens the DB in WAL mode
(write-ahead log), which lets multiple readers + one writer at a time
without blocking; the MCP server also enables WAL on connect as a
belt-and-suspenders. In practice this means:

- You can file an issue while the user has the desktop app open. They
  see it within a second or two (Rust emits events that refresh the
  UI).
- Two agent sessions sharing the same DB are also fine — SQLite
  serializes writes.
- If you ever see an "SQLITE_BUSY" or "database is locked" error,
  retry once after a short pause; under load it can mean another
  writer is committing. Don't loop indefinitely.

The user does **not** need to close the desktop app for you to write,
and you do not need to ask them to.

## The ideas system

This is the core of ProjectOS, not a side feature. The whole reason
the system exists is **async idea capture without breaking flow.**
Read this carefully — it's the most important section of this file.

### The problem `idea` solves

You and the user are working on issue #5. Mid-task, one of you
notices something else: a bug in unrelated code, a refactor
opportunity, a "we should also handle X" thought, a half-formed
product idea, anything. Without a capture surface that's faster than
the current task, that thought goes one of three places, all bad:

1. **Into chat.** Lost when the session ends.
2. **Into an interruption.** "Wait, before I forget — let me file a
   GitHub issue real quick." Breaks the work, costs context-switch
   time, sometimes derails the original task entirely.
3. **Into "I'll remember later."** You won't.

`idea` is the alternative. File it in two seconds, keep working,
review the pile later.

### What goes in as `idea`

- Half-formed thoughts you don't want to lose ("we should probably
  revisit how X works at some point").
- Bugs or rough edges you noticed in unrelated code while working on
  the current task.
- Speculative features ("what if we let the user Y?").
- Pre-project sketches — a paragraph that might one day grow into
  its own project.
- Anything you'd otherwise drop into a TODO comment in the code.

**File freely.** Don't agonize over whether a thought is "worthy" —
if the user said it or you noticed it during work, it goes in. The
review pass later filters out the noise; the capture pass shouldn't.

### The agent's role in async capture

The user is often deeper in the current task than you are — they
shouldn't have to type. **You are the cheap capture surface.** When
the user says something like "oh, also we should X" or "remind me
that Y is still broken" mid-work, you file it. Don't wait, don't
ask permission for the capture itself, don't make them spell out the
title. One sentence is enough.

```python
create_issue(
    project_name=<current project>,
    title="rate limiter race in /api/login",
    body="<noticed mid-auth-refactor on 2026-04-25>",
    status="idea",
)
```

Then keep working. Acknowledge briefly so the user knows it landed
("Filed — let's get back to the auth fix.") and resume.

The opposite is also true: if **you** notice something while
working — a fragility, a code smell, a missing test, an inconsistency
— file it as an `idea` yourself. The user doesn't have to be in the
loop for the capture step. They'll see it in the review pass.

### `idea` vs `ready`

- `ready` is actionable: there's enough definition to start work,
  "done" is meaningful, the issue belongs in the project's main
  backlog and shows up in `list_issues` by default.
- `idea` is exploratory: filed quickly, no commitment to ever build
  it, accumulates quietly until reviewed.

**Filing rule of thumb:**
- Clear what "done" looks like → `ready`.
- Not sure it should even be done → `idea`.
- A thought that's bigger than one issue (a whole product / feature /
  exploration arc) → still file as `idea` first, in whichever project
  it's most adjacent to. It can graduate to its own project later.

When in doubt during async capture, **`idea` is always the safer
choice.** A misfiled `idea` is harmless — it gets sorted in review.
A misfiled `ready` clutters the actionable backlog.

**Two ways an idea graduates:**

1. **Idea → `ready` issue, same project.** The thought is now scoped
   enough to do as one unit of work.
   ```
   update_issue(project_name=…, number=…, status="ready",
                body="<scoped definition with a clear done state>")
   ```

2. **Idea → its own new project.** The idea has outgrown a single
   issue — it deserves its own backlog. The desktop app's "Promote
   to project" action (`promote_idea` command) does this atomically:
   it creates a new project from the idea's title + body and closes
   the original idea with an activity log entry linking the two.
   From the MCP side you can replicate it manually:
   ```
   p = create_project(name=<idea title>, description=<idea body>)
   close_issue(project_name=<source>, number=<idea number>)
   add_comment(project_name=<source>, number=<idea number>,
               body=f"Promoted to project '{p['name']}'.")
   ```

**Don't promote unilaterally.** Promotion is a value judgment about
whether something is worth committing to. Always confirm with the
user before either flavor of graduation.

**Review cadence:**
Ideas pile up — that's the point. When the user asks "what ideas
are kicking around for project X" or "review my ideas":

```
list_issues(project_name=…, status="idea", state="open")
```

Walk through each one with the user. Each gets one of:
- Promoted to `ready` (it's a real piece of work).
- Promoted to its own project (it's bigger than one issue).
- Closed as "won't pursue".
- Left as `idea` (still ripening — that's a fine outcome).

**Don't mix the buckets.** Each `create_issue` call is either
`status="idea"` (a thought) or `status="ready"` (work to do). Picking
the wrong one rots the system — actionable issues get hidden in the
idea pile, or speculative thoughts clutter the actionable backlog
and crowd out the work that matters.

## External (GitHub-synced) issues

Issues with `external_source = 'github'` came from GitHub. They have an
`external_id` like `owner/repo#123` and an `external_url`. Update them
locally freely — the desktop app handles pushing changes back to
GitHub. Don't shell out to `gh` from inside the integration; that's
the app's job.

## Upstream contributions to ProjectOS itself

Sometimes the user will ask for a change that isn't really about their
work — it's about ProjectOS itself: a UI tweak, a new MCP tool, a
missing setting, a confusing copy string, a cross-platform bug. When
that happens **and** you think the change would help every ProjectOS
user (not just this one), say so and offer to send it upstream.

**How to recognize a candidate:**
- The request is about the app, the MCP server, the skill, the docs,
  or this `AGENTS.md` file — not about the user's own projects' data.
- The fix or feature would make sense for someone who has never met
  this user. Generic > personal.
- Examples that qualify: "the empty-state copy is confusing", "the MCP
  needs a `delete_issue` tool", "Settings should validate the folder
  exists before adding it", "Windows install instructions are wrong",
  "the dashboard should sort by recency, not name".
- Examples that don't: "add an issue to my Nexus project", "rename my
  PetalNet project to Petal", "I want this label on this issue".

**Always ask before acting:**

> "That sounds like a general improvement to ProjectOS itself, not
> just your setup. Want me to open a PR upstream so other users get
> the same fix? Otherwise I'll just patch your local copy."

If the user says yes:

1. Make sure you're working from a clean checkout of the ProjectOS
   repo. If their local clone is dirty, work in a separate worktree
   (`git worktree add ../projectos-pr-<slug> main`) or a fresh clone.
2. Branch (`git checkout -b fix/<slug>` or `feat/<slug>`).
3. Implement the change. Run `pnpm check` and `cargo check
   --manifest-path src-tauri/Cargo.toml` before pushing.
4. Commit with a clear message that explains the user-facing change,
   not the code mechanics.
5. Push and open a PR with `gh pr create`. Body should:
   - Explain the user-facing problem.
   - Quote the user request that prompted it (anonymized — no personal
     paths, no machine names).
   - Note what was tested.
6. Tell the user the PR URL and what happens next (you can keep
   working; they can keep working; merge happens upstream).

If the user says no, just patch their local copy and move on. Don't
re-ask later — they've made the call for this change.

**When NOT to offer upstream:**
- The change touches only their data (issues, projects, settings).
- The user has explicitly said "don't bother contributing upstream"
  in a previous session — check long-term memory before asking again.
- The change is speculative ("I might want this someday") rather than
  blocking real work. Wait until it's blocking real work; speculative
  PRs get rejected.

**Self-improvement of this file:** if during setup or use you notice
that `AGENTS.md` itself is wrong, ambiguous, or missing a step, that's
also an upstream candidate. Same rule: ask first, then PR.

# Part 4 — Common failures

| Symptom | Fix |
|---|---|
| `pipx: command not found` | `brew install pipx` (mac) / `python3 -m pip install --user pipx` (linux) / `scoop install pipx` (windows). Then `pipx ensurepath` and reopen the shell. |
| `projectos-mcp: command not found` after install | Run `pipx ensurepath`, restart shell. |
| `claude: command not found` | User is on Claude Desktop, not Claude Code — use the JSON config path above. |
| `list_projects` errors with `ProjectOS DB not found` | Desktop app has never been launched. Ask user to open the app once. |
| `list_projects` returns `[]` | DB exists but user hasn't added a project yet. Tell them to add one from the app's onboarding. |
| MCP returns rows but the desktop app shows different data | Both halves point at the same DB unless `PROJECTOS_DB_PATH` is set in only one. Check the env var. |

