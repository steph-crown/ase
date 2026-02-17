import { motion, AnimatePresence } from "framer-motion";
import { useEffect, useState, useRef } from "react";
import { TerminalCard } from "./TerminalCard";

interface CardData {
  id: number;
  command: string;
  output: string;
  gitBranch?: string;
  x: number;
  y: number;
  rotation: number;
  onGround: boolean;
  createdAt: number;
}

const terminalCommands = [
  { command: "cd ~/projects", output: "~/projects", gitBranch: "main" },
  { command: "ls -la", output: "total 24", gitBranch: "main" },
  { command: "git status", output: "On branch main", gitBranch: "main" },
  { command: "echo 'Hello àṣẹ'", output: "Hello àṣẹ" },
  { command: "pwd", output: "/Users/user/projects" },
  { command: "cd ~/docs", output: "~/docs" },
  { command: "ls", output: "file1.txt file2.md" },
  { command: "history | tail -5", output: "5 commands shown" },
  { command: "cd /tmp", output: "/tmp" },
  { command: "echo $HOME", output: "/Users/user" },
  {
    command: "cd ~/projects/ase",
    output: "~/projects/ase",
    gitBranch: "feature/new-feature",
  },
  {
    command: "git branch",
    output: "* feature/new-feature",
    gitBranch: "feature/new-feature",
  },
  { command: "cd ~/workspace", output: "~/workspace" },
  { command: "ls -la | head -5", output: "5 items shown" },
  {
    command: "cd ~/projects/rust",
    output: "~/projects/rust",
    gitBranch: "dev",
  },
];

const GROUND_Y = 500;
const CARD_WIDTH = 300;
const CARD_HEIGHT = 150;
const CONTAINER_WIDTH = 800;

function getRandomX() {
  return Math.random() * (CONTAINER_WIDTH - CARD_WIDTH);
}

function getRandomRotation() {
  return (Math.random() - 0.5) * 8;
}

function findGroundY(cards: CardData[], x: number): number {
  const cardsInColumn = cards
    .filter(
      (card) =>
        card.onGround && card.x < x + CARD_WIDTH && card.x + CARD_WIDTH > x,
    )
    .sort((a, b) => b.y - a.y);

  if (cardsInColumn.length === 0) {
    return GROUND_Y;
  }

  const topCard = cardsInColumn[0];
  return topCard.y - CARD_HEIGHT - 10;
}

export function TerminalAnimation() {
  const [cards, setCards] = useState<CardData[]>([]);
  const cardIdRef = useRef(0);
  const intervalRef = useRef<NodeJS.Timeout>();

  useEffect(() => {
    const spawnCard = () => {
      const cmd =
        terminalCommands[Math.floor(Math.random() * terminalCommands.length)];
      const x = getRandomX();
      const rotation = getRandomRotation();

      setCards((prev) => {
        const newCard: CardData = {
          id: cardIdRef.current++,
          command: cmd.command,
          output: cmd.output,
          gitBranch: cmd.gitBranch,
          x,
          y: -100,
          rotation,
          onGround: false,
          createdAt: Date.now(),
        };

        setTimeout(() => {
          setCards((current) => {
            const groundY = findGroundY(
              current.filter((c) => c.id !== newCard.id),
              x,
            );
            return current.map((c) =>
              c.id === newCard.id ? { ...c, onGround: true, y: groundY } : c,
            );
          });
        }, 100);

        return [...prev, newCard];
      });
    };

    intervalRef.current = setInterval(spawnCard, 1500);

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, []);

  useEffect(() => {
    const cleanup = setInterval(() => {
      setCards((current) => {
        const now = Date.now();
        const oldCards = current.filter(
          (card) => card.onGround && now - card.createdAt > 8000,
        );

        if (oldCards.length === 0) return current;

        const oldCardIds = new Set(oldCards.map((c) => c.id));

        return current
          .filter((card) => !oldCardIds.has(card.id))
          .map((card) => {
            if (!card.onGround) {
              const cardsAbove = current.filter(
                (c) =>
                  c.onGround &&
                  c.x < card.x + CARD_WIDTH &&
                  c.x + CARD_WIDTH > card.x &&
                  c.y < card.y &&
                  oldCardIds.has(c.id),
              );

              if (cardsAbove.length > 0) {
                const newGroundY = findGroundY(
                  current.filter(
                    (c) => c.id !== card.id && !oldCardIds.has(c.id),
                  ),
                  card.x,
                );
                return { ...card, y: newGroundY, onGround: true };
              }
            }
            return card;
          });
      });
    }, 500);

    return () => clearInterval(cleanup);
  }, []);

  return (
    <div className="relative h-full w-full overflow-hidden">
      {/* Ground with grass effect */}
      <div className="absolute bottom-0 left-0 right-0 h-32 bg-gradient-to-t from-[#8B6914] via-[#9D7A1F] to-[#A6852A]">
        <div className="absolute top-0 left-0 right-0 h-8 bg-gradient-to-b from-transparent via-[#7A5A0F]/50 to-[#8B6914]">
          {/* Grass blades */}
          {Array.from({ length: 50 }).map((_, i) => (
            <div
              key={i}
              className="absolute bottom-0 w-1 bg-[#5A4A0F]"
              style={{
                left: `${(i * 100) / 50}%`,
                height: `${10 + Math.random() * 15}px`,
                transform: `rotate(${(Math.random() - 0.5) * 30}deg)`,
                transformOrigin: "bottom",
              }}
            />
          ))}
        </div>
      </div>

      {/* Falling and stacked cards */}
      <AnimatePresence>
        {cards.map((card) => (
          <TerminalCard
            key={card.id}
            command={card.command}
            output={card.output}
            gitBranch={card.gitBranch}
            onGround={card.onGround}
            x={card.x}
            y={card.y}
            rotation={card.rotation}
            className="origin-center"
          />
        ))}
      </AnimatePresence>

      {/* Dancing animation for cards on ground */}
      <AnimatePresence>
        {cards
          .filter((card) => card.onGround)
          .map((card) => (
            <motion.div
              key={`dance-${card.id}`}
              initial={false}
              animate={{
                y: [0, -3, 0],
                rotate: [card.rotation, card.rotation + 1, card.rotation],
              }}
              transition={{
                duration: 3 + Math.random() * 2,
                repeat: Infinity,
                ease: "easeInOut",
                delay: Math.random() * 2,
              }}
              className="absolute pointer-events-none"
              style={{
                left: `${card.x}px`,
                top: `${card.y}px`,
              }}
            >
              <div className="w-[300px] h-[150px]" />
            </motion.div>
          ))}
      </AnimatePresence>
    </div>
  );
}
