import * as React from "react";
import { ArrowDown, ArrowUp, Copy, Eye, EyeOff, Pencil, Plus, Trash2 } from "lucide-react";
import { readText, writeText } from "@tauri-apps/plugin-clipboard-manager";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { ConfirmDialog } from "@/components/ui/confirm-dialog";
import { Input } from "@/components/ui/input";
import { useCreateVariable, useDeleteVariable, useReorderVariables, useRevealVariable, useUpdateVariable, useVariables } from "@/hooks/use-variables";
import type { VariableRow } from "@/lib/types";
import { cn } from "@/lib/utils";

const KEY_RE = /^[A-Za-z_][A-Za-z0-9_]*$/;

interface Props {
  projectId: string;
  environmentId: string;
  environmentName: string;
}

export function VariableEditor({ projectId, environmentId, environmentName }: Props) {
  const vars = useVariables(environmentId);
  const createVar = useCreateVariable(projectId);
  const updateVar = useUpdateVariable(environmentId);
  const deleteVar = useDeleteVariable(projectId, environmentId);
  const reorderVar = useReorderVariables(environmentId);
  const revealVar = useRevealVariable();

  const [revealed, setRevealed] = React.useState<Record<string, string>>({});
  const revealTimers = React.useRef<Record<string, number>>({});
  const [editingId, setEditingId] = React.useState<string | null>(null);
  const [draft, setDraft] = React.useState<{ key: string; value: string; comment: string }>({ key: "", value: "", comment: "" });
  const [adding, setAdding] = React.useState(false);
  const [addDraft, setAddDraft] = React.useState({ key: "", value: "", comment: "" });
  const [deleteTarget, setDeleteTarget] = React.useState<VariableRow | null>(null);

  React.useEffect(() => {
    return () => {
      Object.values(revealTimers.current).forEach((t) => window.clearTimeout(t));
    };
  }, []);

  React.useEffect(() => {
    setEditingId(null);
    setAdding(false);
    setRevealed({});
    Object.values(revealTimers.current).forEach((t) => window.clearTimeout(t));
    revealTimers.current = {};
  }, [environmentId]);

  const rows = vars.data ?? [];
  const maskValue = (row: VariableRow) => revealed[row.id] ?? row.masked_value;

  const scheduleRemask = (id: string) => {
    if (revealTimers.current[id]) window.clearTimeout(revealTimers.current[id]);
    revealTimers.current[id] = window.setTimeout(() => {
      setRevealed((s) => {
        const next = { ...s };
        delete next[id];
        return next;
      });
      delete revealTimers.current[id];
    }, 15000);
  };

  const toggleReveal = async (row: VariableRow) => {
    if (revealed[row.id]) {
      if (revealTimers.current[row.id]) window.clearTimeout(revealTimers.current[row.id]);
      delete revealTimers.current[row.id];
      setRevealed((s) => { const next = { ...s }; delete next[row.id]; return next; });
      return;
    }
    try {
      const res = await revealVar.mutateAsync(row.id);
      setRevealed((s) => ({ ...s, [row.id]: res.value }));
      scheduleRemask(row.id);
    } catch (e) { toast.error(String(e)); }
  };

  const copyValue = async (row: VariableRow) => {
    try {
      const value = revealed[row.id] ?? (await revealVar.mutateAsync(row.id)).value;
      await writeText(value);
      toast.success("Copied value");
      window.setTimeout(async () => {
        try {
          const current = await readText();
          if (current === value) await writeText("");
        } catch { /* ignore */ }
      }, 30000);
    } catch (e) { toast.error(String(e)); }
  };

  const startEdit = async (row: VariableRow) => {
    try {
      const value = revealed[row.id] ?? (await revealVar.mutateAsync(row.id)).value;
      setDraft({ key: row.key, value, comment: row.comment ?? "" });
      setEditingId(row.id);
    } catch (e) { toast.error(String(e)); }
  };

  const saveEdit = async (row: VariableRow) => {
    const key = draft.key.trim();
    if (!KEY_RE.test(key)) { toast.error("Key must match [A-Za-z_][A-Za-z0-9_]*"); return; }
    if (rows.some((r) => r.id !== row.id && r.key === key)) { toast.error("Duplicate key"); return; }
    try {
      await updateVar.mutateAsync({ id: row.id, key, value: draft.value, comment: draft.comment || null });
      setEditingId(null);
      toast.success("Variable updated");
    } catch (e) { toast.error(String(e)); }
  };

  const addVariable = async () => {
    const key = addDraft.key.trim();
    if (!KEY_RE.test(key)) { toast.error("Key must match [A-Za-z_][A-Za-z0-9_]*"); return; }
    if (rows.some((r) => r.key === key)) { toast.error("Duplicate key"); return; }
    try {
      await createVar.mutateAsync({ environmentId, key, value: addDraft.value, comment: addDraft.comment || null });
      setAddDraft({ key: "", value: "", comment: "" });
      setAdding(false);
      toast.success("Variable added");
    } catch (e) { toast.error(String(e)); }
  };

  const move = async (index: number, dir: -1 | 1) => {
    const next = index + dir;
    if (next < 0 || next >= rows.length) return;
    const ordered = [...rows];
    const [item] = ordered.splice(index, 1);
    ordered.splice(next, 0, item);
    try { await reorderVar.mutateAsync(ordered.map((r) => r.id)); }
    catch (e) { toast.error(String(e)); }
  };

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center justify-between border-b border-border px-4 py-2">
        <div>
          <p className="text-[11px] font-medium uppercase tracking-[0.08em] text-muted-foreground">Variables</p>
          <p className="font-mono text-[13px] text-muted-foreground">{environmentName}</p>
        </div>
        <Button variant="primary" size="sm" onClick={() => setAdding(true)}><Plus size={16} strokeWidth={1.5} /> Add variable</Button>
      </div>

      <div className="min-h-0 flex-1 overflow-auto">
        <table className="w-full border-collapse text-[13px]">
          <thead className="sticky top-0 z-10 bg-background">
            <tr className="border-b border-border text-left">
              <th className="w-[28%] px-4 py-2 font-medium text-muted-foreground">Key</th>
              <th className="w-[34%] px-4 py-2 font-medium text-muted-foreground">Value</th>
              <th className="px-4 py-2 font-medium text-muted-foreground">Comment</th>
              <th className="w-[190px] px-4 py-2 font-medium text-muted-foreground">Actions</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row, index) => {
              const editing = editingId === row.id;
              const isRevealed = !!revealed[row.id];
              return (
                <tr key={row.id} className="border-b border-border align-top">
                  <td className="px-4 py-2 font-mono text-[12px] cursor-text" onClick={() => !editing && void startEdit(row)}>{editing ? <Input value={draft.key} onChange={(e) => setDraft((d) => ({ ...d, key: e.target.value }))} /> : row.key}</td>
                  <td className="px-4 py-2 font-mono text-[12px] cursor-text" onClick={() => !editing && void startEdit(row)}>{editing ? <Input value={draft.value} onChange={(e) => setDraft((d) => ({ ...d, value: e.target.value }))} /> : <span className={cn(!isRevealed && "text-muted-foreground")}>{maskValue(row)}</span>}</td>
                  <td className="px-4 py-2 text-muted-foreground cursor-text" onClick={() => !editing && void startEdit(row)}>{editing ? <Input value={draft.comment} onChange={(e) => setDraft((d) => ({ ...d, comment: e.target.value }))} /> : (row.comment ?? "")}</td>
                  <td className="px-4 py-1.5">
                    <div className="flex items-center gap-1">
                      {editing ? (
                        <>
                          <Button size="sm" variant="primary" onClick={() => void saveEdit(row)} disabled={updateVar.isPending}>Save</Button>
                          <Button size="sm" variant="secondary" onClick={() => setEditingId(null)}>Cancel</Button>
                        </>
                      ) : (
                        <>
                          <IconBtn label={isRevealed ? "Hide value" : "Reveal value"} onClick={() => void toggleReveal(row)}>{isRevealed ? <EyeOff size={16} strokeWidth={1.5} /> : <Eye size={16} strokeWidth={1.5} />}</IconBtn>
                          <IconBtn label="Copy value" onClick={() => void copyValue(row)}><Copy size={16} strokeWidth={1.5} /></IconBtn>
                          <IconBtn label="Edit variable" onClick={() => void startEdit(row)}><Pencil size={16} strokeWidth={1.5} /></IconBtn>
                          <IconBtn label="Move up" onClick={() => void move(index, -1)} disabled={index === 0}><ArrowUp size={16} strokeWidth={1.5} /></IconBtn>
                          <IconBtn label="Move down" onClick={() => void move(index, 1)} disabled={index === rows.length - 1}><ArrowDown size={16} strokeWidth={1.5} /></IconBtn>
                          <IconBtn label="Delete variable" onClick={() => setDeleteTarget(row)}><Trash2 size={16} strokeWidth={1.5} /></IconBtn>
                        </>
                      )}
                    </div>
                  </td>
                </tr>
              );
            })}
            {adding && (
              <tr className="border-b border-border bg-muted/20 align-top">
                <td className="px-4 py-2"><Input autoFocus value={addDraft.key} onChange={(e) => setAddDraft((d) => ({ ...d, key: e.target.value }))} placeholder="KEY_NAME" /></td>
                <td className="px-4 py-2"><Input value={addDraft.value} onChange={(e) => setAddDraft((d) => ({ ...d, value: e.target.value }))} placeholder="value" /></td>
                <td className="px-4 py-2"><Input value={addDraft.comment} onChange={(e) => setAddDraft((d) => ({ ...d, comment: e.target.value }))} placeholder="comment" /></td>
                <td className="px-4 py-1.5"><div className="flex items-center gap-1"><Button size="sm" variant="primary" onClick={() => void addVariable()} disabled={createVar.isPending}>Add</Button><Button size="sm" variant="secondary" onClick={() => setAdding(false)}>Cancel</Button></div></td>
              </tr>
            )}
          </tbody>
        </table>
        {!vars.isLoading && rows.length === 0 && !adding && (
          <div className="p-6">
            <div className="max-w-md border border-border bg-card p-6">
              <p className="text-[11px] font-medium uppercase tracking-[0.08em] text-muted-foreground">Environment</p>
              <h3 className="mt-2 text-lg font-medium tracking-tight">No variables yet</h3>
              <p className="mt-2 text-[13px] text-muted-foreground">Add variables manually. Import/export comes in the next step.</p>
              <Button className="mt-4" variant="primary" size="sm" onClick={() => setAdding(true)}><Plus size={16} strokeWidth={1.5} /> Add variable</Button>
            </div>
          </div>
        )}
      </div>

      <ConfirmDialog
        open={!!deleteTarget}
        onOpenChange={(o) => !o && setDeleteTarget(null)}
        title="Delete variable"
        description={deleteTarget ? `Delete "${deleteTarget.key}"? This cannot be undone.` : ""}
        confirmLabel="Delete"
        destructive
        loading={deleteVar.isPending}
        onConfirm={async () => {
          if (!deleteTarget) return;
          try { await deleteVar.mutateAsync(deleteTarget.id); setDeleteTarget(null); toast.success("Variable deleted"); }
          catch (e) { toast.error(String(e)); }
        }}
      />
    </div>
  );
}

function IconBtn({ label, onClick, children, disabled }: { label: string; onClick: () => void; children: React.ReactNode; disabled?: boolean }) {
  return (
    <button className="inline-flex h-8 w-8 items-center justify-center border border-transparent text-muted-foreground hover:border-border hover:bg-accent hover:text-accent-foreground disabled:opacity-40" aria-label={label} title={label} onClick={onClick} disabled={disabled}>
      {children}
    </button>
  );
}
