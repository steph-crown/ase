import * as React from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { TerminalAnimation } from "./TerminalAnimation";
import { Copy } from "lucide-react";

const installationCommands = {
  mac: "brew install ase-shell",
  windows: "iwr https://ase.sh/install.ps1 | iex",
  linux: "curl -fsSL https://ase.sh/install.sh | sh",
  npm: "npm install -g ase-shell",
  cargo: "cargo install ase-shell",
};

export function HeroSection() {
  const [activeTab, setActiveTab] = React.useState("mac");
  const [copied, setCopied] = React.useState(false);

  const copyToClipboard = () => {
    const command =
      installationCommands[activeTab as keyof typeof installationCommands];
    navigator.clipboard.writeText(command);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className=" min-h-screen bg-background">
      <div className="wrapper py-14">
        <div className="grid lg:grid-cols-2 gap-24 items-center">
          {/* Left Column */}
          <div className="">
            <div className="">
              <h1 className="text-5xl lg:text-6xl text-foreground leading-16 font-air font-medium tracking-[-2%] mb-5">
                Command Your Programs to Work
              </h1>

              <p className="text-base text-[#999999] font-semibold leading-relaxed max-w-[548px] font-air ">
                <span className="text-primary font-agba text-xl">àṣẹ</span>{" "}
                <span className="">("ah-sheh")</span> is a small Unix-style
                shell written in Rust. It gives you a familiar command-line
                experience with builtins, pipelines, history, tab completion,
                and basic expansions, so you can run any (most) thing(s) you'd
                normally do in a shell.
              </p>
            </div>

            <div className="flex gap-4 mt-10 mb-33">
              <Button className="bg-primary text-white hover:bg-primary/90 py-3 px-5 h-11">
                Get Started
              </Button>

              <Button
                size="lg"
                variant="outline"
                className="border-[#31271E] h-11 py-3 px-5"
              >
                View on GitHub
              </Button>
            </div>

            {/* Installation Card */}
            <Card className="bg-[#1B1B1D] border-border rounded-[0.5rem] overflow-hidden max-w-[548px]">
              <CardContent className="p-0">
                <Tabs value={activeTab} onValueChange={setActiveTab}>
                  <TabsList className="w-full justify-start border-b border-border bg-transparent h-auto p-0">
                    {Object.keys(installationCommands).map((key) => (
                      <TabsTrigger
                        key={key}
                        value={key}
                        className="capitalize px-6 py-3"
                      >
                        {key === "mac"
                          ? "Mac"
                          : key.charAt(0).toUpperCase() + key.slice(1)}
                      </TabsTrigger>
                    ))}
                  </TabsList>

                  {Object.entries(installationCommands).map(
                    ([key, command]) => (
                      <TabsContent key={key} value={key} className="mt-0 p-6">
                        <div className="flex items-center gap-3 font-mono text-sm">
                          <span className="text-muted-foreground">$</span>
                          <span className="text-primary flex-1">{command}</span>
                          <button
                            onClick={copyToClipboard}
                            className="text-muted-foreground hover:text-foreground transition-colors p-1"
                            aria-label="Copy command"
                          >
                            <Copy className="h-4 w-4" />
                          </button>
                        </div>
                      </TabsContent>
                    ),
                  )}
                </Tabs>
              </CardContent>
            </Card>
          </div>

          {/* Right Column - Animation */}
          <div className="relative h-[600px] lg:h-[700px] ">
            <TerminalAnimation />
          </div>
        </div>
      </div>
    </div>
  );
}
