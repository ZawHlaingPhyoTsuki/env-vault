import * as React from "react";
import { AlertTriangle } from "lucide-react";
import { Toaster } from "sonner";
import { Sidebar } from "@/components/layout/sidebar";
import { VariableEditor } from "@/components/variables/variable-editor";
import { useInitStatus } from "@/hooks/use-init";
import { useProjects } from "@/hooks/use-projects";
import { useEnvironments } from "@/hooks/use-environments";

function App() {
  const init = useInitStatus();
  const projects = useProjects();
  const [selectedProjectId, setSelectedProjectId] = React.useState<string | null>(null);
  const [selectedEnvironmentId, setSelectedEnvironmentId] = React.useState<string | null>(null);

  React.useEffect(() => {
    if (!selectedProjectId && projects.data && projects.data.length > 0) {
      setSelectedProjectId(projects.data[0].id);
    }
  }, [projects.data, selectedProjectId]);

  React.useEffect(() => {
    if (selectedProjectId && projects.data && !projects.data.some((p) => p.id === selectedProjectId)) {
      setSelectedProjectId(null);
      setSelectedEnvironmentId(null);
    }
  }, [projects.data, selectedProjectId]);

  const selectedProject = projects.data?.find((p) => p.id === selectedProjectId) ?? null;
  const envs = useEnvironments(selectedProjectId);
  const selectedEnvironment = envs.data?.find((e) => e.id === selectedEnvironmentId) ?? null;

  React.useEffect(() => {
    if (selectedEnvironmentId && envs.data && !envs.data.some((e) => e.id === selectedEnvironmentId)) {
      setSelectedEnvironmentId(null);
    }
  }, [envs.data, selectedEnvironmentId]);

  if (init.isLoading) {
    return <div className="flex h-full items-center justify-center bg-background text-[13px] text-muted-foreground">Loading...</div>;
  }

  if (init.data) {
    return (
      <div className="flex h-full items-center justify-center bg-background p-6 text-foreground">
        <div className="w-full max-w-lg border border-destructive bg-card p-6">
          <div className="flex items-center gap-2 text-destructive">
            <AlertTriangle size={16} strokeWidth={1.5} />
            <p className="text-[11px] font-medium uppercase tracking-[0.08em]">Vault initialization error</p>
          </div>
          <h1 className="mt-3 text-lg font-medium tracking-tight">EnvVault cannot open the vault</h1>
          <p className="mt-2 text-[13px] text-muted-foreground">{init.data}</p>
          <p className="mt-4 border border-border bg-muted/30 px-3 py-2 font-mono text-[12px] text-muted-foreground">
            The app will not generate a new key while encrypted data may exist.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-screen bg-background text-foreground">
      <Sidebar
        selectedProjectId={selectedProjectId}
        selectedEnvironmentId={selectedEnvironmentId}
        onSelectProject={(id) => {
          setSelectedProjectId(id);
          setSelectedEnvironmentId(null);
        }}
        onSelectEnvironment={(projectId, envId) => {
          setSelectedProjectId(projectId);
          setSelectedEnvironmentId(envId);
        }}
      />
      <main className="flex min-w-0 flex-1 flex-col">
        <header className="flex h-12 items-center justify-between border-b border-border px-4">
          <div className="flex items-center gap-2 text-[13px]">
            <span className="text-muted-foreground">EnvVault</span>
            {selectedProject && <>
              <span className="text-muted-foreground">/</span>
              <span className={selectedEnvironment ? "text-muted-foreground" : "font-medium"}>{selectedProject.name}</span>
            </>}
            {selectedEnvironment && <>
              <span className="text-muted-foreground">/</span>
              <span className="font-medium font-mono text-[12px]">{selectedEnvironment.name}</span>
            </>}
          </div>
          <div className="font-mono text-[11px] text-muted-foreground">
            {selectedEnvironment ? `${selectedEnvironment.variable_count} vars` : selectedProject ? `${selectedProject.environment_count} envs` : ""}
          </div>
        </header>

        {!selectedProject ? (
          <div className="flex flex-1 items-center justify-center p-6">
            <EmptyState
              label="Workspace"
              title="Create a project"
              description="Projects group environments like dev, stage, and prod. Your .env data stays encrypted in the local vault."
              hint="Ctrl/Cmd+N"
            />
          </div>
        ) : !selectedEnvironment ? (
          <div className="flex flex-1 items-center justify-center p-6">
            <EmptyState
              label="Project"
              title="Select or create an environment"
              description="Add environments inside the project, then paste .env contents into them."
              hint="Use the sidebar + action"
            />
          </div>
        ) : (
          <VariableEditor
            projectId={selectedProject.id}
            environmentId={selectedEnvironment.id}
            environmentName={selectedEnvironment.name}
          />
        )}
      </main>
      <Toaster
        position="bottom-right"
        toastOptions={{
          classNames: {
            toast: "!rounded-none !border !border-border !bg-background !text-foreground !shadow-none",
            description: "!text-muted-foreground",
            error: "!border-destructive !text-destructive",
            success: "!border-success !text-foreground",
          },
        }}
      />
    </div>
  );
}

function EmptyState({ label, title, description, hint }: { label: string; title: string; description: string; hint: string }) {
  return (
    <div className="w-full max-w-md border border-border bg-card p-6 text-card-foreground">
      <p className="text-[11px] font-medium uppercase tracking-[0.08em] text-muted-foreground">{label}</p>
      <h2 className="mt-2 text-lg font-medium tracking-tight">{title}</h2>
      <p className="mt-2 text-[13px] text-muted-foreground">{description}</p>
      <div className="mt-4 border border-border bg-muted/30 px-3 py-2">
        <p className="font-mono text-[12px] text-muted-foreground">{hint}</p>
      </div>
    </div>
  );
}

export default App;
