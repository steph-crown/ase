import { Button } from "@/components/ui/button"
import { Github } from "lucide-react"

export function Navigation() {
  return (
    <nav className="container mx-auto px-4 py-6 flex items-center justify-between">
      <div className="text-primary font-agba text-2xl">àṣẹ</div>
      <div className="flex items-center gap-6">
        <a href="#download" className="text-foreground hover:text-primary transition-colors">
          Download
        </a>
        <a href="#features" className="text-foreground hover:text-primary transition-colors">
          Features
        </a>
        <Button variant="outline" className="border-border flex items-center gap-2">
          <Github className="h-4 w-4" />
          Star on GitHub
        </Button>
      </div>
    </nav>
  )
}
