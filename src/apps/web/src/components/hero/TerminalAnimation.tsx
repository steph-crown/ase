import { motion, AnimatePresence } from "framer-motion";
import { useEffect, useState, useRef } from "react";

interface CommandEntry {
  id: number;
  prompt: string;
  command: string;
  output: string;
  gitBranch?: string;
  typedCommand: string;
  isTyping: boolean;
  showOutput: boolean;
  isActive: boolean;
}

const terminalCommands = [
  { command: "cd ~/projects", output: "~/projects", gitBranch: "main" },
  {
    command: "ls -la",
    output:
      "total 24\ndrwxr-xr-x  5 user  staff  160 Jan 31 10:00 .\ndrwxr-xr-x  3 user  staff   96 Jan 30 14:00 ..",
    gitBranch: "main",
  },
  {
    command: "git status",
    output: "On branch main\nYour branch is up to date with 'origin/main'.",
    gitBranch: "main",
  },
  { command: "echo 'Hello àṣẹ'", output: "Hello àṣẹ" },
  { command: "pwd", output: "/Users/user/projects" },
  { command: "cd ~/docs", output: "~/docs" },
  {
    command: "history | tail -5",
    output:
      "  1  cd ~/projects\n  2  ls -la\n  3  git status\n  4  echo 'Hello àṣẹ'\n  5  history | tail -5",
  },
  {
    command: "cd ~/projects/ase",
    output: "~/projects/ase",
    gitBranch: "feature/new-feature",
  },
  {
    command: "git branch",
    output: "* feature/new-feature\n  main\n  dev",
    gitBranch: "feature/new-feature",
  },
  { command: "ls *.rs | head -3", output: "main.rs\nlib.rs\ncommands.rs" },
  {
    command: "echo 'Building...' && cargo build",
    output:
      "Building...\n   Compiling ase-shell v0.2.0\n    Finished dev [unoptimized + debuginfo] target(s)",
    gitBranch: "main",
  },
  {
    command: "test -f Cargo.toml && echo 'Found!' || echo 'Not found'",
    output: "Found!",
  },
  {
    command: "cd src; ls -1",
    output: "commands\nlib.rs\nmain.rs\nrepl.rs\nutils.rs",
  },
  {
    command: "echo 'Error' > error.log && cat error.log",
    output: "Error",
  },
  {
    command: "echo 'Appended' >> log.txt; cat log.txt",
    output: "Line 1\nLine 2\nAppended",
  },
  {
    command: "ls nonexistent 2> /dev/null || echo 'File not found'",
    output: "File not found",
  },
  {
    command: "echo 'Output' > file.txt && echo 'Success' && cat file.txt",
    output: "Success\nOutput",
  },
  {
    command: "cd ~/projects && pwd && ls | grep -E '^[a-z]'",
    output: "~/projects\nase\ndocs\nrust",
  },
  {
    command: "history | grep 'cd' | tail -3",
    output: "  5  cd ~/projects\n  8  cd ~/projects/ase\n  12  cd ~/docs",
  },
  {
    command: "echo 'Test' > test.txt; cat test.txt; rm test.txt",
    output: "Test",
  },
  {
    command: "type cd && type ls",
    output: "cd is a shell builtin\nls is /usr/bin/ls",
  },
  {
    command: "cd ~/projects/ase && git status && echo 'Ready to commit'",
    output: "On branch main\nChanges not staged for commit\nReady to commit",
    gitBranch: "main",
  },
  {
    command: "ls -la | grep '^d' | wc -l",
    output: "3",
  },
  {
    command:
      "echo 'Line 1' > output.txt; echo 'Line 2' >> output.txt; cat output.txt",
    output: "Line 1\nLine 2",
  },
  {
    command: "pwd; cd ..; pwd; cd -",
    output: "/Users/user/projects\n/Users/user\n/Users/user/projects",
  },
  {
    command: "test -d src && echo 'Directory exists' || echo 'Not a directory'",
    output: "Directory exists",
  },
  {
    command: "echo 'Error message' 2>> errors.log; cat errors.log",
    output: "Previous error\nError message",
  },
];

export function TerminalAnimation() {
  const [commands, setCommands] = useState<CommandEntry[]>([]);
  const [isPaused, setIsPaused] = useState(false);
  const [currentCommandIndex, setCurrentCommandIndex] = useState(0);
  const commandIdRef = useRef(0);
  const typingTimeoutRef = useRef<NodeJS.Timeout>();
  const cycleTimeoutRef = useRef<NodeJS.Timeout>();
  const isTypingRef = useRef(false);
  const scrollContainerRef = useRef<HTMLDivElement>(null);

  // Handle command progression
  useEffect(() => {
    if (isPaused) return;

    const startNextCommand = () => {
      // Loop back to first command (circular flow)
      const cmdIndex = currentCommandIndex % terminalCommands.length;
      const cmd = terminalCommands[cmdIndex];
      const prompt = cmd.gitBranch ? `àṣẹ > (${cmd.gitBranch})` : `àṣẹ >`;

      // Create new command entry
      const newCommand: CommandEntry = {
        id: commandIdRef.current++,
        prompt,
        command: cmd.command,
        output: cmd.output,
        gitBranch: cmd.gitBranch,
        typedCommand: "",
        isTyping: true,
        showOutput: false,
        isActive: true,
      };

      isTypingRef.current = true;
      setCommands((prev) => [
        ...prev.map((c) => ({ ...c, isActive: false })),
        newCommand,
      ]);

      // Type command character by character
      let charIndex = 0;
      const typeNextChar = () => {
        if (isPaused) {
          if (typingTimeoutRef.current) clearTimeout(typingTimeoutRef.current);
          return;
        }

        if (charIndex < cmd.command.length) {
          setCommands((prev) =>
            prev.map((c) =>
              c.id === newCommand.id
                ? { ...c, typedCommand: cmd.command.slice(0, charIndex + 1) }
                : c,
            ),
          );
          charIndex++;
          typingTimeoutRef.current = setTimeout(
            typeNextChar,
            50 + Math.random() * 30,
          );
        } else {
          // Command typed, show output after delay
          setTimeout(() => {
            setCommands((prev) =>
              prev.map((c) =>
                c.id === newCommand.id
                  ? { ...c, isTyping: false, showOutput: true, isActive: false }
                  : c,
              ),
            );
            isTypingRef.current = false;

            // Move to next command after showing output
            setTimeout(() => {
              setCurrentCommandIndex(
                (prev) => (prev + 1) % terminalCommands.length,
              );
            }, 2000);
          }, 300);
        }
      };

      typeNextChar();
    };

    // Check if we need to start a new command
    if (!isTypingRef.current) {
      cycleTimeoutRef.current = setTimeout(startNextCommand, 800);
    }

    return () => {
      if (typingTimeoutRef.current) clearTimeout(typingTimeoutRef.current);
      if (cycleTimeoutRef.current) clearTimeout(cycleTimeoutRef.current);
    };
  }, [currentCommandIndex, isPaused]);

  // Auto-scroll to bottom when new content is added
  useEffect(() => {
    if (scrollContainerRef.current && !isPaused) {
      scrollContainerRef.current.scrollTo({
        top: scrollContainerRef.current.scrollHeight,
        behavior: "smooth",
      });
    }
  }, [commands, isPaused]);

  // Remove old commands (keep only last 8 for better visibility)
  useEffect(() => {
    if (commands.length > 8) {
      setCommands((prev) => prev.slice(-8));
    }
  }, [commands.length]);

  return (
    <div
      className="relative h-full w-full overflow-hidden rounded-lg border border-border bg-card"
      onMouseEnter={() => setIsPaused(true)}
      onMouseLeave={() => setIsPaused(false)}
    >
      {/* Terminal header */}
      <div className="flex items-center gap-2 border-b border-border bg-card/50 px-4 py-2">
        <div className="flex gap-1.5">
          <div className="h-3 w-3 rounded-full bg-red-500/80" />
          <div className="h-3 w-3 rounded-full bg-yellow-500/80" />
          <div className="h-3 w-3 rounded-full bg-green-500/80" />
        </div>
        <span className="text-xs text-muted-foreground font-mono">àṣẹ</span>
        {isPaused && (
          <motion.span
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="ml-auto text-xs text-muted-foreground"
          >
            Paused
          </motion.span>
        )}
      </div>

      {/* Terminal content */}
      <div
        ref={scrollContainerRef}
        className="h-full overflow-y-auto overflow-x-hidden p-4 font-mono text-sm scrollbar-thin scrollbar-thumb-border scrollbar-track-transparent"
        style={{ scrollbarWidth: "thin" }}
      >
        <div className="flex flex-col gap-3 pb-8">
          <AnimatePresence mode="popLayout">
            {commands.map((cmd) => (
              <motion.div
                key={cmd.id}
                initial={{ opacity: 0, y: -20 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: 20 }}
                transition={{ duration: 0.3 }}
                className={`relative ${cmd.isActive ? "ring-2 ring-primary/30 rounded px-2 py-1 -mx-2 -my-1" : ""}`}
              >
                {/* Prompt */}
                <div className="flex items-center gap-2 mb-1">
                  {cmd.gitBranch ? (
                    <>
                      <span className="text-green-500">àṣẹ</span>
                      <span className="text-white font-bold">&gt;</span>
                      <span className="text-yellow-500">({cmd.gitBranch})</span>
                    </>
                  ) : (
                    <>
                      <span className="text-green-500">àṣẹ</span>
                      <span className="text-white font-bold">&gt;</span>
                    </>
                  )}
                </div>

                {/* Command being typed */}
                <div className="flex items-center gap-2">
                  <span className="text-primary">$</span>
                  <span className="text-foreground">
                    {cmd.typedCommand || (cmd.isTyping ? "" : cmd.command)}
                  </span>
                  {cmd.isTyping && (
                    <motion.span
                      animate={{ opacity: [1, 0] }}
                      transition={{
                        duration: 0.8,
                        repeat: Infinity,
                        ease: "easeInOut",
                      }}
                      className="inline-block w-2 h-4 bg-primary ml-1"
                    />
                  )}
                </div>

                {/* Output */}
                {cmd.showOutput && (
                  <motion.div
                    initial={{ opacity: 0, y: -10 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.4, delay: 0.1 }}
                    className="text-muted-foreground mt-2 pl-6 whitespace-pre-wrap"
                  >
                    {cmd.output}
                  </motion.div>
                )}
              </motion.div>
            ))}
          </AnimatePresence>
        </div>
      </div>

      {/* Subtle glow effect on active command */}
      {commands.some((c) => c.isActive) && (
        <motion.div
          className="absolute inset-0 pointer-events-none"
          initial={{ opacity: 0 }}
          animate={{ opacity: [0.3, 0.5, 0.3] }}
          transition={{
            duration: 2,
            repeat: Infinity,
            ease: "easeInOut",
          }}
          style={{
            background: `radial-gradient(circle at center, rgba(250, 145, 42, 0.1) 0%, transparent 70%)`,
          }}
        />
      )}
    </div>
  );
}
