import { Search } from "lucide-react";

import { KeySummary, KeyTable, RuleButton, RuleHelpTooltip } from "../../components/app-ui";
import { getProfileConfig } from "../../lib/config";
import { isMockMode } from "../../lib/tauri-env";
import type {
  EffectRule,
  KeyBinding,
  ProfileDisplaySnapshot,
  ProfilesConfig,
} from "../../types/app-config";
import type { ClassCategory } from "../../types/app-config";
import type { EditTarget } from "../../types/ui";

type AutofirePageProps = {
  autofireClassSearch: string;
  closeTarget: () => void;
  openTarget: (target: EditTarget) => void;
  selectedKeys: KeyBinding[];
  selectedTitle: string;
  setAutofireClassSearch: (value: string) => void;
  profileDisplay: ProfileDisplaySnapshot;
  profiles: ProfilesConfig;
  target: EditTarget | null;
  visibleAutofireClassCategories: ClassCategory[];
  visibleCustomConfigs: [string, ProfilesConfig["customConfigs"][string]][];
  onAddKey: () => void;
  onDeleteKey: (index: number) => void;
  onEffectRuleChange: (effectRule: EffectRule) => void;
  onKeyUpdate: (index: number, patch: Partial<KeyBinding>) => void;
};

export function AutofirePage({
  autofireClassSearch,
  closeTarget,
  openTarget,
  selectedKeys,
  selectedTitle,
  setAutofireClassSearch,
  profileDisplay,
  profiles,
  target,
  visibleAutofireClassCategories,
  visibleCustomConfigs,
  onAddKey: addKey,
  onDeleteKey: deleteKey,
  onEffectRuleChange: updateClassEffectRule,
  onKeyUpdate: updateKey,
}: AutofirePageProps) {
  return (
    <div className="relative h-full min-w-0">
      <section className="h-full min-w-0 overflow-y-auto px-7 py-6" onClick={closeTarget}>
        <header className="mb-5 flex items-start justify-between gap-5">
          <div>
            <div className="flex items-center gap-2">
              <h1 className="text-[22px] font-semibold tracking-tight">按键连发</h1>
            </div>
            <div className="mt-1 space-y-1 text-sm leading-6 text-slate-500">
              <p>选择全局、职业或自定义配置，在右侧编辑键位和连发间隔。</p>
            </div>
          </div>
          <div className="flex shrink-0 items-center gap-3">
            {isMockMode() && (
              <span className="rounded bg-amber-100 px-2 py-1 text-xs text-amber-800">
                浏览器预览
              </span>
            )}
            <label className="flex h-9 w-[260px] items-center gap-2 rounded border border-slate-200 bg-white px-2.5 text-slate-500 shadow-sm focus-within:border-blue-400 focus-within:ring-1 focus-within:ring-blue-100">
              <Search size={15} />
              <input
                className="min-w-0 flex-1 bg-transparent text-sm text-slate-800 outline-none placeholder:text-slate-400"
                placeholder="搜索职业/自定义配置"
                value={autofireClassSearch}
                onChange={(event) => setAutofireClassSearch(event.currentTarget.value)}
              />
            </label>
          </div>
        </header>

        <button
          className={cardClass(target?.type === "global")}
          type="button"
          onClick={(event) => {
            event.stopPropagation();
            openTarget({ type: "global" });
          }}
        >
          <div>
            <div className="text-sm font-semibold">全局配置</div>
            <div className="mt-2 flex flex-wrap gap-1.5">
              <KeySummary active={target?.type === "global"} keys={profiles.globalKeys} />
            </div>
          </div>
        </button>

        {visibleCustomConfigs.length > 0 && (
          <div className="mt-6">
            <h2 className="mb-3 text-sm font-semibold text-slate-700">自定义配置</h2>
            <div className="flex flex-wrap gap-2">
              {visibleCustomConfigs.map(([configId, customConfig]) => {
                const active = target?.type === "profile" && target.configId === configId;
                const configured = profileDisplay.customConfigStates[configId]?.hasKeys ?? false;
                return (
                  <button
                    key={configId}
                    className={classButtonClass(active)}
                    type="button"
                    onClick={(event) => {
                      event.stopPropagation();
                      openTarget({ type: "profile", configId });
                    }}
                  >
                    <span>{customConfig.name || "未命名配置"}</span>
                    {configured && <span className={configuredDotClass(active)} />}
                  </button>
                );
              })}
            </div>
          </div>
        )}

        <div className="mt-6 pb-6">
          <h2 className="mb-3 text-sm font-semibold text-slate-700">职业配置</h2>
          <div className="space-y-4">
            {visibleAutofireClassCategories.map((category) => (
              <div key={category.name} className="grid grid-cols-[84px_1fr] items-center gap-3">
                <div className="text-sm font-medium text-slate-600">{category.name}</div>
                <div className="flex flex-wrap gap-2">
                  {category.classes.map((classInfo) => {
                    const active = target?.type === "profile" && target.configId === classInfo.id;
                    const configured = profileDisplay.classStates[classInfo.id]?.hasKeys ?? false;
                    return (
                      <span key={classInfo.id} className="group relative inline-flex">
                        <button
                          className={classButtonClass(active)}
                          type="button"
                          onClick={(event) => {
                            event.stopPropagation();
                            openTarget({ type: "profile", configId: classInfo.id });
                          }}
                        >
                          <span>{classInfo.name}</span>
                          {configured && <span className={configuredDotClass(active)} />}
                        </button>
                      </span>
                    );
                  })}
                </div>
              </div>
            ))}
            {visibleAutofireClassCategories.length === 0 && (
              <div className="rounded border border-dashed border-slate-200 bg-white px-3 py-8 text-center text-sm text-slate-500">
                没有匹配的职业。
              </div>
            )}
          </div>
        </div>
      </section>

      {target && (
        <div
          className="absolute inset-0 z-20 flex justify-end bg-slate-950/10"
          onClick={closeTarget}
        >
          <aside
            className="flex h-full w-[440px] max-w-[calc(100vw-220px)] flex-col overflow-visible border-l border-slate-200 bg-white shadow-2xl"
            onClick={(event) => event.stopPropagation()}
          >
            <header className="border-b border-slate-200 bg-white px-6 py-5">
              <h2 className="truncate text-xl font-semibold text-slate-900">{selectedTitle}</h2>
              <p className="mt-1 text-xs text-slate-500">
                按键、模式与间隔会自动保存。推荐间隔为20-30ms，1秒=1000毫秒。
              </p>
              <p className="mt-1 text-xs text-slate-500">连发模式说明：</p>
              <p className="mt-1 text-xs text-slate-500">1. 长按：需一直按住这个键</p>
              <p className="mt-1 text-xs text-slate-500">2. 切换：按一次连发，再按一次取消</p>
            </header>

            {target.type === "profile" && (
              <div className="px-6 py-5">
                <div>
                  <div className="mb-2 flex items-center gap-1.5 text-xs font-medium text-slate-500">
                    <span>配置生效规则</span>
                    <RuleHelpTooltip />
                  </div>
                  <div className="grid grid-cols-2 rounded border border-slate-200 bg-slate-50 p-1">
                    <RuleButton
                      active={
                        getProfileConfig(profiles, target.configId).effectRule === "globalAndClass"
                      }
                      label="全局 + 当前配置"
                      onClick={() => updateClassEffectRule("globalAndClass")}
                    />
                    <RuleButton
                      active={
                        getProfileConfig(profiles, target.configId).effectRule === "classOnly"
                      }
                      label="仅当前配置"
                      onClick={() => updateClassEffectRule("classOnly")}
                    />
                  </div>
                </div>
              </div>
            )}

            <div className="min-h-0 flex-1 px-6 py-5">
              <KeyTable
                keys={selectedKeys}
                onAdd={addKey}
                onDelete={deleteKey}
                onUpdate={updateKey}
              />
            </div>
          </aside>
        </div>
      )}
    </div>
  );
}

function cardClass(active: boolean): string {
  return `w-full rounded border px-4 py-3 text-left transition ${
    active
      ? "border-blue-300 bg-blue-50 shadow-sm"
      : "border-slate-200 bg-white shadow-sm hover:border-blue-200 hover:bg-blue-50/40"
  }`;
}

function classButtonClass(active: boolean): string {
  return `relative inline-flex h-9 w-[120px] items-center justify-center rounded border px-2 text-sm transition ${
    active
      ? "border-blue-400 bg-blue-600 text-white shadow-sm"
      : "border-slate-200 bg-white text-slate-700 hover:border-blue-300 hover:bg-blue-50"
  }`;
}

function configuredDotClass(active: boolean): string {
  return `absolute top-1.5 right-1.5 h-1.5 w-1.5 rounded-full ${
    active ? "bg-white" : "bg-blue-500"
  }`;
}
