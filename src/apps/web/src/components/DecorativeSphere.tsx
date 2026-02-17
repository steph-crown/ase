import { useState } from "react";
import { cn } from "@/lib/utils";

export function DecorativeSphere() {
  const [isHovered, setIsHovered] = useState(false);

  return (
    <div className="relative w-full flex items-center justify-center py-16 sm:py-20 lg:py-24 overflow-hidden">
      {/* Sphere Container */}
      <div
        className={cn(
          "relative w-48 h-48 sm:w-64 sm:h-64 lg:w-80 lg:h-80",
          "transition-all duration-500 ease-out",
          isHovered && "scale-110"
        )}
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
      >
        {/* Outer Glow */}
        <div
          className={cn(
            "absolute inset-0 rounded-full",
            "bg-gradient-to-br from-primary/20 via-primary/10 to-primary/5",
            "blur-xl",
            "transition-all duration-500",
            isHovered && "blur-2xl scale-110"
          )}
        />

        {/* Main Sphere */}
        <div
          className={cn(
            "relative w-full h-full rounded-full",
            "bg-gradient-to-br from-primary/30 via-primary/20 to-primary/10",
            "border-2 border-primary/30",
            "transition-all duration-500",
            isHovered && "border-primary/50 shadow-2xl shadow-primary/20"
          )}
          style={{
            transform: isHovered
              ? "rotateY(15deg) rotateX(10deg)"
              : "rotateY(0deg) rotateX(0deg)",
          }}
        >
          {/* African Pattern Overlay - Adinkra-inspired geometric patterns */}
          <svg
            className="absolute inset-0 w-full h-full"
            viewBox="0 0 200 200"
            xmlns="http://www.w3.org/2000/svg"
          >
            <defs>
              <pattern
                id="african-pattern"
                x="0"
                y="0"
                width="40"
                height="40"
                patternUnits="userSpaceOnUse"
              >
                {/* Geometric diamond pattern */}
                <path
                  d="M20,0 L30,10 L20,20 L10,10 Z"
                  fill="none"
                  stroke="#fa912a"
                  strokeWidth="0.5"
                  opacity="0.4"
                />
                <circle cx="20" cy="10" r="2" fill="#fa912a" opacity="0.3" />
              </pattern>
              <radialGradient id="sphere-gradient">
                <stop offset="0%" stopColor="#fa912a" stopOpacity="0.4" />
                <stop offset="50%" stopColor="#fa912a" stopOpacity="0.2" />
                <stop offset="100%" stopColor="#fa912a" stopOpacity="0.05" />
              </radialGradient>
            </defs>

            {/* Background Pattern */}
            <rect
              width="200"
              height="200"
              fill="url(#african-pattern)"
              className={cn(
                "transition-all duration-700",
                isHovered && "rotate-180"
              )}
            />

            {/* Central Adinkra Symbol - Sankofa (bird looking back) */}
            <g
              className={cn(
                "transition-all duration-500 origin-center",
                isHovered ? "scale-110 rotate-12" : "animate-float"
              )}
            >
              {/* Sankofa-inspired pattern */}
              <path
                d="M100,60 Q80,80 70,100 Q80,120 100,140 Q120,120 130,100 Q120,80 100,60 Z"
                fill="none"
                stroke="#fa912a"
                strokeWidth="1.5"
                opacity="0.6"
                className={cn(
                  "transition-all duration-500",
                  isHovered && "opacity-0.8 stroke-[2px]"
                )}
              />
              {/* Inner circle */}
              <circle
                cx="100"
                cy="100"
                r="25"
                fill="none"
                stroke="#fa912a"
                strokeWidth="1"
                opacity="0.5"
                className={cn(
                  "transition-all duration-500",
                  isHovered && "opacity-0.7 stroke-[1.5px]"
                )}
              />
              {/* Decorative dots */}
              <circle
                cx="100"
                cy="75"
                r="2"
                fill="#fa912a"
                opacity="0.6"
                className={cn(
                  "transition-all duration-500",
                  isHovered && "opacity-1 scale-150"
                )}
              />
              <circle
                cx="100"
                cy="125"
                r="2"
                fill="#fa912a"
                opacity="0.6"
                className={cn(
                  "transition-all duration-500",
                  isHovered && "opacity-1 scale-150"
                )}
              />
              <circle
                cx="75"
                cy="100"
                r="2"
                fill="#fa912a"
                opacity="0.6"
                className={cn(
                  "transition-all duration-500",
                  isHovered && "opacity-1 scale-150"
                )}
              />
              <circle
                cx="125"
                cy="100"
                r="2"
                fill="#fa912a"
                opacity="0.6"
                className={cn(
                  "transition-all duration-500",
                  isHovered && "opacity-1 scale-150"
                )}
              />
            </g>

            {/* Decorative Geometric Shapes */}
            {/* Top left pattern */}
            <g
              className={cn(
                "transition-all duration-700 origin-top-left",
                isHovered && "rotate-45 scale-110"
              )}
            >
              <path
                d="M30,30 L50,30 L50,50 L30,50 Z"
                fill="none"
                stroke="#fa912a"
                strokeWidth="0.8"
                opacity="0.3"
              />
              <circle cx="40" cy="40" r="3" fill="#fa912a" opacity="0.2" />
            </g>

            {/* Top right pattern */}
            <g
              className={cn(
                "transition-all duration-700 origin-top-right",
                isHovered && "-rotate-45 scale-110"
              )}
            >
              <path
                d="M150,30 L170,30 L170,50 L150,50 Z"
                fill="none"
                stroke="#fa912a"
                strokeWidth="0.8"
                opacity="0.3"
              />
              <circle cx="160" cy="40" r="3" fill="#fa912a" opacity="0.2" />
            </g>

            {/* Bottom left pattern */}
            <g
              className={cn(
                "transition-all duration-700 origin-bottom-left",
                isHovered && "-rotate-45 scale-110"
              )}
            >
              <path
                d="M30,150 L50,150 L50,170 L30,170 Z"
                fill="none"
                stroke="#fa912a"
                strokeWidth="0.8"
                opacity="0.3"
              />
              <circle cx="40" cy="160" r="3" fill="#fa912a" opacity="0.2" />
            </g>

            {/* Bottom right pattern */}
            <g
              className={cn(
                "transition-all duration-700 origin-bottom-right",
                isHovered && "rotate-45 scale-110"
              )}
            >
              <path
                d="M150,150 L170,150 L170,170 L150,170 Z"
                fill="none"
                stroke="#fa912a"
                strokeWidth="0.8"
                opacity="0.3"
              />
              <circle cx="160" cy="160" r="3" fill="#fa912a" opacity="0.2" />
            </g>

            {/* Orbiting decorative elements */}
            <g
              className={cn(
                "transition-all duration-1000 origin-center",
                isHovered ? "rotate-180" : "animate-spin-slow"
              )}
            >
              <circle cx="100" cy="40" r="1.5" fill="#fa912a" opacity="0.4" />
              <circle cx="160" cy="100" r="1.5" fill="#fa912a" opacity="0.4" />
              <circle cx="100" cy="160" r="1.5" fill="#fa912a" opacity="0.4" />
              <circle cx="40" cy="100" r="1.5" fill="#fa912a" opacity="0.4" />
            </g>
          </svg>

          {/* Shimmer effect */}
          <div
            className={cn(
              "absolute inset-0 rounded-full",
              "bg-gradient-to-r from-transparent via-primary/10 to-transparent",
              "transition-all duration-1000",
              isHovered
                ? "animate-shimmer opacity-100"
                : "animate-shimmer-slow opacity-50"
            )}
            style={{
              transform: "rotate(45deg)",
            }}
          />
        </div>

        {/* Floating particles around sphere */}
        {[...Array(6)].map((_, i) => (
          <div
            key={i}
            className={cn(
              "absolute rounded-full bg-primary/30",
              "transition-all duration-500",
              isHovered && "bg-primary/50 scale-150"
            )}
            style={{
              width: "4px",
              height: "4px",
              left: `${50 + 45 * Math.cos((i * Math.PI * 2) / 6)}%`,
              top: `${50 + 45 * Math.sin((i * Math.PI * 2) / 6)}%`,
              animation: `float-particle-${i} ${3 + i * 0.5}s ease-in-out infinite`,
              animationDelay: `${i * 0.3}s`,
            }}
          />
        ))}
      </div>
    </div>
  );
}
