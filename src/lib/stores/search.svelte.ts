import type { SearchResults } from "$lib/types";
import { searchIssues } from "$lib/commands";

let open = $state(false);
let query = $state("");
let results = $state<SearchResults | null>(null);
let loading = $state(false);

export function openSearch() {
  open = true;
  query = "";
  results = null;
}

export function closeSearch() {
  open = false;
  query = "";
  results = null;
}

export function isOpen() {
  return open;
}

export function getQuery() {
  return query;
}

export function getResults() {
  return results;
}

export function isLoading() {
  return loading;
}

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

export function setQuery(q: string) {
  query = q;
  if (debounceTimer) clearTimeout(debounceTimer);
  if (!q.trim()) {
    results = null;
    return;
  }
  debounceTimer = setTimeout(async () => {
    loading = true;
    try {
      results = await searchIssues(q);
    } catch {
      results = null;
    } finally {
      loading = false;
    }
  }, 250);
}
