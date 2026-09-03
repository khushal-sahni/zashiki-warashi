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
  projectHasCompose,
} from "./logs";

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
