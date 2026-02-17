import { Navigation } from "@/components/Navigation";
import { HeroSection } from "@/components/hero/HeroSection";

export function App() {
  return (
    <div className="min-h-screen bg-background">
      <Navigation />
      <HeroSection />
    </div>
  );
}

export default App;
