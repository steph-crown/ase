import { cn } from "@/lib/utils";

interface TerminalCardProps {
  command: string;
  output: string;
  gitBranch?: string;
  delay?: number;
  className?: string;
  onGround?: boolean;
  x?: number;
  y?: number;
  rotation?: number;
}

export function TerminalCard({
  command,
  output,
  gitBranch,
  delay = 0,
  className,
  onGround = false,
  x = 0,
  y = 0,
  rotation = 0,
}: TerminalCardProps) {
  const prompt = gitBranch
    ? `\x1b[32màṣẹ\x1b[0m \x1b[1m>\x1b[0m \x1b[33m(${gitBranch})\x1b[0m`
    : `\x1b[32màṣẹ\x1b[0m \x1b[1m>\x1b[0m`;

  return (
    <div
      className={cn(
        "rounded-lg border border-border bg-card p-4 shadow-lg w-[300px]",
        className,
      )}
    >
      <div className="mb-2 flex items-center gap-2">
        <div className="flex gap-1.5">
          <div className="h-3 w-3 rounded-full bg-red-500/80" />
          <div className="h-3 w-3 rounded-full bg-yellow-500/80" />
          <div className="h-3 w-3 rounded-full bg-green-500/80" />
        </div>
        <span className="text-xs text-muted-foreground font-mono">àṣẹ</span>
      </div>
      <div className="space-y-1 font-mono text-sm">
        <div className="flex items-center gap-2">
          {gitBranch ? (
            <>
              <span className="text-green-500">àṣẹ</span>
              <span className="text-white font-bold">&gt;</span>
              <span className="text-yellow-500">({gitBranch})</span>
            </>
          ) : (
            <>
              <span className="text-green-500">àṣẹ</span>
              <span className="text-white font-bold">&gt;</span>
            </>
          )}
        </div>
        <div className="flex items-center gap-2">
          <span className="text-primary">$</span>
          <span className="text-foreground">{command}</span>
        </div>
        <div className="text-muted-foreground pl-6">{output}</div>
      </div>
    </div>
  );
}
