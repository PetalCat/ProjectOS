<script lang="ts">
  type Props = {
    status: "next" | "ready" | "blocked" | "idea" | null;
    state?: "open" | "closed";
  };

  let { status, state = "open" }: Props = $props();

  const label = $derived(() => {
    if (state === "closed") return "Closed";
    switch (status) {
      case "next": return "Next";
      case "ready": return "Ready";
      case "blocked": return "Blocked";
      case "idea": return "Idea";
      default: return "Open";
    }
  });

  const cls = $derived(() => {
    if (state === "closed") return "badge closed";
    switch (status) {
      case "next": return "badge next";
      case "blocked": return "badge blocked";
      case "idea": return "badge idea";
      default: return "badge ready";
    }
  });
</script>

<span class={cls()}>{label()}</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .next {
    background: rgba(184, 224, 96, 0.15);
    color: #b8e060;
    border: 1px solid rgba(184, 224, 96, 0.3);
  }

  .ready {
    background: rgba(140, 140, 120, 0.12);
    color: #8a8a7a;
    border: 1px solid rgba(140, 140, 120, 0.25);
  }

  .blocked {
    background: rgba(232, 160, 64, 0.15);
    color: #e8a040;
    border: 1px solid rgba(232, 160, 64, 0.3);
  }

  .idea {
    background: rgba(96, 96, 80, 0.12);
    color: #6a6a5a;
    border: 1px solid rgba(100, 100, 80, 0.2);
  }

  .closed {
    background: rgba(100, 100, 90, 0.12);
    color: #6a6a5a;
    border: 1px solid rgba(100, 100, 90, 0.2);
  }
</style>
