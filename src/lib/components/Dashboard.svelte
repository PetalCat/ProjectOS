<script lang="ts">
  import { loadDashboard, getDashboardData, isLoading } from "$lib/stores/dashboard.svelte";
  import { navigate } from "$lib/stores/navigation.svelte";
  import { getProjects, projectColor } from "$lib/stores/projects.svelte";
  import ActivityFeed from "./ActivityFeed.svelte";
  import type { DashboardProject, Issue } from "$lib/types";

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
    return new Date().toLocaleDateString(undefined, {
      weekday: "long",
      month: "long",
      day: "numeric",
    });
  }

  type UpNextRow = { dp: DashboardProject; issue: Issue };

  // Cross-project Up Next: every project that has a `next` issue contributes
  // one row, ordered by project recency (Rust already orders projects by
  // updated_at desc).
  const upNext = $derived<UpNextRow[]>(
    (dashboard?.projects ?? [])
      .filter((dp) => dp.next_issue)
      .map((dp) => ({ dp, issue: dp.next_issue as Issue })),
  );

  // Active projects: anything with open issues OR a `next`. Empty silent
  // projects fall into "Other".
  const activeProjects = $derived(
    (dashboard?.projects ?? []).filter(
      (dp) => dp.next_issue || dp.open_count > 0,
    ),
  );
  const otherProjects = $derived(
    (dashboard?.projects ?? []).filter(
      (dp) => !dp.next_issue && dp.open_count === 0,
    ),
  );

  let showOther = $state(false);

  function statusGlyph(status: string | null): { char: string; cls: string } {
    switch (status) {
      case "next":
        return { char: "▶", cls: "next" };
      case "blocked":
        return { char: "!", cls: "blocked" };
      case "idea":
        return { char: "·", cls: "idea" };
      default:
        return { char: "", cls: "ready" };
    }
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
      {#if dashboard.projects.length === 0}
        <!-- No projects: friendly empty hero. Onboarding wizard normally
             takes over before we get here, but if a user added a folder
             that scanned zero projects we still need this. -->
        <section class="hero">
          <h2 class="hero-title">Local issue tracking for agent-driven development.</h2>
          <p class="hero-body">
            Agents file issues. You triage. ProjectOS tracks the loop locally —
            a polished desktop UI for humans, an MCP server for Claude.
          </p>
          <div class="hero-actions">
            <button class="btn-primary" onclick={() => navigate({ kind: "settings" })}>
              Add a project
            </button>
          </div>
        </section>
      {:else}
        <!-- ── Up Next ── -->
        {#if upNext.length > 0}
          <section>
            <div class="section-label">Up Next</div>
            <div class="up-next">
              {#each upNext as { dp, issue } (issue.id)}
                {@const idx = projectIdx(dp.project.id)}
                {@const color = projectColor(idx)}
                {@const g = statusGlyph(issue.status)}
                <button
                  class="up-next-row"
                  onclick={() => navigate({ kind: "issue", issueId: issue.id })}
                >
                  <span class={`circle ${g.cls}`} aria-hidden="true">{g.char}</span>
                  <span class="project-tag" style:--accent={color}>
                    <span class="project-dot" style:background={color}></span>
                    {dp.project.name}
                  </span>
                  <span class="up-next-title">{issue.title}</span>
                  <span class="up-next-count">{dp.open_count} open</span>
                </button>
              {/each}
            </div>
          </section>
        {:else}
          <section>
            <div class="section-label">Up Next</div>
            <div class="up-next-empty">
              No project has a <code>next</code> issue set yet. Open a project
              and pick what to work on.
            </div>
          </section>
        {/if}

        <!-- ── Recent activity ── -->
        <section>
          <div class="section-label">Recent Activity</div>
          <ActivityFeed entries={dashboard.recent_activity} />
        </section>

        <!-- ── Active projects (compact list) ── -->
        {#if activeProjects.length > 0}
          <section>
            <div class="section-label">Active Projects</div>
            <div class="project-list">
              {#each activeProjects as dp (dp.project.id)}
                {@const idx = projectIdx(dp.project.id)}
                {@const color = projectColor(idx)}
                <button
                  class="project-row"
                  style:--accent={color}
                  onclick={() => navigate({ kind: "project", projectId: dp.project.id })}
                >
                  <span class="project-dot" style:background={color}></span>
                  <span class="project-row-name">{dp.project.name}</span>
                  <span class="project-row-meta">
                    {#if dp.next_issue}
                      <span class="row-next">{dp.next_issue.title}</span>
                    {/if}
                    <span class="row-open">{dp.open_count} open</span>
                  </span>
                </button>
              {/each}
            </div>
          </section>
        {/if}

        <!-- ── Other (idle) projects ── -->
        {#if otherProjects.length > 0}
          <section>
            <button
              class="other-toggle"
              onclick={() => (showOther = !showOther)}
            >
              {showOther ? "▼" : "▶"} All projects ({otherProjects.length} idle)
            </button>
            {#if showOther}
              <div class="project-list quiet">
                {#each otherProjects as dp (dp.project.id)}
                  {@const idx = projectIdx(dp.project.id)}
                  {@const color = projectColor(idx)}
                  <button
                    class="project-row"
                    style:--accent={color}
                    onclick={() => navigate({ kind: "project", projectId: dp.project.id })}
                  >
                    <span class="project-dot" style:background={color}></span>
                    <span class="project-row-name">{dp.project.name}</span>
                    <span class="project-row-meta">
                      <span class="row-open quiet">no open issues</span>
                    </span>
                  </button>
                {/each}
              </div>
            {/if}
          </section>
        {/if}
      {/if}
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
    padding: 24px 32px 60px;
    display: flex;
    flex-direction: column;
    gap: 28px;
  }

  .loading {
    padding: 48px 32px;
    color: #5a5a4a;
    font-size: 14px;
  }

  /* ── Hero (empty state) ── */
  .hero {
    background: #141410;
    border: 1px solid #1e1e1a;
    border-radius: 14px;
    padding: 32px 28px;
    max-width: 720px;
  }

  .hero-title {
    margin: 0 0 12px;
    font-size: 22px;
    font-weight: 800;
    color: #f0ead6;
    letter-spacing: -0.01em;
  }

  .hero-body {
    margin: 0 0 22px;
    font-size: 14px;
    line-height: 1.55;
    color: #8a8a7a;
    max-width: 560px;
  }

  .hero-actions {
    display: flex;
    gap: 10px;
  }

  /* ── Section label ── */
  .section-label {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: #4a4a3a;
    margin-bottom: 10px;
  }

  /* ── Up Next list ── */
  .up-next {
    background: #131310;
    border: 1px solid #1e1e1a;
    border-radius: 10px;
    overflow: hidden;
  }

  .up-next-row {
    display: grid;
    grid-template-columns: auto auto 1fr auto;
    gap: 14px;
    align-items: center;
    width: 100%;
    padding: 12px 16px;
    background: none;
    border: none;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
    cursor: pointer;
    text-align: left;
    transition: background 0.1s;
  }
  .up-next-row:last-child {
    border-bottom: none;
  }
  .up-next-row:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  .circle {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 9px;
    font-weight: 700;
    flex-shrink: 0;
  }
  .circle.next {
    background: #b8e060;
    color: #0a1a00;
    font-size: 8px;
  }
  .circle.ready {
    border: 2px solid #3a3a2a;
    background: transparent;
  }
  .circle.blocked {
    border: 2px solid #e8a040;
    color: #e8a040;
    font-size: 11px;
    font-weight: 900;
  }
  .circle.idea {
    border: 2px dashed #4a4a3a;
    color: #4a4a3a;
    font-size: 14px;
    line-height: 1;
  }

  .project-tag {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: #8a8a7a;
    padding: 3px 8px;
    border-radius: 5px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.05);
  }

  .project-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .up-next-title {
    font-size: 14px;
    font-weight: 500;
    color: #d8d8c8;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .up-next-count {
    font-size: 11px;
    color: #5a5a4a;
    flex-shrink: 0;
  }

  .up-next-empty {
    background: #131310;
    border: 1px dashed #2a2a22;
    border-radius: 10px;
    padding: 16px;
    color: #6a6a5a;
    font-size: 13px;
    line-height: 1.5;
  }
  .up-next-empty code {
    font-family: ui-monospace, SFMono-Regular, monospace;
    background: rgba(255, 255, 255, 0.05);
    padding: 1px 5px;
    border-radius: 3px;
    color: #b8e060;
    font-size: 12px;
  }

  /* ── Project list (compact rows) ── */
  .project-list {
    display: flex;
    flex-direction: column;
    background: #131310;
    border: 1px solid #1e1e1a;
    border-radius: 10px;
    overflow: hidden;
  }
  .project-list.quiet {
    margin-top: 10px;
    background: #100f0d;
    opacity: 0.85;
  }

  .project-row {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: 12px;
    align-items: center;
    width: 100%;
    padding: 10px 16px;
    background: none;
    border: none;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
    cursor: pointer;
    text-align: left;
    transition: background 0.1s;
  }
  .project-row:last-child {
    border-bottom: none;
  }
  .project-row:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  .project-row-name {
    font-size: 13px;
    font-weight: 600;
    color: #c8c8b0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .project-row-meta {
    display: flex;
    gap: 12px;
    align-items: center;
    flex-shrink: 0;
    overflow: hidden;
  }

  .row-next {
    font-size: 12px;
    color: #a09870;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 320px;
  }

  .row-open {
    font-size: 11px;
    color: #5a5a4a;
    flex-shrink: 0;
  }
  .row-open.quiet {
    color: #3a3a2a;
    font-style: italic;
  }

  /* ── Other-projects disclosure ── */
  .other-toggle {
    background: none;
    border: none;
    color: #5a5a4a;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    cursor: pointer;
    padding: 0;
    text-align: left;
  }
  .other-toggle:hover {
    color: #a09870;
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
</style>
