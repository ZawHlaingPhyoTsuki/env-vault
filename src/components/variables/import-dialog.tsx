import * as React from "react";
import { toast } from "sonner";
import { Dialog } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { api } from "@/lib/api";
import type { ImportPreviewRow } from "@/lib/types";
import { cn } from "@/lib/utils";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  environmentId: string;
  onImported: () => void;
}

export function ImportDialog({ open, onOpenChange, environmentId, onImported }: Props) {
  const [text, setText] = React.useState("");
  const [rows, setRows] = React.useState<ImportPreviewRow[]>([]);
  const [loading, setLoading] = React.useState(false);
  const [importing, setImporting] = React.useState(false);

  React.useEffect(() => {
    if (!open) { setText(""); setRows([]); }
  }, [open]);

  const preview = async () => {
    setLoading(true);
    try {
      const res = await api.previewEnvImport(environmentId, text);
      setRows(res.rows);
    } catch (e) { toast.error(String(e)); }
    finally { setLoading(false); }
  };

  const doImport = async () => {
    setImporting(true);
    try {
      const count = await api.importEnvText(environmentId, text);
      toast.success(`Imported ${count} variable(s)`);
      onImported();
      onOpenChange(false);
    } catch (e) { toast.error(String(e)); }
    finally { setImporting(false); }
  };

  const validCount = rows.filter((r) => r.status !== "invalid").length;

  return (
    <Dialog open={open} onOpenChange={onOpenChange} title="Import from .env" description="Paste .env contents, preview, then import." className="max-w-3xl">
      <div className="space-y-3">
        <textarea
          value={text}
          onChange={(e) => setText(e.target.value)}
          placeholder={"# paste .env here\nAPI_KEY=...\n"}
          className="h-44 w-full border border-input bg-background px-3 py-2 font-mono text-[12px] text-foreground placeholder:text-muted-foreground focus-visible:outline-1 focus-visible:outline-ring"
        />
        <div className="flex items-center gap-2">
          <Button variant="secondary" size="sm" onClick={() => void preview()} disabled={loading || !text.trim()}>
            {loading ? "Parsing..." : "Preview"}
          </Button>
          <Button variant="primary" size="sm" onClick={() => void doImport()} disabled={importing || validCount === 0}>
            {importing ? "Importing..." : `Import ${validCount} variable(s)`}
          </Button>
        </div>

        {rows.length > 0 && (
          <div className="max-h-64 overflow-auto border border-border">
            <table className="w-full border-collapse text-[12px]">
              <thead className="sticky top-0 bg-background">
                <tr className="border-b border-border text-left">
                  <th className="px-3 py-2 font-medium text-muted-foreground">Key</th>
                  <th className="px-3 py-2 font-medium text-muted-foreground">Value</th>
                  <th className="px-3 py-2 font-medium text-muted-foreground">Status</th>
                  <th className="px-3 py-2 font-medium text-muted-foreground">Line</th>
                  <th className="px-3 py-2 font-medium text-muted-foreground">Message</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((r, i) => (
                  <tr key={`${r.line}-${r.key ?? "err"}-${i}`} className="border-b border-border last:border-b-0">
                    <td className="px-3 py-2 font-mono">{r.key ?? "—"}</td>
                    <td className="px-3 py-2 font-mono text-muted-foreground break-all">{r.masked_value || "—"}</td>
                    <td className="px-3 py-2">
                      <span className={cn(
                        "inline-flex border px-1.5 py-0.5 text-[11px] uppercase tracking-[0.04em]",
                        r.status === "new" && "border-success/40 text-success",
                        r.status === "will_overwrite" && "border-warning/40 text-warning",
                        r.status === "invalid" && "border-destructive/40 text-destructive",
                      )}>
                        {r.status.replace(/_/g, " ")}
                      </span>
                    </td>
                    <td className="px-3 py-2 font-mono text-muted-foreground">{r.line}</td>
                    <td className="px-3 py-2 text-muted-foreground">{r.message ?? ""}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </Dialog>
  );
}
