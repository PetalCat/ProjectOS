<script lang="ts">
  import type { ActivityEntry } from "$lib/types";

  type Props = {
    entries: ActivityEntry[];
  };

  let { entries }: Props = $props();

  function dayLabel(ms: number): string {
    const d = new Date(ms);
    const today = new Date();
    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);

    if (d.toDateString() === today.toDateString()) return "Today";
    if (d.toDateString() === yesterday.toDateString()) return "Yesterday";
    return d.toLocaleDateString("en-US", { month: "short", day: "numeric" });
  }

  function formatTime(ms: number): string {
    return new Date(ms).toLocaleTimeString("en-US", {
      hour: "numeric",
      minute: "2-digit",
    });
  }

  function actionIcon(action: string): string {
    switch (action) {
      case "created": return "+";
      case "closed": return "✓";
      case "reopened": return "↩";
      case "updated": return "✎";
      case "moved": return "→";
      case "labeled": return "◈";
      case "commented": return "◉";
      default: return "·";
    }
  }

  function actionColor(action: string): string {
    switch (action) {
      case "created": return "#60b8e0";
      case "closed": return "#b8e060";
      case "reopened": return "#e8a040";
      default: return "#6a6a5a";
    }
  }

  // Group by day
  const grouped = $derived.by(() => {
    const map = new Map<string, ActivityEntry[]>();
    for (const e of entries) {
      const key = dayLabel(e.created_at);
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(e);
    }
    return [...map.entries()];
  });
</script>

<div class="activity-feed">
  {#if grouped.length === 0}
    <div class="empty">No recent activity.</div>
  {/if}
  {#each grouped as [day, dayEntries] (day)}
    <div class="day-group">
      <div class="day-label">{day}</div>
      {#each dayEntries as entry (entry.id)}
        <div class="entry">
          <span class="entry-icon" style:color={actionColor(entry.action)}>
            {actionIcon(entry.action)}
          </span>
          <div class="entry-body">
            <span class="entry-action">{entry.action}</span>
            {#if entry.detail}
              <span class="entry-detail"> — {entry.detail}</span>
            {/if}
          </div>
          <span class="entry-time">{formatTime(entry.created_at)}</span>
        </div>
      {/each}
    </div>
  {/each}
</div>

<style>
  .activity-feed {
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .day-group {
    margin-bottom: 16px;
  }

  .day-label {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: #4a4a3a;
    padding: 8px 0 6px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    margin-bottom: 4px;
  }

  .entry {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 6px 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.03);
  }

  .entry-icon {
    font-size: 12px;
    font-weight: 700;
    width: 16px;
    text-align: center;
    flex-shrink: 0;
    margin-top: 1px;
  }

  .entry-body {
    flex: 1;
    font-size: 12px;
    color: #8a8a7a;
    line-height: 1.4;
  }

  .entry-action {
    font-weight: 600;
    color: #a0a090;
  }

  .entry-detail {
    color: #6a6a5a;
  }

  .entry-time {
    font-size: 10px;
    color: #4a4a3a;
    flex-shrink: 0;
  }

  .empty {
    color: #4a4a3a;
    font-size: 13px;
    padding: 20px 0;
  }
</style>
