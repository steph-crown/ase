const footerLinks = [
  { href: "#install", label: "Install" },
  { href: "#features", label: "Features" },
  { href: "https://github.com/steph-crown/ase", label: "GitHub", external: true },
];

export function Footer() {
  return (
    <footer className="border-t border-[#3b3440] bg-background">
      <div className="wrapper py-10 sm:py-12">
        <div className="flex flex-col sm:flex-row items-center justify-between gap-6">
          <div className="text-primary font-agba text-xl sm:text-2xl">
            àṣẹ
          </div>
          <nav className="flex items-center gap-6 sm:gap-8">
            {footerLinks.map((link) => (
              <a
                key={link.href}
                href={link.href}
                target={link.external ? "_blank" : undefined}
                rel={link.external ? "noopener noreferrer" : undefined}
                className="text-sm font-semibold text-[#999999] hover:text-primary transition-colors font-air"
              >
                {link.label}
              </a>
            ))}
          </nav>
        </div>
        <div className="mt-8 pt-8 border-t border-[#3b3440] flex flex-col sm:flex-row items-center justify-between gap-4">
          <p className="text-xs text-[#777778] font-air order-2 sm:order-1">
            MIT OR Apache-2.0 · A small Unix-style shell in Rust
          </p>
          <p className="text-xs text-[#777778] font-air order-1 sm:order-2">
            Built with <span className="text-primary">❤</span> by{" "}
            <a
              href="https://stephcrown.com"
              target="_blank"
              rel="noopener noreferrer"
              className="text-primary hover:underline"
            >
              Stephen
            </a>
          </p>
        </div>
      </div>
    </footer>
  );
}
