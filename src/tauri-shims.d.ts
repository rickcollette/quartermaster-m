declare module "@tauri-apps/api/core" {
  export function invoke<T = unknown>(command: string, args?: Record<string, unknown>): Promise<T>;
}
declare module "@tauri-apps/plugin-dialog" {
  export interface DialogFilter { name: string; extensions: string[]; }
  export interface OpenDialogOptions {
    multiple?: boolean;
    directory?: boolean;
    defaultPath?: string;
    title?: string;
    filters?: DialogFilter[];
  }
  export interface SaveDialogOptions { defaultPath?: string; title?: string; filters?: DialogFilter[]; }
  export function open(options?: OpenDialogOptions): Promise<string | null>;
  export function save(options?: SaveDialogOptions): Promise<string | null>;
}
declare module "*?raw" {
  const content: string;
  export default content;
}
