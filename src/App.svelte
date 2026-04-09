<script lang="ts">
  import { onMount } from "svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import Dashboard from "$lib/components/Dashboard.svelte";
  import ProjectView from "$lib/components/ProjectView.svelte";
  import IssueDetail from "$lib/components/IssueDetail.svelte";
  import MachineView from "$lib/components/MachineView.svelte";
  import Settings from "$lib/components/Settings.svelte";
  import SearchModal from "$lib/components/SearchModal.svelte";
  import QuickCapture from "$lib/components/QuickCapture.svelte";
  import { loadProjects, getProjects } from "$lib/stores/projects.svelte";
  import { loadMachines, getMachines } from "$lib/stores/machines.svelte";
  import { currentView, navigate } from "$lib/stores/navigation.svelte";
  import { openSearch, isOpen as isSearchOpen } from "$lib/stores/search.svelte";
  import { onProjectsChanged, onIssuesChanged } from "$lib/events";
  import { moveSelection, getSelectedIssue, getIssues } from "$lib/stores/issues.svelte";
  import { closeIssue, reopenIssue } from "$lib/commands";
  import { loadIssues } from "$lib/stores/issues.svelte";

  const view = $derived(currentView());
  const projects = $derived(getProjects());
  const machines = $derived(getMachines());

  let captureOpen = $state(false);

  // Find current project for project view
  const currentProject = $derived(
    view.kind === "project"
      ? projects.find((p) => p.id === (view as { kind: "project"; projectId: string }).projectId) ?? null
      : null
  );

  // Find current machine for machine view
  const currentMachine = $derived(
    view.kind === "machine"
      ? machines.find((m) => m.id === (view as { kind: "machine"; machineId: string }).machineId) ?? null
      : null
  );

  onMount(async () => {
    await Promise.all([loadProjects(), loadMachines()]);

    // Listen for Rust events and refresh
    const unProjects = await onProjectsChanged(() => loadProjects());
    const unIssues = await onIssuesChanged(() => {
      // issues store refreshes itself via $effect in ProjectView
    });

    return () => {
      unProjects();
      unIssues();
    };
  });

  // Global keyboard shortcuts
  function handleGlobalKey(e: KeyboardEvent) {
    // Cmd+K → search
    if (e.metaKey && e.key === "k") {
      e.preventDefault();
      openSearch();
      return;
    }
    // Cmd+N → quick capture
    if (e.metaKey && e.key === "n") {
      e.preventDefault();
      captureOpen = true;
      return;
    }
    // Escape → close overlays or go home
    if (e.key === "Escape") {
      if (captureOpen) { captureOpen = false; return; }
    }

    // Issue list navigation — only when in project view and no input focused
    const tag = (e.target as HTMLElement)?.tagName?.toLowerCase();
    const isInput = tag === "input" || tag === "textarea" || (e.target as HTMLElement)?.isContentEditable;
    if (isInput) return;

    if (view.kind === "project") {
      if (e.key === "j" || e.key === "J") {
        e.preventDefault();
        moveSelection(1);
        return;
      }
      if (e.key === "k" || e.key === "K") {
        e.preventDefault();
        moveSelection(-1);
        return;
      }
      if (e.key === "Enter") {
        const issue = getSelectedIssue();
        if (issue) {
          e.preventDefault();
          navigate({ kind: "issue", issueId: issue.id });
        }
        return;
      }
      if (e.key === "x" || e.key === "X") {
        const issue = getSelectedIssue();
        if (issue) {
          e.preventDefault();
          const pid = (view as { kind: "project"; projectId: string }).projectId;
          if (issue.state === "open") {
            closeIssue(issue.id).then(() => loadIssues(pid, false));
          } else {
            reopenIssue(issue.id).then(() => loadIssues(pid, false));
          }
        }
        return;
      }
    }
  }
</script>

<svelte:window onkeydown={handleGlobalKey} />

<div class="app">
  <Sidebar onNewIssue={() => { captureOpen = true; }} />

  <main class="content">
    {#if view.kind === "home"}
      <Dashboard />
    {:else if view.kind === "project"}
      {#if currentProject}
        {#key currentProject.id}
          <ProjectView project={currentProject} />
        {/key}
      {:else}
        <div class="not-found">Project not found.</div>
      {/if}
    {:else if view.kind === "issue"}
      <IssueDetail issueId={(view as { kind: "issue"; issueId: string }).issueId} />
    {:else if view.kind === "machine"}
      {#if currentMachine}
        <MachineView machine={currentMachine} />
      {:else}
        <div class="not-found">Machine not found.</div>
      {/if}
    {:else if view.kind === "settings"}
      <div class="settings-container">
        <Settings />
      </div>
    {/if}
  </main>
</div>

<SearchModal />
<QuickCapture
  open={captureOpen}
  onClose={() => { captureOpen = false; }}
  onCreated={() => { captureOpen = false; }}
/>

<style>
  :global(*, *::before, *::after) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  :global(html, body) {
    height: 100%;
    background: #0a0a0a;
    color: #d8d8c8;
    font-family: "Inter", system-ui, -apple-system, sans-serif;
    font-size: 14px;
    -webkit-font-smoothing: antialiased;
    overflow: hidden;
  }

  :global(#app) {
    height: 100%;
    display: flex;
  }

  :global(button) {
    font-family: inherit;
  }

  :global(::-webkit-scrollbar) {
    width: 6px;
  }

  :global(::-webkit-scrollbar-track) {
    background: transparent;
  }

  :global(::-webkit-scrollbar-thumb) {
    background: rgba(255, 255, 255, 0.08);
    border-radius: 3px;
  }

  :global(::-webkit-scrollbar-thumb:hover) {
    background: rgba(255, 255, 255, 0.14);
  }

  /* Focus ring for keyboard nav */
  :global(:focus-visible) {
    outline: 2px solid rgba(184, 224, 96, 0.5);
    outline-offset: 2px;
  }

  /* Remove default focus for mouse users */
  :global(:focus:not(:focus-visible)) {
    outline: none;
  }

  /* Smooth transitions on interactive elements */
  :global(button, a, [role="button"]) {
    transition-property: background, color, border-color, opacity, box-shadow;
    transition-duration: 0.12s;
    transition-timing-function: ease;
  }

  .app {
    display: flex;
    height: 100vh;
    width: 100vw;
    background: #0a0a0a;
    overflow: hidden;
  }

  .content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: #111110;
  }

  .not-found {
    padding: 48px 32px;
    color: #4a4a3a;
    font-size: 14px;
  }

  .settings-container {
    flex: 1;
    overflow-y: auto;
    padding: 36px 48px;
  }
</style>
