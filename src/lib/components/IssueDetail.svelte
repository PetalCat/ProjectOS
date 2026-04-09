<script lang="ts">
  import type { Issue, Label, Milestone } from "$lib/types";
  import {
    getIssue,
    updateIssue,
    closeIssue,
    reopenIssue,
    getIssueLabels,
    listMilestones,
  } from "$lib/commands";
  import { navigate, navigateBack } from "$lib/stores/navigation.svelte";
  import { getProjects, projectColor } from "$lib/stores/projects.svelte";
  import StatusBadge from "./StatusBadge.svelte";
  import LabelBadge from "./LabelBadge.svelte";
  import CommentThread from "./CommentThread.svelte";

  type Props = {
    issueId: string;
  };

  let { issueId }: Props = $props();

  let issue = $state<Issue | null>(null);
  let labels = $state<Label[]>([]);
  let milestone = $state<Milestone | null>(null);
  let loading = $state(true);
  let editingContext = $state(false);
  let contextDraft = $state("");
  let editingTitle = $state(false);
  let titleDraft = $state("");

  const projects = $derived(getProjects());
  const project = $derived(issue?.project_id ? projects.find((p) => p.id === issue!.project_id) ?? null : null);
  const projectIdx = $derived(project ? projects.findIndex((p) => p.id === project.id) : 0);
  const accent = $derived(projectColor(projectIdx));

  $effect(() => {
    const id = issueId;
    loading = true;
    getIssue(id)
      .then(async (i) => {
        issue = i;
        contextDraft = i.context ?? "";
        titleDraft = i.title;
        labels = await getIssueLabels(id).catch(() => []);
        if (i.project_id && i.milestone_id) {
          const ms = await listMilestones(i.project_id).catch(() => []);
          milestone = ms.find((m) => m.id === i.milestone_id) ?? null;
        }
      })
      .catch(() => {})
      .finally(() => { loading = false; });
  });

  function formatTime(ms: number): string {
    return new Date(ms).toLocaleDateString("en-US", {
      month: "long",
      day: "numeric",
      year: "numeric",
    });
  }

  async function handleClose() {
    if (!issue) return;
    issue = await closeIssue(issue.id);
  }

  async function handleReopen() {
    if (!issue) return;
    issue = await reopenIssue(issue.id);
  }

  async function saveContext() {
    if (!issue) return;
    issue = await updateIssue({ id: issue.id, context: contextDraft });
    editingContext = false;
  }

  async function saveTitle() {
    if (!issue || !titleDraft.trim()) return;
    issue = await updateIssue({ id: issue.id, title: titleDraft.trim() });
    editingTitle = false;
  }

  function backLabel(): string {
    return project ? `← ${project.name}` : "← Back";
  }

  function goBack() {
    if (project) {
      navigate({ kind: "project", projectId: project.id });
    } else {
      navigateBack();
    }
  }
</script>

{#if loading}
  <div class="loading">Loading…</div>
{:else if issue}
  <div class="issue-detail">
    <div class="detail-header">
      <button class="back-btn" onclick={goBack}>{backLabel()}</button>
      <div class="header-row">
        <div class="header-left">
          <div class="header-badges">
            <StatusBadge status={issue.status} state={issue.state} />
            {#if issue.state === "open"}
              <button class="action-btn close-btn" onclick={handleClose}>Close issue</button>
            {:else}
              <button class="action-btn reopen-btn" onclick={handleReopen}>Reopen</button>
            {/if}
          </div>

          {#if editingTitle}
            <div class="title-edit-row">
              <input
                class="title-input"
                type="text"
                bind:value={titleDraft}
                onkeydown={(e) => { if (e.key === "Enter") saveTitle(); if (e.key === "Escape") editingTitle = false; }}
              />
              <button class="confirm-btn" onclick={saveTitle}>Save</button>
              <button class="cancel-small-btn" onclick={() => editingTitle = false}>Cancel</button>
            </div>
          {:else}
            <button
              class="issue-title"
              onclick={() => { editingTitle = true; titleDraft = issue!.title; }}
            >
              {issue.title}
            </button>
          {/if}

          <div class="issue-meta">
            {#if issue.number}<span>#{issue.number}</span>{/if}
            <span>·</span>
            <span>opened {formatTime(issue.created_at)}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="detail-body">
      <div class="detail-main">
        {#if issue.body}
          <div class="body-card">
            <div class="body-content">{issue.body}</div>
          </div>
        {:else}
          <div class="body-card empty-body">
            <span>No description.</span>
          </div>
        {/if}

        <div class="context-block" class:has-context={!!issue.context}>
          <div class="context-header">
            <span class="context-icon">📍</span>
            <span class="context-label">Where I left off</span>
            {#if !editingContext}
              <button class="edit-context-btn" onclick={() => { editingContext = true; contextDraft = issue!.context ?? ""; }}>
                Edit
              </button>
            {/if}
          </div>
          {#if editingContext}
            <textarea
              class="context-input"
              bind:value={contextDraft}
              placeholder="Add a bookmark note…"
              rows="3"
            ></textarea>
            <div class="context-actions">
              <button class="confirm-btn" onclick={saveContext}>Save</button>
              <button class="cancel-small-btn" onclick={() => editingContext = false}>Cancel</button>
            </div>
          {:else if issue.context}
            <div class="context-text">{issue.context}</div>
          {:else}
            <div class="context-empty">No context saved yet.</div>
          {/if}
        </div>

        <CommentThread issueId={issue.id} locked={issue.locked} />
      </div>

      <aside class="detail-sidebar">
        <div class="sidebar-section">
          <div class="sidebar-label">Status</div>
          <div class="sidebar-value">
            <StatusBadge status={issue.status} state={issue.state} />
          </div>
        </div>

        {#if labels.length > 0}
          <div class="sidebar-section">
            <div class="sidebar-label">Labels</div>
            <div class="sidebar-value label-list">
              {#each labels as lbl (lbl.id)}
                <LabelBadge label={lbl} />
              {/each}
            </div>
          </div>
        {/if}

        {#if milestone}
          {@const total = (milestone.open_count ?? 0) + (milestone.closed_count ?? 0)}
          {@const pct = total > 0 ? Math.round(((milestone.closed_count ?? 0) / total) * 100) : 0}
          <div class="sidebar-section">
            <div class="sidebar-label">Milestone</div>
            <div class="sidebar-value">
              <div class="milestone-name">{milestone.title}</div>
              <div class="milestone-mini-bar">
                <div class="mini-fill" style:width="{pct}%"></div>
              </div>
              <div class="milestone-pct">{pct}%</div>
            </div>
          </div>
        {/if}

        {#if project}
          <div class="sidebar-section">
            <div class="sidebar-label">Project</div>
            <div class="sidebar-value">
              <button
                class="project-link"
                style:color={accent}
                onclick={() => navigate({ kind: "project", projectId: project!.id })}
              >
                <span class="dot" style:background={accent}></span>
                {project.name}
              </button>
            </div>
          </div>
        {/if}

        <div class="sidebar-section">
          <div class="sidebar-label">Actions</div>
          <div class="sidebar-value actions-list">
            {#if issue.state === "open"}
              <button class="action-link" onclick={handleClose}>Close issue</button>
            {:else}
              <button class="action-link" onclick={handleReopen}>Reopen issue</button>
            {/if}
          </div>
        </div>
      </aside>
    </div>
  </div>
{:else}
  <div class="loading">Issue not found.</div>
{/if}

<style>
  .loading {
    padding: 48px 32px;
    color: #4a4a3a;
    font-size: 14px;
  }

  .issue-detail {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .detail-header {
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
    padding: 0 0 10px;
    display: block;
  }

  .back-btn:hover { color: #8a8a7a; }

  .header-row {
    display: flex;
    align-items: flex-start;
    gap: 16px;
  }

  .header-left { flex: 1; }

  .header-badges {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 10px;
  }

  .issue-title {
    font-size: 22px;
    font-weight: 800;
    color: #e8e8d8;
    margin: 0 0 8px;
    cursor: pointer;
    line-height: 1.3;
    background: none;
    border: none;
    padding: 0;
    text-align: left;
    display: block;
    width: 100%;
    font-family: inherit;
  }

  .issue-title:hover { color: #ffffff; }

  .title-edit-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .title-input {
    flex: 1;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.18);
    border-radius: 6px;
    padding: 6px 10px;
    font-size: 18px;
    font-weight: 700;
    color: #e8e8d8;
    font-family: inherit;
    outline: none;
  }

  .issue-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: #5a5a4a;
  }

  .detail-body {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .detail-main {
    flex: 1;
    padding: 24px 28px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .body-card {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 10px;
    padding: 16px;
  }

  .body-content {
    font-size: 14px;
    color: #c8c8b8;
    line-height: 1.6;
    white-space: pre-wrap;
  }

  .empty-body {
    color: #4a4a3a;
    font-style: italic;
    font-size: 13px;
  }

  .context-block {
    border: 1px dashed rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    padding: 14px;
  }

  .context-block.has-context {
    border-color: rgba(184, 224, 96, 0.15);
  }

  .context-header {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 8px;
  }

  .context-icon { font-size: 13px; }

  .context-label {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: #5a5a4a;
    flex: 1;
  }

  .edit-context-btn {
    font-size: 11px;
    color: #5a5a4a;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
  }

  .edit-context-btn:hover { color: #8a8a7a; }

  .context-text {
    font-size: 13px;
    color: #a0a090;
    line-height: 1.5;
    white-space: pre-wrap;
  }

  .context-empty {
    font-size: 12px;
    color: #4a4a3a;
    font-style: italic;
  }

  .context-input {
    width: 100%;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    padding: 8px 10px;
    font-size: 13px;
    color: #d0d0c0;
    font-family: inherit;
    resize: vertical;
    outline: none;
    box-sizing: border-box;
  }

  .context-actions {
    display: flex;
    gap: 8px;
    margin-top: 8px;
  }

  .confirm-btn {
    font-size: 12px;
    font-weight: 700;
    color: #0a0a0a;
    background: #b8e060;
    border: none;
    border-radius: 6px;
    padding: 6px 14px;
    cursor: pointer;
  }

  .cancel-small-btn {
    font-size: 12px;
    color: #6a6a5a;
    background: none;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    padding: 6px 12px;
    cursor: pointer;
  }

  .detail-sidebar {
    width: 260px;
    flex-shrink: 0;
    border-left: 1px solid rgba(255, 255, 255, 0.06);
    padding: 20px 18px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .sidebar-section {}

  .sidebar-label {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: #4a4a3a;
    margin-bottom: 7px;
  }

  .sidebar-value {
    font-size: 13px;
    color: #8a8a7a;
  }

  .label-list {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
  }

  .milestone-name {
    font-size: 12px;
    font-weight: 600;
    color: #a0a090;
    margin-bottom: 5px;
  }

  .milestone-mini-bar {
    height: 3px;
    background: rgba(255, 255, 255, 0.08);
    border-radius: 2px;
    overflow: hidden;
    margin-bottom: 3px;
  }

  .mini-fill {
    height: 100%;
    background: #b8e060;
    border-radius: 2px;
  }

  .milestone-pct {
    font-size: 11px;
    color: #b8e060;
    font-weight: 700;
  }

  .project-link {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    font-weight: 600;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .actions-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .action-link {
    font-size: 12px;
    font-weight: 600;
    color: #6a6a5a;
    background: none;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 6px;
    padding: 6px 10px;
    cursor: pointer;
    text-align: left;
    transition: all 0.12s;
  }

  .action-link:hover {
    color: #a0a090;
    border-color: rgba(255, 255, 255, 0.15);
  }

  .action-btn {
    font-size: 12px;
    font-weight: 600;
    border: none;
    border-radius: 6px;
    padding: 5px 12px;
    cursor: pointer;
  }

  .close-btn {
    background: rgba(232, 160, 64, 0.15);
    color: #e8a040;
    border: 1px solid rgba(232, 160, 64, 0.3);
  }

  .reopen-btn {
    background: rgba(184, 224, 96, 0.15);
    color: #b8e060;
    border: 1px solid rgba(184, 224, 96, 0.3);
  }
</style>
