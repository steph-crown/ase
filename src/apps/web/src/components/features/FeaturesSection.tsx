import { FeatureCard } from "./FeatureCard";

const features = [
  {
    title: "Essential Builtins",
    description:
      "Navigate directories, inspect commands, and manage your session with familiar builtins like `cd`, `pwd`, `type`, and `exit`.",
    commands: [
      {
        prompt: "àṣẹ",
        command: "cd ~/projects",
        gitBranch: "main",
      },
      {
        prompt: "àṣẹ",
        command: "pwd",
        output: "/Users/user/projects",
        gitBranch: "main",
      },
      {
        prompt: "àṣẹ",
        command: "type cd ls",
        output: "cd is a shell builtin\nls is /usr/bin/ls",
        gitBranch: "main",
      },
      {
        prompt: "àṣẹ",
        command: "exit 0",
        gitBranch: "main",
      },
    ],
  },
  {
    title: "Powerful Command Composition",
    description:
      "Chain commands with pipelines (`|`), control flow (`&&`, `||`), and sequential execution (`;`) to build complex workflows.",
    commands: [
      {
        prompt: "àṣẹ",
        command: "ls | grep .rs | wc -l",
        output: "42",
        gitBranch: "main",
      },
      {
        prompt: "àṣẹ",
        command: "cargo build && echo 'Build succeeded!'",
        output:
          "   Compiling ase-shell v0.2.0\n    Finished dev target(s)\nBuild succeeded!",
        gitBranch: "main",
      },
      {
        prompt: "àṣẹ",
        command: "test -f Cargo.toml || echo 'Not found'",
        gitBranch: "main",
      },
    ],
  },
  {
    title: "Flexible I/O Redirection",
    description:
      "Redirect stdout and stderr with `>`, `>>`, `2>`, and `2>>`. Supports tilde expansion and environment variables in paths.",
    commands: [
      {
        prompt: "àṣẹ",
        command: "echo 'Hello' > ~/output.txt",
        gitBranch: "main",
      },
      {
        prompt: "àṣẹ",
        command: "echo 'World' >> ~/output.txt",
        gitBranch: "main",
      },
      {
        prompt: "àṣẹ",
        command: "cat ~/output.txt",
        output: "Hello\nWorld",
        gitBranch: "main",
      },
      {
        prompt: "àṣẹ",
        command: "ls nonexistent 2> $HOME/errors.log",
        gitBranch: "main",
      },
    ],
  },
  {
    title: "Smart Interactive Features",
    description:
      "Tab completion, command history navigation, colorful prompts with Git branch detection, and multi-line input support.",
    commands: [
      {
        prompt: "àṣẹ",
        command: 'echo "unclosed quote',
        gitBranch: "main",
      },
      {
        prompt: "àṣẹ",
        command: "still typing...\"",
        output: 'unclosed quote\nstill typing...',
        gitBranch: "main",
      },
      {
        prompt: "àṣẹ",
        command: "history 5",
        output:
          "    1  cd ~/projects\n    2  ls -la\n    3  git status\n    4  echo 'Hello àṣẹ'\n    5  history 5",
        gitBranch: "main",
      },
      {
        prompt: "àṣẹ",
        command: "cd ~/pro[TAB]",
        gitBranch: "main",
      },
    ],
  },
];

export function FeaturesSection() {
  return (
    <section className="py-16 sm:py-24 lg:py-32 bg-background">
      <div className="wrapper">
        {/* Section Header */}
        <div className="text-center mb-10 sm:mb-12 lg:mb-16">
          <h2 className="text-3xl sm:text-4xl lg:text-5xl xl:text-6xl font-medium text-foreground leading-[106%] font-air tracking-[-2%] mb-3 sm:mb-4 px-4 sm:px-0">
            Everything You Need
          </h2>
          <p className="text-sm sm:text-base lg:text-lg text-[#999999] font-semibold max-w-2xl mx-auto font-air px-4 sm:px-0">
            A complete Unix-style shell experience with all the features you
            expect, built for modern development workflows.
          </p>
        </div>

        {/* Features Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8 sm:gap-10 lg:gap-12">
          {features.map((feature, idx) => (
            <FeatureCard
              key={idx}
              title={feature.title}
              description={feature.description}
              commands={feature.commands}
            />
          ))}
        </div>
      </div>
    </section>
  );
}
