import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";

export function useEnvironments(projectId: string | null) {
  return useQuery({
    queryKey: ["environments", projectId],
    queryFn: () => api.listEnvironments(projectId!),
    enabled: !!projectId,
  });
}

export function useCreateEnvironment() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ projectId, name }: { projectId: string; name: string }) =>
      api.createEnvironment(projectId, name),
    onSuccess: (_, vars) => {
      qc.invalidateQueries({ queryKey: ["environments", vars.projectId] });
      qc.invalidateQueries({ queryKey: ["projects"] });
    },
  });
}

export function useRenameEnvironment() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, name }: { id: string; name: string; projectId: string }) =>
      api.renameEnvironment(id, name),
    onSuccess: (_, vars) => {
      qc.invalidateQueries({ queryKey: ["environments", vars.projectId] });
    },
  });
}

export function useDeleteEnvironment() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id }: { id: string; projectId: string }) => api.deleteEnvironment(id),
    onSuccess: (_, vars) => {
      qc.invalidateQueries({ queryKey: ["environments", vars.projectId] });
      qc.invalidateQueries({ queryKey: ["projects"] });
    },
  });
}

export function useDuplicateEnvironment() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ sourceId, newName }: { sourceId: string; newName: string; projectId: string }) =>
      api.duplicateEnvironment(sourceId, newName),
    onSuccess: (_, vars) => {
      qc.invalidateQueries({ queryKey: ["environments", vars.projectId] });
      qc.invalidateQueries({ queryKey: ["projects"] });
    },
  });
}
