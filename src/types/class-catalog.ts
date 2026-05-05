export type ClassInfo = {
  id: string;
  name: string;
  detectionIndex: number;
};

export type ClassCategory = {
  name: string;
  classes: ClassInfo[];
};
