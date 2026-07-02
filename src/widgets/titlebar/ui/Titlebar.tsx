import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, PanelLeftClose, PanelLeftOpen, Square, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/shared/ui/button";
import { cn } from "@/shared/lib/utils";

const appWindow = getCurrentWindow();

interface TitlebarProps {
  collapsed: boolean;
  onToggleCollapse: () => void;
}

export function Titlebar({ collapsed, onToggleCollapse }: TitlebarProps) {
  const { t } = useTranslation();
  return (
    <header
      data-tauri-drag-region
      className="flex shrink-0 items-center justify-between bg-sidebar select-none py-0.5"
    >
      <div className="flex h-full items-center gap-1 pl-1">
        <Button
          variant="ghost"
          size="icon"
          onClick={onToggleCollapse}
          aria-label={t("titlebar.toggleSidebar")}
          className="hover:border-border"
        >
          {collapsed ? (
            <PanelLeftOpen className="size-4" />
          ) : (
            <PanelLeftClose className="size-4" />
          )}
        </Button>
      </div>

      <div className="flex h-full items-center gap-1 pr-1">
        <TitlebarButton onClick={() => appWindow.minimize()} aria-label={t("titlebar.minimize")}>
          <Minus className="size-4" />
        </TitlebarButton>
        <TitlebarButton onClick={() => appWindow.toggleMaximize()} aria-label={t("titlebar.maximize")}>
          <Square className="size-3.5" />
        </TitlebarButton>
        <TitlebarButton onClick={() => appWindow.close()} aria-label={t("titlebar.close")} variant="close">
          <X className="size-4" />
        </TitlebarButton>
      </div>
    </header>
  );
}

function TitlebarButton({
  children,
  variant = "plain",
  ...props
}: {
  children: React.ReactNode;
  variant?: "plain" | "close";
} & Omit<React.ComponentProps<typeof Button>, "variant" | "size">) {
  return (
    <Button
      variant="ghost"
      size="icon"
      className={cn(
        "hover:border-border",
        variant === "close" && "hover:border-destructive/30",
      )}
      {...props}
    >
      {children}
    </Button>
  );
}