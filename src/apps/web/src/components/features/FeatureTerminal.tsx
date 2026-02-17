import { cn } from "@/lib/utils";

interface FeatureTerminalProps {
  commands: Array<{
    prompt: string;
    command: string;
    output?: string;
    gitBranch?: string;
  }>;
  className?: string;
}

export function FeatureTerminal({
  commands,
  className,
}: FeatureTerminalProps) {
  return (
    <div
      className={cn(
        "relative h-full w-full overflow-hidden border border-[#29292b] bg-[#1B1B1D] rounded-[0.5rem]",
        className,
      )}
    >
      <div className="flex items-center gap-2 border-b border-[#0A0A0B] bg-card/50 px-3 sm:px-4 lg:px-5 py-2 sm:py-2.5 lg:py-3">
        <div className="flex gap-1 sm:gap-1.5">
          <div className="h-2.5 w-2.5 sm:h-3 sm:w-3 rounded-full bg-red-500/80" />
          <div className="h-2.5 w-2.5 sm:h-3 sm:w-3 rounded-full bg-yellow-500/80" />
          <div className="h-2.5 w-2.5 sm:h-3 sm:w-3 rounded-full bg-green-500/80" />
        </div>
        <span className="text-[10px] sm:text-xs text-muted-foreground font-mono">
          àṣẹ
        </span>
      </div>

      <div className="h-full overflow-y-auto overflow-x-hidden p-3 sm:p-4 lg:p-5 font-mono text-xs sm:text-sm scrollbar-thin scrollbar-thumb-border scrollbar-track-transparent space-y-1.5 sm:space-y-2">
        {commands.map((cmd, idx) => (
          <div key={idx} className="space-y-0.5 sm:space-y-1">
            <div className="flex items-center gap-1.5 sm:gap-2 flex-wrap">
              {cmd.gitBranch ? (
                <>
                  <span className="text-green-500 text-xs sm:text-sm">àṣẹ</span>
                  <span className="text-white font-bold text-xs sm:text-sm">
                    &gt;
                  </span>
                  <span className="text-yellow-500 text-xs sm:text-sm">
                    ({cmd.gitBranch})
                  </span>
                </>
              ) : (
                <>
                  <span className="text-green-500 text-xs sm:text-sm">àṣẹ</span>
                  <span className="text-white font-bold text-xs sm:text-sm">
                    &gt;
                  </span>
                </>
              )}
              <span className="text-primary text-xs sm:text-sm">$</span>
              <span className="text-foreground text-xs sm:text-sm break-all">
                {cmd.command}
              </span>
            </div>
            {cmd.output && (
              <div className="text-muted-foreground pl-4 sm:pl-6 whitespace-pre-wrap text-xs sm:text-sm break-words">
                {cmd.output}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
