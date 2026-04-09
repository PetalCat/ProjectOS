import type { Machine } from "$lib/types";
import { listMachines } from "$lib/commands";

let machines = $state<Machine[]>([]);
let loading = $state(false);
let error = $state<string | null>(null);

export async function loadMachines() {
  loading = true;
  error = null;
  try {
    machines = await listMachines();
  } catch (e) {
    error = String(e);
  } finally {
    loading = false;
  }
}

export function getMachines() {
  return machines;
}

export function isLoading() {
  return loading;
}

export function getError() {
  return error;
}
