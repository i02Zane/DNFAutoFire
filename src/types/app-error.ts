import type {
  AppError as GeneratedAppError,
  AppErrorKind as GeneratedAppErrorKind,
} from "../generated/backend-types";

export type AppErrorKind = GeneratedAppErrorKind | "transport";
export type BackendAppError = GeneratedAppError;
type FrontendAppError = {
  kind: AppErrorKind;
  message: string;
};

export class TauriCommandError extends Error {
  readonly kind: AppErrorKind;

  constructor(error: FrontendAppError) {
    super(error.message);
    this.name = "TauriCommandError";
    this.kind = error.kind;
  }
}

export function normalizeAppError(reason: unknown): TauriCommandError {
  if (isBackendAppError(reason)) {
    return new TauriCommandError(reason);
  }
  if (reason instanceof Error) {
    return new TauriCommandError({
      kind: "transport",
      message: reason.message || "调用后端失败",
    });
  }
  return new TauriCommandError({
    kind: "transport",
    message: "调用后端失败",
  });
}

export function appErrorMessage(reason: unknown): string {
  return normalizeAppError(reason).message;
}

function isBackendAppError(value: unknown): value is BackendAppError {
  if (!value || typeof value !== "object") return false;
  const candidate = value as Partial<BackendAppError>;
  return typeof candidate.kind === "string" && typeof candidate.message === "string";
}
