import type { View } from "$lib/types";

// Current view state
let current = $state<View>({ kind: "home" });

// Navigation history for back navigation
let history = $state<View[]>([]);

export function navigate(view: View) {
  history.push(current);
  current = view;
}

export function navigateBack() {
  const prev = history.pop();
  if (prev) {
    current = prev;
  } else {
    current = { kind: "home" };
  }
}

export function currentView() {
  return current;
}
