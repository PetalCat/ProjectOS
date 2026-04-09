import type { Issue } from "$lib/types";
import { listIssues } from "$lib/commands";

let issues = $state<Issue[]>([]);
let currentProjectId = $state<string | null>(null);
let showClosed = $state(false);
let loading = $state(false);
let error = $state<string | null>(null);

export async function loadIssues(projectId: string | null, includeClosed = false) {
  loading = true;
  error = null;
  currentProjectId = projectId;
  showClosed = includeClosed;
  try {
    issues = await listIssues(projectId, includeClosed);
  } catch (e) {
    error = String(e);
  } finally {
    loading = false;
  }
}

export function getIssues() {
  return issues;
}

export function getCurrentProjectId() {
  return currentProjectId;
}

export function getShowClosed() {
  return showClosed;
}

export function isLoading() {
  return loading;
}

export function getError() {
  return error;
}

// Group issues by status for display
export function groupedIssues() {
  const open = issues.filter((i) => i.state === "open");
  const closed = issues.filter((i) => i.state === "closed");

  if (showClosed) {
    return { closed };
  }

  return {
    pinned: open.filter((i) => i.pinned),
    next: open.filter((i) => !i.pinned && i.status === "next"),
    ready: open.filter((i) => !i.pinned && (i.status === "ready" || i.status === null)),
    blocked: open.filter((i) => !i.pinned && i.status === "blocked"),
    ideas: open.filter((i) => !i.pinned && i.status === "idea"),
  };
}
