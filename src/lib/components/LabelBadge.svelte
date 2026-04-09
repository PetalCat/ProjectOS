<script lang="ts">
  import type { Label } from "$lib/types";

  type Props = {
    label: Label;
  };

  let { label }: Props = $props();

  function hexToRgb(hex: string) {
    const r = parseInt(hex.slice(1, 3), 16);
    const g = parseInt(hex.slice(3, 5), 16);
    const b = parseInt(hex.slice(5, 7), 16);
    return `${r}, ${g}, ${b}`;
  }
</script>

<span
  class="label-badge"
  style:--color={label.color}
  style:--rgb={hexToRgb(label.color || "#888888")}
>
  {label.name}
</span>

<style>
  .label-badge {
    display: inline-flex;
    align-items: center;
    padding: 2px 7px;
    border-radius: 10px;
    font-size: 11px;
    font-weight: 600;
    color: var(--color, #888);
    background: rgba(var(--rgb, 136, 136, 136), 0.12);
    border: 1px solid rgba(var(--rgb, 136, 136, 136), 0.28);
    white-space: nowrap;
  }
</style>
