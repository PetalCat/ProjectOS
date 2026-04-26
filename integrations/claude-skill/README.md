# ProjectOS — Claude Code skill

A small Claude Code skill that lets the agent peek at and update your
ProjectOS issues from the shell. The MCP server (sister directory,
`integrations/claude-mcp/`) is the preferred path — install both; the
skill is useful as a fallback when MCP isn't connected.

## Install

Symlink (or copy) `skill.md` into your Claude Code skills directory:

```bash
mkdir -p ~/.claude/skills/projectos
ln -s "$PWD/skill.md" ~/.claude/skills/projectos/skill.md
```

On Windows (PowerShell):

```powershell
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\.claude\skills\projectos"
Copy-Item -Force skill.md "$env:USERPROFILE\.claude\skills\projectos\skill.md"
```

That's it. Claude Code picks the skill up on the next session.

## Pointing at a custom DB location

If you've moved the ProjectOS database (or you're testing against a
fixture DB), set `PROJECTOS_DB_PATH` in your shell profile:

```bash
export PROJECTOS_DB_PATH="/path/to/projectos.db"
```

The skill checks this env var before falling back to the per-OS default
location.
