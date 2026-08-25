export {
  addProject,
  getAppStatus,
  getSettings,
  listProjects,
  removeProject,
  restartProject,
  scanProjects,
  setScanRoots,
  startProject,
  stopProject,
  updateProjectCommands,
} from "./api";

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
