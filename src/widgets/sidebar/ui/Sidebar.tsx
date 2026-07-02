import type { ReactNode } from "react";
import { Mic, Plug, Settings } from "lucide-react";
import { AnimatePresence, motion } from "framer-motion";
import { Link, useLocation } from "@tanstack/react-router";
import { useTranslation } from "react-i18next";
import { buttonVariants } from "@/shared/ui/button";
import { cn } from "@/shared/lib/utils";

interface SidebarProps {
  collapsed: boolean;
}

type NavTo = "/" | "/plugins" | "/settings";

// ponytail: icon in fixed 32px span (aligns with collapse button), text slides+fades
function SideNavItem({
  to,
  label,
  icon,
  collapsed,
}: {
  to: NavTo;
  label: string;
  icon: ReactNode;
  collapsed: boolean;
}) {
  const location = useLocation();
  const isActive = location.pathname === to;

  return (
    <Link
      to={to}
      aria-label={label}
      className={cn(
        buttonVariants({ variant: "ghost", size: "default" }),
        "h-8 w-full justify-start gap-0 px-0 hover:border-border",
        isActive
          ? "border-border bg-muted text-foreground"
          : "text-muted-foreground",
      )}
    >
      <span className="flex size-8 shrink-0 items-center justify-center">
        {icon}
      </span>
      <AnimatePresence>
        {!collapsed && (
          <motion.span
            initial={{ opacity: 0, x: -12 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -12 }}
            transition={{ duration: 0.2, ease: "easeInOut" }}
            className="truncate pr-2 text-sm"
          >
            {label}
          </motion.span>
        )}
      </AnimatePresence>
    </Link>
  );
}

export function Sidebar({ collapsed }: SidebarProps) {
  const { t } = useTranslation();
  return (
    <nav
      className={cn(
        "flex shrink-0 flex-col items-start overflow-hidden bg-sidebar py-1 px-1 gap-1 transition-[width] duration-200 ease-in-out",
        collapsed ? "w-9" : "w-36",
      )}
    >
      <SideNavItem
        to="/"
        label={t("sidebar.home")}
        icon={<Mic className="size-4" />}
        collapsed={collapsed}
      />
      <SideNavItem
        to="/plugins"
        label={t("sidebar.plugins")}
        icon={<Plug className="size-4" />}
        collapsed={collapsed}
      />
      <div className="mt-auto w-full">
        <SideNavItem
          to="/settings"
          label={t("sidebar.settings")}
          icon={<Settings className="size-4" />}
          collapsed={collapsed}
        />
      </div>
    </nav>
  );
}
