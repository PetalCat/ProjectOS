export interface Project {
  id: string;
  name: string;
  description: string | null;
  notes: string | null;
  created_at: number;
  updated_at: number;
}

export interface Issue {
  id: string;
  project_id: string | null;
  number: number | null;
  title: string;
  body: string | null;
  state: "open" | "closed";
  status: "next" | "ready" | "blocked" | "idea" | null;
  sort_order: number;
  context: string | null;
  machine_id: string | null;
  milestone_id: string | null;
  locked: boolean;
  pinned: boolean;
  created_at: number;
  updated_at: number;
  closed_at: number | null;
}

export interface Comment {
  id: string;
  issue_id: string;
  body: string;
  created_at: number;
  updated_at: number;
}

export interface Label {
  id: string;
  name: string;
  color: string;
  project_id: string | null;
}

export interface Milestone {
  id: string;
  project_id: string;
  title: string;
  description: string | null;
  due_date: number | null;
  state: "open" | "closed";
  created_at: number;
  updated_at: number;
  open_count: number | null;
  closed_count: number | null;
}

export interface Machine {
  id: string;
  name: string;
  hostname: string | null;
  ip: string | null;
  user: string | null;
  os: string | null;
  notes: string | null;
  created_at: number;
  updated_at: number;
}

export interface MachineDoc {
  id: string;
  machine_id: string;
  title: string;
  content: string | null;
  url: string | null;
  created_at: number;
}

export interface ActivityEntry {
  id: number;
  issue_id: string | null;
  project_id: string | null;
  action: string;
  detail: string | null;
  created_at: number;
}

export interface ReactionGroup {
  emoji: string;
  count: number;
  ids: string[];
}

export interface DashboardProject {
  project: Project;
  next_issue: Issue | null;
  open_count: number;
}

export interface Dashboard {
  projects: DashboardProject[];
  recent_activity: ActivityEntry[];
}

export interface SearchResults {
  issues: Issue[];
  projects: Project[];
  machines: Machine[];
}

export type View =
  | { kind: "home" }
  | { kind: "project"; projectId: string }
  | { kind: "issue"; issueId: string }
  | { kind: "machine"; machineId: string };
