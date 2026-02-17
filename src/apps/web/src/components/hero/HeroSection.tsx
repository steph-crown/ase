import * as React from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { TerminalAnimation } from "./TerminalAnimation";
import { cn } from "@/lib/utils";

const installationCommands = {
  mac: "brew install ase-shell",
  windows: "iwr https://ase.sh/install.ps1 | iex",
  linux: "curl -fsSL https://ase.sh/install.sh | sh",
  npm: "npm install -g ase-shell",
  cargo: "cargo install ase-shell",
};

export function HeroSection() {
  const [activeTab, setActiveTab] = React.useState("mac");

  const copyToClipboard = async () => {
    const command =
      installationCommands[activeTab as keyof typeof installationCommands];
    try {
      await navigator.clipboard.writeText(command);
      toast.success("Copied to clipboard");
    } catch {
      toast.error("Failed to copy");
    }
  };

  return (
    <div className=" bg-background">
      <div className="wrapper py-5 sm:py-14">
        <div className="grid lg:grid-cols-2 gap-24 items-center">
          {/* Left Column - constrained by wrapper on all screens */}
          <div className="min-w-0 overflow-hidden">
            <div className="">
              <h1 className="text-[40px] sm:text-5xl lg:text-[54px] xl:text-6xl text-foreground leading-[106%] font-air font-medium tracking-[-2%] mb-5">
                Command Your Programs to Work
              </h1>

              <p className="text-sm sm:text-base text-[#999999] font-semibold leading-relaxed max-w-[548px] font-air ">
                <span className="text-primary font-agba text-base sm:text-xl">
                  àṣẹ
                </span>{" "}
                <span className="">("ah-sheh")</span> is a small Unix-style
                shell written in Rust. It gives you a familiar command-line
                experience with builtins, pipelines, history, tab completion,
                and basic expansions, so you can run any (most) thing(s) you'd
                normally do in a shell.
              </p>
            </div>

            <div className="flex gap-4 mt-10 mb-16 sm:mb-20 lg:mb-33">
              <Button className="bg-primary text-white hover:bg-primary/90 py-3 px-5 h-11 text-sm font-medium">
                Get Started
              </Button>

              <Button
                size="lg"
                variant="outline"
                className="border-[#31271E] h-11 py-3 px-5 text-sm font-medium"
              >
                View on GitHub
              </Button>
            </div>

            {/* Installation Card */}
            <Card
              className="bg-[#1B1B1D] border-0 shadow-none rounded-[0.5rem] overflow-hidden w-full max-w-[548px] ring-0 py-0"
              style={{
                boxShadow: "0 2px 4px 2px rgba(0, 0, 0, 0.1)",
              }}
            >
              <CardContent className="p-0 min-w-0">
                <Tabs value={activeTab} onValueChange={setActiveTab}>
                  <div
                    className={cn(
                      "border-b border-[#0A0A0B] min-w-0 overflow-x-auto overflow-y-hidden",
                      "md:overflow-visible",
                      "[scrollbar-width:none] [-ms-overflow-style:none] [&::-webkit-scrollbar]:hidden",
                    )}
                  >
                    <TabsList className="w-max min-w-full justify-start border-b-0 border-transparent bg-transparent p-0 px-5 h-auto gap-4 flex-nowrap">
                      {Object.keys(installationCommands).map((key) => (
                        <TabsTrigger
                          key={key}
                          value={key}
                          className={cn(
                            "shrink-0 px-2 py-3.5 text-sm leading-[20px] font-air font-medium cursor-pointer ",
                            activeTab === key
                              ? "text-[#ccc]"
                              : "text-[#777778]",
                          )}
                        >
                          {key === "mac"
                            ? "Mac"
                            : key.charAt(0).toUpperCase() + key.slice(1)}
                        </TabsTrigger>
                      ))}
                    </TabsList>
                  </div>

                  {Object.entries(installationCommands).map(
                    ([key, command]) => (
                      <TabsContent
                        key={key}
                        value={key}
                        className="mt-0 p-6 min-w-0"
                      >
                        <div className="flex items-center gap-3 font-mono text-sm min-w-0">
                          <span className="text-muted-foreground flex-shrink-0">
                            $
                          </span>
                          <span className="text-primary flex-1 min-w-0 truncate">
                            {command}
                          </span>

                          <button
                            onClick={copyToClipboard}
                            className="text-muted-foreground hover:text-foreground transition-colors p-1 h-10 w-10 border border-[#474747] rounded-[0.375rem] flex items-center justify-center cursor-pointer"
                            aria-label="Copy command"
                          >
                            <svg
                              width="20"
                              height="20"
                              viewBox="0 0 20 20"
                              fill="none"
                              xmlns="http://www.w3.org/2000/svg"
                            >
                              <path
                                d="M14.1666 3.33335H13.3333C13.3333 2.89133 13.1577 2.4674 12.8452 2.15484C12.5326 1.84228 12.1087 1.66669 11.6666 1.66669H8.33331C7.89129 1.66669 7.46736 1.84228 7.1548 2.15484C6.84224 2.4674 6.66665 2.89133 6.66665 3.33335H5.83331C5.17027 3.33335 4.53439 3.59675 4.06555 4.06559C3.59671 4.53443 3.33331 5.17031 3.33331 5.83335V15.8334C3.33331 16.4964 3.59671 17.1323 4.06555 17.6011C4.53439 18.07 5.17027 18.3334 5.83331 18.3334H14.1666C14.8297 18.3334 15.4656 18.07 15.9344 17.6011C16.4033 17.1323 16.6666 16.4964 16.6666 15.8334V5.83335C16.6666 5.17031 16.4033 4.53443 15.9344 4.06559C15.4656 3.59675 14.8297 3.33335 14.1666 3.33335ZM8.33331 3.33335H11.6666V4.16669V5.00002H8.33331V3.33335ZM15 15.8334C15 16.0544 14.9122 16.2663 14.7559 16.4226C14.5996 16.5789 14.3877 16.6667 14.1666 16.6667H5.83331C5.6123 16.6667 5.40034 16.5789 5.24406 16.4226C5.08778 16.2663 4.99998 16.0544 4.99998 15.8334V5.83335C4.99998 5.61234 5.08778 5.40038 5.24406 5.2441C5.40034 5.08782 5.6123 5.00002 5.83331 5.00002H6.66665C6.66665 5.44205 6.84224 5.86597 7.1548 6.17853C7.46736 6.49109 7.89129 6.66669 8.33331 6.66669H11.6666C12.1087 6.66669 12.5326 6.49109 12.8452 6.17853C13.1577 5.86597 13.3333 5.44205 13.3333 5.00002H14.1666C14.3877 5.00002 14.5996 5.08782 14.7559 5.2441C14.9122 5.40038 15 5.61234 15 5.83335V15.8334Z"
                                fill="#999999"
                              />
                            </svg>
                          </button>
                        </div>
                      </TabsContent>
                    ),
                  )}
                </Tabs>
              </CardContent>
            </Card>
          </div>

          {/* Desktop Terminal - inside wrapper, hidden on mobile */}
          <div className="hidden lg:block relative h-[680px]">
            <TerminalAnimation />
          </div>
        </div>
      </div>

      {/* Mobile Terminal - full-bleed, outside wrapper */}
      <div className="relative h-[420px] sm:h-[600px] lg:hidden mt-8">
        <TerminalAnimation />
      </div>
    </div>
  );
}
