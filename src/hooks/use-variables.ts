import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";

export function useVariables(environmentId: string | null) {
  return useQuery({
    queryKey: ["variables", environmentId],
    queryFn: () => api.listVariables(environmentId!),
    enabled: !!environmentId,
  });
}

export function useCreateVariable(projectId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: api.createVariable,
    onSuccess: (_, vars) => {
      qc.invalidateQueries({ queryKey: ["variables", vars.environmentId] });
      qc.invalidateQueries({ queryKey: ["environments", projectId] });
      qc.invalidateQueries({ queryKey: ["projects"] });
    },
  });
}

export function useUpdateVariable(environmentId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: api.updateVariable,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["variables", environmentId] });
    },
  });
}

export function useDeleteVariable(projectId: string, environmentId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.deleteVariable(id),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["variables", environmentId] });
      qc.invalidateQueries({ queryKey: ["environments", projectId] });
      qc.invalidateQueries({ queryKey: ["projects"] });
    },
  });
}

export function useReorderVariables(environmentId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (orderedIds: string[]) => api.reorderVariables(environmentId, orderedIds),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["variables", environmentId] }),
  });
}

export function useRevealVariable() {
  return useMutation({
    mutationFn: (id: string) => api.revealVariable(id),
  });
}
