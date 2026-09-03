export type DbKind = "postgres" | "mysql" | "mongo" | "redis" | "search";

export type PortOccupant =
  | {
      readonly kind: "catalogProject";
      readonly projectId: string;
      readonly name: string;
      readonly service: string;
    }
  | {
      readonly kind: "dockerOther";
      readonly containerName: string;
      readonly service: string | null;
    }
  | {
      readonly kind: "nativeProcess";
      readonly pid: number;
      readonly command: string;
    }
  | { readonly kind: "unknown" };

export interface PortConflict {
  readonly projectId: string;
  readonly service: string;
  readonly hostPort: number;
  readonly suggestedPort: number;
  readonly occupant: PortOccupant;
}

export interface DbEndpoint {
  readonly service: string;
  readonly kind: DbKind;
  readonly host: string;
  readonly port: number;
  readonly user: string | null;
  readonly database: string | null;
  readonly uriMasked: string;
  readonly uri: string;
  readonly running: boolean;
}

export interface ProjectStack {
  readonly composeFile: string | null;
  readonly endpoints: readonly DbEndpoint[];
}

export type ReconcileAction = "stopOccupant" | "remap";
