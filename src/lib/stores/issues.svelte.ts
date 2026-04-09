import type { Issue } from "$lib/types";
import { listIssues } from "$lib/commands";

let issues = $state<Issue[]>([]);
let currentProjectId = $state<string | null>(null);
let showClosed = $state(false);
let loading = $state(false);
let error = $state<string | null>(null);
let selectedIndex = $state<number>(-1);

export async function loadIssues(projectId: string | null, includeClosed = false) {
  loading = true;
  error = null;
  currentProjectId = projectId;
  showClosed = includeClosed;
  selectedIndex = -1;
  try {
    issues = await listIssues(projectId, includeClosed);
  } catch (e) {
    console.error(`[issues store] error loading issues:`, e);
    error = String(e);
  } finally {
    loading = false;
  }
}

export function getIssues() {
  return issues;
}

export function getSelectedIndex() {
  return selectedIndex;
}

export function setSelectedIndex(idx: number) {
  selectedIndex = idx;
}

export function moveSelection(delta: number) {
  const len = issues.length;
  if (len === 0) return;
  if (selectedIndex < 0) {
    selectedIndex = delta > 0 ? 0 : len - 1;
  } else {
    selectedIndex = Math.max(0, Math.min(len - 1, selectedIndex + delta));
  }
}

export function getSelectedIssue(): Issue | null {
  if (selectedIndex < 0 || selectedIndex >= issues.length) return null;
  return issues[selectedIndex];
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
