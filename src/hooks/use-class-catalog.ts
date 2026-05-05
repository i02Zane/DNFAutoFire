import { useEffect, useState } from "react";
import { tauriCommands, type ClassCategory } from "../lib/tauri";

export function useClassCatalog(): ClassCategory[] {
  const [classCategories, setLoadedClassCategories] = useState<ClassCategory[]>([]);

  useEffect(() => {
    let disposed = false;
    void tauriCommands
      .loadClassCategories()
      .then((loadedClassCategories) => {
        if (disposed) return;
        setLoadedClassCategories(loadedClassCategories);
      })
      .catch(() => undefined);

    return () => {
      disposed = true;
    };
  }, []);

  return classCategories;
}
