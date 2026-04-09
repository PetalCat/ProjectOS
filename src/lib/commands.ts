import { invoke } from "@tauri-apps/api/core";
import type {
  ActivityEntry,
  Comment,
  Dashboard,
  Issue,
  Label,
  Machine,
  MachineDoc,
  Milestone,
  Project,
  ReactionGroup,
  SearchResults,
} from "./types";

// ── Projects ──────────────────────────────────────────────────────────────────

export const createProject = (name: string, description?: string) =>
  invoke<Project>("create_project", { input: { name, description } });

export const listProjects = () => invoke<Project[]>("list_projects");

export const updateProject = (
  id: string,
  fields: { name?: string; description?: string; notes?: string }
) => invoke<Project>("update_project", { input: { id, ...fields } });

export const deleteProject = (id: string) =>
  invoke<void>("delete_project", { id });

// ── Issues ────────────────────────────────────────────────────────────────────

export const createIssue = (input: {
  title: string;
  project_id?: string | null;
  body?: string | null;
  status?: string | null;
  machine_id?: string | null;
  milestone_id?: string | null;
}) => invoke<Issue>("create_issue", { input });

export const listIssues = (
  project_id?: string | null,
  include_closed?: boolean
) => invoke<Issue[]>("list_issues", { projectId: project_id, includeClosed: include_closed });

export const getIssue = (id: string) => invoke<Issue>("get_issue", { id });

export const updateIssue = (input: {
  id: string;
  title?: string;
  body?: string | null;
  status?: string | null;
  context?: string | null;
  machine_id?: string | null;
  milestone_id?: string | null;
}) => invoke<Issue>("update_issue", { input });

export const closeIssue = (id: string) => invoke<Issue>("close_issue", { id });

export const reopenIssue = (id: string) =>
  invoke<Issue>("reopen_issue", { id });

export const deleteIssue = (id: string) =>
  invoke<void>("delete_issue", { id });

export const reorderIssue = (id: string, new_sort_order: number) =>
  invoke<Issue>("reorder_issue", { id, newSortOrder: new_sort_order });

export const transferIssue = (id: string, to_project_id: string) =>
  invoke<Issue>("transfer_issue", { id, toProjectId: to_project_id });

export const promoteIdea = (id: string) =>
  invoke<Issue>("promote_idea", { id });

// ── Dependencies & Relations ──────────────────────────────────────────────────

export const addDependency = (blocker_id: string, blocked_id: string) =>
  invoke<void>("add_dependency", { blockerId: blocker_id, blockedId: blocked_id });

export const removeDependency = (blocker_id: string, blocked_id: string) =>
  invoke<void>("remove_dependency", { blockerId: blocker_id, blockedId: blocked_id });

export const addRelation = (issue_a_id: string, issue_b_id: string) =>
  invoke<void>("add_relation", { issueAId: issue_a_id, issueBId: issue_b_id });

export const removeRelation = (issue_a_id: string, issue_b_id: string) =>
  invoke<void>("remove_relation", { issueAId: issue_a_id, issueBId: issue_b_id });

// ── Assignees ─────────────────────────────────────────────────────────────────

export const assignIssue = (issue_id: string, assignee_name: string) =>
  invoke<void>("assign_issue", { issueId: issue_id, assigneeName: assignee_name });

export const unassignIssue = (issue_id: string, assignee_name: string) =>
  invoke<void>("unassign_issue", { issueId: issue_id, assigneeName: assignee_name });

export const getIssueAssignees = (issue_id: string) =>
  invoke<string[]>("get_issue_assignees", { issueId: issue_id });

// ── Comments ──────────────────────────────────────────────────────────────────

export const createComment = (issue_id: string, body: string) =>
  invoke<Comment>("create_comment", { input: { issue_id, body } });

export const listComments = (issue_id: string) =>
  invoke<Comment[]>("list_comments", { issueId: issue_id });

export const updateComment = (id: string, body: string) =>
  invoke<Comment>("update_comment", { input: { id, body } });

export const deleteComment = (id: string) =>
  invoke<void>("delete_comment", { id });

// ── Reactions ─────────────────────────────────────────────────────────────────

export const addReaction = (
  emoji: string,
  issue_id?: string | null,
  comment_id?: string | null
) => invoke<void>("add_reaction", { issueId: issue_id, commentId: comment_id, emoji });

export const removeReaction = (id: string) =>
  invoke<void>("remove_reaction", { id });

export const listReactions = (
  issue_id?: string | null,
  comment_id?: string | null
) => invoke<ReactionGroup[]>("list_reactions", { issueId: issue_id, commentId: comment_id });

// ── Labels ────────────────────────────────────────────────────────────────────

export const createLabel = (
  name: string,
  color: string,
  project_id?: string | null
) => invoke<Label>("create_label", { input: { name, color, project_id } });

export const listLabels = (project_id?: string | null) =>
  invoke<Label[]>("list_labels", { projectId: project_id });

export const deleteLabel = (id: string) =>
  invoke<void>("delete_label", { id });

export const addLabelToIssue = (issue_id: string, label_id: string) =>
  invoke<void>("add_label_to_issue", { issueId: issue_id, labelId: label_id });

export const removeLabelFromIssue = (issue_id: string, label_id: string) =>
  invoke<void>("remove_label_from_issue", { issueId: issue_id, labelId: label_id });

export const getIssueLabels = (issue_id: string) =>
  invoke<Label[]>("get_issue_labels", { issueId: issue_id });

// ── Milestones ────────────────────────────────────────────────────────────────

export const createMilestone = (input: {
  project_id: string;
  title: string;
  description?: string | null;
  due_date?: number | null;
}) => invoke<Milestone>("create_milestone", { input });

export const listMilestones = (project_id: string) =>
  invoke<Milestone[]>("list_milestones", { projectId: project_id });

export const closeMilestone = (id: string) =>
  invoke<void>("close_milestone", { id });

export const setMilestone = (
  issue_id: string,
  milestone_id: string | null
) => invoke<void>("set_milestone", { issueId: issue_id, milestoneId: milestone_id });

// ── Machines ──────────────────────────────────────────────────────────────────

export const createMachine = (input: {
  name: string;
  hostname?: string | null;
  ip?: string | null;
  user?: string | null;
  os?: string | null;
  notes?: string | null;
}) => invoke<Machine>("create_machine", { input });

export const listMachines = () => invoke<Machine[]>("list_machines");

export const updateMachine = (input: {
  id: string;
  name?: string;
  hostname?: string | null;
  ip?: string | null;
  user?: string | null;
  os?: string | null;
  notes?: string | null;
}) => invoke<Machine>("update_machine", { input });

export const deleteMachine = (id: string) =>
  invoke<void>("delete_machine", { id });

export const getCurrentMachine = () =>
  invoke<Machine>("get_current_machine");

// ── Machine Docs ──────────────────────────────────────────────────────────────

export const createMachineDoc = (input: {
  machine_id: string;
  title: string;
  content?: string | null;
  url?: string | null;
}) => invoke<MachineDoc>("create_machine_doc", { input });

export const listMachineDocs = (machine_id: string) =>
  invoke<MachineDoc[]>("list_machine_docs", { machineId: machine_id });

export const updateMachineDoc = (
  id: string,
  title?: string,
  content?: string | null,
  url?: string | null
) => invoke<MachineDoc>("update_machine_doc", { id, title, content, url });

export const deleteMachineDoc = (id: string) =>
  invoke<void>("delete_machine_doc", { id });

// ── Activity ──────────────────────────────────────────────────────────────────

export const getActivityLog = (
  project_id?: string | null,
  limit?: number | null
) => invoke<ActivityEntry[]>("get_activity_log", { projectId: project_id, limit });

// ── Search ────────────────────────────────────────────────────────────────────

export const searchIssues = (query: string) =>
  invoke<SearchResults>("search_issues", { query });

// ── Dashboard ─────────────────────────────────────────────────────────────────

export const getDashboard = () => invoke<Dashboard>("get_dashboard");

// ── Settings ─────────────────────────────────────────────────────────────────

export const scanDeveloperFolder = (path: string) =>
  invoke<Project[]>("scan_developer_folder", { path });

export const rescanTimestamps = () => invoke<number>("rescan_timestamps");

export const syncGithubIssues = (project_id: string) =>
  invoke<number>("sync_github_issues", { projectId: project_id });
