// 前端入口：先安装桌面端交互保护，再挂载 React 应用。
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./app";
import "./index.css";
import { installWindowInteractionGuards } from "./lib/window-interactions";

// 生产版桌面窗口不需要浏览器默认右键、拖拽和非输入区选中文本行为。
installWindowInteractionGuards();

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
