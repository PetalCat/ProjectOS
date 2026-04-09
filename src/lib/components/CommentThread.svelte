<script lang="ts">
  import type { Comment } from "$lib/types";
  import { createComment, listComments } from "$lib/commands";

  type Props = {
    issueId: string;
    locked?: boolean;
  };

  let { issueId, locked = false }: Props = $props();

  let comments = $state<Comment[]>([]);
  let newBody = $state("");
  let submitting = $state(false);

  $effect(() => {
    const id = issueId;
    listComments(id)
      .then((c) => { comments = c; })
      .catch(() => {});
  });

  async function handleSubmit() {
    if (!newBody.trim() || submitting || locked) return;
    submitting = true;
    try {
      const comment = await createComment(issueId, newBody.trim());
      comments = [...comments, comment];
      newBody = "";
    } finally {
      submitting = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
      handleSubmit();
    }
  }

  function formatTime(ms: number): string {
    return new Date(ms).toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
      hour: "numeric",
      minute: "2-digit",
    });
  }
</script>

<div class="comment-thread">
  <div class="thread-label">
    {comments.length} comment{comments.length !== 1 ? "s" : ""}
  </div>

  {#each comments as comment (comment.id)}
    <div class="comment">
      <div class="comment-header">
        <span class="comment-author">comment</span>
        <span class="comment-time">{formatTime(comment.created_at)}</span>
      </div>
      <div class="comment-body">{comment.body}</div>
    </div>
  {/each}

  {#if !locked}
    <div class="comment-input-area">
      <textarea
        class="comment-input"
        placeholder="Leave a comment… (⌘↵ to submit)"
        bind:value={newBody}
        onkeydown={handleKeydown}
        rows="3"
      ></textarea>
      <div class="comment-actions">
        <button class="submit-btn" onclick={handleSubmit} disabled={submitting || !newBody.trim()}>
          {submitting ? "Sending…" : "Comment"}
        </button>
      </div>
    </div>
  {:else}
    <div class="locked-notice">This issue is locked. No new comments.</div>
  {/if}
</div>

<style>
  .comment-thread {
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .thread-label {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: #4a4a3a;
    padding-bottom: 10px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    margin-bottom: 8px;
  }

  .comment {
    padding: 12px 0 12px 14px;
    border-left: 2px solid rgba(255, 255, 255, 0.07);
    margin-bottom: 8px;
  }

  .comment-header {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 6px;
  }

  .comment-author {
    font-size: 12px;
    font-weight: 700;
    color: #8a8a7a;
  }

  .comment-time {
    font-size: 11px;
    color: #4a4a3a;
  }

  .comment-body {
    font-size: 13px;
    color: #c0c0b0;
    line-height: 1.55;
    white-space: pre-wrap;
  }

  .comment-input-area {
    margin-top: 12px;
  }

  .comment-input {
    width: 100%;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 10px 12px;
    font-size: 13px;
    color: #d0d0c0;
    font-family: inherit;
    resize: vertical;
    outline: none;
    box-sizing: border-box;
  }

  .comment-input:focus {
    border-color: rgba(255, 255, 255, 0.2);
  }

  .comment-actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 8px;
  }

  .submit-btn {
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

  .submit-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .locked-notice {
    font-size: 12px;
    color: #4a4a3a;
    font-style: italic;
    padding: 12px 0;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    margin-top: 8px;
  }
</style>
