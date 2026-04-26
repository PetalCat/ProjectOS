<script lang="ts">
  import {
    listScanFolders,
    addScanFolder,
    removeScanFolder,
    scanFolder,
    scanAllFolders,
    rescanTimestamps,
    createProject,
  } from "$lib/commands";
  import { loadProjects } from "$lib/stores/projects.svelte";
  import type { Project, ScanFolder } from "$lib/types";

  let folders = $state<ScanFolder[]>([]);
  let foldersError = $state<string | null>(null);

  let newFolderPath = $state("");
  let addingFolder = $state(false);
  let addError = $state<string | null>(null);

  let scanningId = $state<string | null>(null);
  let scanningAll = $state(false);
  let scanResult = $state<{ folder: string; created: Project[] } | null>(null);
  let scanResultAll = $state<Project[] | null>(null);
  let scanError = $state<string | null>(null);

  let rescanning = $state(false);
  let rescanResult = $state<number | null>(null);
  let rescanError = $state<string | null>(null);

  // Manual project form
  let manualName = $state("");
  let manualPath = $state("");
  let creatingManual = $state(false);
  let manualResult = $state<Project | null>(null);
  let manualError = $state<string | null>(null);

  async function refreshFolders() {
    try {
      folders = await listScanFolders();
      foldersError = null;
    } catch (e) {
      foldersError = String(e);
    }
  }

  $effect(() => {
    refreshFolders();
  });

  async function handleAddFolder() {
    addError = null;
    scanResult = null;
    scanResultAll = null;
    if (!newFolderPath.trim()) {
      addError = "Enter a folder path.";
      return;
    }
    addingFolder = true;
    try {
      const folder = await addScanFolder(newFolderPath.trim());
      newFolderPath = "";
      await refreshFolders();
      // Auto-scan the freshly added folder.
      scanningId = folder.id;
      const created = await scanFolder(folder.id);
      scanResult = { folder: folder.path, created };
      await Promise.all([refreshFolders(), loadProjects()]);
    } catch (e) {
      addError = String(e);
    } finally {
      addingFolder = false;
      scanningId = null;
    }
  }

  async function handleScanFolder(folder: ScanFolder) {
    scanError = null;
    scanResult = null;
    scanResultAll = null;
    scanningId = folder.id;
    try {
      const created = await scanFolder(folder.id);
      scanResult = { folder: folder.path, created };
      await Promise.all([refreshFolders(), loadProjects()]);
    } catch (e) {
      scanError = String(e);
    } finally {
      scanningId = null;
    }
  }

  async function handleRemoveFolder(folder: ScanFolder) {
    try {
      await removeScanFolder(folder.id);
      await refreshFolders();
    } catch (e) {
      foldersError = String(e);
    }
  }

  async function handleScanAll() {
    scanError = null;
    scanResult = null;
    scanResultAll = null;
    scanningAll = true;
    try {
      const created = await scanAllFolders();
      scanResultAll = created;
      await Promise.all([refreshFolders(), loadProjects()]);
    } catch (e) {
      scanError = String(e);
    } finally {
      scanningAll = false;
    }
  }

  async function handleRescan() {
    rescanning = true;
    rescanResult = null;
    rescanError = null;
    try {
      rescanResult = await rescanTimestamps();
      await loadProjects();
    } catch (e) {
      rescanError = String(e);
    } finally {
      rescanning = false;
    }
  }

  async function handleCreateManual() {
    manualError = null;
    manualResult = null;
    if (!manualName.trim()) {
      manualError = "Project name is required.";
      return;
    }
    creatingManual = true;
    try {
      const project = await createProject(
        manualName.trim(),
        manualPath.trim() || undefined,
      );
      manualResult = project;
      manualName = "";
      manualPath = "";
      await loadProjects();
    } catch (e) {
      manualError = String(e);
    } finally {
      creatingManual = false;
    }
  }

  function formatTime(ms: number | null): string {
    if (!ms) return "never";
    const diff = Date.now() - ms;
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return "just now";
    if (mins < 60) return `${mins}m ago`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    return `${days}d ago`;
  }
</script>

<div class="settings">
  <h2>Settings</h2>

  <!-- ── Scan folders ── -->
  <div class="section">
    <div class="section-head">
      <div>
        <h3>Project folders</h3>
        <p class="description">
          Add folders containing projects you want tracked. Each immediate
          subdirectory becomes a project.
        </p>
      </div>
      {#if folders.length > 0}
        <button
          class="btn-secondary"
          onclick={handleScanAll}
          disabled={scanningAll || scanningId !== null}
        >
          {scanningAll ? "Scanning…" : "Scan all"}
        </button>
      {/if}
    </div>

    {#if foldersError}
      <div class="error">{foldersError}</div>
    {/if}

    {#if folders.length > 0}
      <ul class="folder-list">
        {#each folders as folder (folder.id)}
          <li class="folder-row">
            <div class="folder-info">
              <div class="folder-path">{folder.path}</div>
              <div class="folder-meta">
                Last scanned {formatTime(folder.last_scanned_at)}
              </div>
            </div>
            <div class="folder-actions">
              <button
                class="btn-link"
                onclick={() => handleScanFolder(folder)}
                disabled={scanningId !== null || scanningAll}
              >
                {scanningId === folder.id ? "Scanning…" : "Scan"}
              </button>
              <button
                class="btn-link danger"
                onclick={() => handleRemoveFolder(folder)}
                disabled={scanningId !== null || scanningAll}
              >
                Remove
              </button>
            </div>
          </li>
        {/each}
      </ul>
    {:else}
      <div class="empty-folders">
        No folders yet. Add one below to start scanning.
      </div>
    {/if}

    <div class="add-row">
      <input
        class="input"
        type="text"
        bind:value={newFolderPath}
        placeholder="Path to a folder containing your projects"
        disabled={addingFolder}
        onkeydown={(e) => e.key === "Enter" && handleAddFolder()}
      />
      <button
        class="btn-primary"
        onclick={handleAddFolder}
        disabled={addingFolder}
      >
        {addingFolder ? "Adding…" : "Add folder"}
      </button>
    </div>

    {#if addError}
      <div class="error">{addError}</div>
    {/if}

    {#if scanError}
      <div class="error">{scanError}</div>
    {/if}

    {#if scanResult}
      <div class="result">
        {#if scanResult.created.length === 0}
          No new projects in <code>{scanResult.folder}</code> — already
          imported.
        {:else}
          <strong>{scanResult.created.length}</strong> new project{scanResult.created.length === 1 ? "" : "s"} from <code>{scanResult.folder}</code>:
          <ul class="result-list">
            {#each scanResult.created as project}
              <li>{project.name}</li>
            {/each}
          </ul>
        {/if}
      </div>
    {/if}

    {#if scanResultAll}
      <div class="result">
        {#if scanResultAll.length === 0}
          No new projects across {folders.length} folder{folders.length === 1 ? "" : "s"}.
        {:else}
          Scanned all folders — found <strong>{scanResultAll.length}</strong>
          new project{scanResultAll.length === 1 ? "" : "s"}.
        {/if}
      </div>
    {/if}
  </div>

  <!-- ── Manual project ── -->
  <div class="section">
    <h3>Add a project manually</h3>
    <p class="description">
      Track a project without scanning. Folder path is optional.
    </p>

    <div class="manual-grid">
      <input
        class="input"
        type="text"
        bind:value={manualName}
        placeholder="Project name (required)"
        disabled={creatingManual}
      />
      <input
        class="input"
        type="text"
        bind:value={manualPath}
        placeholder="Folder path (optional)"
        disabled={creatingManual}
      />
      <button
        class="btn-primary"
        onclick={handleCreateManual}
        disabled={creatingManual}
      >
        {creatingManual ? "Creating…" : "Create"}
      </button>
    </div>

    {#if manualError}
      <div class="error">{manualError}</div>
    {/if}
    {#if manualResult}
      <div class="result">
        Created project <strong>{manualResult.name}</strong>.
      </div>
    {/if}
  </div>

  <!-- ── Maintenance ── -->
  <div class="section">
    <h3>Maintenance</h3>
    <p class="description">
      Refresh "last activity" timestamps from disk for any project that has a
      folder path.
    </p>
    <button
      class="btn-secondary"
      onclick={handleRescan}
      disabled={rescanning}
    >
      {rescanning ? "Rescanning…" : "Rescan timestamps"}
    </button>
    {#if rescanResult !== null}
      <div class="result">
        Updated timestamps for {rescanResult} project{rescanResult === 1 ? "" : "s"}.
      </div>
    {/if}
    {#if rescanError}
      <div class="error">{rescanError}</div>
    {/if}
  </div>
</div>

<style>
  .settings {
    max-width: 720px;
  }

  h2 {
    font-size: 28px;
    color: #f0ead6;
    font-weight: 800;
    letter-spacing: -1px;
    margin-bottom: 32px;
  }

  .section {
    background: #141410;
    border: 1px solid #1e1e1a;
    border-radius: 12px;
    padding: 22px 24px;
    margin-bottom: 20px;
  }

  .section-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 16px;
    margin-bottom: 8px;
  }

  h3 {
    font-size: 16px;
    color: #e0dac6;
    font-weight: 700;
    margin: 0 0 4px;
  }

  .description {
    font-size: 13px;
    color: #6a6a5a;
    margin: 0 0 16px;
  }

  .folder-list {
    list-style: none;
    padding: 0;
    margin: 0 0 14px;
    border: 1px solid #20201c;
    border-radius: 8px;
    overflow: hidden;
  }

  .folder-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 14px;
    border-bottom: 1px solid #1a1a16;
    gap: 12px;
  }
  .folder-row:last-child {
    border-bottom: none;
  }

  .folder-info {
    min-width: 0;
    flex: 1;
  }

  .folder-path {
    font-family: ui-monospace, SFMono-Regular, monospace;
    font-size: 13px;
    color: #c8c0a4;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .folder-meta {
    margin-top: 3px;
    font-size: 11px;
    color: #5a5a4a;
  }

  .folder-actions {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }

  .empty-folders {
    border: 1px dashed #2a2a22;
    border-radius: 8px;
    padding: 14px;
    color: #5a5a4a;
    font-size: 13px;
    margin-bottom: 14px;
  }

  .add-row {
    display: flex;
    gap: 10px;
  }

  .manual-grid {
    display: grid;
    grid-template-columns: 1fr 1fr auto;
    gap: 10px;
  }

  .input {
    flex: 1;
    background: #0e0e0a;
    border: 1px solid #2a2a22;
    border-radius: 8px;
    padding: 9px 13px;
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

  .btn-primary {
    background: #e8a040;
    color: #0a0a0a;
    border: none;
    border-radius: 8px;
    padding: 9px 18px;
    font-family: "Inter", sans-serif;
    font-size: 13px;
    font-weight: 700;
    cursor: pointer;
    transition: opacity 0.15s;
    white-space: nowrap;
  }

  .btn-primary:hover {
    opacity: 0.9;
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    background: #3a3a2a;
    color: #c0b89a;
    border: 1px solid #4a4a38;
    border-radius: 8px;
    padding: 7px 14px;
    font-family: "Inter", sans-serif;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.15s;
    white-space: nowrap;
  }

  .btn-secondary:hover {
    opacity: 0.85;
  }

  .btn-secondary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-link {
    background: none;
    border: none;
    color: #a09870;
    font-family: "Inter", sans-serif;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    padding: 4px 6px;
  }

  .btn-link:hover {
    color: #e0dac6;
  }

  .btn-link.danger {
    color: #b06868;
  }
  .btn-link.danger:hover {
    color: #e07070;
  }

  .btn-link:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .result {
    margin-top: 16px;
    padding: 12px 14px;
    background: #1a1a16;
    border-radius: 8px;
    font-size: 13px;
    color: #c0b89a;
  }

  .result code {
    font-family: ui-monospace, SFMono-Regular, monospace;
    color: #d8d8c8;
  }

  .result-list {
    padding-left: 20px;
    margin-top: 6px;
    color: #a09870;
  }

  .result-list li {
    margin-bottom: 2px;
  }

  .error {
    margin-top: 14px;
    padding: 12px 14px;
    background: #1a1210;
    border: 1px solid #3a2218;
    border-radius: 8px;
    font-size: 13px;
    color: #e08080;
  }
</style>
