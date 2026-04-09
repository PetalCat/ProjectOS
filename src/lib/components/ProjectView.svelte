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
  let newIssueBody = $state("");
  let newIssueStatus = $state("ready");
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
    await createIssue({
      title: newIssueTitle.trim(),
      project_id: project.id,
      body: newIssueBody.trim() || undefined,
      status: newIssueStatus,
    });
    newIssueTitle = "";
    newIssueBody = "";
    newIssueStatus = "ready";
    creatingIssue = false;
    await refresh();
  }

  function cancelCreate() {
    creatingIssue = false;
    newIssueTitle = "";
    newIssueBody = "";
    newIssueStatus = "ready";
  }

  function handleNewIssueKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") cancelCreate();
    if (e.key === "Enter" && e.metaKey) handleCreateIssue();
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
    <div class="new-issue-panel">
      <div class="new-issue-header">
        <h3>New Issue</h3>
        <button class="cancel-x" onclick={cancelCreate}>×</button>
      </div>
      <input
        bind:this={newIssueInputEl}
        class="new-issue-title-input"
        type="text"
        placeholder="Title"
        bind:value={newIssueTitle}
        onkeydown={handleNewIssueKeydown}
      />
      <textarea
        class="new-issue-body-input"
        placeholder="Description (optional)"
        bind:value={newIssueBody}
        onkeydown={handleNewIssueKeydown}
        rows="4"
      ></textarea>
      <div class="new-issue-footer">
        <div class="status-picker">
          <label class="status-option">
            <input type="radio" name="status" value="ready" bind:group={newIssueStatus} />
            <span class="status-dot ready"></span> Ready
          </label>
          <label class="status-option">
            <input type="radio" name="status" value="idea" bind:group={newIssueStatus} />
            <span class="status-dot idea"></span> Idea
          </label>
          <label class="status-option">
            <input type="radio" name="status" value="next" bind:group={newIssueStatus} />
            <span class="status-dot next"></span> Next
          </label>
        </div>
        <div class="new-issue-actions">
          <button class="cancel-btn" onclick={cancelCreate}>Cancel</button>
          <button class="create-btn" onclick={handleCreateIssue} disabled={!newIssueTitle.trim()}>
            Create Issue
          </button>
        </div>
      </div>
      <div class="new-issue-hint">⌘↵ to submit</div>
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

  .new-issue-panel {
    margin: 0 24px;
    padding: 20px;
    background: #141410;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    flex-shrink: 0;
    margin-bottom: 8px;
  }

  .new-issue-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 14px;
  }

  .new-issue-header h3 {
    font-size: 15px;
    font-weight: 700;
    color: #c8c8b0;
  }

  .cancel-x {
    font-size: 20px;
    color: #5a5a4a;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0 4px;
    line-height: 1;
  }
  .cancel-x:hover { color: #8a8a7a; }

  .new-issue-title-input {
    width: 100%;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 10px 14px;
    font-size: 15px;
    font-weight: 600;
    color: #e8e8d8;
    font-family: inherit;
    outline: none;
    margin-bottom: 10px;
  }
  .new-issue-title-input:focus { border-color: rgba(255, 255, 255, 0.2); }
  .new-issue-title-input::placeholder { color: #4a4a3a; }

  .new-issue-body-input {
    width: 100%;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    padding: 10px 14px;
    font-size: 13px;
    color: #a8a898;
    font-family: inherit;
    outline: none;
    resize: vertical;
    min-height: 80px;
    margin-bottom: 14px;
  }
  .new-issue-body-input:focus { border-color: rgba(255, 255, 255, 0.15); }
  .new-issue-body-input::placeholder { color: #3a3a2a; }

  .new-issue-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .status-picker {
    display: flex;
    gap: 12px;
  }

  .status-option {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    color: #8a8a7a;
    cursor: pointer;
  }
  .status-option input[type="radio"] { display: none; }
  .status-option:has(input:checked) { color: #d8d8c8; font-weight: 600; }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    border: 2px solid #4a4a40;
  }
  .status-dot.ready { border-color: #6a6a5a; }
  .status-dot.idea { border-style: dashed; border-color: #4a4a40; }
  .status-dot.next { background: #b8e060; border-color: #b8e060; }
  .status-option:has(input:checked) .status-dot.ready { border-color: #a0a090; }
  .status-option:has(input:checked) .status-dot.idea { border-color: #6a6a5a; }

  .new-issue-actions {
    display: flex;
    gap: 8px;
  }

  .cancel-btn {
    font-size: 12px;
    font-weight: 600;
    color: #6a6a5a;
    background: none;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 7px;
    padding: 7px 14px;
    cursor: pointer;
  }

  .create-btn {
    font-size: 12px;
    font-weight: 700;
    color: #0a0a0a;
    background: #b8e060;
    border: none;
    border-radius: 7px;
    padding: 7px 16px;
    cursor: pointer;
  }
  .create-btn:disabled { opacity: 0.4; cursor: not-allowed; }
  .create-btn:hover:not(:disabled) { opacity: 0.9; }

  .new-issue-hint {
    text-align: right;
    font-size: 10px;
    color: #3a3a2a;
    margin-top: 8px;
  }

  .loading {
    padding: 48px 24px;
    color: #4a4a3a;
    font-size: 14px;
  }
</style>
