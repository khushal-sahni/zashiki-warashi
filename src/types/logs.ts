export type LogSource = "process" | "compose";

export interface LogChunk {
  readonly projectId: string;
  readonly source: LogSource;
  readonly lines: readonly string[];
  readonly truncated: boolean;
}
