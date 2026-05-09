import {
  Activity,
  CircleHelp,
  Footprints,
  Keyboard,
  ListChecks,
  Settings,
  Wand2,
} from "lucide-react";
import type { ReactNode } from "react";

import type { Page } from "../types/ui";
import { AppTitleBar, MessageDialog, NavButton } from "./app-ui";

type AppShellProps = {
  children: ReactNode;
  message: string | null;
  page: Page;
  statusBar: ReactNode;
  onMessageClose: () => void;
  onPageChange: (page: Page) => void;
};

export function AppShell({
  children,
  message,
  page,
  statusBar,
  onMessageClose,
  onPageChange,
}: AppShellProps) {
  const changePage = onPageChange;
  const clearMessage = onMessageClose;

  return (
    <div className="flex h-screen w-screen flex-col overflow-hidden bg-[#eef3f8] text-slate-950">
      <AppTitleBar />
      <div className="flex min-h-0 flex-1">
        <aside className="flex w-[188px] shrink-0 flex-col border-r border-slate-200 bg-[#111827] px-4 py-5 text-slate-200">
          <nav className="space-y-2">
            <NavButton
              active={page === "autofire"}
              icon={<Keyboard size={18} />}
              label="按键连发"
              onClick={() => changePage("autofire")}
            />
            <NavButton
              active={page === "combo"}
              icon={<Wand2 size={18} />}
              label="一键连招(Beta)"
              onClick={() => changePage("combo")}
            />
            <NavButton
              active={page === "auto-run"}
              icon={<Footprints size={18} />}
              label="一键奔跑"
              onClick={() => changePage("auto-run")}
            />
          </nav>

          <div className="mt-auto space-y-2">
            <NavButton
              active={page === "config-management"}
              icon={<ListChecks size={18} />}
              label="配置管理"
              onClick={() => changePage("config-management")}
            />
            <NavButton
              active={page === "runtime-diagnostics"}
              icon={<Activity size={18} />}
              label="运行诊断"
              onClick={() => changePage("runtime-diagnostics")}
            />
            <NavButton
              active={page === "settings"}
              icon={<Settings size={18} />}
              label="设置"
              onClick={() => changePage("settings")}
            />
            <NavButton
              active={page === "about"}
              icon={<CircleHelp size={18} />}
              label="关于"
              onClick={() => changePage("about")}
            />
          </div>
        </aside>

        <main className="grid min-h-0 min-w-0 flex-1 grid-rows-[1fr_76px]">
          <div className="min-h-0 overflow-hidden">{children}</div>
          {statusBar}
        </main>
      </div>
      {message && <MessageDialog message={message} onClose={clearMessage} />}
    </div>
  );
}
