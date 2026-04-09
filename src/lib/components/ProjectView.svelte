<script lang="ts">
  import type { Project, Milestone, Label } from "$lib/types";
  import { loadIssues, getIssues, isLoading, getShowClosed } from "$lib/stores/issues.svelte";
  import { listMilestones, createIssue, getIssueLabels, syncGithubIssues } from "$lib/commands";
  import { projectColor, getProjects } from "$lib/stores/projects.svelte";
  import IssueList from "./IssueList.svelte";
  import MilestoneBar from "./MilestoneBar.svelte";
  import { navigate } from "$lib/stores/navigation.svelte";

  type Props = {
    project: Project;
  };

  let { project }: Props = $props();

  let milestones = $state<Milestone[]>([]);
  let labelMap = $state<Record<string, Label[]>>({});
  let showClosed = $state(false);
  let creatingIssue = $state(false);
  let newIssueTitle = $state("");
  let newIssueInputEl = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (creatingIssue && newIssueInputEl) {
      newIssueInputEl.focus();
    }
  });

  const projects = $derived(getProjects());
  const projectIdx = $derived(projects.findIndex((p) => p.id === project.id));
  const accent = $derived(projectColor(projectIdx));

  const issues = $derived(getIssues());
  const loading = $derived(isLoading());

  const activeMilestone = $derived(milestones.find((m) => m.state === "open") ?? null);

  const openCount = $derived(issues.filter((i) => i.state === "open").length);
  const closedCount = $derived(issues.filter((i) => i.state === "closed").length);

  async function refresh() {
    await loadIssues(project.id, showClosed);
    // load labels for each issue
    const map: Record<string, Label[]> = {};
    for (const issue of issues) {
      try {
        map[issue.id] = await getIssueLabels(issue.id);
      } catch {
        map[issue.id] = [];
      }
    }
    labelMap = map;
  }

  $effect(() => {
    // re-run when project or showClosed changes
    const pid = project.id;
    const sc = showClosed;
    loadIssues(pid, sc).then(() => {
      const issueList = getIssues();
      const map: Record<string, Label[]> = {};
      Promise.all(
        issueList.map((issue) =>
          getIssueLabels(issue.id)
            .then((labels) => { map[issue.id] = labels; })
            .catch(() => { map[issue.id] = []; })
        )
      ).then(() => { labelMap = map; });
    });

    listMilestones(project.id)
      .then((ms) => { milestones = ms; })
      .catch(() => {});
  });

  async function handleCreateIssue() {
    if (!newIssueTitle.trim()) return;
    await createIssue({ title: newIssueTitle.trim(), project_id: project.id, status: "ready" });
    newIssueTitle = "";
    creatingIssue = false;
    await refresh();
  }

  function handleNewIssueKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") handleCreateIssue();
    if (e.key === "Escape") { creatingIssue = false; newIssueTitle = ""; }
  }

  function toggleClosed() {
    showClosed = !showClosed;
  }

  let syncingGithub = $state(false);
  let syncGithubResult = $state<number | null>(null);

  async function handleSyncGithub() {
    syncingGithub = true;
    syncGithubResult = null;
    try {
      const count = await syncGithubIssues(project.id);
      syncGithubResult = count;
      await refresh();
    } catch {
      // ignore
    } finally {
      syncingGithub = false;
    }
  }
</script>

<div class="project-view">
  <div class="project-header" style:--accent={accent}>
    <div class="project-header-left">
      <button class="back-btn" onclick={() => navigate({ kind: "home" })}>← Home</button>
      <div class="project-dot" style:background={accent}></div>
      <h1>{project.name}</h1>
      <div class="issue-counts">
        <button class="count-btn" class:active={!showClosed} onclick={() => { showClosed = false; }}>
          {openCount} open
        </button>
        <button class="count-btn" class:active={showClosed} onclick={() => { showClosed = true; }}>
          {closedCount} closed
        </button>
      </div>
    </div>
    <div class="header-actions">
      {#if project.github_repo}
        <button class="github-sync-btn" onclick={handleSyncGithub} disabled={syncingGithub} title="Sync issues from GitHub: {project.github_repo}">
          {syncingGithub ? "Syncing…" : syncGithubResult !== null ? `Synced ${syncGithubResult}` : "Sync GitHub Issues"}
        </button>
      {/if}
      <button class="new-issue-btn" style:background={accent} onclick={() => { creatingIssue = true; }}>
        + New Issue
      </button>
    </div>
  </div>

  {#if activeMilestone}
    <div class="milestone-wrap">
      <MilestoneBar milestone={activeMilestone} />
    </div>
  {/if}

  {#if creatingIssue}
    <div class="new-issue-bar">
      <input
        bind:this={newIssueInputEl}
        class="new-issue-input"
        type="text"
        placeholder="Issue title…"
        bind:value={newIssueTitle}
        onkeydown={handleNewIssueKeydown}
      />
      <button class="confirm-btn" onclick={handleCreateIssue}>Create</button>
      <button class="cancel-btn" onclick={() => { creatingIssue = false; newIssueTitle = ""; }}>Cancel</button>
    </div>
  {/if}

  {#if loading}
    <div class="loading">Loading…</div>
  {:else}
    <IssueList {issues} {labelMap} {accent} {showClosed} />
  {/if}
</div>

<style>
  .project-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .project-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    flex-shrink: 0;
    gap: 12px;
  }

  .project-header-left {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
  }

  .back-btn {
    font-size: 12px;
    color: #5a5a4a;
    background: none;
    border: none;
    cursor: pointer;
    padding: 4px 0;
    flex-shrink: 0;
  }

  .back-btn:hover {
    color: #8a8a7a;
  }

  .project-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  h1 {
    font-size: 22px;
    font-weight: 800;
    color: #e8e8d8;
    margin: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .issue-counts {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  .count-btn {
    font-size: 12px;
    font-weight: 600;
    color: #5a5a4a;
    background: none;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 6px;
    padding: 3px 10px;
    cursor: pointer;
    transition: all 0.12s;
  }

  .count-btn.active {
    color: #c8c8b0;
    border-color: rgba(255, 255, 255, 0.15);
    background: rgba(255, 255, 255, 0.05);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .new-issue-btn {
    font-size: 12px;
    font-weight: 700;
    color: #0a0a0a;
    border: none;
    border-radius: 7px;
    padding: 7px 14px;
    cursor: pointer;
    flex-shrink: 0;
    transition: opacity 0.12s;
  }

  .new-issue-btn:hover {
    opacity: 0.85;
  }

  .github-sync-btn {
    font-size: 12px;
    font-weight: 600;
    color: #c0b89a;
    background: #1e1e18;
    border: 1px solid #3a3a2a;
    border-radius: 7px;
    padding: 6px 12px;
    cursor: pointer;
    flex-shrink: 0;
    transition: opacity 0.12s;
  }

  .github-sync-btn:hover {
    opacity: 0.85;
  }

  .github-sync-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .milestone-wrap {
    padding: 10px 24px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
    flex-shrink: 0;
  }

  .new-issue-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 24px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    background: rgba(255, 255, 255, 0.02);
    flex-shrink: 0;
  }

  .new-issue-input {
    flex: 1;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 6px;
    padding: 7px 12px;
    font-size: 13px;
    color: #d8d8c8;
    font-family: inherit;
    outline: none;
  }

  .new-issue-input:focus {
    border-color: rgba(255, 255, 255, 0.25);
  }

  .confirm-btn {
    font-size: 12px;
    font-weight: 600;
    color: #0a0a0a;
    background: #b8e060;
    border: none;
    border-radius: 6px;
    padding: 7px 14px;
    cursor: pointer;
  }

  .cancel-btn {
    font-size: 12px;
    font-weight: 600;
    color: #6a6a5a;
    background: none;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    padding: 7px 12px;
    cursor: pointer;
  }

  .loading {
    padding: 48px 24px;
    color: #4a4a3a;
    font-size: 14px;
  }
</style>
