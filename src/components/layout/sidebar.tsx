import * as React from "react";
import { ChevronDown, ChevronRight, Copy, FolderPlus, Pencil, Plus, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { ConfirmDialog } from "@/components/ui/confirm-dialog";
import { Dialog } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Kbd } from "@/components/ui/kbd";
import { useCreateEnvironment, useDeleteEnvironment, useDuplicateEnvironment, useEnvironments, useRenameEnvironment } from "@/hooks/use-environments";
import { useCreateProject, useDeleteProject, useProjects, useRenameProject } from "@/hooks/use-projects";
import type { EnvironmentSummary, ProjectSummary } from "@/lib/types";
import { cn } from "@/lib/utils";

interface SidebarProps {
  selectedProjectId: string | null;
  selectedEnvironmentId: string | null;
  onSelectProject: (id: string) => void;
  onSelectEnvironment: (projectId: string, environmentId: string) => void;
}

type NameDialogState =
  | { kind: "create-project" }
  | { kind: "rename-project"; project: ProjectSummary }
  | { kind: "create-environment"; projectId: string }
  | { kind: "rename-environment"; env: EnvironmentSummary }
  | { kind: "duplicate-environment"; env: EnvironmentSummary }
  | null;

type ConfirmState =
  | { kind: "delete-project"; project: ProjectSummary }
  | { kind: "delete-environment"; env: EnvironmentSummary }
  | null;

export function Sidebar({ selectedProjectId, selectedEnvironmentId, onSelectProject, onSelectEnvironment }: SidebarProps) {
  const projects = useProjects();
  const createProject = useCreateProject();
  const renameProject = useRenameProject();
  const deleteProject = useDeleteProject();
  const createEnv = useCreateEnvironment();
  const renameEnv = useRenameEnvironment();
  const deleteEnv = useDeleteEnvironment();
  const duplicateEnv = useDuplicateEnvironment();

  const [expanded, setExpanded] = React.useState<Record<string, boolean>>({});
  const [nameDialog, setNameDialog] = React.useState<NameDialogState>(null);
  const [confirm, setConfirm] = React.useState<ConfirmState>(null);
  const [nameValue, setNameValue] = React.useState("");

  React.useEffect(() => {
    if (selectedProjectId) setExpanded((s) => ({ ...s, [selectedProjectId]: true }));
  }, [selectedProjectId]);

  React.useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "n") {
        e.preventDefault();
        setNameValue("");
        setNameDialog({ kind: "create-project" });
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const openNameDialog = (state: NameDialogState, initial = "") => {
    setNameValue(initial);
    setNameDialog(state);
  };

  const submitName = async () => {
    const name = nameValue.trim();
    if (!name || !nameDialog) return;
    try {
      if (nameDialog.kind === "create-project") {
        const p = await createProject.mutateAsync(name);
        onSelectProject(p.id);
        setExpanded((s) => ({ ...s, [p.id]: true }));
        toast.success("Project created");
      } else if (nameDialog.kind === "rename-project") {
        await renameProject.mutateAsync({ id: nameDialog.project.id, name });
        toast.success("Project renamed");
      } else if (nameDialog.kind === "create-environment") {
        const env = await createEnv.mutateAsync({ projectId: nameDialog.projectId, name });
        onSelectProject(nameDialog.projectId);
        onSelectEnvironment(nameDialog.projectId, env.id);
        toast.success("Environment created");
      } else if (nameDialog.kind === "rename-environment") {
        await renameEnv.mutateAsync({ id: nameDialog.env.id, name, projectId: nameDialog.env.project_id });
        toast.success("Environment renamed");
      } else if (nameDialog.kind === "duplicate-environment") {
        const env = await duplicateEnv.mutateAsync({ sourceId: nameDialog.env.id, newName: name, projectId: nameDialog.env.project_id });
        onSelectEnvironment(nameDialog.env.project_id, env.id);
        toast.success("Environment duplicated");
      }
      setNameDialog(null);
    } catch (e) {
      toast.error(String(e));
    }
  };

  const submitConfirm = async () => {
    if (!confirm) return;
    try {
      if (confirm.kind === "delete-project") {
        await deleteProject.mutateAsync(confirm.project.id);
        toast.success("Project deleted");
      } else {
        await deleteEnv.mutateAsync({ id: confirm.env.id, projectId: confirm.env.project_id });
        toast.success("Environment deleted");
      }
      setConfirm(null);
    } catch (e) {
      toast.error(String(e));
    }
  };

  const nameDialogTitle =
    nameDialog?.kind === "create-project" ? "New project" :
    nameDialog?.kind === "rename-project" ? "Rename project" :
    nameDialog?.kind === "create-environment" ? "New environment" :
    nameDialog?.kind === "rename-environment" ? "Rename environment" :
    nameDialog?.kind === "duplicate-environment" ? "Duplicate environment" : "";

  const nameBusy = createProject.isPending || renameProject.isPending || createEnv.isPending || renameEnv.isPending || duplicateEnv.isPending;

  return (
    <aside className="flex w-[260px] shrink-0 flex-col border-r border-border bg-sidebar text-sidebar-foreground">
      <div className="flex h-12 items-center justify-between border-b border-sidebar-border px-4">
        <span className="text-[13px] font-medium tracking-tight">EnvVault</span>
        <Button variant="ghost" size="icon" aria-label="New project" onClick={() => openNameDialog({ kind: "create-project" })}>
          <FolderPlus size={16} strokeWidth={1.5} />
        </Button>
      </div>

      <div className="flex-1 overflow-auto p-2">
        <p className="px-2 py-2 text-[11px] font-medium uppercase tracking-[0.08em] text-muted-foreground">Projects</p>
        {projects.isLoading && <p className="px-2 py-2 text-[13px] text-muted-foreground">Loading...</p>}
        {projects.data?.length === 0 && (
          <div className="px-2 py-4">
            <p className="text-[13px] text-muted-foreground">No projects yet.</p>
            <Button className="mt-3 w-full" variant="primary" size="sm" onClick={() => openNameDialog({ kind: "create-project" })}>
              Create project
            </Button>
            <div className="mt-2 text-muted-foreground"><Kbd>Ctrl/Cmd+N</Kbd></div>
          </div>
        )}
        {projects.data?.map((project) => (
          <ProjectItem
            key={project.id}
            project={project}
            expanded={!!expanded[project.id]}
            selected={selectedProjectId === project.id && !selectedEnvironmentId}
            selectedEnvironmentId={selectedProjectId === project.id ? selectedEnvironmentId : null}
            onToggle={() => {
              onSelectProject(project.id);
              setExpanded((s) => ({ ...s, [project.id]: !s[project.id] }));
            }}
            onSelectProject={() => onSelectProject(project.id)}
            onSelectEnvironment={(envId) => onSelectEnvironment(project.id, envId)}
            onRename={() => openNameDialog({ kind: "rename-project", project }, project.name)}
            onDelete={() => setConfirm({ kind: "delete-project", project })}
            onCreateEnv={() => openNameDialog({ kind: "create-environment", projectId: project.id })}
            onRenameEnv={(env) => openNameDialog({ kind: "rename-environment", env }, env.name)}
            onDeleteEnv={(env) => setConfirm({ kind: "delete-environment", env })}
            onDuplicateEnv={(env) => openNameDialog({ kind: "duplicate-environment", env }, `${env.name}-copy`)}
          />
        ))}
      </div>

      <div className="border-t border-sidebar-border px-4 py-3 text-[11px] text-muted-foreground">
        <div className="flex items-center gap-2"><Kbd>Ctrl/Cmd+N</Kbd><span>new project</span></div>
      </div>

      <Dialog open={!!nameDialog} onOpenChange={(o) => !o && setNameDialog(null)} title={nameDialogTitle}>
        <form
          className="space-y-4"
          onSubmit={(e) => { e.preventDefault(); void submitName(); }}
        >
          <Input autoFocus value={nameValue} onChange={(e) => setNameValue(e.target.value)} placeholder="Name" />
          <div className="flex justify-end gap-2">
            <Button variant="secondary" onClick={() => setNameDialog(null)}>Cancel</Button>
            <Button variant="primary" type="submit" disabled={!nameValue.trim() || nameBusy}>
              {nameBusy ? "Saving..." : "Save"}
            </Button>
          </div>
        </form>
      </Dialog>

      <ConfirmDialog
        open={!!confirm}
        onOpenChange={(o) => !o && setConfirm(null)}
        title={confirm?.kind === "delete-project" ? "Delete project" : "Delete environment"}
        description={
          confirm?.kind === "delete-project"
            ? `Delete "${confirm.project.name}"? This will remove ${confirm.project.environment_count} environment(s) and ${confirm.project.variable_count} variable(s).`
            : confirm?.kind === "delete-environment"
            ? `Delete "${confirm.env.name}"? This will remove ${confirm.env.variable_count} variable(s).`
            : ""
        }
        confirmLabel="Delete"
        destructive
        loading={deleteProject.isPending || deleteEnv.isPending}
        onConfirm={() => void submitConfirm()}
      />
    </aside>
  );
}

function ProjectItem(props: {
  project: ProjectSummary;
  expanded: boolean;
  selected: boolean;
  selectedEnvironmentId: string | null;
  onToggle: () => void;
  onSelectProject: () => void;
  onSelectEnvironment: (id: string) => void;
  onRename: () => void;
  onDelete: () => void;
  onCreateEnv: () => void;
  onRenameEnv: (env: EnvironmentSummary) => void;
  onDeleteEnv: (env: EnvironmentSummary) => void;
  onDuplicateEnv: (env: EnvironmentSummary) => void;
}) {
  const envs = useEnvironments(props.expanded ? props.project.id : null);
  return (
    <div className="mb-1">
      <div
        className={cn(
          "group flex items-center border border-transparent",
          props.selected && "border-sidebar-border bg-sidebar-accent text-sidebar-accent-foreground",
        )}
      >
        <button
          className="flex min-w-0 flex-1 items-center gap-1 px-2 py-1.5 text-left text-[13px]"
          onClick={props.onToggle}
        >
          {props.expanded ? <ChevronDown size={16} strokeWidth={1.5} /> : <ChevronRight size={16} strokeWidth={1.5} />}
          <span className="truncate">{props.project.name}</span>
        </button>
        <div className="hidden items-center pr-1 group-hover:flex group-focus-within:flex">
          <IconButton label="Add environment" onClick={props.onCreateEnv}><Plus size={16} strokeWidth={1.5} /></IconButton>
          <IconButton label="Rename project" onClick={props.onRename}><Pencil size={16} strokeWidth={1.5} /></IconButton>
          <IconButton label="Delete project" onClick={props.onDelete}><Trash2 size={16} strokeWidth={1.5} /></IconButton>
        </div>
      </div>

      {props.expanded && (
        <div className="ml-4 border-l border-sidebar-border pl-2">
          {envs.data?.length === 0 && (
            <div className="px-2 py-2 text-[12px] text-muted-foreground">No environments</div>
          )}
          {envs.data?.map((env) => (
            <div
              key={env.id}
              className={cn(
                "group/env flex items-center border border-transparent",
                props.selectedEnvironmentId === env.id && "border-sidebar-border bg-sidebar-accent text-sidebar-accent-foreground",
              )}
            >
              <button
                className="min-w-0 flex-1 px-2 py-1.5 text-left text-[13px]"
                onClick={() => props.onSelectEnvironment(env.id)}
              >
                <span className="truncate font-mono text-[12px]">{env.name}</span>
              </button>
              <div className="hidden items-center pr-1 group-hover/env:flex group-focus-within:flex">
                <IconButton label="Duplicate environment" onClick={() => props.onDuplicateEnv(env)}><Copy size={16} strokeWidth={1.5} /></IconButton>
                <IconButton label="Rename environment" onClick={() => props.onRenameEnv(env)}><Pencil size={16} strokeWidth={1.5} /></IconButton>
                <IconButton label="Delete environment" onClick={() => props.onDeleteEnv(env)}><Trash2 size={16} strokeWidth={1.5} /></IconButton>
              </div>
            </div>
          ))}
          <button
            className="flex w-full items-center gap-1 px-2 py-1.5 text-left text-[12px] text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
            onClick={props.onCreateEnv}
          >
            <Plus size={14} strokeWidth={1.5} /> Add environment
          </button>
        </div>
      )}
    </div>
  );
}

function IconButton({ label, onClick, children }: { label: string; onClick: () => void; children: React.ReactNode }) {
  return (
    <button
      className="inline-flex h-7 w-7 items-center justify-center text-muted-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
      aria-label={label}
      title={label}
      onClick={(e) => { e.stopPropagation(); onClick(); }}
    >
      {children}
    </button>
  );
}
