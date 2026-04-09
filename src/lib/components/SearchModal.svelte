<script lang="ts">
  import { isOpen, closeSearch, getQuery, setQuery, getResults, isLoading } from "$lib/stores/search.svelte";
  import { navigate } from "$lib/stores/navigation.svelte";

  const open = $derived(isOpen());
  const query = $derived(getQuery());
  const results = $derived(getResults());
  const loading = $derived(isLoading());

  let inputEl = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (open && inputEl) {
      setTimeout(() => inputEl?.focus(), 30);
    }
  });

  function handleKey(e: KeyboardEvent) {
    if (e.key === "Escape") closeSearch();
  }

  function goToIssue(issueId: string) {
    closeSearch();
    navigate({ kind: "issue", issueId });
  }

  function goToProject(projectId: string) {
    closeSearch();
    navigate({ kind: "project", projectId });
  }

  function goToMachine(machineId: string) {
    closeSearch();
    navigate({ kind: "machine", machineId });
  }

  const hasResults = $derived(
    results && (results.issues.length > 0 || results.projects.length > 0 || results.machines.length > 0)
  );

  const showEmpty = $derived(query.trim().length > 0 && !loading && results && !hasResults);
</script>

{#if open}
  <div
    class="modal-overlay"
    onclick={closeSearch}
    onkeydown={handleKey}
    role="dialog"
    aria-modal="true"
    aria-label="Search"
    tabindex="-1"
  >
    <div
      class="modal-panel"
      onclick={(e) => e.stopPropagation()}
      onkeydown={handleKey}
      role="none"
    >
      <div class="search-bar">
        <span class="search-icon">⌕</span>
        <input
          bind:this={inputEl}
          class="search-input"
          type="text"
          placeholder="Search issues, projects, machines…"
          value={query}
          oninput={(e) => setQuery((e.target as HTMLInputElement).value)}
          onkeydown={handleKey}
        />
        {#if loading}
          <span class="search-spinner">…</span>
        {/if}
      </div>

      <div class="results">
        {#if !query.trim()}
          <div class="hint-text">Start typing to search across everything.</div>
        {:else if showEmpty}
          <div class="hint-text">No results for "{query}"</div>
        {:else if results}
          {#if results.issues.length > 0}
            <div class="results-group">
              <div class="results-group-label">Issues</div>
              {#each results.issues as issue (issue.id)}
                <button class="result-row" onclick={() => goToIssue(issue.id)}>
                  <span class="result-icon">◎</span>
                  <span class="result-title">{issue.title}</span>
                  {#if issue.number}
                    <span class="result-meta">#{issue.number}</span>
                  {/if}
                </button>
              {/each}
            </div>
          {/if}

          {#if results.projects.length > 0}
            <div class="results-group">
              <div class="results-group-label">Projects</div>
              {#each results.projects as project (project.id)}
                <button class="result-row" onclick={() => goToProject(project.id)}>
                  <span class="result-icon">◈</span>
                  <span class="result-title">{project.name}</span>
                </button>
              {/each}
            </div>
          {/if}

          {#if results.machines.length > 0}
            <div class="results-group">
              <div class="results-group-label">Machines</div>
              {#each results.machines as machine (machine.id)}
                <button class="result-row" onclick={() => goToMachine(machine.id)}>
                  <span class="result-icon">⬡</span>
                  <span class="result-title">{machine.name}</span>
                  {#if machine.hostname}
                    <span class="result-meta">{machine.hostname}</span>
                  {/if}
                </button>
              {/each}
            </div>
          {/if}
        {/if}
      </div>

      <div class="modal-footer">
        <span class="kbd">Esc</span><span class="kbd-label"> to close</span>
        <span class="kbd-sep">·</span>
        <span class="kbd">↵</span><span class="kbd-label"> to open</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 80px;
    z-index: 100;
    backdrop-filter: blur(4px);
  }

  .modal-panel {
    background: #1a1a16;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    width: 580px;
    max-width: calc(100vw - 40px);
    max-height: 60vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: 0 24px 80px rgba(0, 0, 0, 0.6);
  }

  .search-bar {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.07);
  }

  .search-icon {
    font-size: 18px;
    color: #5a5a4a;
    line-height: 1;
  }

  .search-input {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    font-size: 16px;
    font-weight: 500;
    color: #e0e0d0;
    font-family: inherit;
    caret-color: #b8e060;
  }

  .search-input::placeholder { color: #4a4a3a; }

  .search-spinner {
    color: #5a5a4a;
    font-size: 14px;
    animation: pulse 1s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 0.4; }
    50% { opacity: 1; }
  }

  .results {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
  }

  .hint-text {
    padding: 20px 18px;
    font-size: 13px;
    color: #4a4a3a;
  }

  .results-group {
    margin-bottom: 4px;
  }

  .results-group-label {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    color: #4a4a3a;
    padding: 6px 18px 4px;
  }

  .result-row {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 8px 18px;
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    transition: background 0.1s;
  }

  .result-row:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .result-icon {
    font-size: 13px;
    color: #5a5a4a;
    width: 16px;
    flex-shrink: 0;
  }

  .result-title {
    flex: 1;
    font-size: 13px;
    color: #c8c8b8;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .result-meta {
    font-size: 11px;
    color: #5a5a4a;
    flex-shrink: 0;
  }

  .modal-footer {
    padding: 8px 16px;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: #4a4a3a;
  }

  .kbd {
    background: rgba(255, 255, 255, 0.07);
    border-radius: 4px;
    padding: 1px 5px;
    font-family: monospace;
    font-size: 10px;
    color: #6a6a5a;
  }

  .kbd-label, .kbd-sep { color: #3a3a2a; }
  .kbd-sep { margin: 0 4px; }
</style>
