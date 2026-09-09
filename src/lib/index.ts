export {
  addProject,
  getAppStatus,
  getKeepAwakeStatus,
  getSettings,
  listProjects,
  removeProject,
  restartProject,
  scanProjects,
  setKeepAwakeEnabled,
  setScanRoots,
  startProject,
  stopProject,
  updateProjectCommands,
} from "./api";
export {
  clearProjectLogs,
  getProjectLogs,
  listenProjectLogs,
} from "./logs";
export {
  getProjectStack,
  openCompass,
  peekProjectStack,
  resolvePortConflict,
  startProjectStack,
  stopProjectStack,
} from "./docker";
import type { AppErrorPayload, PortConflict } from "../types";

export function formatInvokeError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  if (
    typeof error === "object" &&
    error !== null &&
    "message" in error &&
    typeof error.message === "string"
  ) {
    return error.message;
  }
  return "Something went wrong";
}

export function parsePortConflict(error: unknown): PortConflict | null {
  if (typeof error !== "object" || error === null) {
    return null;
  }
  const payload = error as AppErrorPayload;
  if (payload.code !== "port_conflict" || !payload.portConflict) {
    return null;
  }
  return payload.portConflict;
}
