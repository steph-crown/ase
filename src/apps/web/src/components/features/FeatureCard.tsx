import { FeatureTerminal } from "./FeatureTerminal";
import { cn } from "@/lib/utils";

interface FeatureCardProps {
  title: string;
  description: string;
  commands: Array<{
    prompt: string;
    command: string;
    output?: string;
    gitBranch?: string;
  }>;
  className?: string;
}

export function FeatureCard({
  title,
  description,
  commands,
  className,
}: FeatureCardProps) {
  return (
    <div className={cn("flex flex-col gap-6", className)}>
      {/* Title and Description */}
      <div className="space-y-2 sm:space-y-3">
        <h3 className="text-xl sm:text-2xl lg:text-3xl font-medium text-foreground font-air leading-tight">
          {title}
        </h3>
        <p className="text-xs sm:text-sm lg:text-base text-[#999999] font-semibold leading-relaxed font-air">
          {description}
        </p>
      </div>

      {/* Terminal Container with Animated Patterned Background */}
      <div className="relative w-full h-[280px] sm:h-[380px] lg:h-[450px] rounded-[0.5rem] overflow-hidden">
        {/* Animated Patterned Background */}
        <div className="absolute inset-0 bg-gradient-to-br from-primary/20 via-primary/10 to-primary/5">
          {/* Animated Wave Pattern */}
          <div className="absolute inset-0 opacity-30">
            <svg
              className="absolute inset-0 w-full h-full"
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 1200 600"
              preserveAspectRatio="none"
            >
              <defs>
                <linearGradient
                  id={`wave-gradient-${title.replace(/\s+/g, "-").toLowerCase()}`}
                  x1="0%"
                  y1="0%"
                  x2="100%"
                  y2="100%"
                >
                  <stop offset="0%" stopColor="#fa912a" stopOpacity="0.4" />
                  <stop offset="50%" stopColor="#fa912a" stopOpacity="0.2" />
                  <stop offset="100%" stopColor="#fa912a" stopOpacity="0.1" />
                </linearGradient>
              </defs>
              {/* Animated Wave 1 */}
              <path
                d="M0,300 Q300,200 600,300 T1200,300 L1200,600 L0,600 Z"
                fill={`url(#wave-gradient-${title.replace(/\s+/g, "-").toLowerCase()})`}
                className="animate-wave-1"
              />
              {/* Animated Wave 2 */}
              <path
                d="M0,400 Q400,300 800,400 T1200,400 L1200,600 L0,600 Z"
                fill={`url(#wave-gradient-${title.replace(/\s+/g, "-").toLowerCase()})`}
                className="animate-wave-2"
                opacity="0.6"
              />
              {/* Animated Wave 3 */}
              <path
                d="M0,500 Q500,400 1000,500 T1200,500 L1200,600 L0,600 Z"
                fill={`url(#wave-gradient-${title.replace(/\s+/g, "-").toLowerCase()})`}
                className="animate-wave-3"
                opacity="0.4"
              />
            </svg>
          </div>

          {/* Animated Stripes Pattern */}
          <div className="absolute inset-0 opacity-20">
            <div
              className="absolute inset-0"
              style={{
                backgroundImage: `repeating-linear-gradient(
                  45deg,
                  transparent,
                  transparent 20px,
                  rgba(250, 145, 42, 0.1) 20px,
                  rgba(250, 145, 42, 0.1) 40px
                )`,
                animation: "slide-stripes 20s linear infinite",
                backgroundSize: "40px 40px",
              }}
            />
          </div>
        </div>

        {/* Terminal - ~10px offset on mobile, ~40px on desktop */}
        <div className="absolute inset-0 flex items-center justify-center p-2.5 lg:p-10">
          <div className="w-full h-full">
            <FeatureTerminal commands={commands} />
          </div>
        </div>
      </div>
    </div>
  );
}
