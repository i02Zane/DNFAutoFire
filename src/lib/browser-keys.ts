// 浏览器 KeyboardEvent 到 Windows VK 码的转换，前后端统一只传 VK number。
export function isValidComboHotkey(event: KeyboardEvent, vk: number): boolean {
  return (event.ctrlKey || event.altKey || event.shiftKey) && !isModifierVk(vk);
}

export function isModifierVk(vk: number): boolean {
  return vk === 0x10 || vk === 0x11 || vk === 0x12 || (vk >= 0xa0 && vk <= 0xa5);
}

export function browserKeyToVk(event: KeyboardEvent): number | null {
  // 使用 event.code 读取物理键位，避免不同输入法或键盘布局影响 key 字符。
  if (/^Key[A-Z]$/.test(event.code)) return event.code.charCodeAt(3);
  if (/^Digit[0-9]$/.test(event.code)) return event.code.charCodeAt(5);
  if (/^F([1-9]|1[0-2])$/.test(event.code)) return 0x6f + Number(event.code.slice(1));
  if (/^Numpad[0-9]$/.test(event.code)) return 0x60 + Number(event.code.slice(6));
  if (event.code === "Space") return 0x20;
  if (event.code === "Tab") return 0x09;
  if (event.code === "Enter") return 0x0d;
  if (event.code === "ShiftLeft") return 0xa0;
  if (event.code === "ShiftRight") return 0xa1;
  if (event.code === "ControlLeft") return 0xa2;
  if (event.code === "ControlRight") return 0xa3;
  if (event.code === "AltLeft") return 0xa4;
  if (event.code === "AltRight") return 0xa5;
  if (event.code === "Pause") return 0x13;
  if (event.code === "CapsLock") return 0x14;
  if (event.code === "Escape") return 0x1b;
  if (event.code === "ArrowLeft") return 0x25;
  if (event.code === "ArrowUp") return 0x26;
  if (event.code === "ArrowRight") return 0x27;
  if (event.code === "ArrowDown") return 0x28;
  if (event.code === "Insert") return 0x2d;
  if (event.code === "Delete") return 0x2e;
  if (event.code === "NumpadMultiply") return 0x6a;
  if (event.code === "NumpadAdd") return 0x6b;
  if (event.code === "NumpadSubtract") return 0x6d;
  if (event.code === "NumpadDecimal") return 0x6e;
  if (event.code === "NumpadDivide") return 0x6f;
  if (event.code === "Semicolon") return 0xba;
  if (event.code === "Equal") return 0xbb;
  if (event.code === "Comma") return 0xbc;
  if (event.code === "Minus") return 0xbd;
  if (event.code === "Period") return 0xbe;
  if (event.code === "Slash") return 0xbf;
  if (event.code === "Backquote") return 0xc0;
  if (event.code === "BracketLeft") return 0xdb;
  if (event.code === "Backslash") return 0xdc;
  if (event.code === "BracketRight") return 0xdd;
  if (event.code === "Quote") return 0xde;
  return null;
}
