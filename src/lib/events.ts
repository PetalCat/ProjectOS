import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export function onIssuesChanged(
  callback: (projectId: string | null) => void
): Promise<UnlistenFn> {
  return listen<{ project_id: string | null }>("issues-changed", (event) => {
    callback(event.payload.project_id);
  });
}

export function onProjectsChanged(callback: () => void): Promise<UnlistenFn> {
  return listen("projects-changed", () => callback());
}

export function onActivity(
  callback: (entry: { action: string; detail: string | null }) => void
): Promise<UnlistenFn> {
  return listen("activity", (event) => callback(event.payload as any));
}

export function onGithubPushError(
  callback: (payload: { issue_id: string; message: string }) => void
): Promise<UnlistenFn> {
  return listen<{ issue_id: string; message: string }>(
    "github-push-error",
    (event) => callback(event.payload)
  );
}
