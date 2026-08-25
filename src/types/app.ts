export interface AppStatus {
  readonly name: string;
  readonly version: string;
  readonly databaseReady: boolean;
  readonly appDataDir: string;
  readonly schemaVersion: number;
}

export interface AppErrorPayload {
  readonly code: string;
  readonly message: string;
}
