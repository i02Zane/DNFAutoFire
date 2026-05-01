import { SettingsSelect, SettingsSwitch } from "../components/app-ui";
import { APP_DISPLAY_NAME } from "../lib/app-meta";
import { type LogLevelSetting } from "../lib/tauri";

const LOG_LEVEL_OPTIONS: { label: string; value: LogLevelSetting }[] = [
  { label: "Trace", value: "trace" },
  { label: "Debug", value: "debug" },
  { label: "Info", value: "info" },
  { label: "Warn", value: "warn" },
  { label: "Error", value: "error" },
  { label: "关闭", value: "off" },
];

export function SettingsPage({
  launchAtStartup,
  logLevel,
  minimizeToTray,
  openFloatingControlOnStart,
  startMinimized,
  onLaunchAtStartupChange,
  onLogLevelChange,
  onMinimizeToTrayChange,
  onOpenFloatingControlOnStartChange,
  onStartMinimizedChange,
}: {
  launchAtStartup: boolean;
  logLevel: LogLevelSetting;
  minimizeToTray: boolean;
  openFloatingControlOnStart: boolean;
  startMinimized: boolean;
  onLaunchAtStartupChange: (checked: boolean) => void;
  onLogLevelChange: (logLevel: LogLevelSetting) => void;
  onMinimizeToTrayChange: (checked: boolean) => void;
  onOpenFloatingControlOnStartChange: (checked: boolean) => void;
  onStartMinimizedChange: (checked: boolean) => void;
}) {
  return (
    <main className="min-w-0 flex-1 overflow-auto px-7 py-6">
      <section className="max-w-[760px]">
        <div className="flex items-center gap-2">
          <h1 className="text-[22px] font-semibold tracking-tight">设置</h1>
        </div>
        <div className="mt-1 space-y-1 text-sm leading-6 text-slate-500 hidden">
          <p>程序级设置。</p>
        </div>

        <div className="mt-6 overflow-hidden rounded border border-slate-200 bg-white shadow-sm">
          <SettingsSwitch
            checked={launchAtStartup}
            description={`打开 Windows 后自动启动 ${APP_DISPLAY_NAME}。`}
            label="开机时启动"
            onChange={onLaunchAtStartupChange}
          />
          <SettingsSwitch
            checked={startMinimized}
            description="启动应用后收起到最小化状态。"
            label="启动时最小化"
            onChange={onStartMinimizedChange}
          />
          <SettingsSwitch
            checked={minimizeToTray}
            description="开启后，最小化按钮会隐藏主窗口到系统托盘；启动时最小化也会按这个方式处理。"
            label="最小化到托盘"
            onChange={onMinimizeToTrayChange}
          />
          <SettingsSwitch
            checked={openFloatingControlOnStart}
            description="进入应用后自动显示助手悬浮窗。"
            label="启动时自动打开悬浮窗"
            onChange={onOpenFloatingControlOnStartChange}
          />
        </div>
        <div className="mt-6 overflow-hidden rounded border border-slate-200 bg-white shadow-sm">
          <SettingsSelect
            description="控制日志输出等级"
            label="日志等级"
            options={LOG_LEVEL_OPTIONS}
            value={logLevel}
            onChange={(value) => onLogLevelChange(value as LogLevelSetting)}
          />
        </div>
      </section>
    </main>
  );
}
