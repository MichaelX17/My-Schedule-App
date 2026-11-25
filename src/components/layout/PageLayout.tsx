import type { ReactNode } from "react";

type PageMaxWidth = "4xl" | "5xl" | "6xl" | "7xl" | "full";

const MAX_WIDTH_CLASSES: Record<PageMaxWidth, string> = {
  "4xl": "max-w-4xl",
  "5xl": "max-w-5xl",
  "6xl": "max-w-6xl",
  "7xl": "max-w-7xl",
  full: "max-w-none",
};

interface PageLayoutProps {
  header: ReactNode;
  children: ReactNode;
  maxWidth?: PageMaxWidth;
  contentClassName?: string;
}

/**
 * Standard desktop page layout with a non-scrolling fixed header and an
 * independently scrollable content region below it.
 */
export function PageLayout({
  header,
  children,
  maxWidth = "5xl",
  contentClassName = "",
}: PageLayoutProps) {
  const widthClass = MAX_WIDTH_CLASSES[maxWidth];

  return (
    <div className="flex-1 min-h-0 flex flex-col overflow-hidden">
      <header className="shrink-0 bg-background z-10 shadow-[0_1px_0_rgb(128_128_128/0.14)]">
        <div
          className={`mx-auto w-full ${widthClass} px-4 sm:px-6 lg:px-8 py-4 sm:py-5`}
        >
          {header}
        </div>
      </header>
      <div className="flex-1 min-h-0 overflow-y-auto">
        <div
          className={`mx-auto w-full ${widthClass} px-4 sm:px-6 lg:px-8 py-5 sm:py-6 space-y-6 ${contentClassName}`}
        >
          {children}
        </div>
      </div>
    </div>
  );
}
