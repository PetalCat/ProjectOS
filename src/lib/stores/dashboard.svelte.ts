import type { Dashboard } from "$lib/types";
import { getDashboard } from "$lib/commands";

let dashboard = $state<Dashboard | null>(null);
let loading = $state(false);
let error = $state<string | null>(null);

export async function loadDashboard() {
  loading = true;
  error = null;
  try {
    dashboard = await getDashboard();
  } catch (e) {
    error = String(e);
  } finally {
    loading = false;
  }
}

export function getDashboardData() {
  return dashboard;
}

export function isLoading() {
  return loading;
}

export function getError() {
  return error;
}
