import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";

const root = process.cwd();
const ignoredDirs = new Set([".git", "dist", "node_modules", "target"]);
const textExtensions = new Set([
  ".css",
  ".html",
  ".js",
  ".jsx",
  ".json",
  ".md",
  ".mjs",
  ".rs",
  ".toml",
  ".ts",
  ".tsx",
]);

const suspiciousFragments = [
  "\ufffd",
  "\u93c8\ue041\u8a2d\u7f03", // "未设置" read/written with the wrong encoding.
  "\u93c8\u5930\u6554",
  "\u5ba5\u30e5\u53bf",
  "\u93c8\u20ac\u704f",
  "\u936f\u62bd\u68f4",
  "\u95b9\u7ed8\u5297",
  "\u7ead\ue3c6\u7568",
  "\u934f\u3125\u772c",
  "\u95b0\u5d88\u5f7f",
  "\u95ab\u5925\u5ad3",
  "\u9366\u5938\u5f42",
  "\u93b8\ue1bc\u60d6",
  "\u95c2\u64ae\u6bab",
  "\u59e3\ue081\ue757",
  "\u9362\u7281\u6afa",
  "\u7487\u950b\u5bdc",
  "\u9422\u719a\u6645",
  "\u7479\u52eb\u579d",
  "\u9441\u807d\u7b1f",
  "\u935a\u5c7e\u693f",
  "\u6d63\u8de8\u657e",
  "\u95bf",
  "\u6d60\u545a\u657e",
  "\u6d63\u7474\u6564",
];

function collectTextFiles(dir, files = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (entry.isDirectory()) {
      if (!ignoredDirs.has(entry.name)) {
        collectTextFiles(path.join(dir, entry.name), files);
      }
      continue;
    }

    if (textExtensions.has(path.extname(entry.name))) {
      files.push(path.join(dir, entry.name));
    }
  }

  return files;
}

const findings = [];

for (const file of collectTextFiles(root)) {
  const relativePath = path.relative(root, file);
  const content = readFileSync(file, "utf8");
  const lines = content.split(/\r?\n/);

  for (const [index, line] of lines.entries()) {
    if (suspiciousFragments.some((fragment) => line.includes(fragment))) {
      findings.push(`${relativePath}:${index + 1}: ${line.trim()}`);
    }
  }
}

if (findings.length > 0) {
  console.error("Suspicious mojibake or replacement characters found:");
  console.error(findings.join("\n"));
  process.exitCode = 1;
}
