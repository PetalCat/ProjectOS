<script lang="ts">
  import { createIssue } from "$lib/commands";

  type Props = {
    open?: boolean;
    onClose?: () => void;
    onCreated?: () => void;
  };

  let { open = false, onClose, onCreated }: Props = $props();

  let title = $state("");
  let submitting = $state(false);
  let inputEl = $state<HTMLInputElement | null>(null);

  $effect(() => {
    if (open && inputEl) {
      setTimeout(() => inputEl?.focus(), 30);
    }
    if (!open) {
      title = "";
    }
  });

  async function handleSubmit() {
    if (!title.trim() || submitting) return;
    submitting = true;
    try {
      await createIssue({ title: title.trim(), status: "idea" });
      title = "";
      onCreated?.();
      onClose?.();
    } finally {
      submitting = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") handleSubmit();
    if (e.key === "Escape") onClose?.();
  }
</script>

{#if open}
  <div
    class="capture-overlay"
    onclick={onClose}
    onkeydown={(e) => e.key === "Escape" && onClose?.()}
    role="dialog"
    aria-modal="true"
    aria-label="Quick Capture"
    tabindex="-1"
  >
    <div
      class="capture-panel"
      onclick={(e) => e.stopPropagation()}
      onkeydown={handleKeydown}
      role="none"
    >
      <div class="capture-label">
        <span class="capture-icon">💡</span>
        Quick Capture
      </div>
      <input
        bind:this={inputEl}
        class="capture-input"
        type="text"
        placeholder="What's on your mind? Press ↵ to capture."
        bind:value={title}
        onkeydown={handleKeydown}
      />
      <div class="capture-footer">
        <span class="capture-hint">Creates an Idea · no project</span>
        <button
          class="capture-submit"
          onclick={handleSubmit}
          disabled={submitting || !title.trim()}
        >
          {submitting ? "Saving…" : "Capture ↵"}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .capture-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.65);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    backdrop-filter: blur(3px);
  }

  .capture-panel {
    background: #1a1a16;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    width: 520px;
    max-width: calc(100vw - 40px);
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.5);
    overflow: hidden;
  }

  .capture-label {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 12px 16px 10px;
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: #5a5a4a;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .capture-icon { font-size: 14px; }

  .capture-input {
    display: block;
    width: 100%;
    background: none;
    border: none;
    outline: none;
    padding: 16px 18px;
    font-size: 18px;
    font-weight: 500;
    color: #e0e0d0;
    font-family: inherit;
    caret-color: #b8e060;
    box-sizing: border-box;
  }

  .capture-input::placeholder { color: #3a3a2a; }

  .capture-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
  }

  .capture-hint {
    font-size: 11px;
    color: #4a4a3a;
  }

  .capture-submit {
    font-size: 12px;
    font-weight: 700;
    color: #0a0a0a;
    background: #b8e060;
    border: none;
    border-radius: 7px;
    padding: 7px 16px;
    cursor: pointer;
    transition: opacity 0.12s;
  }

  .capture-submit:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
