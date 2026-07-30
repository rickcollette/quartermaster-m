export type DocumentMode = "atascii" | "ascii";

export interface Cell {
  byte: number;
  inverse: boolean;
  display: string;
}

export interface LoadedDocument {
  path: string | null;
  mode: DocumentMode;
  width: number;
  height: number;
  cells: Cell[];
  warnings: string[];
}

export interface SaveDocumentRequest {
  path: string;
  mode: DocumentMode;
  width: number;
  height: number;
  cells: Cell[];
  trimTrailingSpaces: boolean;
}

export interface UpdateInfo {
  currentVersion: string;
  latestVersion: string;
  state: "available" | "current" | "newer";
  platform: "windows" | "macos" | "other";
  exeFile: string | null;
  msiFile: string | null;
  dmgFile: string | null;
}

export interface UpdateDownload {
  version: string;
  path: string;
  launched: boolean;
}

export interface AtrTreeEntry {
  name: string;
  path: string;
  isDirectory: boolean;
  sizeBytes: number;
  children: AtrTreeEntry[];
}

export interface AtrDriveStatus {
  drive: string;
  mounted: boolean;
  active: boolean;
  path: string | null;
  filesystem: string | null;
  entries: string[];
  info: string[];
  tree: AtrTreeEntry[];
}

export interface LocalTreeEntry {
  name: string;
  path: string;
  isDirectory: boolean;
  sizeBytes: number;
  children: LocalTreeEntry[];
}

export interface AtrStatus {
  mounted: boolean;
  activeDrive: string | null;
  path: string | null;
  filesystem: string | null;
  entries: string[];
  info: string[];
  tree: AtrTreeEntry[];
  drives: AtrDriveStatus[];
  localFolder: string | null;
  localTree: LocalTreeEntry[];
}
