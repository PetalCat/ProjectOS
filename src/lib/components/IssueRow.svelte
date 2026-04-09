<script lang="ts">
  import type { Issue, Label } from "$lib/types";
  import StatusBadge from "./StatusBadge.svelte";
  import LabelBadge from "./LabelBadge.svelte";
  import { navigate } from "$lib/stores/navigation.svelte";

  type Props = {
    issue: Issue;
    labels?: Label[];
    accent?: string;
  };

  let { issue, labels = [], accent = "#6a6a5a" }: Props = $props();

  function formatRelativeTime(ms: number): string {
    const diff = Date.now() - ms;
    const mins = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);
    if (mins < 60) return `${mins}m ago`;
    if (hours < 24) return `${hours}h ago`;
    if (days < 30) return `${days}d ago`;
    return new Date(ms).toLocaleDateString();
  }

  const circleClass = $derived(() => {
    if (issue.pinned) return "circle pinned";
    switch (issue.status) {
      case "next": return "circle next";
      case "blocked": return "circle blocked";
      case "idea": return "circle idea";
      default: return "circle ready";
    }
  });

  const circleIcon = $derived(() => {
    if (issue.pinned) return "📌";
    switch (issue.status) {
      case "next": return "▶";
      case "blocked": return "!";
      case "idea": return "·";
      default: return "";
    }
  });

  function handleClick() {
    navigate({ kind: "issue", issueId: issue.id });
  }
</script>

<div
  class="issue-row"
  onclick={handleClick}
  onkeydown={(e) => e.key === "Enter" && handleClick()}
  role="button"
  tabindex="0"
>
  <span class={circleClass()} title={issue.status ?? ""}>
    {circleIcon()}
  </span>

  <div class="issue-content">
    <div class="issue-title">{issue.title}</div>
    <div class="issue-subtitle">
      {#if issue.number}
        <span class="issue-number">#{issue.number}</span>
      {/if}
      <span class="issue-time">opened {formatRelativeTime(issue.created_at)}</span>
      {#if issue.machine_id}
        <span class="issue-machine">· machine</span>
      {/if}
    </div>
  </div>

  <div class="issue-meta">
    {#each labels as lbl (lbl.id)}
      <LabelBadge label={lbl} />
    {/each}
    <StatusBadge status={issue.status} state={issue.state} />
    {#if issue.number}
      <span class="issue-num-badge">#{issue.number}</span>
    {/if}
  </div>
</div>

<style>
  .issue-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    cursor: pointer;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
    transition: background 0.12s ease;
  }

  .issue-row:hover {
    background: rgba(255, 255, 255, 0.04);
  }

  .circle {
    width: 20px;
    height: 20px;
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
    color: transparent;
  }

  .circle.blocked {
    border: 2px solid #e8a040;
    background: transparent;
    color: #e8a040;
    font-size: 11px;
    font-weight: 900;
  }

  .circle.idea {
    border: 2px dashed #4a4a3a;
    background: transparent;
    color: #4a4a3a;
    font-size: 18px;
    line-height: 1;
    padding-bottom: 2px;
  }

  .circle.pinned {
    background: transparent;
    font-size: 12px;
  }

  .issue-content {
    flex: 1;
    min-width: 0;
  }

  .issue-title {
    font-size: 13px;
    font-weight: 500;
    color: #d8d8c8;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .issue-subtitle {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 2px;
    font-size: 11px;
    color: #5a5a4a;
  }

  .issue-number {
    color: #6a6a5a;
    font-weight: 600;
  }

  .issue-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .issue-num-badge {
    font-size: 11px;
    color: #4a4a3a;
    font-weight: 600;
  }
</style>
