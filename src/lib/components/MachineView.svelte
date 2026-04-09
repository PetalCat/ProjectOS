<script lang="ts">
  import type { Machine, MachineDoc, Issue } from "$lib/types";
  import { listMachineDocs, listIssues } from "$lib/commands";
  import { navigate } from "$lib/stores/navigation.svelte";
  import IssueRow from "./IssueRow.svelte";

  type Props = {
    machine: Machine;
  };

  let { machine }: Props = $props();

  let docs = $state<MachineDoc[]>([]);
  let taggedIssues = $state<Issue[]>([]);
  let loading = $state(true);

  $effect(() => {
    const id = machine.id;
    loading = true;
    Promise.all([
      listMachineDocs(id),
      listIssues(null, false),
    ])
      .then(([d, all]) => {
        docs = d;
        taggedIssues = all.filter((i) => i.machine_id === id);
      })
      .catch(() => {})
      .finally(() => { loading = false; });
  });

  function formatDate(ms: number): string {
    return new Date(ms).toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" });
  }
</script>

<div class="machine-view">
  <div class="machine-header">
    <button class="back-btn" onclick={() => navigate({ kind: "home" })}>← Home</button>
    <div class="machine-icon">⬡</div>
    <h1>{machine.name}</h1>
  </div>

  <div class="machine-body">
    <div class="machine-main">
      <section class="info-section">
        <div class="section-label">Machine Info</div>
        <div class="info-grid">
          {#if machine.hostname}
            <div class="info-row">
              <span class="info-key">Hostname</span>
              <span class="info-val">{machine.hostname}</span>
            </div>
          {/if}
          {#if machine.ip}
            <div class="info-row">
              <span class="info-key">IP</span>
              <span class="info-val">{machine.ip}</span>
            </div>
          {/if}
          {#if machine.user}
            <div class="info-row">
              <span class="info-key">User</span>
              <span class="info-val">{machine.user}</span>
            </div>
          {/if}
          {#if machine.os}
            <div class="info-row">
              <span class="info-key">OS</span>
              <span class="info-val">{machine.os}</span>
            </div>
          {/if}
          <div class="info-row">
            <span class="info-key">Added</span>
            <span class="info-val">{formatDate(machine.created_at)}</span>
          </div>
        </div>
        {#if machine.notes}
          <div class="notes-block">
            <div class="notes-label">Notes</div>
            <div class="notes-content">{machine.notes}</div>
          </div>
        {/if}
      </section>

      {#if docs.length > 0}
        <section class="docs-section">
          <div class="section-label">Docs ({docs.length})</div>
          <div class="docs-list">
            {#each docs as doc (doc.id)}
              <div class="doc-card">
                <div class="doc-title">{doc.title}</div>
                {#if doc.url}
                  <a class="doc-url" href={doc.url} target="_blank" rel="noopener">{doc.url}</a>
                {/if}
                {#if doc.content}
                  <div class="doc-content">{doc.content}</div>
                {/if}
              </div>
            {/each}
          </div>
        </section>
      {/if}

      {#if taggedIssues.length > 0}
        <section class="issues-section">
          <div class="section-label">Tagged Issues ({taggedIssues.length})</div>
          <div class="issues-list">
            {#each taggedIssues as issue (issue.id)}
              <IssueRow {issue} />
            {/each}
          </div>
        </section>
      {/if}

      {#if loading}
        <div class="loading">Loading…</div>
      {/if}
    </div>
  </div>
</div>

<style>
  .machine-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .machine-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 20px 28px 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    flex-shrink: 0;
  }

  .back-btn {
    font-size: 12px;
    color: #5a5a4a;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    flex-shrink: 0;
  }

  .back-btn:hover { color: #8a8a7a; }

  .machine-icon {
    font-size: 18px;
    color: #6a8a6a;
  }

  h1 {
    font-size: 22px;
    font-weight: 800;
    color: #e8e8d8;
    margin: 0;
  }

  .machine-body {
    flex: 1;
    overflow-y: auto;
  }

  .machine-main {
    padding: 24px 28px;
    display: flex;
    flex-direction: column;
    gap: 28px;
    max-width: 720px;
  }

  .section-label {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: #4a4a3a;
    margin-bottom: 12px;
  }

  .info-grid {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .info-row {
    display: flex;
    align-items: center;
    gap: 0;
    font-size: 13px;
  }

  .info-key {
    width: 90px;
    color: #5a5a4a;
    font-weight: 600;
    flex-shrink: 0;
  }

  .info-val {
    color: #c0c0b0;
    font-family: "SF Mono", monospace;
    font-size: 12px;
  }

  .notes-block {
    margin-top: 14px;
    padding: 12px;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.06);
  }

  .notes-label {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: #4a4a3a;
    margin-bottom: 6px;
  }

  .notes-content {
    font-size: 13px;
    color: #a0a090;
    line-height: 1.55;
    white-space: pre-wrap;
  }

  .docs-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .doc-card {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 8px;
    padding: 12px 14px;
  }

  .doc-title {
    font-size: 13px;
    font-weight: 600;
    color: #c8c8b8;
    margin-bottom: 5px;
  }

  .doc-url {
    font-size: 12px;
    color: #60b8e0;
    text-decoration: none;
    display: block;
    margin-bottom: 5px;
    word-break: break-all;
  }

  .doc-url:hover { text-decoration: underline; }

  .doc-content {
    font-size: 12px;
    color: #8a8a7a;
    line-height: 1.5;
    white-space: pre-wrap;
  }

  .issues-list {
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 8px;
    overflow: hidden;
  }

  .loading {
    color: #4a4a3a;
    font-size: 14px;
    padding: 20px 0;
  }
</style>
