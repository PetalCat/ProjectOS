<script lang="ts">
  import {
    addScanFolder,
    scanFolder,
    createProject,
    listScanFolders,
  } from "$lib/commands";
  import { loadProjects, getProjects } from "$lib/stores/projects.svelte";

  type Props = {
    onComplete: () => void;
  };

  let { onComplete }: Props = $props();

  type Step = "welcome" | "projects" | "connect" | "done";
  let step = $state<Step>("welcome");

  // ── Step 2: Add a project ───────────────────────────────────────
  type ProjectMode = "folder" | "manual";
  let mode = $state<ProjectMode>("folder");

  let folderPath = $state("");
  let manualName = $state("");
  let manualPath = $state("");
  let working = $state(false);
  let error = $state<string | null>(null);
  let added = $state<string[]>([]);

  async function handleAddFolder() {
    error = null;
    if (!folderPath.trim()) {
      error = "Enter a folder path.";
      return;
    }
    working = true;
    try {
      const folder = await addScanFolder(folderPath.trim());
      const created = await scanFolder(folder.id);
      added = created.map((p) => p.name);
      folderPath = "";
      await loadProjects();
      step = "connect";
    } catch (e) {
      error = String(e);
    } finally {
      working = false;
    }
  }

  async function handleManualCreate() {
    error = null;
    if (!manualName.trim()) {
      error = "Project name is required.";
      return;
    }
    working = true;
    try {
      const project = await createProject(
        manualName.trim(),
        manualPath.trim() || undefined,
      );
      added = [project.name];
      manualName = "";
      manualPath = "";
      await loadProjects();
      step = "connect";
    } catch (e) {
      error = String(e);
    } finally {
      working = false;
    }
  }

  // ── Step 3: Connect Claude ──────────────────────────────────────
  const AGENT_PROMPT = `Set up ProjectOS for me. Read https://github.com/PetalCat/ProjectOS/blob/main/AGENTS.md and follow the install instructions for the MCP server. When you're done, run \`claude mcp list | grep projectos\` and \`list_projects\` to verify, then walk me through the configuration questions in part 2 of that file.`;

  const MANUAL_COMMANDS = `# in a clone of the ProjectOS repo
pipx install ./integrations/claude-mcp
claude mcp add projectos -- projectos-mcp
claude mcp list | grep projectos`;

  let copied = $state<"prompt" | "manual" | null>(null);
  let showManual = $state(false);

  async function copyText(text: string, key: "prompt" | "manual") {
    try {
      await navigator.clipboard.writeText(text);
      copied = key;
      setTimeout(() => {
        if (copied === key) copied = null;
      }, 2000);
    } catch (_e) {
      // Clipboard can be blocked in some webview contexts; user can
      // still triple-click + cmd-c.
    }
  }

  function finish() {
    onComplete();
  }
</script>

<div class="overlay">
  <div class="card">
    <header class="header">
      <div class="brand">
        <span class="logo">⌂</span>
        <span class="brand-name">ProjectOS</span>
      </div>
      <div class="progress">
        <span class="dot" class:active={step === "welcome"}></span>
        <span class="dot" class:active={step === "projects"}></span>
        <span class="dot" class:active={step === "connect"}></span>
      </div>
    </header>

    <!-- ── Step 1: Welcome ── -->
    {#if step === "welcome"}
      <section class="body">
        <h1>Local issue tracking for agent-driven development.</h1>
        <p class="lede">
          You're working on issue #5 with Claude. Mid-task, you spot a
          rate-limiter race. Or Claude notices a fragile migration. Or
          you remember the empty-state copy is wrong.
        </p>
        <p>
          Without ProjectOS, that thought goes into chat (lost) or
          interrupts the work (kills flow) or onto a sticky note (gone
          by tomorrow). With ProjectOS, either of you files it as an
          <code>idea</code> in two seconds and you keep working.
        </p>
        <p class="lede">
          <strong>Async idea capture is the core of the system;
          everything else is plumbing around it.</strong>
        </p>
      </section>
      <footer class="footer">
        <div></div>
        <button class="btn-primary" onclick={() => (step = "projects")}>
          Get started →
        </button>
      </footer>
    {/if}

    <!-- ── Step 2: Add a project ── -->
    {#if step === "projects"}
      <section class="body">
        <h2>Add your first project</h2>
        <p class="muted">
          ProjectOS tracks issues per-project. Pick how you'd like to
          set up the first one — you can always add more later from
          Settings.
        </p>

        <div class="tabs">
          <button
            class="tab"
            class:active={mode === "folder"}
            onclick={() => {
              mode = "folder";
              error = null;
            }}
          >
            Scan a folder
          </button>
          <button
            class="tab"
            class:active={mode === "manual"}
            onclick={() => {
              mode = "manual";
              error = null;
            }}
          >
            Add manually
          </button>
        </div>

        {#if mode === "folder"}
          <p class="muted-small">
            Point at a folder. Each immediate subdirectory becomes a
            project — ideal if you keep code under one umbrella like
            <code>~/code</code> or <code>~/Developer</code>.
          </p>
          <div class="form-row">
            <input
              class="input"
              type="text"
              bind:value={folderPath}
              placeholder="Path to a folder containing your projects"
              disabled={working}
              onkeydown={(e) => e.key === "Enter" && handleAddFolder()}
            />
            <button
              class="btn-primary"
              onclick={handleAddFolder}
              disabled={working}
            >
              {working ? "Scanning…" : "Add & scan"}
            </button>
          </div>
        {:else}
          <p class="muted-small">
            Just give it a name. Folder path is optional — you can
            always attach one later.
          </p>
          <div class="form-grid">
            <input
              class="input"
              type="text"
              bind:value={manualName}
              placeholder="Project name (required)"
              disabled={working}
            />
            <input
              class="input"
              type="text"
              bind:value={manualPath}
              placeholder="Folder path (optional)"
              disabled={working}
              onkeydown={(e) => e.key === "Enter" && handleManualCreate()}
            />
            <button
              class="btn-primary"
              onclick={handleManualCreate}
              disabled={working}
            >
              {working ? "Creating…" : "Create"}
            </button>
          </div>
        {/if}

        {#if error}
          <div class="error">{error}</div>
        {/if}
      </section>
      <footer class="footer">
        <button class="btn-link" onclick={() => (step = "connect")}>
          Skip — I'll add projects later
        </button>
        <div></div>
      </footer>
    {/if}

    <!-- ── Step 3: Connect Claude ── -->
    {#if step === "connect"}
      <section class="body">
        <h2>Connect your agent</h2>
        {#if added.length > 0}
          <p class="muted">
            <span class="check">✓</span>
            Added {added.length} project{added.length === 1 ? "" : "s"}:
            <strong>{added.slice(0, 3).join(", ")}</strong>{added.length >
            3
              ? `, +${added.length - 3} more`
              : ""}.
          </p>
        {/if}
        <p>
          ProjectOS is most useful when your agent can file and read
          issues directly. Easiest path: paste this prompt into your
          Claude session and let it do the install.
        </p>

        <div class="prompt-box">
          <pre>{AGENT_PROMPT}</pre>
          <button
            class="copy-btn"
            onclick={() => copyText(AGENT_PROMPT, "prompt")}
          >
            {copied === "prompt" ? "Copied ✓" : "Copy"}
          </button>
        </div>

        <details class="manual" bind:open={showManual}>
          <summary>Or install manually</summary>
          <div class="manual-body">
            <p class="muted-small">
              From a clone of the ProjectOS repo:
            </p>
            <div class="prompt-box">
              <pre>{MANUAL_COMMANDS}</pre>
              <button
                class="copy-btn"
                onclick={() => copyText(MANUAL_COMMANDS, "manual")}
              >
                {copied === "manual" ? "Copied ✓" : "Copy"}
              </button>
            </div>
            <p class="muted-small">
              For Claude Desktop, add <code>projectos</code> to
              <code>claude_desktop_config.json</code>.
              <a
                href="https://github.com/PetalCat/ProjectOS/blob/main/AGENTS.md"
                target="_blank"
                rel="noopener noreferrer">Full instructions in AGENTS.md →</a
              >
            </p>
          </div>
        </details>
      </section>
      <footer class="footer">
        <button class="btn-link" onclick={finish}>
          Skip — I'll connect later
        </button>
        <button class="btn-primary" onclick={finish}>
          Done — open ProjectOS
        </button>
      </footer>
    {/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: radial-gradient(
        circle at 30% 20%,
        rgba(232, 160, 64, 0.06),
        transparent 60%
      ),
      radial-gradient(
        circle at 70% 80%,
        rgba(184, 224, 96, 0.04),
        transparent 60%
      ),
      #0a0a0a;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 32px;
  }

  .card {
    width: min(720px, 100%);
    background: #131310;
    border: 1px solid #1e1e1a;
    border-radius: 16px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 18px 28px;
    border-bottom: 1px solid #1e1e1a;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .logo {
    font-size: 16px;
    color: #c8c0a4;
  }

  .brand-name {
    font-size: 14px;
    font-weight: 800;
    color: #e0dac6;
    letter-spacing: -0.01em;
  }

  .progress {
    display: flex;
    gap: 6px;
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #2a2a22;
    transition: background 0.2s;
  }

  .dot.active {
    background: #e8a040;
  }

  .body {
    padding: 36px 36px 24px;
    min-height: 320px;
  }

  h1 {
    font-size: 26px;
    font-weight: 800;
    color: #f0ead6;
    letter-spacing: -0.015em;
    line-height: 1.2;
    margin: 0 0 18px;
  }

  h2 {
    font-size: 22px;
    font-weight: 800;
    color: #f0ead6;
    margin: 0 0 8px;
    letter-spacing: -0.01em;
  }

  .lede {
    font-size: 14px;
    line-height: 1.55;
    color: #c0b89a;
    margin: 0 0 14px;
  }

  .body p {
    font-size: 13.5px;
    line-height: 1.6;
    color: #8a8a7a;
    margin: 0 0 14px;
  }

  .body p:last-child {
    margin-bottom: 0;
  }

  .body code {
    font-family: ui-monospace, SFMono-Regular, monospace;
    font-size: 12px;
    background: rgba(255, 255, 255, 0.05);
    color: #b8e060;
    padding: 1px 5px;
    border-radius: 3px;
  }

  .muted {
    color: #8a8a7a;
    font-size: 13px;
    margin: 0 0 18px;
  }

  .muted-small {
    color: #6a6a5a;
    font-size: 12px;
    margin: 12px 0 14px;
  }

  .check {
    color: #b8e060;
    font-weight: 700;
    margin-right: 4px;
  }

  .tabs {
    display: flex;
    gap: 4px;
    background: #0e0e0a;
    border: 1px solid #1e1e1a;
    border-radius: 9px;
    padding: 4px;
    margin-bottom: 6px;
  }

  .tab {
    flex: 1;
    padding: 8px 12px;
    background: none;
    border: none;
    border-radius: 6px;
    font-family: "Inter", sans-serif;
    font-size: 13px;
    font-weight: 600;
    color: #6a6a5a;
    cursor: pointer;
    transition: background 0.12s, color 0.12s;
  }

  .tab:hover {
    color: #a09870;
  }

  .tab.active {
    background: #1e1e1a;
    color: #e0dac6;
  }

  .form-row {
    display: flex;
    gap: 10px;
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr 1fr auto;
    gap: 10px;
  }

  .input {
    flex: 1;
    background: #0e0e0a;
    border: 1px solid #2a2a22;
    border-radius: 8px;
    padding: 10px 14px;
    color: #c0b89a;
    font-family: "Inter", sans-serif;
    font-size: 13px;
    outline: none;
    transition: border-color 0.15s;
    min-width: 0;
  }

  .input:focus {
    border-color: #4a4a40;
  }

  .input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .prompt-box {
    position: relative;
    background: #0e0e0a;
    border: 1px solid #1e1e1a;
    border-radius: 8px;
    padding: 14px 16px;
    margin: 14px 0 6px;
  }

  .prompt-box pre {
    font-family: ui-monospace, SFMono-Regular, monospace;
    font-size: 12px;
    color: #c8c0a4;
    line-height: 1.55;
    white-space: pre-wrap;
    word-break: break-word;
    margin: 0;
    padding-right: 70px;
  }

  .copy-btn {
    position: absolute;
    top: 10px;
    right: 10px;
    background: #1e1e1a;
    border: 1px solid #2a2a22;
    color: #a09870;
    border-radius: 6px;
    padding: 4px 10px;
    font-family: "Inter", sans-serif;
    font-size: 11px;
    font-weight: 700;
    cursor: pointer;
    transition: opacity 0.12s;
  }

  .copy-btn:hover {
    color: #e0dac6;
    background: #2a2a22;
  }

  .manual {
    margin-top: 18px;
    border-top: 1px solid #1e1e1a;
    padding-top: 16px;
  }

  .manual summary {
    cursor: pointer;
    font-size: 12px;
    font-weight: 700;
    color: #6a6a5a;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    list-style: none;
  }

  .manual summary::-webkit-details-marker {
    display: none;
  }

  .manual summary:hover {
    color: #a09870;
  }

  .manual summary::before {
    content: "▶";
    font-size: 9px;
    margin-right: 6px;
    transition: transform 0.15s;
    display: inline-block;
  }

  .manual[open] summary::before {
    transform: rotate(90deg);
  }

  .manual-body {
    margin-top: 12px;
  }

  .manual-body a {
    color: #a09870;
    text-decoration: underline;
  }

  .manual-body a:hover {
    color: #e8a040;
  }

  .footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 18px 28px;
    border-top: 1px solid #1e1e1a;
    background: #100f0d;
  }

  .btn-primary {
    background: #e8a040;
    color: #0a0a0a;
    border: none;
    border-radius: 8px;
    padding: 10px 22px;
    font-family: "Inter", sans-serif;
    font-size: 13px;
    font-weight: 700;
    cursor: pointer;
    transition: opacity 0.15s;
  }

  .btn-primary:hover {
    opacity: 0.9;
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-link {
    background: none;
    border: none;
    color: #5a5a4a;
    font-family: "Inter", sans-serif;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    padding: 4px 6px;
    text-decoration: underline;
    text-decoration-color: transparent;
    transition: color 0.12s, text-decoration-color 0.12s;
  }

  .btn-link:hover {
    color: #a09870;
    text-decoration-color: #a09870;
  }

  .error {
    margin-top: 14px;
    padding: 10px 14px;
    background: #1a1210;
    border: 1px solid #3a2218;
    border-radius: 8px;
    font-size: 12.5px;
    color: #e08080;
  }
</style>
