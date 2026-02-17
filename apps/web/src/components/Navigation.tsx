import * as React from "react";
import { motion, AnimatePresence } from "framer-motion";
import { X } from "lucide-react";

export function Navigation() {
  const [isMenuOpen, setIsMenuOpen] = React.useState(false);

  const navLinks = [
    { href: "#install", label: "Install" },
    { href: "#features", label: "Features" },
  ];

  return (
    <>
      <nav className="wrapper py-7 sm:py-10 flex items-center justify-between" aria-label="Main">
        <a href="/" className="text-primary font-agba text-2xl" aria-label="àṣẹ home">àṣẹ</a>

        <div className="hidden md:flex items-center gap-6">
          {navLinks.map((link) => (
            <a
              key={link.href}
              href={link.href}
              className="text-[#9898A0] hover:text-primary transition-colors font-semibold text-sm"
            >
              {link.label}
            </a>
          ))}

          <a
            href="https://github.com/steph-crown/ase"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-2 text-[#9898A0] hover:text-primary transition-colors font-semibold text-sm rounded-[0.5rem] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
          >
            <svg
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
              className="size-6"
            >
              <path
                d="M12 2.2467C9.6255 2.2468 7.32849 3.09181 5.51999 4.63055C3.71149 6.16928 2.50953 8.30132 2.12916 10.6452C1.74879 12.989 2.21485 15.3918 3.44393 17.4235C4.67301 19.4551 6.58491 20.9832 8.83755 21.7342C9.33755 21.8217 9.52505 21.5217 9.52505 21.2592C9.52505 21.0217 9.51254 20.2342 9.51254 19.3967C7.00003 19.8592 6.35003 18.7842 6.15003 18.2217C5.9281 17.6747 5.5763 17.1898 5.12503 16.8092C4.77503 16.6217 4.27503 16.1592 5.11252 16.1467C5.4323 16.1814 5.73901 16.2927 6.00666 16.4711C6.2743 16.6495 6.49499 16.8899 6.65003 17.1717C6.7868 17.4174 6.97071 17.6337 7.19122 17.8082C7.41173 17.9827 7.6645 18.112 7.93506 18.1886C8.20562 18.2652 8.48864 18.2877 8.76791 18.2548C9.04717 18.2219 9.3172 18.1342 9.56251 17.9967C9.6058 17.4883 9.83237 17.013 10.2 16.6592C7.97503 16.4092 5.65003 15.5467 5.65003 11.7217C5.63597 10.7279 6.00271 9.76631 6.67503 9.03423C6.36931 8.17045 6.40508 7.22251 6.77503 6.38423C6.77503 6.38423 7.6125 6.12172 9.52503 7.40923C11.1613 6.9592 12.8887 6.9592 14.525 7.40923C16.4375 6.10923 17.275 6.38423 17.275 6.38423C17.645 7.2225 17.6808 8.17046 17.375 9.03423C18.0494 9.76505 18.4164 10.7275 18.4 11.7217C18.4 15.5592 16.0625 16.4092 13.8375 16.6592C14.0762 16.9011 14.26 17.1915 14.3764 17.5107C14.4929 17.83 14.5393 18.1705 14.5125 18.5092C14.5125 19.8468 14.5 20.9217 14.5 21.2592C14.5 21.5217 14.6875 21.8342 15.1875 21.7342C17.4362 20.9771 19.3426 19.4455 20.5664 17.4127C21.7903 15.38 22.2519 12.9785 21.8689 10.6369C21.4859 8.29535 20.2832 6.16607 18.4755 4.62921C16.6678 3.09235 14.3727 2.24794 12 2.2467Z"
                fill="#9898A0"
              />
            </svg>
            Star on GitHub
          </a>
        </div>

        <button
          onClick={() => setIsMenuOpen(!isMenuOpen)}
          className="md:hidden relative w-12 h-12 rounded-full border-2 border-primary/30 hover:border-primary transition-all duration-300 flex items-center justify-center group"
          aria-label="Toggle menu"
        >
          <div className="flex flex-col gap-1.5">
            <motion.div
              animate={isMenuOpen ? { rotate: 45, y: 6 } : { rotate: 0, y: 0 }}
              transition={{ duration: 0.3 }}
              className="w-5 h-0.5 bg-primary rounded-full"
            />
            <motion.div
              animate={isMenuOpen ? { opacity: 0 } : { opacity: 1 }}
              transition={{ duration: 0.2 }}
              className="w-5 h-0.5 bg-primary rounded-full"
            />
            <motion.div
              animate={
                isMenuOpen ? { rotate: -45, y: -6 } : { rotate: 0, y: 0 }
              }
              transition={{ duration: 0.3 }}
              className="w-5 h-0.5 bg-primary rounded-full"
            />
          </div>

          {[0, 1, 2, 3, 4, 5, 6, 7].map((i) => (
            <motion.div
              key={i}
              className="absolute w-1.5 h-1.5 bg-primary/40 rounded-full"
              style={{
                top: "50%",
                left: "50%",
                x: "-50%",
                y: "-50%",
              }}
              animate={
                isMenuOpen
                  ? {
                      x: `calc(-50% + ${Math.cos((i * Math.PI) / 4) * 20}px)`,
                      y: `calc(-50% + ${Math.sin((i * Math.PI) / 4) * 20}px)`,
                      opacity: 0,
                    }
                  : {
                      x: `calc(-50% + ${Math.cos((i * Math.PI) / 4) * 16}px)`,
                      y: `calc(-50% + ${Math.sin((i * Math.PI) / 4) * 16}px)`,
                      opacity: 0.6,
                    }
              }
              transition={{
                duration: 0.4,
                delay: i * 0.05,
              }}
            />
          ))}
        </button>
      </nav>

      <AnimatePresence>
        {isMenuOpen && (
          <>
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.3 }}
              className="fixed inset-0 bg-black/60 backdrop-blur-sm z-40 md:hidden"
              onClick={() => setIsMenuOpen(false)}
            />

            <motion.div
              initial={{ x: "100%", opacity: 0 }}
              animate={{ x: 0, opacity: 1 }}
              exit={{ x: "100%", opacity: 0 }}
              transition={{
                type: "spring",
                damping: 25,
                stiffness: 200,
              }}
              className="fixed top-0 right-0 h-full w-[85%] max-w-sm bg-[#1B1B1D] border-l border-[#29292b] z-50 md:hidden shadow-2xl"
              style={{
                boxShadow: "0 2px 4px 2px rgba(0, 0, 0, 0.1)",
              }}
            >
              <div className="flex flex-col h-full">
                <div className="flex items-center justify-between p-6 border-b border-[#0A0A0B]">
                  <div className="text-primary font-agba text-xl">àṣẹ</div>
                  <button
                    onClick={() => setIsMenuOpen(false)}
                    className="w-10 h-10 rounded-full border border-[#474747] flex items-center justify-center hover:border-primary transition-colors"
                    aria-label="Close menu"
                  >
                    <X className="w-5 h-5 text-[#9898A0]" />
                  </button>
                </div>

                <div className="flex-1 flex flex-col gap-2 p-6">
                  {navLinks.map((link, index) => (
                    <motion.a
                      key={link.href}
                      href={link.href}
                      onClick={() => setIsMenuOpen(false)}
                      initial={{ x: 50, opacity: 0 }}
                      animate={{ x: 0, opacity: 1 }}
                      transition={{
                        delay: index * 0.1,
                        type: "spring",
                        damping: 20,
                      }}
                      className="text-[#9898A0] hover:text-primary transition-colors font-semibold text-lg py-4 px-4 rounded-lg hover:bg-[#29292b]"
                    >
                      {link.label}
                    </motion.a>
                  ))}

                  <motion.div
                    initial={{ x: 50, opacity: 0 }}
                    animate={{ x: 0, opacity: 1 }}
                    transition={{
                      delay: navLinks.length * 0.1,
                      type: "spring",
                      damping: 20,
                    }}
                    className="mt-4 pt-4 border-t border-[#0A0A0B]"
                  >
                    <a
                      href="https://github.com/steph-crown/ase"
                      target="_blank"
                      rel="noopener noreferrer"
                      className="w-full flex items-center justify-center gap-2 text-[#9898A0] hover:text-primary transition-colors font-semibold text-lg py-4 rounded-[0.5rem] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    >
                      <svg
                        width="24"
                        height="24"
                        viewBox="0 0 24 24"
                        fill="none"
                        xmlns="http://www.w3.org/2000/svg"
                        className="size-6"
                      >
                        <path
                          d="M12 2.2467C9.6255 2.2468 7.32849 3.09181 5.51999 4.63055C3.71149 6.16928 2.50953 8.30132 2.12916 10.6452C1.74879 12.989 2.21485 15.3918 3.44393 17.4235C4.67301 19.4551 6.58491 20.9832 8.83755 21.7342C9.33755 21.8217 9.52505 21.5217 9.52505 21.2592C9.52505 21.0217 9.51254 20.2342 9.51254 19.3967C7.00003 19.8592 6.35003 18.7842 6.15003 18.2217C5.9281 17.6747 5.5763 17.1898 5.12503 16.8092C4.77503 16.6217 4.27503 16.1592 5.11252 16.1467C5.4323 16.1814 5.73901 16.2927 6.00666 16.4711C6.2743 16.6495 6.49499 16.8899 6.65003 17.1717C6.7868 17.4174 6.97071 17.6337 7.19122 17.8082C7.41173 17.9827 7.6645 18.112 7.93506 18.1886C8.20562 18.2652 8.48864 18.2877 8.76791 18.2548C9.04717 18.2219 9.3172 18.1342 9.56251 17.9967C9.6058 17.4883 9.83237 17.013 10.2 16.6592C7.97503 16.4092 5.65003 15.5467 5.65003 11.7217C5.63597 10.7279 6.00271 9.76631 6.67503 9.03423C6.36931 8.17045 6.40508 7.22251 6.77503 6.38423C6.77503 6.38423 7.6125 6.12172 9.52503 7.40923C11.1613 6.9592 12.8887 6.9592 14.525 7.40923C16.4375 6.10923 17.275 6.38423 17.275 6.38423C17.645 7.2225 17.6808 8.17046 17.375 9.03423C18.0494 9.76505 18.4164 10.7275 18.4 11.7217C18.4 15.5592 16.0625 16.4092 13.8375 16.6592C14.0762 16.9011 14.26 17.1915 14.3764 17.5107C14.4929 17.83 14.5393 18.1705 14.5125 18.5092C14.5125 19.8468 14.5 20.9217 14.5 21.2592C14.5 21.5217 14.6875 21.8342 15.1875 21.7342C17.4362 20.9771 19.3426 19.4455 20.5664 17.4127C21.7903 15.38 22.2519 12.9785 21.8689 10.6369C21.4859 8.29535 20.2832 6.16607 18.4755 4.62921C16.6678 3.09235 14.3727 2.24794 12 2.2467Z"
                          fill="#9898A0"
                        />
                      </svg>
                      Star on GitHub
                    </a>
                  </motion.div>
                </div>
              </div>
            </motion.div>
          </>
        )}
      </AnimatePresence>
    </>
  );
}
