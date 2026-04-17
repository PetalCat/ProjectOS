<script lang="ts">
  import type { Issue, Label } from "$lib/types";
  import IssueRow from "./IssueRow.svelte";
  import { getSelectedIndex, setSelectedIndex } from "$lib/stores/issues.svelte";
  import { reorderIssue, updateIssue } from "$lib/commands";

  function effectiveStatus(i: Issue): string {
    if (i.pinned) return "pinned";
    return i.status ?? "ready";
  }

  type Props = {
    issues: Issue[];
    labelMap?: Record<string, Label[]>;
    accent?: string;
    showClosed?: boolean;
  };

  let { issues, labelMap = {}, accent = "#6a6a5a", showClosed = false }: Props = $props();

  // Flat ordered list for keyboard nav index mapping
  const flatIssues = $derived.by(() => {
    const open = issues.filter((i) => i.state === "open");
    const closed = issues.filter((i) => i.state === "closed");

    if (showClosed) return closed;

    return [
      ...open.filter((i) => i.pinned),
      ...open.filter((i) => !i.pinned && i.status === "next"),
      ...open.filter((i) => !i.pinned && (i.status === "ready" || i.status === null)),
      ...open.filter((i) => !i.pinned && i.status === "blocked"),
      ...open.filter((i) => !i.pinned && i.status === "idea"),
    ];
  });

  // Group issues by status
  const groups = $derived.by(() => {
    const open = issues.filter((i) => i.state === "open");
    const closed = issues.filter((i) => i.state === "closed");

    if (showClosed) {
      return [{ key: "closed", label: "Closed", icon: "✓", issues: closed }];
    }

    return [
      { key: "pinned", label: "Pinned", icon: "📌", issues: open.filter((i) => i.pinned) },
      { key: "next", label: "Next", icon: "▶", issues: open.filter((i) => !i.pinned && i.status === "next") },
      { key: "ready", label: "Ready", icon: "", issues: open.filter((i) => !i.pinned && (i.status === "ready" || i.status === null)) },
      { key: "blocked", label: "Blocked", icon: "⚠", issues: open.filter((i) => !i.pinned && i.status === "blocked") },
      { key: "ideas", label: "Ideas", icon: "💡", issues: open.filter((i) => !i.pinned && i.status === "idea") },
    ].filter((g) => g.issues.length > 0);
  });

  const selectedIndex = $derived(getSelectedIndex());

  // Drag state
  let draggedId = $state<string | null>(null);
  let dragOverId = $state<string | null>(null);

  function handleDragStart(issue: Issue) {
    draggedId = issue.id;
  }

  function handleDragOver(e: DragEvent, issue: Issue) {
    e.preventDefault();
    dragOverId = issue.id;
  }

  function handleDragLeave() {
    dragOverId = null;
  }

  async function handleDrop(e: DragEvent, targetIssue: Issue) {
    e.preventDefault();
    const dragId = draggedId;
    if (!dragId || dragId === targetIssue.id) {
      draggedId = null;
      dragOverId = null;
      return;
    }

    const dragged = issues.find((i) => i.id === dragId);
    if (!dragged) {
      draggedId = null;
      dragOverId = null;
      return;
    }

    const draggedStatus = effectiveStatus(dragged);
    const targetStatus = effectiveStatus(targetIssue);
    const crossGroup = draggedStatus !== targetStatus;

    // If crossing status groups (and neither is "pinned"), compute sort_order
    // relative to the target group so the item lands near the drop target.
    let neighbors: Issue[];
    if (crossGroup && targetStatus !== "pinned" && draggedStatus !== "pinned") {
      neighbors = flatIssues.filter((i) => effectiveStatus(i) === targetStatus);
    } else {
      neighbors = flatIssues;
    }

    const targetIdx = neighbors.findIndex((i) => i.id === targetIssue.id);
    const prev = targetIdx > 0 ? neighbors[targetIdx - 1] : null;
    const next = targetIdx < neighbors.length - 1 ? neighbors[targetIdx + 1] : null;

    let newSortOrder: number;
    if (prev && prev.id !== dragId && next && next.id !== dragId) {
      newSortOrder = ((prev.sort_order ?? 0) + (next.sort_order ?? 0)) / 2;
    } else if (prev && prev.id !== dragId) {
      newSortOrder = (prev.sort_order ?? 0) + 1;
    } else if (next && next.id !== dragId) {
      newSortOrder = (next.sort_order ?? 1) - 0.5;
    } else {
      newSortOrder = targetIssue.sort_order ?? 1;
    }

    // Optimistic UI update
    dragged.sort_order = newSortOrder;
    if (crossGroup && targetStatus !== "pinned" && draggedStatus !== "pinned") {
      dragged.status = targetStatus as Issue["status"];
    }

    draggedId = null;
    dragOverId = null;

    if (crossGroup && targetStatus !== "pinned" && draggedStatus !== "pinned") {
      await updateIssue({ id: dragId, status: targetStatus });
    }
    await reorderIssue(dragId, newSortOrder);
  }

  function handleDragEnd() {
    draggedId = null;
    dragOverId = null;
  }
</script>

<div class="issue-list">
  {#if groups.length === 0}
    <div class="empty">No issues yet.</div>
  {/if}
  {#each groups as group (group.key)}
    <div class="group">
      <div class="group-header">
        {#if group.icon}
          <span class="group-icon">{group.icon}</span>
        {/if}
        <span class="group-label">{group.label}</span>
        <span class="group-count">{group.issues.length}</span>
      </div>
      {#each group.issues as issue (issue.id)}
        {@const flatIdx = flatIssues.findIndex((i) => i.id === issue.id)}
        <IssueRow
          {issue}
          labels={labelMap[issue.id] ?? []}
          {accent}
          selected={selectedIndex === flatIdx}
          dragging={draggedId === issue.id}
          dragOver={dragOverId === issue.id}
          onSelect={() => setSelectedIndex(flatIdx)}
          onDragStart={() => handleDragStart(issue)}
          onDragOver={(e) => handleDragOver(e, issue)}
          onDragLeave={handleDragLeave}
          onDrop={(e) => handleDrop(e, issue)}
          onDragEnd={handleDragEnd}
        />
      {/each}
    </div>
  {/each}
</div>

<style>
  .issue-list {
    flex: 1;
    overflow-y: auto;
  }

  .group {
    margin-bottom: 4px;
  }

  .group-header {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 16px 6px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: #5a5a4a;
    background: rgba(0, 0, 0, 0.2);
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
    position: sticky;
    top: 0;
    z-index: 1;
  }

  .group-icon {
    font-size: 12px;
  }

  .group-label {
    flex: 1;
  }

  .group-count {
    background: rgba(255, 255, 255, 0.06);
    padding: 1px 6px;
    border-radius: 8px;
    font-size: 10px;
  }

  .empty {
    padding: 48px 24px;
    text-align: center;
    color: #4a4a3a;
    font-size: 14px;
  }
</style>
