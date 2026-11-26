import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";

export function useInitStatus() {
  return useQuery({
    queryKey: ["init-status"],
    queryFn: api.getInitStatus,
    staleTime: Infinity,
  });
}
