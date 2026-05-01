import { ChevronDown, ChevronRight, Eye, EyeOff, Plus, Trash2 } from "lucide-react";
import { useState, type FormEvent } from "react";
import { ConfirmDialog } from "../components/app-ui";
import { classCategories } from "../data/classes";
import { hasClassConfig } from "../lib/config";
import type { AppConfig } from "../lib/tauri";

export function ConfigManagementPage({
  config,
  onAddCustomConfig,
  onDeleteCustomConfig,
  onToggleClassHidden,
}: {
  config: AppConfig;
  onAddCustomConfig: (name: string) => void;
  onDeleteCustomConfig: (configId: string) => void;
  onToggleClassHidden: (classId: string, hidden: boolean) => void;
}) {
  const [customConfigName, setCustomConfigName] = useState("");
  const [deletingCustomConfigId, setDeletingCustomConfigId] = useState<string | null>(null);
  const [collapsedCategoryNames, setCollapsedCategoryNames] = useState<Set<string>>(
    () => new Set(),
  );
  const deletingCustomConfig = deletingCustomConfigId
    ? config.customConfigs[deletingCustomConfigId]
    : null;

  function submitCustomConfig(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    onAddCustomConfig(customConfigName);
    setCustomConfigName("");
  }

  function toggleCategory(categoryName: string) {
    setCollapsedCategoryNames((current) => {
      const next = new Set(current);
      if (next.has(categoryName)) {
        next.delete(categoryName);
      } else {
        next.add(categoryName);
      }
      return next;
    });
  }

  return (
    <main className="h-full min-w-0 overflow-hidden px-7 py-6">
      {deletingCustomConfigId && deletingCustomConfig && (
        <ConfirmDialog
          confirmText="删除"
          description={`“${deletingCustomConfig.name || "未命名配置"}”的连发和连招设置都会被移除，无法恢复。`}
          title="删除自定义配置"
          onCancel={() => setDeletingCustomConfigId(null)}
          onConfirm={() => {
            onDeleteCustomConfig(deletingCustomConfigId);
            setDeletingCustomConfigId(null);
          }}
        />
      )}
      <section className="grid h-full min-w-0 grid-rows-[auto_1fr]">
        <header>
          <div className="flex items-center gap-2">
            <h1 className="text-[22px] font-semibold tracking-tight">配置管理</h1>
          </div>
          <div className="mt-1 space-y-1 text-sm leading-6 text-slate-500">
            <p>管理用户自定义配置，并精简两处编辑页中的职业入口。</p>
          </div>
        </header>

        <div className="mt-6 grid min-h-0 grid-cols-2 gap-5">
          <section className="grid min-h-0 grid-rows-[auto_1fr] rounded border border-slate-200 bg-white shadow-sm">
            <header className="flex items-center justify-between gap-3 border-b border-slate-100 px-4 py-3">
              <div>
                <h2 className="text-sm font-semibold text-slate-900">自定义配置</h2>
                <p className="mt-1 text-xs text-slate-500">
                  可新增或删除；设置连发键或连招后，会出现在底部和悬浮窗选择器。
                </p>
              </div>
            </header>
            <div className="min-h-0 overflow-y-auto p-4">
              <form className="flex gap-2" onSubmit={submitCustomConfig}>
                <input
                  className="h-9 min-w-0 flex-1 rounded border border-slate-300 px-3 text-sm outline-none focus:border-blue-400 focus:ring-1 focus:ring-blue-100"
                  placeholder="配置名，例如：太宗、短宗、金刀剑魂"
                  value={customConfigName}
                  onChange={(event) => setCustomConfigName(event.currentTarget.value)}
                />
                <button
                  className="inline-flex h-9 items-center gap-1.5 rounded bg-blue-600 px-3 text-sm font-semibold text-white shadow-sm transition hover:bg-blue-700"
                  type="submit"
                >
                  <Plus size={16} />
                  添加
                </button>
              </form>

              <div className="mt-3 space-y-2">
                {Object.entries(config.customConfigs).map(([configId, customConfig]) => (
                  <div
                    key={configId}
                    className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-3 rounded border border-slate-200 bg-slate-50 px-3 py-2.5"
                  >
                    <div className="min-w-0">
                      <div className="truncate text-sm font-semibold text-slate-800">
                        {customConfig.name || "未命名配置"}
                      </div>
                      <div className="mt-1 text-xs text-slate-500">
                        {customConfig.enabledKeys.length} 个连发键 / {customConfig.comboDefs.length}{" "}
                        个连招
                      </div>
                    </div>
                    <button
                      className="inline-flex h-8 w-8 items-center justify-center rounded border border-slate-300 bg-white text-slate-500 transition hover:border-red-300 hover:bg-red-50 hover:text-red-600"
                      type="button"
                      onClick={() => setDeletingCustomConfigId(configId)}
                    >
                      <Trash2 size={15} />
                    </button>
                  </div>
                ))}
                {Object.keys(config.customConfigs).length === 0 && (
                  <div className="rounded border border-dashed border-slate-200 px-3 py-6 text-center text-sm text-slate-500">
                    还没有自定义配置。
                  </div>
                )}
              </div>
            </div>
          </section>

          <section className="grid min-h-0 grid-rows-[auto_1fr] rounded border border-slate-200 bg-white shadow-sm">
            <header className="border-b border-slate-100 px-4 py-3">
              <h2 className="text-sm font-semibold text-slate-900">职业配置管理</h2>
              <p className="mt-1 text-xs text-slate-500">不能隐藏已有连发或连招设置的职业配置。</p>
            </header>
            <div className="min-h-0 overflow-y-auto p-2.5">
              <div className="space-y-2">
                {classCategories.map((category) => {
                  const collapsed = collapsedCategoryNames.has(category.name);
                  const configuredCount = category.classes.filter((classInfo) =>
                    hasClassConfig(config.classes[classInfo.id]),
                  ).length;
                  const hiddenCount = category.classes.filter(
                    (classInfo) =>
                      config.hiddenClassIds.includes(classInfo.id) &&
                      !hasClassConfig(config.classes[classInfo.id]),
                  ).length;

                  return (
                    <section
                      key={category.name}
                      className="overflow-hidden rounded border border-slate-200 bg-white"
                    >
                      <button
                        className="grid h-10 w-full grid-cols-[24px_minmax(0,1fr)_auto] items-center gap-2 bg-slate-50 px-3 text-left transition hover:bg-slate-100"
                        type="button"
                        onClick={() => toggleCategory(category.name)}
                      >
                        <span className="inline-flex h-6 w-6 items-center justify-center text-slate-500">
                          {collapsed ? <ChevronRight size={16} /> : <ChevronDown size={16} />}
                        </span>
                        <span className="truncate text-sm font-semibold text-slate-800">
                          {category.name}
                        </span>
                        <span className="text-xs text-slate-500">
                          {configuredCount > 0 && `${configuredCount} 已配置`}
                          {configuredCount > 0 && hiddenCount > 0 ? " / " : ""}
                          {hiddenCount > 0 && `${hiddenCount} 已隐藏`}
                        </span>
                      </button>

                      {!collapsed && (
                        <div className="divide-y divide-slate-100">
                          {category.classes.map((classInfo) => {
                            const configured = hasClassConfig(config.classes[classInfo.id]);
                            const hidden =
                              config.hiddenClassIds.includes(classInfo.id) && !configured;
                            return (
                              <div
                                key={classInfo.id}
                                className="grid grid-cols-[minmax(0,1fr)_84px] items-center gap-2 px-3 py-1.5 transition hover:bg-slate-50"
                              >
                                <div className="min-w-0">
                                  <div className="truncate text-sm font-medium text-slate-800">
                                    {classInfo.name}
                                  </div>
                                </div>
                                <button
                                  className={`inline-flex h-8 items-center justify-center gap-1.5 rounded border px-2 text-xs font-medium transition ${
                                    configured
                                      ? "border-slate-200 bg-slate-100 text-slate-400"
                                      : hidden
                                        ? "border-slate-300 bg-slate-100 text-slate-600 hover:bg-slate-200"
                                        : "border-emerald-200 bg-emerald-50 text-emerald-700 hover:border-slate-300 hover:bg-slate-100 hover:text-slate-700"
                                  } ${configured ? "cursor-not-allowed" : ""}`}
                                  type="button"
                                  disabled={configured}
                                  aria-label={
                                    configured
                                      ? `${classInfo.name} 已配置，不能隐藏`
                                      : hidden
                                        ? `显示 ${classInfo.name}`
                                        : `隐藏 ${classInfo.name}`
                                  }
                                  onClick={() => onToggleClassHidden(classInfo.id, !hidden)}
                                >
                                  {!configured &&
                                    (hidden ? <EyeOff size={14} /> : <Eye size={14} />)}
                                  {configured ? "不能隐藏" : hidden ? "已隐藏" : "显示中"}
                                </button>
                              </div>
                            );
                          })}
                        </div>
                      )}
                    </section>
                  );
                })}
              </div>
            </div>
          </section>
        </div>
      </section>
    </main>
  );
}
