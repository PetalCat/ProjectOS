import type { Project } from "$lib/types";
import { listProjects } from "$lib/commands";

let projects = $state<Project[]>([]);
let loading = $state(false);
let error = $state<string | null>(null);

export async function loadProjects() {
  loading = true;
  error = null;
  try {
    projects = await listProjects();
  } catch (e) {
    error = String(e);
  } finally {
    loading = false;
  }
}

export function getProjects() {
  return projects;
}

export function isLoading() {
  return loading;
}

export function getError() {
  return error;
}

// Project accent colors — cycle through palette
const PROJECT_COLORS = [
  "#e8a040", // amber
  "#60b8e0", // teal
  "#b8e060", // lime
  "#e06080", // rose
  "#a060e0", // purple
  "#e0a060", // orange
  "#60e0a0", // mint
  "#e060c0", // pink
];

export function projectColor(index: number): string {
  return PROJECT_COLORS[index % PROJECT_COLORS.length];
}
