import { Navigation } from "@/components/Navigation";
import { FeaturesSection } from "@/components/features/FeaturesSection";
import { HeroSection } from "@/components/hero/HeroSection";
import { Toaster } from "@/components/ui/sonner";

export function App() {
  return (
    <div className="min-h-screen bg-background">
      <Navigation />
      <HeroSection />
      <FeaturesSection />
      <Toaster position="bottom-center" />
    </div>
  );
}

export default App;
