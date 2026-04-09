<script lang="ts">
  import { getProjects, projectColor } from "$lib/stores/projects.svelte";
  import { getMachines } from "$lib/stores/machines.svelte";
  import { currentView, navigate } from "$lib/stores/navigation.svelte";
  import { openSearch } from "$lib/stores/search.svelte";

  type Props = {
    onNewIssue?: () => void;
  };

  let { onNewIssue }: Props = $props();

  const projects = $derived(getProjects());
  const machines = $derived(getMachines());
  const view = $derived(currentView());
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    <button
      class="home-btn"
      class:active={view.kind === "home"}
      onclick={() => navigate({ kind: "home" })}
    >
      <span class="home-icon">⌂</span>
      <span class="home-label">ProjectOS</span>
    </button>
  </div>

  <div class="sidebar-scroll">
    <div class="sidebar-section">
      <div class="section-header">Projects</div>
      {#if projects.length === 0}
        <div class="empty-list">No projects yet</div>
      {/if}
      {#each projects as project, i (project.id)}
        {@const color = projectColor(i)}
        {@const active = view.kind === "project" && view.projectId === project.id}
        <button
          class="sidebar-item"
          class:active
          onclick={() => navigate({ kind: "project", projectId: project.id })}
        >
          <span class="project-dot" style:background={color}></span>
          <span class="item-name">{project.name}</span>
        </button>
      {/each}
    </div>

    {#if machines.length > 0}
      <div class="sidebar-section">
        <div class="section-header">Machines</div>
        {#each machines as machine (machine.id)}
          {@const active = view.kind === "machine" && view.machineId === machine.id}
          <button
            class="sidebar-item"
            class:active
            onclick={() => navigate({ kind: "machine", machineId: machine.id })}
          >
            <span class="machine-icon">⬡</span>
            <span class="item-name">{machine.name}</span>
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <div class="sidebar-footer">
    <button
      class="settings-btn"
      class:active={view.kind === "settings"}
      onclick={() => navigate({ kind: "settings" })}
    >
      <span class="settings-icon">⚙</span>
      <span>Settings</span>
    </button>
    <button class="kbd-hint" onclick={openSearch}>
      <kbd>⌘K</kbd> Search
    </button>
    <button class="kbd-hint" onclick={onNewIssue}>
      <kbd>⌘N</kbd> Capture
    </button>
    {#if view.kind === "project"}
      <div class="kbd-divider"></div>
      <div class="kbd-hint static">
        <kbd>J</kbd><kbd>K</kbd> Navigate
      </div>
      <div class="kbd-hint static">
        <kbd>↵</kbd> Open · <kbd>X</kbd> Toggle
      </div>
    {/if}
  </div>
</aside>

<style>
  .sidebar {
    width: 220px;
    flex-shrink: 0;
    background: #0f0f0c;
    border-right: 1px solid rgba(255, 255, 255, 0.06);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .sidebar-header {
    padding: 16px 12px 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    flex-shrink: 0;
  }

  .home-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 7px 10px;
    border-radius: 7px;
    background: none;
    border: none;
    cursor: pointer;
    transition: background 0.12s;
  }

  .home-btn.active,
  .home-btn:hover {
    background: rgba(255, 255, 255, 0.06);
  }

  .home-icon {
    font-size: 16px;
    color: #7a7a6a;
    line-height: 1;
  }

  .home-label {
    font-size: 14px;
    font-weight: 800;
    color: #c8c8b0;
    letter-spacing: -0.01em;
  }

  .sidebar-scroll {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
  }

  .sidebar-section {
    padding: 12px 12px 4px;
  }

  .section-header {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: #3a3a2a;
    padding: 0 6px 6px;
  }

  .sidebar-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 10px;
    border-radius: 6px;
    background: none;
    border: none;
    cursor: pointer;
    transition: background 0.1s;
    margin-bottom: 1px;
  }

  .sidebar-item:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .sidebar-item.active {
    background: rgba(255, 255, 255, 0.08);
  }

  .project-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .machine-icon {
    font-size: 13px;
    color: #5a7a5a;
    width: 8px;
    flex-shrink: 0;
    text-align: center;
  }

  .item-name {
    font-size: 13px;
    font-weight: 500;
    color: #a0a090;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    text-align: left;
  }

  .sidebar-item.active .item-name {
    color: #d8d8c8;
    font-weight: 600;
  }

  .empty-list {
    font-size: 12px;
    color: #3a3a2a;
    padding: 4px 10px 8px;
    font-style: italic;
  }

  .sidebar-footer {
    margin-top: auto;
    padding: 12px 14px;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .kbd-hint {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 11px;
    color: #3a3a2a;
    background: none;
    border: none;
    cursor: pointer;
    padding: 2px 0;
    text-align: left;
  }

  .kbd-hint:hover { color: #5a5a4a; }

  .settings-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 0;
    background: none;
    border: none;
    cursor: pointer;
    font-size: 12px;
    color: #3a3a2a;
    font-family: "Inter", sans-serif;
    margin-bottom: 6px;
    transition: color 0.12s;
  }
  .settings-btn:hover { color: #6a6a5a; }
  .settings-btn.active { color: #c0b89a; }
  .settings-icon { font-size: 13px; }

  .kbd-hint.static {
    pointer-events: none;
    cursor: default;
  }

  .kbd-divider {
    height: 1px;
    background: rgba(255, 255, 255, 0.04);
    margin: 4px 0;
  }

  kbd {
    font-size: 10px;
    background: rgba(255, 255, 255, 0.06);
    border-radius: 4px;
    padding: 1px 5px;
    font-family: monospace;
    color: #5a5a4a;
  }
</style>
