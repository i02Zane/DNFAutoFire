import { ExternalLink } from "lucide-react";
import { APP_DISPLAY_NAME, APP_VERSION } from "../lib/app-meta";

const CONTRIBUTORS = [
  {
    href: "https://github.com/mouyase",
    name: "mouyase",
    avatarSrc: "/avatars/mouyase.png",
    role: "原项目作者",
  },
  {
    href: "https://github.com/i02Zane",
    name: "i02Zane",
    avatarSrc: "/avatars/i02zane.png",
    role: "当前维护者",
  },
] as const;

const NOTO_SANS_SC_LICENSE_URL = "https://scripts.sil.org/OFL";
const QQ_GROUP_JOIN_URL = "https://qm.qq.com/q/Y3hwQe1Og4";

export function AboutPage() {
  return (
    <main className="h-full min-w-0 overflow-auto px-7 py-6">
      <section className="max-w-[760px]">
        <div className="flex items-center gap-2">
          <h1 className="text-[22px] font-semibold tracking-tight">关于</h1>
        </div>

        <div className="mt-6 space-y-4">
          <section className="rounded border border-slate-200 bg-white p-5 shadow-sm">
            <h2 className="text-base font-semibold text-slate-900">
              {APP_DISPLAY_NAME} {APP_VERSION}
            </h2>
            <p className="mt-2 text-sm leading-6 text-slate-600">
              源码公开项目，仅供学习、研究和个人使用，禁止商业用途。
            </p>
            <p className="mt-2 text-sm leading-6 text-slate-600">
              本工具免费使用。如果你是付费获取的本工具，请申请退款并举报卖家。
            </p>
          </section>

          <section className="rounded border border-slate-200 bg-white p-5 shadow-sm">
            <h2 className="text-base font-semibold text-slate-900">安全声明</h2>
            <p className="mt-2 text-sm leading-6 text-slate-600">
              本工具在过去、现在、将来都不提供任何关于自动刷图、搬砖等涉及工作室外挂行为的功能。
            </p>
            <p className="mt-2 text-sm leading-6 text-slate-600">
              但本工具仍属于非官方提供的第三方软件，使用本工具造成的风险由使用者自行承担。
            </p>
          </section>

          <section className="rounded border border-slate-200 bg-white p-5 shadow-sm">
            <h2 className="text-sm font-semibold text-slate-900">项目来源 / 贡献者</h2>
            <div className="mt-3 grid gap-3 sm:grid-cols-2">
              {CONTRIBUTORS.map((contributor) => (
                <ContributorCard key={contributor.name} {...contributor} />
              ))}
            </div>
          </section>

          <section className="rounded border border-slate-200 bg-white p-5 shadow-sm">
            <h2 className="text-sm font-semibold text-slate-900">交流群</h2>
            <a
              className="mt-3 inline-flex items-center gap-1.5 text-sm font-medium text-blue-700 hover:text-blue-800"
              href={QQ_GROUP_JOIN_URL}
              rel="noreferrer"
              target="_blank"
            >
              QQ群号：810286069
              <ExternalLink size={14} />
            </a>
          </section>

          <section className="rounded border border-slate-200 bg-white p-5 shadow-sm">
            <h2 className="text-sm font-semibold text-slate-900">字体声明</h2>
            <p className="mt-2 text-sm leading-6 text-slate-600">
              本应用界面使用 Noto Sans SC 字体，字体遵循
              <a
                className="mx-1 font-medium text-blue-700 hover:text-blue-800"
                href={NOTO_SANS_SC_LICENSE_URL}
                rel="noreferrer"
                target="_blank"
              >
                SIL Open Font License 1.1
              </a>
              。
            </p>
          </section>
        </div>
      </section>
    </main>
  );
}

function ContributorCard({
  href,
  name,
  avatarSrc,
  role,
}: {
  href: string;
  name: string;
  avatarSrc: string;
  role: string;
}) {
  return (
    <a
      className="flex min-w-0 items-center gap-3 rounded border border-slate-200 bg-slate-50 px-3 py-3 transition hover:border-blue-200 hover:bg-blue-50/40"
      href={href}
      rel="noreferrer"
      target="_blank"
    >
      <AvatarImage name={name} src={avatarSrc} />
      <div className="min-w-0 flex-1">
        <div className="flex min-w-0 items-center gap-1.5">
          <span className="truncate text-sm font-semibold text-slate-900">{name}</span>
          <ExternalLink className="shrink-0 text-slate-400" size={14} />
        </div>
        <div className="mt-0.5 text-xs text-slate-500">{role}</div>
      </div>
    </a>
  );
}

function AvatarImage({ name, src }: { name: string; src: string }) {
  return (
    <img
      alt={`${name} avatar`}
      className="h-12 w-12 shrink-0 rounded-full border border-slate-200 bg-white object-cover shadow-sm"
      src={src}
    />
  );
}
