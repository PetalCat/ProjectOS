<script lang="ts">
  import { loadDashboard, getDashboardData, isLoading } from "$lib/stores/dashboard.svelte";
  import { navigate } from "$lib/stores/navigation.svelte";
  import { getProjects, projectColor } from "$lib/stores/projects.svelte";
  import ActivityFeed from "./ActivityFeed.svelte";

  $effect(() => {
    loadDashboard();
  });

  const dashboard = $derived(getDashboardData());
  const loading = $derived(isLoading());
  const projects = $derived(getProjects());

  function projectIdx(projectId: string): number {
    return projects.findIndex((p) => p.id === projectId);
  }

  function today(): string {
    return new Date().toLocaleDateString("en-US", {
      weekday: "long",
      month: "long",
      day: "numeric",
    });
  }
</script>

<div class="dashboard">
  <div class="dashboard-header">
    <div class="header-title">
      <h1>What's Next</h1>
      <span class="date">{today()}</span>
    </div>
  </div>

  {#if loading}
    <div class="loading">Loading…</div>
  {:else if dashboard}
    <div class="content">
      <section class="next-section">
        <div class="card-grid">
          {#each dashboard.projects as dp (dp.project.id)}
            {@const idx = projectIdx(dp.project.id)}
            {@const color = projectColor(idx)}
            <button
              class="project-card"
              style:--accent={color}
              onclick={() => navigate({ kind: "project", projectId: dp.project.id })}
            >
              <div class="card-top-bar"></div>
              <div class="card-body">
                <div class="card-project-name">
                  <span class="project-dot" style:background={color}></span>
                  {dp.project.name}
                </div>
                {#if dp.next_issue}
                  <div class="card-issue-title">{dp.next_issue.title}</div>
                  <div class="card-meta">
                    {#if dp.next_issue.machine_id}
                      <span class="card-machine">⬡ machine</span>
                    {/if}
                    <span class="card-count">{dp.open_count} open</span>
                  </div>
                {:else}
                  <div class="card-no-next">No next issue</div>
                  <div class="card-meta">
                    <span class="card-count">{dp.open_count} open</span>
                  </div>
                {/if}
              </div>
            </button>
          {/each}

          {#if dashboard.projects.length === 0}
            <div class="no-projects">
              <p>No projects yet.</p>
              <p class="hint">Create a project to get started.</p>
            </div>
          {/if}
        </div>
      </section>

      <section class="activity-section">
        <div class="section-label">Recent Activity</div>
        <ActivityFeed entries={dashboard.recent_activity} />
      </section>
    </div>
  {/if}
</div>

<style>
  .dashboard {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .dashboard-header {
    padding: 28px 32px 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    flex-shrink: 0;
  }

  .header-title {
    display: flex;
    align-items: baseline;
    gap: 16px;
  }

  h1 {
    font-size: 26px;
    font-weight: 800;
    color: #e8e8d8;
    margin: 0;
  }

  .date {
    font-size: 13px;
    color: #5a5a4a;
    font-weight: 500;
  }

  .content {
    flex: 1;
    overflow-y: auto;
    padding: 24px 32px;
    display: flex;
    flex-direction: column;
    gap: 32px;
  }

  .loading {
    padding: 48px 32px;
    color: #5a5a4a;
    font-size: 14px;
  }

  .card-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 16px;
  }

  .project-card {
    background: #141410;
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 10px;
    overflow: hidden;
    cursor: pointer;
    text-align: left;
    transition: border-color 0.15s ease, transform 0.1s ease;
    padding: 0;
    width: 100%;
  }

  .project-card:hover {
    border-color: rgba(var(--accent, 200, 200, 150), 0.3);
    transform: translateY(-1px);
  }

  .card-top-bar {
    height: 3px;
    background: var(--accent, #6a6a5a);
  }

  .card-body {
    padding: 14px 16px;
  }

  .card-project-name {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: #7a7a6a;
    margin-bottom: 8px;
  }

  .project-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .card-issue-title {
    font-size: 14px;
    font-weight: 600;
    color: #d8d8c8;
    line-height: 1.4;
    margin-bottom: 10px;
  }

  .card-no-next {
    font-size: 13px;
    color: #4a4a3a;
    font-style: italic;
    margin-bottom: 10px;
  }

  .card-meta {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 11px;
    color: #5a5a4a;
  }

  .card-machine {
    color: #6a8a6a;
  }

  .section-label {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: #4a4a3a;
    margin-bottom: 12px;
  }

  .no-projects {
    padding: 32px;
    color: #4a4a3a;
    font-size: 14px;
    grid-column: 1 / -1;
  }

  .hint {
    margin-top: 4px;
    font-size: 12px;
    color: #3a3a2a;
  }
</style>
