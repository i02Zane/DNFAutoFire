import { Activity } from "lucide-react";
import { keyLabel } from "../../lib/keys";
import type { RuntimeDiagnostics } from "../../types/app-config";
import { useRuntimeDiagnostics } from "./use-runtime-diagnostics";

type RuntimeDiagnosticsPageProps = {
  onError: (message: string) => void;
};

export function RuntimeDiagnosticsPage({ onError }: RuntimeDiagnosticsPageProps) {
  const { diagnostics, lastUpdatedAt } = useRuntimeDiagnostics({ onError });

  return (
    <section className="h-full min-w-0 overflow-y-auto px-7 py-6">
      <header className="mb-5">
        <div>
          <div className="flex items-center gap-2">
            <Activity size={20} className="text-blue-600" />
            <h1 className="text-[22px] font-semibold tracking-tight">运行诊断</h1>
          </div>
          <p className="mt-1 text-sm leading-6 text-slate-500">
            查看助手、前台窗口和各运行时引擎的当前状态。
          </p>
        </div>
      </header>

      {diagnostics ? (
        <div className="grid gap-4 pb-6">
          <div className="grid grid-cols-3 gap-4">
            <StatusCard
              items={[
                ["状态", diagnostics.assistant.running ? "运行中" : "已停止"],
                [
                  "运行快照",
                  `${diagnostics.assistant.profileKeyCount} 个连发 / ${diagnostics.assistant.profileComboCount} 个连招`,
                ],
                ["当前配置", diagnostics.activeConfig.activeClassId ?? "全局配置"],
              ]}
              title="助手"
            />
            <StatusCard
              items={[
                ["目标前台", diagnostics.foreground.targetActive ? "是" : "否"],
                ["窗口类名", diagnostics.foreground.className || "-"],
                ["更新时间", lastUpdatedAt ? lastUpdatedAt.toLocaleTimeString() : "-"],
              ]}
              title="前台窗口"
            />
            <StatusCard
              items={[
                ["自动识别", diagnostics.activeConfig.detectionEnabled ? "已启用" : "已关闭"],
                ["识别服务", diagnostics.detection.running ? "运行中" : "已停止"],
                ["识别间隔", `${diagnostics.detection.intervalMs} ms`],
                ["城镇状态", townStateLabel(diagnostics.detection.townActive)],
                [
                  "识别状态",
                  detectionReasonLabel(diagnostics.detection.lastResult?.reason ?? null),
                ],
                ["识别结果", detectionResultLabel(diagnostics.detection.lastResult)],
              ]}
              title="职业识别"
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <StatusCard
              items={[
                ["引擎", diagnostics.autofire.running ? "运行中" : "已停止"],
                ["配置键数", `${diagnostics.autofire.keys.length} 个`],
                [
                  "激活切换",
                  diagnostics.autofire.keys
                    .filter((key) => key.toggleActive)
                    .map((key) => keyLabel(key.vk))
                    .join("、") || "-",
                ],
              ]}
              title="按键连发"
            />
            <StatusCard
              items={[
                ["引擎", diagnostics.combo.running ? "运行中" : "已停止"],
                [
                  "连招数量",
                  `${diagnostics.combo.enabledComboCount} / ${diagnostics.combo.comboCount}`,
                ],
                ["执行中", diagnostics.combo.executing ? "是" : "否"],
                [
                  "触发键",
                  diagnostics.combo.triggerVks.map((vk) => keyLabel(vk)).join("、") || "-",
                ],
              ]}
              title="一键连招"
            />
          </div>

          <StatusCard
            items={[
              ["设置", diagnostics.autoRun.enabled ? "已启用" : "已关闭"],
              ["引擎", diagnostics.autoRun.running ? "运行中" : "已停止"],
              ["左键", keyLabel(diagnostics.autoRun.leftVk)],
              ["右键", keyLabel(diagnostics.autoRun.rightVk)],
              ["脉冲延迟", `${diagnostics.autoRun.pulseDelayMs} ms`],
            ]}
            title="一键奔跑"
          />

          <div className="rounded border border-slate-200 bg-white shadow-sm">
            <div className="border-b border-slate-100 px-4 py-3 text-sm font-semibold text-slate-800">
              连发键明细
            </div>
            {diagnostics.autofire.keys.length > 0 ? (
              <div className="divide-y divide-slate-100">
                {diagnostics.autofire.keys.map((key) => (
                  <div
                    key={key.vk}
                    className="grid grid-cols-[160px_100px_100px_100px_1fr] items-center gap-3 px-4 py-3 text-sm"
                  >
                    <span className="font-medium text-slate-800">{keyLabel(key.vk)}</span>
                    <span className="text-slate-600">{key.intervalMs} ms</span>
                    <StatePill
                      active={key.mode === "toggle"}
                      label={key.mode === "toggle" ? "切换" : "按住"}
                    />
                    <StatePill active={key.pressed} label={key.pressed ? "按下" : "未按下"} />
                    <StatePill
                      active={key.toggleActive}
                      label={key.toggleActive ? "切换激活" : "切换未激活"}
                    />
                  </div>
                ))}
              </div>
            ) : (
              <div className="px-4 py-8 text-center text-sm text-slate-500">当前没有连发键。</div>
            )}
          </div>
        </div>
      ) : (
        <div className="rounded border border-dashed border-slate-200 bg-white px-3 py-10 text-center text-sm text-slate-500">
          正在读取运行状态...
        </div>
      )}
    </section>
  );
}

function StatusCard({ title, items }: { title: string; items: [string, string][] }) {
  return (
    <section className="rounded border border-slate-200 bg-white shadow-sm">
      <div className="border-b border-slate-100 px-4 py-3 text-sm font-semibold text-slate-800">
        {title}
      </div>
      <dl className="divide-y divide-slate-100">
        {items.map(([label, value]) => (
          <div key={label} className="grid grid-cols-[92px_1fr] gap-3 px-4 py-3 text-sm">
            <dt className="text-slate-500">{label}</dt>
            <dd className="min-w-0 break-words font-medium text-slate-800">{value}</dd>
          </div>
        ))}
      </dl>
    </section>
  );
}

function StatePill({ active, label }: { active: boolean; label: string }) {
  return (
    <span
      className={`inline-flex h-7 w-fit items-center rounded border px-2 text-xs font-medium ${
        active
          ? "border-blue-200 bg-blue-50 text-blue-700"
          : "border-slate-200 bg-slate-50 text-slate-500"
      }`}
    >
      {label}
    </span>
  );
}

function townStateLabel(townActive: boolean | null): string {
  if (townActive === true) return "城镇";
  if (townActive === false) return "副本/非城镇";
  return "未知";
}

function detectionReasonLabel(reason: string | null): string {
  switch (reason) {
    case "matched":
      return "已识别";
    case "notFound":
      return "未识别到职业";
    case "notInTown":
      return "不在城镇";
    case "foregroundInactive":
      return "目标窗口未在前台";
    case "captureError":
      return "捕获失败";
    case null:
      return "暂无结果";
    default:
      return reason;
  }
}

function detectionResultLabel(result: RuntimeDiagnostics["detection"]["lastResult"]): string {
  if (result?.classIndex === null || result === null) return "-";

  return result.className ? `#${result.classIndex} ${result.className}` : `#${result.classIndex}`;
}
