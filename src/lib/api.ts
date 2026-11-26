import { invoke } from "@tauri-apps/api/core";
import type {
  Environment,
  EnvironmentSummary,
  Project,
  ProjectSummary,
  RevealedValue,
  Variable,
  VariableRow,
} from "./types";

export const api = {
  getInitStatus: () => invoke<string | null>("get_init_status"),
  listProjects: () => invoke<ProjectSummary[]>("list_projects"),
  createProject: (name: string) => invoke<Project>("create_project", { name }),
  renameProject: (id: string, name: string) => invoke<Project>("rename_project", { id, name }),
  deleteProject: (id: string) => invoke<void>("delete_project", { id }),

  listEnvironments: (projectId: string) =>
    invoke<EnvironmentSummary[]>("list_environments", { projectId }),
  createEnvironment: (projectId: string, name: string) =>
    invoke<Environment>("create_environment", { projectId, name }),
  renameEnvironment: (id: string, name: string) =>
    invoke<Environment>("rename_environment", { id, name }),
  deleteEnvironment: (id: string) => invoke<void>("delete_environment", { id }),
  duplicateEnvironment: (sourceId: string, newName: string) =>
    invoke<Environment>("duplicate_environment", { sourceId, newName }),

  listVariables: (environmentId: string) =>
    invoke<VariableRow[]>("list_variables", { environmentId }),
  createVariable: (input: {
    environmentId: string;
    key: string;
    value: string;
    comment?: string | null;
    position?: number | null;
  }) =>
    invoke<Variable>("create_variable", {
      environmentId: input.environmentId,
      key: input.key,
      value: input.value,
      comment: input.comment ?? null,
      position: input.position ?? null,
    }),
  updateVariable: (input: {
    id: string;
    key: string;
    value?: string | null;
    comment?: string | null;
  }) =>
    invoke<Variable>("update_variable", {
      id: input.id,
      key: input.key,
      value: input.value ?? null,
      comment: input.comment ?? null,
    }),
  deleteVariable: (id: string) => invoke<void>("delete_variable", { id }),
  reorderVariables: (environmentId: string, orderedIds: string[]) =>
    invoke<void>("reorder_variables", { environmentId, orderedIds }),
  revealVariable: (id: string) => invoke<RevealedValue>("reveal_variable", { id }),
};
