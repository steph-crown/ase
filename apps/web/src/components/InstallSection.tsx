import { InstallCard } from "@/components/InstallCard";

export function InstallSection() {
  return (
    <section id="install" className="py-16 sm:py-24 sm:pt-10 bg-background" aria-labelledby="install-heading">
      <div className="wrapper">
        <div className="text-center mb-10 sm:mb-14">
          <h2 id="install-heading" className="text-3xl sm:text-4xl lg:text-5xl xl:text-6xl font-medium text-foreground leading-[106%] font-air tracking-[-2%] mb-3 sm:mb-4 ">
            Install <span className="text-primary font-agba">àṣẹ</span>
          </h2>
          <p className="text-sm sm:text-base text-[#999999] font-semibold max-w-xl mx-auto font-air">
            Pick your platform and run the command. That’s it.
          </p>
        </div>
        <div className="flex justify-center">
          <InstallCard className="w-full max-w-[748px] " />
        </div>
      </div>
    </section>
  );
}
