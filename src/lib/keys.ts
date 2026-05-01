// Windows Virtual Key 码表：UI 展示、键位录制和后端 SendInput 都使用同一套编号。
export type KeyOption = {
  vk: number;
  label: string;
};

export const keyOptions: KeyOption[] = [
  { vk: 0x41, label: "A" },
  { vk: 0x42, label: "B" },
  { vk: 0x43, label: "C" },
  { vk: 0x44, label: "D" },
  { vk: 0x45, label: "E" },
  { vk: 0x46, label: "F" },
  { vk: 0x47, label: "G" },
  { vk: 0x48, label: "H" },
  { vk: 0x49, label: "I" },
  { vk: 0x4a, label: "J" },
  { vk: 0x4b, label: "K" },
  { vk: 0x4c, label: "L" },
  { vk: 0x4d, label: "M" },
  { vk: 0x4e, label: "N" },
  { vk: 0x4f, label: "O" },
  { vk: 0x50, label: "P" },
  { vk: 0x51, label: "Q" },
  { vk: 0x52, label: "R" },
  { vk: 0x53, label: "S" },
  { vk: 0x54, label: "T" },
  { vk: 0x55, label: "U" },
  { vk: 0x56, label: "V" },
  { vk: 0x57, label: "W" },
  { vk: 0x58, label: "X" },
  { vk: 0x59, label: "Y" },
  { vk: 0x5a, label: "Z" },
  { vk: 0x30, label: "0" },
  { vk: 0x31, label: "1" },
  { vk: 0x32, label: "2" },
  { vk: 0x33, label: "3" },
  { vk: 0x34, label: "4" },
  { vk: 0x35, label: "5" },
  { vk: 0x36, label: "6" },
  { vk: 0x37, label: "7" },
  { vk: 0x38, label: "8" },
  { vk: 0x39, label: "9" },
  { vk: 0x09, label: "Tab" },
  { vk: 0x0d, label: "Enter" },
  { vk: 0x10, label: "Shift" },
  { vk: 0x11, label: "Ctrl" },
  { vk: 0x12, label: "Alt" },
  { vk: 0x13, label: "Pause" },
  { vk: 0x14, label: "Caps" },
  { vk: 0x1b, label: "Esc" },
  { vk: 0x20, label: "Space" },
  { vk: 0x25, label: "←" },
  { vk: 0x26, label: "↑" },
  { vk: 0x27, label: "→" },
  { vk: 0x28, label: "↓" },
  { vk: 0x2d, label: "Insert" },
  { vk: 0x2e, label: "Delete" },
  { vk: 0x60, label: "NUM0" },
  { vk: 0x61, label: "NUM1" },
  { vk: 0x62, label: "NUM2" },
  { vk: 0x63, label: "NUM3" },
  { vk: 0x64, label: "NUM4" },
  { vk: 0x65, label: "NUM5" },
  { vk: 0x66, label: "NUM6" },
  { vk: 0x67, label: "NUM7" },
  { vk: 0x68, label: "NUM8" },
  { vk: 0x69, label: "NUM9" },
  { vk: 0x6a, label: "NUM*" },
  { vk: 0x6b, label: "NUM+" },
  { vk: 0x6d, label: "NUM-" },
  { vk: 0x6e, label: "NUM." },
  { vk: 0x6f, label: "NUM/" },
  { vk: 0x70, label: "F1" },
  { vk: 0x71, label: "F2" },
  { vk: 0x72, label: "F3" },
  { vk: 0x73, label: "F4" },
  { vk: 0x74, label: "F5" },
  { vk: 0x75, label: "F6" },
  { vk: 0x76, label: "F7" },
  { vk: 0x77, label: "F8" },
  { vk: 0x78, label: "F9" },
  { vk: 0x79, label: "F10" },
  { vk: 0x7a, label: "F11" },
  { vk: 0x7b, label: "F12" },
  { vk: 0xa0, label: "LShift" },
  { vk: 0xa1, label: "RShift" },
  { vk: 0xa2, label: "LCtrl" },
  { vk: 0xa3, label: "RCtrl" },
  { vk: 0xa4, label: "LAlt" },
  { vk: 0xa5, label: "RAlt" },
  { vk: 0xba, label: ";" },
  { vk: 0xbb, label: "=" },
  { vk: 0xbc, label: "," },
  { vk: 0xbd, label: "-" },
  { vk: 0xbe, label: "." },
  { vk: 0xbf, label: "/" },
  { vk: 0xc0, label: "`" },
  { vk: 0xdb, label: "[" },
  { vk: 0xdc, label: "\\" },
  { vk: 0xdd, label: "]" },
  { vk: 0xde, label: "'" },
];

export function keyLabel(vk: number): string {
  if (vk >= 0x41 && vk <= 0x5a) return String.fromCharCode(vk);
  if (vk >= 0x30 && vk <= 0x39) return String.fromCharCode(vk);
  if (vk >= 0x60 && vk <= 0x69) return `NUM${vk - 0x60}`;
  if (vk >= 0x70 && vk <= 0x87) return `F${vk - 0x6f}`;
  return keyOptions.find((option) => option.vk === vk)?.label ?? `VK ${vk}`;
}

export function normalizeInterval(value: number): number {
  if (!Number.isFinite(value)) return 20;
  return Math.max(10, Math.min(1000, Math.trunc(value)));
}

export function hotkeyKeyLabel(vk: number): string {
  return keyLabel(vk);
}
