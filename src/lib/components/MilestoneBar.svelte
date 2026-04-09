<script lang="ts">
  import type { Milestone } from "$lib/types";

  type Props = {
    milestone: Milestone;
  };

  let { milestone }: Props = $props();

  const total = $derived((milestone.open_count ?? 0) + (milestone.closed_count ?? 0));
  const pct = $derived(total > 0 ? Math.round(((milestone.closed_count ?? 0) / total) * 100) : 0);
</script>

<div class="milestone-bar">
  <div class="milestone-header">
    <span class="milestone-title">{milestone.title}</span>
    <span class="milestone-pct">{pct}%</span>
  </div>
  <div class="track">
    <div class="fill" style:width="{pct}%"></div>
  </div>
  <div class="milestone-detail">
    {milestone.closed_count ?? 0} of {total} issues closed
  </div>
</div>

<style>
  .milestone-bar {
    padding: 8px 12px;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.06);
  }

  .milestone-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 6px;
  }

  .milestone-title {
    font-size: 12px;
    font-weight: 700;
    color: #c8c8b0;
  }

  .milestone-pct {
    font-size: 11px;
    font-weight: 700;
    color: #b8e060;
  }

  .track {
    height: 4px;
    background: rgba(255, 255, 255, 0.08);
    border-radius: 2px;
    overflow: hidden;
    margin-bottom: 4px;
  }

  .fill {
    height: 100%;
    background: #b8e060;
    border-radius: 2px;
    transition: width 0.3s ease;
  }

  .milestone-detail {
    font-size: 10px;
    color: #5a5a4a;
  }
</style>
