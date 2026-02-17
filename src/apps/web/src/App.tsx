import { Toaster } from "@/components/ui/sonner";
import { Navigation } from "@/components/Navigation";
import { HeroSection } from "@/components/hero/HeroSection";

export function App() {
  return (
    <div className="min-h-screen bg-background">
      <Navigation />
      <HeroSection />
      <Toaster position="bottom-center" />
    </div>
  );
}

export default App;
