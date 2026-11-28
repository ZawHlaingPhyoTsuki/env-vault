import * as React from "react";
import { Eye, EyeOff } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Dialog } from "@/components/ui/dialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { api } from "@/lib/api";
import type { CompareRow, EnvironmentSummary } from "@/lib/types";
import { cn } from "@/lib/utils";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  currentEnvironmentId: string;
  environments: EnvironmentSummary[];
}

const STATUS_LABEL: Record<CompareRow["status"], string> = {
  only_left: "Only in A",
  only_right: "Only in B",
  same: "Same",
  different: "Different",
};

export function CompareDialog({ open, onOpenChange, currentEnvironmentId, environments }: Props) {
  const [leftId, setLeftId] = React.useState(currentEnvironmentId);
  const [rightId, setRightId] = React.useState("");
  const [rows, setRows] = React.useState<CompareRow[]>([]);
  const [loading, setLoading] = React.useState(false);
  const [missingOnly, setMissingOnly] = React.useState(false);
  const [revealed, setRevealed] = React.useState<Record<string, string>>({});
  const timers = React.useRef<Record<string, number>>({});

  React.useEffect(() => {
    if (!open) return;
    setLeftId(currentEnvironmentId);
    const other = environments.find((e) => e.id !== currentEnvironmentId);
    setRightId(other?.id ?? "");
    setRows([]);
    setMissingOnly(false);
    setRevealed({});
    Object.values(timers.current).forEach((t) => window.clearTimeout(t));
    timers.current = {};
  }, [open, currentEnvironmentId, environments]);

  React.useEffect(() => {
    return () => { Object.values(timers.current).forEach((t) => window.clearTimeout(t)); };
  }, []);

  const scheduleRemask = (id: string) => {
    if (timers.current[id]) window.clearTimeout(timers.current[id]);
    timers.current[id] = window.setTimeout(() => {
      setRevealed((s) => { const n = { ...s }; delete n[id]; return n; });
      delete timers.current[id];
    }, 15000);
  };

  const toggleReveal = async (varId: string | null) => {
    if (!varId) return;
    if (revealed[varId]) {
      if (timers.current[varId]) window.clearTimeout(timers.current[varId]);
      delete timers.current[varId];
      setRevealed((s) => { const n = { ...s }; delete n[varId]; return n; });
      return;
    }
    try {
      const res = await api.revealVariable(varId);
      setRevealed((s) => ({ ...s, [varId]: res.value }));
      scheduleRemask(varId);
    } catch (e) { toast.error(String(e)); }
  };

  const compare = async () => {
    if (!leftId || !rightId) return;
    if (leftId === rightId) { toast.error("Select two different environments"); return; }
    setLoading(true);
    try {
      const res = await api.compareEnvironments(leftId, rightId);
      setRows(res.rows);
      setRevealed({});
      Object.values(timers.current).forEach((t) => window.clearTimeout(t));
      timers.current = {};
    } catch (e) { toast.error(String(e)); }
    finally { setLoading(false); }
  };

  const visible = missingOnly ? rows.filter((r) => r.status === "only_left") : rows;
  const leftName = environments.find((e) => e.id === leftId)?.name ?? "A";
  const rightName = environments.find((e) => e.id === rightId)?.name ?? "B";
  const envItems = environments.map((e) => ({ value: e.id, label: e.name }));

  return (
    <Dialog open={open} onOpenChange={onOpenChange} title="Compare environments" description="Compare keys and values across two environments. Values are masked by default." className="max-w-5xl">
      <div className="space-y-3">
        <div className="flex flex-wrap items-end gap-2">
          <label className="flex flex-col gap-1 text-[12px] text-muted-foreground">
            <span className="uppercase tracking-[0.04em]">Env A</span>
            <Select value={leftId} onValueChange={(v) => setLeftId(v ?? "")} items={envItems}>
              <SelectTrigger className="w-[180px] font-mono">
                <SelectValue placeholder="Select..." />
              </SelectTrigger>
              <SelectContent>
                {environments.map((e) => <SelectItem key={e.id} value={e.id} className="font-mono">{e.name}</SelectItem>)}
              </SelectContent>
            </Select>
          </label>
          <label className="flex flex-col gap-1 text-[12px] text-muted-foreground">
            <span className="uppercase tracking-[0.04em]">Env B</span>
            <Select value={rightId} onValueChange={(v) => setRightId(v ?? "")} items={envItems}>
              <SelectTrigger className="w-[180px] font-mono">
                <SelectValue placeholder="Select..." />
              </SelectTrigger>
              <SelectContent>
                {environments.map((e) => <SelectItem key={e.id} value={e.id} className="font-mono">{e.name}</SelectItem>)}
              </SelectContent>
            </Select>
          </label>
          <Button size="sm" variant="primary" onClick={() => void compare()} disabled={loading || !leftId || !rightId || leftId === rightId}>
            {loading ? "Comparing..." : "Compare"}
          </Button>
          <Button size="sm" variant={missingOnly ? "primary" : "secondary"} onClick={() => setMissingOnly((v) => !v)} disabled={rows.length === 0}>
            Missing in B
          </Button>
        </div>

        {rows.length > 0 && (
          <div className="max-h-[60vh] overflow-auto border border-border">
            <table className="w-full border-collapse text-[12px]">
              <thead className="sticky top-0 bg-background">
                <tr className="border-b border-border text-left">
                  <th className="px-3 py-2 font-medium text-muted-foreground">Key</th>
                  <th className="px-3 py-2 font-medium text-muted-foreground">Status</th>
                  <th className="px-3 py-2 font-medium text-muted-foreground">{leftName}</th>
                  <th className="px-3 py-2 font-medium text-muted-foreground">{rightName}</th>
                </tr>
              </thead>
              <tbody>
                {visible.map((r) => (
                  <tr key={r.key} className="border-b border-border align-middle last:border-b-0">
                    <td className="px-3 py-2 font-mono">{r.key}</td>
                    <td className="px-3 py-2">
                      <span className={cn(
                        "inline-flex border px-1.5 py-0.5 text-[11px] uppercase tracking-[0.04em]",
                        r.status === "same" && "border-success/40 text-success",
                        r.status === "different" && "border-warning/40 text-warning",
                        (r.status === "only_left" || r.status === "only_right") && "border-destructive/40 text-destructive",
                      )}>{STATUS_LABEL[r.status]}</span>
                    </td>
                    <td className="px-3 py-2 font-mono text-muted-foreground"><ValueCell value={r.left_masked} varId={r.left_variable_id} revealed={revealed} onToggle={toggleReveal} /></td>
                    <td className="px-3 py-2 font-mono text-muted-foreground"><ValueCell value={r.right_masked} varId={r.right_variable_id} revealed={revealed} onToggle={toggleReveal} /></td>
                  </tr>
                ))}
                {visible.length === 0 && (
                  <tr><td colSpan={4} className="px-3 py-6 text-center text-muted-foreground">No missing keys in {rightName}.</td></tr>
                )}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </Dialog>
  );
}

function ValueCell({ value, varId, revealed, onToggle }: { value: string | null; varId: string | null; revealed: Record<string, string>; onToggle: (id: string | null) => void }) {
  if (value === null) return <span className="text-muted-foreground">—</span>;
  const isRevealed = varId ? !!revealed[varId] : false;
  const display = isRevealed && varId ? revealed[varId] : (value || "—");
  const canReveal = !!varId && value !== "";
  return (
    <div className="flex items-center gap-2">
      <span className="min-w-0 break-all">{display}</span>
      {canReveal && (
        <button type="button" className="inline-flex h-7 w-7 shrink-0 items-center justify-center border border-transparent text-muted-foreground hover:border-border hover:bg-accent hover:text-accent-foreground" aria-label={isRevealed ? "Hide value" : "Reveal value"} title={isRevealed ? "Hide value" : "Reveal value"} onClick={() => onToggle(varId)}>
          {isRevealed ? <EyeOff size={14} strokeWidth={1.5} /> : <Eye size={14} strokeWidth={1.5} />}
        </button>
      )}
    </div>
  );
}
