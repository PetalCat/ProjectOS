<script lang="ts">
  import { scanDeveloperFolder, rescanTimestamps } from "$lib/commands";
  import { loadProjects } from "$lib/stores/projects.svelte";
  import type { Project } from "$lib/types";

  let scanPath = $state("");
  let scanning = $state(false);
  let scanResult = $state<Project[] | null>(null);
  let scanError = $state<string | null>(null);

  let rescanning = $state(false);
  let rescanResult = $state<number | null>(null);
  let rescanError = $state<string | null>(null);

  async function handleScan() {
    scanning = true;
    scanResult = null;
    scanError = null;
    try {
      const created = await scanDeveloperFolder(scanPath);
      scanResult = created;
      await loadProjects();
    } catch (e) {
      scanError = String(e);
    } finally {
      scanning = false;
    }
  }

  async function handleRescan() {
    rescanning = true;
    rescanResult = null;
    rescanError = null;
    try {
      const count = await rescanTimestamps();
      rescanResult = count;
      await loadProjects();
    } catch (e) {
      rescanError = String(e);
    } finally {
      rescanning = false;
    }
  }
</script>

<div class="settings">
  <h2>Settings</h2>

  <div class="section">
    <h3>Import Projects</h3>
    <p class="description">Scan a folder and create a project for each subdirectory.</p>

    <div class="scan-row">
      <input
        class="scan-input"
        type="text"
        bind:value={scanPath}
        placeholder="Path to a folder containing your projects"
      />
      <button class="scan-btn" onclick={handleScan} disabled={scanning}>
        {scanning ? "Scanning..." : "Scan"}
      </button>
      <button class="rescan-btn" onclick={handleRescan} disabled={rescanning}>
        {rescanning ? "Rescanning..." : "Rescan Timestamps"}
      </button>
    </div>

    {#if rescanResult !== null}
      <div class="scan-result">
        <p class="result-success">Updated timestamps for {rescanResult} project{rescanResult === 1 ? "" : "s"}.</p>
      </div>
    {/if}

    {#if rescanError}
      <div class="scan-error">{rescanError}</div>
    {/if}

    {#if scanResult !== null}
      <div class="scan-result">
        {#if scanResult.length === 0}
          <p class="result-empty">No new projects found. All directories already imported.</p>
        {:else}
          <p class="result-success">Created {scanResult.length} project{scanResult.length === 1 ? "" : "s"}:</p>
          <ul class="result-list">
            {#each scanResult as project}
              <li>{project.name}</li>
            {/each}
          </ul>
        {/if}
      </div>
    {/if}

    {#if scanError}
      <div class="scan-error">{scanError}</div>
    {/if}
  </div>
</div>

<style>
  .settings {
    max-width: 640px;
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
    padding: 24px;
    margin-bottom: 20px;
  }

  h3 {
    font-size: 16px;
    color: #e0dac6;
    font-weight: 700;
    margin-bottom: 6px;
  }

  .description {
    font-size: 13px;
    color: #6a6a5a;
    margin-bottom: 16px;
  }

  .scan-row {
    display: flex;
    gap: 10px;
  }

  .scan-input {
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
  }

  .scan-input:focus {
    border-color: #4a4a40;
  }

  .scan-btn {
    background: #e8a040;
    color: #0a0a0a;
    border: none;
    border-radius: 8px;
    padding: 10px 20px;
    font-family: "Inter", sans-serif;
    font-size: 13px;
    font-weight: 700;
    cursor: pointer;
    transition: opacity 0.15s;
  }

  .scan-btn:hover {
    opacity: 0.9;
  }

  .scan-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .rescan-btn {
    background: #3a3a2a;
    color: #c0b89a;
    border: 1px solid #4a4a38;
    border-radius: 8px;
    padding: 10px 16px;
    font-family: "Inter", sans-serif;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.15s;
    white-space: nowrap;
  }

  .rescan-btn:hover {
    opacity: 0.85;
  }

  .rescan-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .scan-result {
    margin-top: 16px;
    padding: 14px 16px;
    background: #1a1a16;
    border-radius: 8px;
  }

  .result-empty {
    font-size: 13px;
    color: #6a6a5a;
  }

  .result-success {
    font-size: 13px;
    color: #b8e060;
    font-weight: 600;
    margin-bottom: 8px;
  }

  .result-list {
    padding-left: 20px;
    font-size: 13px;
    color: #c0b89a;
  }

  .result-list li {
    margin-bottom: 3px;
  }

  .scan-error {
    margin-top: 16px;
    padding: 14px 16px;
    background: #1a1210;
    border: 1px solid #3a2218;
    border-radius: 8px;
    font-size: 13px;
    color: #e06060;
  }
</style>
