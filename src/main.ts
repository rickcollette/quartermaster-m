import "./style.css";
import "./generated/atari-glyphs.css";
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open, save } from "@tauri-apps/plugin-dialog";
import { atariGlyphClass, atariGlyphLabel, controlByteForKey } from "./atariGlyphs";
import { showHelpCenter } from "./help";
import { APP_VERSION } from "./version";
import LICENSE_TEXT from "../LICENSE?raw";
import type { AtrDriveStatus, AtrStatus, AtrTreeEntry, Cell, DocumentMode, LoadedDocument, LocalTreeEntry, SaveDocumentRequest, UpdateDownload, UpdateInfo } from "./types";

const DOCUMENT_ROWS = 357;
const VIEW_ROWS = 24;
const blankCell = (): Cell => ({ byte: 0x20, inverse: false, display: " " });
const ACTIVITY_LABELS: Record<string,string> = {
  load_document:"Loading",save_document:"Saving",local_open_folder:"Loading Folder",
  atr_status:"Refreshing",atr_create:"Creating Disk",atr_mount:"Mounting",atr_select_drive:"Loading Directory",atr_close:"Unmounting",
  atr_open_document:"Loading",atr_write_document:"Saving",atr_add_host_file:"Importing",atr_import_host_files:"Importing",
  atr_copy_file:"Copying",atr_extract_file:"Extracting",atr_export_ascii:"Exporting",atr_delete_file:"Deleting",
  atr_delete_entry:"Deleting",atr_rename_entry:"Renaming",atr_mkdir:"Creating Folder",
  basic_detokenize_host:"Detokenizing",basic_tokenize_host:"Tokenizing",basic_save_listing_host:"Saving",
  basic_detokenize_atr:"Detokenizing",basic_tokenize_to_atr:"Tokenizing",basic_save_listing_to_atr:"Saving",
  check_for_updates:"Checking for Updates",download_portable_update:"Downloading Update",download_and_install_update:"Downloading Installer",download_macos_update:"Downloading Update"
};
type ActivityRunner = <T>(label:string,operation:()=>Promise<T>)=>Promise<T>;
let activityRunner:ActivityRunner|null=null;
const invoke=<T=unknown>(command:string,args?:Record<string,unknown>):Promise<T>=>{
  const operation=()=>tauriInvoke<T>(command,args);
  return activityRunner?activityRunner(ACTIVITY_LABELS[command]??"Working",operation):operation();
};
type CellChange = { start: number; end: number };
type AtrFormatConfig = {
  drive: number;
  filesystem: "DOS2" | "SPARTA";
  sectors: number;
  sectorSize: 128 | 256;
  volumeLabel: string | null;
};
type ExplorerDropTarget = { drive: number; directory: string; element: HTMLElement };
type ExplorerDragSource =
  | { kind: "atr"; entry: AtrTreeEntry; drive: number }
  | { kind: "local"; path: string };
type ActiveStorageLocation =
  | { kind: "local"; directory: string }
  | { kind: "atr"; drive: number; directory: string };
type AtrDocumentLocation = { drive: number; path: string };
type SelectionBounds = { top:number;left:number;bottom:number;right:number;width:number;height:number };
type EditorClipboard = { width:number;height:number;cells:Cell[];text:string };

class Editor {
  private width = 40;
  private height = DOCUMENT_ROWS;
  private cells: Cell[] = Array.from({ length: this.width * this.height }, blankCell);
  private cursor = 0;
  private mode: DocumentMode = "atascii";
  private inverse = false;
  private insertMode = false;
  private path: string | null = null;
  private dirty = false;
  private atr: AtrStatus = { mounted: false, activeDrive: null, path: null, filesystem: null, entries: [], info: [], tree: [], drives: [], localFolder: null, localTree: [] };
  private selectedAtrPath: string | null = null;
  private selectedAtrDrive = 1;
  private selectedLocalPath: string | null = null;
  private activeLocationKind: "local" | "atr" = "local";
  private atrDocument: AtrDocumentLocation | null = null;
  private collapsedAtrPaths = new Set<string>();
  private collapsedLocalPaths = new Set<string>();
  private screen!: HTMLElement;
  private hiddenInput!: HTMLInputElement;
  private atrTree!: HTMLElement;
  private atrContextMenu!: HTMLElement;
  private editorContextMenu!: HTMLElement;
  private localTree!: HTMLElement;
  private cellElements: HTMLElement[] = [];
  private rowElements: HTMLElement[] = [];
  private dragGestureCleanup: (() => void) | null = null;
  private dropHighlight: HTMLElement | null = null;
  private activityOverlay!: HTMLElement;
  private activityLabel!: HTMLElement;
  private activitySequence = 0;
  private activities = new Map<number,string>();
  private selectionAnchor = 0;
  private selectionFocus = 0;
  private selectionActive = false;
  private renderedSelection = new Set<number>();
  private editorContextIndex = 0;
  private internalClipboard: EditorClipboard | null = null;
  private findPanel: HTMLElement | null = null;

  async mount(): Promise<void> {
    document.querySelector<HTMLDivElement>("#app")!.innerHTML = `
      <main class="app-shell">
        <header class="titlebar"><span class="brand">QUARTERMASTER/M — ATASCII EDITOR</span><span class="version">V${APP_VERSION}</span></header>
        <nav class="menubar">
          ${this.menu("file", "File", [
            ["new","New"],["open","Open..."],["save","Save"],["saveAs","Save As..."],["exportAscii","Export ASCII..."]
          ])}
          ${this.menu("edit", "Edit", [["find","Find..."],["replace","Find and Replace..."]])}
          ${this.menu("view", "View", [["columns40","40 Column Atari"],["columns80","80 Column XEP-80"]])}
          ${this.menu("atr", "ATR", [
            ["-","-"],["atrMount","Open/Mount"],["atrRefresh","Refresh Directory"],
            ["atrOpenDocument","Open File From ATR"],["atrCreate","Create ATR"],["atrClose","Unmount ATR"]
          ])}
          ${this.menu("basic", "BASIC", [
            ["basicDetokHost","Open Tokenized BASIC From Disk..."],["basicTokHost","Save Tokenized BASIC To Disk..."],
            ["basicListHost","Save Detokenized Listing To Disk..."],["-","-"],
            ["basicDetokAtr","Open Tokenized BASIC From ATR..."],["basicTokAtr","Save Tokenized BASIC To ATR..."],
            ["basicListAtr","Save Detokenized Listing To ATR..."]
          ])}
          ${this.menu("help", "Help", [
            ["help","Help Center"],["helpShortcuts","Keyboard & Mouse"],["helpAtascii","ATASCII Map"],["-","-"],
            ["helpSupport","Get Help / Report Issue"],["helpUpdates","Check for Updates"],["-","-"],["helpLicense","License"],["helpAbout","About"]
          ])}
          <span id="atrMountLabel" class="mount-label">NO ATR MOUNTED</span>
        </nav>
        <nav class="toolbar">
          <div class="toolbar-view-controls">
            <button data-cmd="columns40">40 COL</button><button data-cmd="columns80">XEP-80</button>
          </div>
          <div class="toolbar-editor-controls">
            <div class="toolbar-file-controls"><button data-cmd="new">New</button><button data-cmd="open">Open</button><button data-cmd="save">Save</button></div>
            <span id="activeLocationLabel" class="active-location-label">LOCAL</span>
            <span class="spacer"></span>
            <select id="mode"><option value="atascii">ATASCII</option><option value="ascii">ASCII</option></select>
            <button data-cmd="inverse">Inverse</button><button data-cmd="insert">Insert</button><button data-cmd="clear">Clear</button>
          </div>
        </nav>
        <section class="workbench">
          <aside class="explorer" aria-label="File management">
            <div class="explorer-title">Explorer</div>
            <div class="explorer-section-title">Local Folder</div>
            <div class="explorer-actions">
              <button data-cmd="openFolder">Open Folder</button><button data-cmd="openLocalFile">Open File</button>
            </div>
            <div id="localRootName" class="explorer-drive">NO FOLDER OPENED</div>
            <div id="localTree" class="tree-view" role="tree"></div>
            <div class="explorer-section-title">ATR Mounts</div>
            <div class="explorer-actions">
              <button data-cmd="atrMount">Mount</button><button data-cmd="atrClose">Unmount</button><button data-cmd="atrRefresh">Refresh</button><button data-cmd="atrAddHost">Add</button><button data-cmd="atrMkdir">New Dir</button>
            </div>
            <div id="atrDriveName" class="explorer-drive">D1: NO ATR MOUNTED</div>
            <div id="atrTree" class="tree-view" role="tree"></div>
            <div class="tree-actions">
              <button id="atrTreeOpen" data-tree-cmd="open">Open</button><button id="atrTreeAdd" data-tree-cmd="add">Add</button><button id="atrTreeExtract" data-tree-cmd="extract">Extract</button><button id="atrTreeMkdir" data-tree-cmd="mkdir">New Dir</button><button id="atrTreeDelete" data-tree-cmd="delete">Delete</button><button id="atrTreeUnmount" data-tree-cmd="unmount">Unmount</button>
            </div>
            <div id="atrSelection" class="tree-selection">No file selected</div>
            <div class="drag-hint">Drop host text files onto an ATR. .BAS files are tokenized; other text becomes ATASCII. Drag between ATR drives to copy bytes unchanged.</div>
          </aside>
          <section class="workspace"><div class="screen-frame"><div id="screen" class="screen" role="textbox" tabindex="0"></div></div><input id="keyboard" class="hidden-input" /></section>
        </section>
        <footer class="statusbar"><span class="status-left"><span id="fileName">UNTITLED.ATA</span><span id="position"></span></span><span class="status-right"><span id="modeStatus"></span><span id="byteStatus"></span><span id="geometry"></span></span></footer>
      </main>
      <div id="activityOverlay" class="activity-overlay" hidden>
        <div class="activity-card" role="status" aria-live="polite">
          <span id="activityLabel">Working</span><span class="activity-dots" aria-hidden="true">...</span>
        </div>
      </div>
      <div id="atrContextMenu" class="context-menu" role="menu" aria-label="ATR file actions" hidden></div>
      <div id="editorContextMenu" class="context-menu editor-context-menu" role="menu" aria-label="Editor actions" hidden></div>`;

    this.screen = document.querySelector("#screen")!;
    this.hiddenInput = document.querySelector("#keyboard")!;
    this.atrTree = document.querySelector("#atrTree")!;
    this.atrContextMenu = document.querySelector("#atrContextMenu")!;
    this.editorContextMenu = document.querySelector("#editorContextMenu")!;
    this.localTree = document.querySelector("#localTree")!;
    this.activityOverlay = document.querySelector("#activityOverlay")!;
    this.activityLabel = document.querySelector("#activityLabel")!;
    activityRunner=(label,operation)=>this.withActivity(label,operation);
    this.screen.addEventListener("pointerdown", e => this.onScreenPointerDown(e));
    this.screen.addEventListener("contextmenu", e => this.showEditorContextMenu(e));
    this.screen.addEventListener("focus", () => this.hiddenInput.focus());
    this.hiddenInput.addEventListener("keydown", e => this.onKeyDown(e));
    this.hiddenInput.addEventListener("input", () => this.onTextInput());
    this.hiddenInput.addEventListener("copy",e=>this.onClipboardCopy(e));
    this.hiddenInput.addEventListener("cut",e=>this.onClipboardCut(e));
    this.hiddenInput.addEventListener("paste",e=>this.onClipboardPaste(e));
    document.querySelectorAll<HTMLButtonElement>("[data-cmd]").forEach(b => b.addEventListener("click", () => { this.closeMenus(); void this.command(b.dataset.cmd!); }));
    document.querySelectorAll<HTMLButtonElement>("[data-tree-cmd]").forEach(b => b.addEventListener("click", () => { this.closeMenus(); void this.treeCommand(b.dataset.treeCmd!); }));
    document.querySelectorAll<HTMLButtonElement>("[data-menu]").forEach(b => b.addEventListener("click", e => {
      e.stopPropagation(); const panel = document.querySelector<HTMLElement>(`[data-panel="${b.dataset.menu}"]`)!; const was = panel.classList.contains("open"); this.closeMenus(); if (!was) panel.classList.add("open");
    }));
    document.addEventListener("click", () => { this.closeMenus();this.hideAtrContextMenu();this.hideEditorContextMenu(); });
    window.addEventListener("blur", () => {this.hideAtrContextMenu();this.hideEditorContextMenu();});
    window.addEventListener("resize", () => {this.hideAtrContextMenu();this.hideEditorContextMenu();});
    document.addEventListener("scroll", () => {this.hideAtrContextMenu();this.hideEditorContextMenu();}, true);
    document.querySelector<HTMLSelectElement>("#mode")!.addEventListener("change", e => { this.mode = (e.target as HTMLSelectElement).value as DocumentMode; this.dirty = true; this.renderStatus(); });
    window.addEventListener("keydown", e => {
      if(this.isBusy){if(e.ctrlKey||e.metaKey)e.preventDefault();return;}
      const shiftedCommand=(e.ctrlKey||e.metaKey)&&e.shiftKey;
      if (shiftedCommand && e.key.toLowerCase() === "s") { e.preventDefault(); void this.save(false); }
      if (shiftedCommand && e.key.toLowerCase() === "o") { e.preventDefault(); void this.openFile(); }
      if (shiftedCommand && e.key.toLowerCase() === "n") { e.preventDefault(); void this.newFile(); }
      if (shiftedCommand && e.key.toLowerCase() === "f") { e.preventDefault();this.showFindPanel(false); }
      if (shiftedCommand && e.key.toLowerCase() === "h") { e.preventDefault();this.showFindPanel(true); }
      if (shiftedCommand && e.key.toLowerCase() === "a" && this.editorHasFocus()) { e.preventDefault();this.selectAllCells(); }
      if (shiftedCommand && e.key.toLowerCase() === "c" && this.editorHasFocus()) { e.preventDefault();void this.copySelectionToSystem(false); }
      if (shiftedCommand && e.key.toLowerCase() === "x" && this.editorHasFocus()) { e.preventDefault();void this.copySelectionToSystem(true); }
      if (shiftedCommand && e.key.toLowerCase() === "v" && this.editorHasFocus()) { e.preventDefault();void this.pasteFromSystemClipboard(); }
      if (e.key === "F2") { e.preventDefault(); this.inverse = !this.inverse; this.renderStatus(); }
    });
    this.render();this.renderAtrTree();void this.installHostDropHandler();this.hiddenInput.focus();
    await new Promise<void>(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>resolve())));
    try{await tauriInvoke("app_ready");}catch(error){console.error("Could not finish the splash-screen transition:",error);}

    // Restoring a large local tree or an ATR image can take time (and persisted
    // Windows paths may not exist on macOS). It must not hold the splash screen
    // open, so restore the explorer state after the editor is already usable.
    void this.refreshAtr(false);
  }

  private menu(id: string, title: string, items: string[][]): string {
    const body = items.map(([cmd,label]) => cmd === "-" ? "<hr>" : `<button data-cmd="${cmd}">${label}</button>`).join("");
    return `<div class="menu"><button class="menu-button" data-menu="${id}">${title}</button><div class="menu-panel" data-panel="${id}">${body}</div></div>`;
  }
  private get cellCount(): number { return this.width * this.height; }
  private get isBusy():boolean{return this.activities.size>0;}
  private closeMenus(): void { document.querySelectorAll(".menu-panel.open").forEach(p => p.classList.remove("open")); }
  private updateActivity():void{
    let current:string|null=null;for(const label of this.activities.values())current=label;
    this.activityOverlay.hidden=current===null;
    this.activityLabel.textContent=current??"Working";
    document.querySelector(".app-shell")?.setAttribute("aria-busy",String(current!==null));
  }
  private async withActivity<T>(label:string,operation:()=>Promise<T>):Promise<T>{
    const id=++this.activitySequence;this.activities.set(id,label);this.updateActivity();
    try{return await operation();}finally{this.activities.delete(id);this.updateActivity();}
  }
  private editorHasFocus():boolean{
    const active=document.activeElement;
    return active===this.hiddenInput||active===this.screen||this.editorContextMenu?.contains(active);
  }
  private selectionBounds():SelectionBounds|null{
    if(!this.selectionActive)return null;
    const anchorRow=Math.floor(this.selectionAnchor/this.width),anchorCol=this.selectionAnchor%this.width;
    const focusRow=Math.floor(this.selectionFocus/this.width),focusCol=this.selectionFocus%this.width;
    const top=Math.min(anchorRow,focusRow),bottom=Math.max(anchorRow,focusRow);
    const left=Math.min(anchorCol,focusCol),right=Math.max(anchorCol,focusCol);
    return{top,left,bottom,right,width:right-left+1,height:bottom-top+1};
  }
  private selectedCellIndexes(bounds=this.selectionBounds()):number[]{
    if(!bounds)return[];
    const indexes:number[]=[];
    for(let row=bounds.top;row<=bounds.bottom;row++)for(let col=bounds.left;col<=bounds.right;col++)indexes.push(row*this.width+col);
    return indexes;
  }
  private selectionContains(index:number):boolean{
    const bounds=this.selectionBounds();if(!bounds)return false;
    const row=Math.floor(index/this.width),col=index%this.width;
    return row>=bounds.top&&row<=bounds.bottom&&col>=bounds.left&&col<=bounds.right;
  }
  private renderSelection():void{
    const next=new Set(this.selectedCellIndexes());
    for(const index of this.renderedSelection)if(!next.has(index))this.cellElements[index]?.classList.remove("selected");
    for(const index of next)if(!this.renderedSelection.has(index))this.cellElements[index]?.classList.add("selected");
    this.renderedSelection=next;this.renderStatus();
  }
  private clearEditorSelection(render=true):void{
    this.selectionActive=false;this.selectionAnchor=this.cursor;this.selectionFocus=this.cursor;
    if(render)this.renderSelection();
  }
  private selectAllCells():void{
    const previousCursor=this.cursor;
    this.selectionAnchor=0;this.selectionFocus=this.cellCount-1;this.selectionActive=true;this.cursor=0;
    this.renderSelection();this.updateCell(previousCursor);this.updateCell(0);this.hiddenInput.focus();
  }

  private render(): void {
    this.screen.style.setProperty("--columns", String(this.width));
    this.screen.classList.toggle("xep80", this.width === 80);
    const fragment = document.createDocumentFragment();
    this.cellElements = [];
    this.rowElements = [];
    for(let row=0;row<this.height;row++){
      const rowElement=document.createElement("div");
      rowElement.className="screen-row";
      for(let column=0;column<this.width;column++){
        const index=row*this.width+column,element=this.createCellElement(index);
        this.cellElements.push(element);
        rowElement.appendChild(element);
      }
      this.rowElements.push(rowElement);
      fragment.appendChild(rowElement);
    }
    this.screen.replaceChildren(fragment);
    this.renderedSelection=new Set(this.selectedCellIndexes());
    this.renderStatus();this.scrollCursorIntoView();
  }

  private createCellElement(index:number):HTMLElement{
    const el=document.createElement("span");
    el.dataset.index=String(index);
    this.applyCellElement(el,index);
    return el;
  }

  private applyCellElement(el:HTMLElement,index:number):void{
    const c=this.cells[index]??blankCell();
    const byte=c.byte&0x7f;
    el.className=`cell ${atariGlyphClass(byte)}${c.inverse?" inverse":""}${this.selectionContains(index)?" selected":""}${index===this.cursor?" cursor":""}`;
    el.setAttribute("aria-label",atariGlyphLabel(byte));
  }

  private updateCell(index:number):void{
    if(index<0||index>=this.cellCount)return;
    const el=this.cellElements[index];
    if(el)this.applyCellElement(el,index);
  }
  private updateCellRange(start:number,endExclusive:number):void{
    const from=Math.max(0,start),to=Math.min(this.cellCount,endExclusive);
    for(let index=from;index<to;index++)this.updateCell(index);
  }

  private refreshEditorView(previousCursor:number,change?:CellChange):void{
    if(change){
      const start=Math.max(0,change.start),end=Math.min(this.cellCount-1,change.end);
      for(let i=start;i<=end;i++)this.updateCell(i);
    }
    if(previousCursor!==this.cursor)this.updateCell(previousCursor);
    this.updateCell(this.cursor);
    this.renderStatus();
    this.scrollCursorIntoView();
  }

  private renderStatus(): void {
    const row=Math.floor(this.cursor/this.width)+1, col=this.cursor%this.width+1, current=this.cells[this.cursor]??blankCell();
    const name=this.atrDocument?`${this.basename(this.atrDocument.path)} [D${this.atrDocument.drive}:]`:this.path?.split(/[\\/]/).pop()??(this.mode==="atascii"?"UNTITLED.ATA":"UNTITLED.TXT");
    const f=document.querySelector("#fileName")!; f.textContent=name; f.classList.toggle("dirty",this.dirty);
    const selection=this.selectionBounds();
    document.querySelector("#position")!.textContent=`ROW ${String(row).padStart(3,"0")} COL ${String(col).padStart(2,"0")}${selection?`  SEL ${selection.width}×${selection.height}`:""}`;
    document.querySelector("#modeStatus")!.textContent=`${this.mode.toUpperCase()} ${this.inverse?"INV":"NOR"} ${this.insertMode?"INS":"OVR"}`;
    document.querySelector("#byteStatus")!.textContent=`$${current.byte.toString(16).padStart(2,"0").toUpperCase()}`;
    document.querySelector("#geometry")!.textContent=`${this.width}×${this.height} / ${VIEW_ROWS} ROW VIEW`;
    document.querySelector<HTMLButtonElement>("[data-cmd='inverse']")?.classList.toggle("active",this.inverse);
    document.querySelector<HTMLButtonElement>("[data-cmd='insert']")?.classList.toggle("active",this.insertMode);
    document.querySelector<HTMLButtonElement>("[data-cmd='columns40']")?.classList.toggle("active",this.width===40);
    document.querySelector<HTMLButtonElement>("[data-cmd='columns80']")?.classList.toggle("active",this.width===80);
    const a=document.querySelector("#atrMountLabel"); if(a)a.textContent=this.atr.mounted?`ATR: ${this.atr.path?.split(/[\\/]/).pop()} [${this.atr.filesystem}]`:"NO ATR MOUNTED";
    this.updateActiveLocationLabel();
  }

  private setAtr(status:AtrStatus):void{const firstStatus=this.atr.drives.length===0,drives=status.drives?.length?status.drives:this.defaultDrives();this.atr={...status,tree:status.tree??[],drives,localTree:status.localTree??[]};if(firstStatus)this.selectedAtrDrive=this.driveNumber(this.atr.activeDrive)??this.selectedAtrDrive;if(this.selectedAtrPath&&!this.findAtrTreeEntry(this.selectedDriveStatus()?.tree??[],this.selectedAtrPath))this.selectedAtrPath=null;if(this.selectedLocalPath&&!this.findLocalTreeEntry(this.atr.localTree,this.selectedLocalPath))this.selectedLocalPath=null;this.renderStatus();this.renderAtrTree();}
  private defaultDrives():AtrDriveStatus[]{return[1,2,3,4].map(n=>({drive:`D${n}:`,mounted:false,active:n===1,path:null,filesystem:null,entries:[],info:[],tree:[]}));}
  private renderAtrTree():void{this.renderLocalTree();this.renderDriveTree();}
  private renderLocalTree():void{
    if(!this.localTree)return;
    const rootLabel=document.querySelector("#localRootName");if(rootLabel)rootLabel.textContent=this.atr.localFolder?this.atr.localFolder:"NO FOLDER OPENED";
    this.localTree.replaceChildren();
    if(!this.atr.localFolder){const empty=document.createElement("div");empty.className="tree-empty";empty.textContent="Open a local folder to browse files.";this.localTree.appendChild(empty);return;}
    const root=document.createElement("button");root.type="button";root.className=`tree-root location-root${this.activeLocationKind==="local"&&!this.selectedLocalPath?" location-active":""}`;root.textContent=`- ${this.basename(this.atr.localFolder)}`;root.addEventListener("click",()=>{this.activeLocationKind="local";this.selectedLocalPath=null;this.renderAtrTree();});this.localTree.appendChild(root);
    if(!this.atr.localTree.length){const empty=document.createElement("div");empty.className="tree-empty";empty.textContent="(empty folder)";this.localTree.appendChild(empty);return;}
    this.atr.localTree.forEach(entry=>this.appendLocalTreeEntry(entry,1));
  }
  private renderDriveTree():void{
    if(!this.atrTree)return;
    this.updateAtrSelectionUi();
    this.atrTree.replaceChildren();
    this.atr.drives.forEach(driveStatus=>this.appendDriveRoot(driveStatus));
  }
  private updateAtrSelectionUi():void{
    const selected=this.selectedAtrPath?this.findAtrTreeEntry(this.selectedDriveStatus()?.tree??[],this.selectedAtrPath):null;
    const activeDrive=this.selectedDriveStatus();
    const drive=document.querySelector("#atrDriveName");if(drive)drive.textContent=`D${this.selectedAtrDrive}: ${activeDrive?.mounted?this.basename(activeDrive.path??"ATR"):"NO ATR MOUNTED"}`;
    const selection=document.querySelector("#atrSelection");if(selection)selection.textContent=selected?`D${this.selectedAtrDrive}: ${selected.isDirectory?"DIR ":""}${selected.path}`:"No ATR file selected";
    const isSparta=activeDrive?.filesystem?.toLowerCase().includes("sparta")??false;
    const isMounted=activeDrive?.mounted??false;
    const openButton=document.querySelector<HTMLButtonElement>("#atrTreeOpen");if(openButton)openButton.disabled=!selected||selected.isDirectory;
    const extractButton=document.querySelector<HTMLButtonElement>("#atrTreeExtract");if(extractButton)extractButton.disabled=!selected||selected.isDirectory;
    const deleteButton=document.querySelector<HTMLButtonElement>("#atrTreeDelete");if(deleteButton)deleteButton.disabled=!selected;
    const addButton=document.querySelector<HTMLButtonElement>("#atrTreeAdd");if(addButton)addButton.disabled=!isMounted;
    const mkdirButton=document.querySelector<HTMLButtonElement>("#atrTreeMkdir");if(mkdirButton)mkdirButton.disabled=!isMounted||!isSparta;
    const unmountButton=document.querySelector<HTMLButtonElement>("#atrTreeUnmount");if(unmountButton)unmountButton.disabled=!isMounted;
  }
  private appendDriveRoot(driveStatus:AtrDriveStatus):void{
    const driveNumber=this.driveNumber(driveStatus.drive)??1;
    const collapsed=this.collapsedAtrPaths.has(driveStatus.drive);
    const locationActive=this.activeLocationKind==="atr"&&driveNumber===this.selectedAtrDrive&&!this.selectedAtrPath;
    const row=document.createElement("button");row.type="button";row.className=`tree-row drive-root${driveStatus.active?" active":""}${locationActive?" location-active":""}`;row.style.setProperty("--depth","0");row.title=driveStatus.path??`${driveStatus.drive} empty`;
    if(driveStatus.mounted){row.dataset.atrDrop="true";row.dataset.atrDrive=String(driveNumber);row.dataset.atrDirectory="";}
    row.textContent=`${driveStatus.mounted?(collapsed?"+":"-"):" "} ${driveStatus.drive} ${driveStatus.mounted?`${this.basename(driveStatus.path??"ATR")} [${driveStatus.filesystem}]`:"(empty)"}`;
    row.addEventListener("click",()=>{this.hideAtrContextMenu();this.activeLocationKind="atr";this.selectedAtrDrive=driveNumber;this.selectedAtrPath=null;if(driveStatus.mounted){if(collapsed)this.collapsedAtrPaths.delete(driveStatus.drive);else this.collapsedAtrPaths.add(driveStatus.drive);}void this.selectDrive(driveNumber);this.renderAtrTree();});
    row.addEventListener("contextmenu",event=>{event.preventDefault();event.stopPropagation();this.activeLocationKind="atr";this.selectedAtrDrive=driveNumber;this.selectedAtrPath=null;this.renderLocalTree();this.updateAtrSelectionUi();if(driveStatus.mounted){void this.activateAtrDrive(driveNumber);this.showAtrContextMenu(event,null,driveNumber,row);}});
    this.atrTree.appendChild(row);
    if(!driveStatus.mounted||collapsed)return;
    if(!driveStatus.tree.length){const empty=document.createElement("div");empty.className="tree-empty nested";empty.textContent="(empty image)";this.atrTree.appendChild(empty);return;}
    driveStatus.tree.forEach(entry=>this.appendAtrTreeEntry(entry,1,driveNumber));
  }
  private appendAtrTreeEntry(entry:AtrTreeEntry,depth:number,drive:number):void{
    const key=`D${drive}:${entry.path}`,collapsed=this.collapsedAtrPaths.has(key),selected=this.activeLocationKind==="atr"&&drive===this.selectedAtrDrive&&entry.path===this.selectedAtrPath;
    const row=document.createElement("button");row.type="button";row.className=`tree-row${entry.isDirectory?" directory":" file"}${selected?" selected":""}`;row.style.setProperty("--depth",String(depth));row.title=entry.isDirectory?entry.path:`${entry.path} (${entry.sizeBytes} bytes)`;row.setAttribute("role","treeitem");row.setAttribute("aria-selected",String(selected));
    row.dataset.atrDrop="true";row.dataset.atrDrive=String(drive);row.dataset.atrDirectory=entry.isDirectory?entry.path:this.parentAtrPath(entry.path);
    const twist=document.createElement("span");twist.className="tree-twist";twist.textContent=entry.isDirectory?(collapsed?"+":"-"):"";
    const icon=document.createElement("span");icon.className="tree-icon";icon.textContent=entry.isDirectory?"DIR":"FILE";
    const name=document.createElement("span");name.className="tree-name";name.textContent=entry.name;
    row.append(twist,icon,name);
    row.addEventListener("click",()=>{this.hideAtrContextMenu();this.selectAtrTreeEntry(entry,drive,row);if(entry.isDirectory){if(collapsed)this.collapsedAtrPaths.delete(key);else this.collapsedAtrPaths.add(key);this.renderDriveTree();}});
    row.addEventListener("dblclick",event=>{event.preventDefault();event.stopPropagation();if(!entry.isDirectory)void this.openAtrTreeFile(entry.path,drive);});
    row.addEventListener("contextmenu",event=>{event.preventDefault();event.stopPropagation();this.selectAtrTreeEntry(entry,drive,row);this.showAtrContextMenu(event,entry,drive,row);});
    if(!entry.isDirectory)row.addEventListener("pointerdown",event=>this.armExplorerDrag(event,row,{kind:"atr",entry,drive}));
    this.atrTree.appendChild(row);
    if(entry.isDirectory&&!collapsed)entry.children.forEach(child=>this.appendAtrTreeEntry(child,depth+1,drive));
  }
  private selectAtrTreeEntry(entry:AtrTreeEntry,drive:number,row:HTMLElement):void{
    this.activeLocationKind="atr";this.selectedAtrDrive=drive;this.selectedAtrPath=entry.path;
    this.atrTree.querySelectorAll(".tree-row.selected").forEach(selected=>selected.classList.remove("selected"));
    row.classList.add("selected");row.setAttribute("aria-selected","true");
    this.renderLocalTree();this.updateAtrSelectionUi();this.updateActiveLocationLabel();
  }
  private async activateAtrDrive(drive:number):Promise<void>{if(this.driveNumber(this.atr.activeDrive)!==drive)await this.selectDrive(drive);}
  private showAtrContextMenu(event:MouseEvent,entry:AtrTreeEntry|null,drive:number,anchor:HTMLElement):void{
    const driveStatus=this.atr.drives.find(status=>status.drive===`D${drive}:`);
    if(!driveStatus?.mounted)return;
    const actions:Array<{action:string;label:string;disabled?:boolean}|null>=entry
      ? entry.isDirectory
        ? [{action:"add",label:"Add File..."},{action:"mkdir",label:"New Folder...",disabled:!this.isSpartaDrive(drive)},null,{action:"rename",label:"Rename..."},{action:"delete",label:"Delete"}]
        : [{action:"open",label:"Open"},{action:"exportAscii",label:"Export ASCII..."},{action:"extract",label:"Extract Raw..."},null,{action:"rename",label:"Rename..."},{action:"delete",label:"Delete"}]
      : [{action:"add",label:"Add File..."},{action:"mkdir",label:"New Folder...",disabled:!this.isSpartaDrive(drive)},null,{action:"refresh",label:"Refresh"},{action:"unmount",label:"Unmount"}];
    this.atrContextMenu.replaceChildren();
    for(const item of actions){
      if(!item){const separator=document.createElement("div");separator.className="context-menu-separator";separator.setAttribute("role","separator");this.atrContextMenu.appendChild(separator);continue;}
      const button=document.createElement("button");button.type="button";button.role="menuitem";button.textContent=item.label;button.disabled=item.disabled??false;
      button.addEventListener("click",click=>{click.stopPropagation();this.hideAtrContextMenu();void this.runAtrContextAction(item.action,entry,drive);});
      this.atrContextMenu.appendChild(button);
    }
    this.atrContextMenu.hidden=false;
    const bounds=anchor.getBoundingClientRect();
    const requestedX=event.clientX||bounds.left+18,requestedY=event.clientY||bounds.top+bounds.height;
    const x=Math.max(6,Math.min(requestedX,window.innerWidth-this.atrContextMenu.offsetWidth-6));
    const y=Math.max(6,Math.min(requestedY,window.innerHeight-this.atrContextMenu.offsetHeight-6));
    this.atrContextMenu.style.left=`${x}px`;this.atrContextMenu.style.top=`${y}px`;
    this.atrContextMenu.querySelector<HTMLButtonElement>("button:not(:disabled)")?.focus();
  }
  private hideAtrContextMenu():void{if(this.atrContextMenu)this.atrContextMenu.hidden=true;}
  private isSpartaDrive(drive:number):boolean{return this.atr.drives.find(status=>status.drive===`D${drive}:`)?.filesystem?.toLowerCase().includes("sparta")??false;}
  private async runAtrContextAction(action:string,entry:AtrTreeEntry|null,drive:number):Promise<void>{
    this.activeLocationKind="atr";this.selectedAtrDrive=drive;this.selectedAtrPath=entry?.path??null;
    try{
      if(action==="open"&&entry&&!entry.isDirectory)await this.openAtrTreeFile(entry.path,drive);
      else if(action==="exportAscii"&&entry&&!entry.isDirectory)await this.exportAtrFileAscii(entry,drive);
      else if(action==="extract"&&entry&&!entry.isDirectory)await this.extractAtrFile(entry,drive);
      else if(action==="rename"&&entry)await this.renameAtrEntry(entry,drive);
      else if(action==="delete"&&entry)await this.deleteAtrEntry(entry,drive);
      else if(action==="add")await this.atrAddHost();
      else if(action==="mkdir")await this.atrMkdir();
      else if(action==="refresh")await this.refreshAtr(false);
      else if(action==="unmount")await this.atrClose();
    }finally{this.hiddenInput.focus();}
  }
  private appendLocalTreeEntry(entry:LocalTreeEntry,depth:number):void{
    const collapsed=this.collapsedLocalPaths.has(entry.path);
    const selected=this.activeLocationKind==="local"&&entry.path===this.selectedLocalPath;
    const row=document.createElement("button");row.type="button";row.className=`tree-row${entry.isDirectory?" directory":" file"}${selected?" selected location-active":""}`;row.style.setProperty("--depth",String(depth));row.title=entry.path;
    const twist=document.createElement("span");twist.className="tree-twist";twist.textContent=entry.isDirectory?(collapsed?"+":"-"):"";
    const icon=document.createElement("span");icon.className="tree-icon";icon.textContent=entry.isDirectory?"DIR":"FILE";
    const name=document.createElement("span");name.className="tree-name";name.textContent=entry.name;
    row.append(twist,icon,name);
    row.addEventListener("click",()=>{this.activeLocationKind="local";this.selectedLocalPath=entry.path;if(entry.isDirectory){if(collapsed)this.collapsedLocalPaths.delete(entry.path);else this.collapsedLocalPaths.add(entry.path);}this.renderAtrTree();});
    row.addEventListener("dblclick",()=>{if(!entry.isDirectory)void this.openLocalTreeFile(entry.path);});
    if(!entry.isDirectory)row.addEventListener("pointerdown",event=>this.armExplorerDrag(event,row,{kind:"local",path:entry.path}));
    this.localTree.appendChild(row);
    if(entry.isDirectory&&!collapsed)entry.children.forEach(child=>this.appendLocalTreeEntry(child,depth+1));
  }
  private async treeCommand(action:string):Promise<void>{if(action==="open")await this.openSelectedAtrFile();else if(action==="add")await this.atrAddHost();else if(action==="extract")await this.extractSelectedAtrFile();else if(action==="mkdir")await this.atrMkdir();else if(action==="delete")await this.deleteSelectedAtrEntry();else if(action==="unmount")await this.atrClose();}
  private selectedDriveStatus():AtrDriveStatus|null{return this.atr.drives.find(d=>d.drive===`D${this.selectedAtrDrive}:`)??null;}
  private selectedAtrEntry():AtrTreeEntry|null{return this.selectedAtrPath?this.findAtrTreeEntry(this.selectedDriveStatus()?.tree??[],this.selectedAtrPath):null;}
  private selectedAtrFile():AtrTreeEntry|null{const entry=this.selectedAtrEntry();if(!entry||entry.isDirectory){window.alert("Select an ATR file in D1: through D4: first.");return null;}return entry;}
  private async selectDrive(drive:number):Promise<void>{try{this.setAtr(await invoke("atr_select_drive",{drive}));}catch(e){window.alert(String(e));}}
  private async openSelectedAtrFile():Promise<void>{const entry=this.selectedAtrFile();if(entry)await this.openAtrTreeFile(entry.path,this.selectedAtrDrive);}
  private async openAtrTreeFile(path:string,drive:number,confirmDiscard=true):Promise<void>{if(confirmDiscard&&this.dirty&&!window.confirm("Discard unsaved changes?"))return;try{this.activeLocationKind="atr";this.selectedAtrDrive=drive;this.selectedAtrPath=path;this.load(await invoke("atr_open_document",{imageName:path,drive,mode:this.mode,width:this.width,height:this.height}),undefined,{drive,path});this.renderAtrTree();this.hiddenInput.focus();}catch(e){window.alert(String(e));}}
  private async extractSelectedAtrFile():Promise<void>{const entry=this.selectedAtrFile();if(entry)await this.extractAtrFile(entry,this.selectedAtrDrive);}
  private async extractAtrFile(entry:AtrTreeEntry,drive:number):Promise<void>{const p=await save({defaultPath:this.basename(entry.path)});if(!p)return;try{this.setAtr(await invoke("atr_extract_file",{imageName:entry.path,hostPath:p,drive}));}catch(e){window.alert(String(e));}}
  private async exportAtrFileAscii(entry:AtrTreeEntry,drive:number):Promise<void>{const p=await save({defaultPath:this.basename(entry.path),filters:[{name:"ASCII Text",extensions:["txt","bas","lst","ata"]}]});if(!p)return;try{this.setAtr(await invoke("atr_export_ascii",{imageName:entry.path,hostPath:p,drive}));}catch(e){window.alert(String(e));}}
  private async renameAtrEntry(entry:AtrTreeEntry,drive:number):Promise<void>{
    const newName=window.prompt(`Rename D${drive}: ${entry.path}`,entry.name)?.trim();if(!newName||newName===entry.name)return;
    try{
      const parent=this.parentAtrPath(entry.path),renamedPath=parent?`${parent}>${newName.toUpperCase()}`:newName.toUpperCase();
      this.selectedAtrPath=renamedPath;
      this.setAtr(await invoke("atr_rename_entry",{imageName:entry.path,newName,drive}));
    }catch(e){this.selectedAtrPath=entry.path;window.alert(String(e));this.renderDriveTree();}
  }
  private async deleteSelectedAtrEntry():Promise<void>{const entry=this.selectedAtrEntry();if(!entry){window.alert("Select an ATR file or directory first.");return;}await this.deleteAtrEntry(entry,this.selectedAtrDrive);}
  private async deleteAtrEntry(entry:AtrTreeEntry,drive:number):Promise<void>{const kind=entry.isDirectory?"directory":"file";if(!window.confirm(`Delete ${kind} D${drive}: ${entry.path}?${entry.isDirectory?"\n\nDirectory must be empty.":""}`))return;try{this.selectedAtrPath=null;this.setAtr(await invoke("atr_delete_entry",{imageName:entry.path,isDirectory:entry.isDirectory,drive}));}catch(e){this.selectedAtrPath=entry.path;window.alert(String(e));this.renderDriveTree();}}
  private async openLocalTreeFile(path:string):Promise<void>{if(this.dirty&&!window.confirm("Discard unsaved changes?"))return;try{this.activeLocationKind="local";this.selectedLocalPath=path;this.load(await invoke("load_document",{path,mode:this.mode,width:this.width,height:this.height}));await this.setLocalFolderForPath(path);}catch(e){window.alert(String(e));}}
  private async setLocalFolderForPath(path:string):Promise<void>{const slash=Math.max(path.lastIndexOf("\\"),path.lastIndexOf("/"));const folder=slash>0?path.slice(0,slash):"";if(folder)try{this.setAtr(await invoke("local_open_folder",{path:folder}));}catch{}}
  private driveNumber(label:string|null):number|null{const match=label?.match(/^D([1-4]):$/);return match?Number(match[1]):null;}
  private basename(path:string):string{return path.split(/[\\/>]/).filter(Boolean).pop()??path;}
  private parentLocalPath(path:string):string{const index=Math.max(path.lastIndexOf("\\"),path.lastIndexOf("/"));return index>0?path.slice(0,index):"";}
  private joinLocalPath(directory:string,name:string):string{if(!directory)return name;const separator=directory.includes("\\")?"\\":"/";return `${directory.replace(/[\\/]+$/,"")}${separator}${name}`;}
  private parentAtrPath(path:string):string{const index=Math.max(path.lastIndexOf(">"),path.lastIndexOf("/"),path.lastIndexOf("\\"));return index>0?path.slice(0,index):"";}
  private selectedAtrDirectory():string{const entry=this.selectedAtrEntry();if(!entry)return"";return entry.isDirectory?entry.path:this.parentAtrPath(entry.path);}
  private defaultAtrPath(name:string):string{const directory=this.selectedAtrDirectory();return directory?`${directory}>${name}`:name;}
  private selectedLocalEntry():LocalTreeEntry|null{return this.selectedLocalPath?this.findLocalTreeEntry(this.atr.localTree,this.selectedLocalPath):null;}
  private selectedLocalDirectory():string{const entry=this.selectedLocalEntry();if(!entry)return this.atr.localFolder??"";return entry.isDirectory?entry.path:this.parentLocalPath(entry.path);}
  private activeStorageLocation():ActiveStorageLocation{
    return this.activeLocationKind==="atr"
      ?{kind:"atr",drive:this.selectedAtrDrive,directory:this.selectedAtrDirectory()}
      :{kind:"local",directory:this.selectedLocalDirectory()};
  }
  private currentDocumentName():string{return this.atrDocument?this.basename(this.atrDocument.path):this.path?this.basename(this.path):(this.mode==="atascii"?"UNTITLED.ATA":"UNTITLED.TXT");}
  private updateActiveLocationLabel():void{
    const label=document.querySelector<HTMLElement>("#activeLocationLabel");if(!label)return;
    const location=this.activeStorageLocation();
    if(location.kind==="atr"){
      const mounted=this.atr.drives.find(status=>status.drive===`D${location.drive}:`)?.mounted??false;
      label.textContent=`D${location.drive}:${location.directory||"\\"}${mounted?"":" (NOT MOUNTED)"}`;
    }else label.textContent=`LOCAL: ${location.directory||"(CHOOSE FOLDER)"}`;
    label.title=label.textContent;
  }
  private findAtrTreeEntry(entries:AtrTreeEntry[],path:string):AtrTreeEntry|null{for(const entry of entries){if(entry.path===path)return entry;const child=this.findAtrTreeEntry(entry.children,path);if(child)return child;}return null;}
  private findLocalTreeEntry(entries:LocalTreeEntry[],path:string):LocalTreeEntry|null{for(const entry of entries){if(entry.path===path)return entry;const child=this.findLocalTreeEntry(entry.children,path);if(child)return child;}return null;}
  private atrDropTargetAt(clientX:number,clientY:number):ExplorerDropTarget|null{
    const element=document.elementFromPoint(clientX,clientY)?.closest<HTMLElement>("[data-atr-drop='true']");
    if(!element)return null;
    const drive=Number(element.dataset.atrDrive);
    const driveStatus=this.atr.drives.find(status=>status.drive===`D${drive}:`);
    if(!Number.isInteger(drive)||!driveStatus?.mounted)return null;
    return{drive,directory:element.dataset.atrDirectory??"",element};
  }
  private physicalAtrDropTarget(position:{x:number;y:number}):ExplorerDropTarget|null{
    const scale=window.devicePixelRatio||1;
    return this.atrDropTargetAt(position.x/scale,position.y/scale);
  }
  private showDropHighlight(target:ExplorerDropTarget|null):void{
    if(this.dropHighlight===target?.element)return;
    this.dropHighlight?.classList.remove("drop-target");
    this.dropHighlight=target?.element??null;
    this.dropHighlight?.classList.add("drop-target");
  }
  private clearDropHighlight():void{this.showDropHighlight(null);}
  private async installHostDropHandler():Promise<void>{
    try{
      await getCurrentWebview().onDragDropEvent(event=>{
        const payload=event.payload;
        if(payload.type==="leave"){this.clearDropHighlight();return;}
        const target=this.physicalAtrDropTarget(payload.position);
        if(payload.type==="enter"||payload.type==="over"){this.showDropHighlight(target);return;}
        this.clearDropHighlight();
        if(target&&!this.isBusy)void this.importHostDrop(payload.paths,target);
      });
    }catch{}
  }
  private async importHostDrop(paths:string[],target:ExplorerDropTarget):Promise<void>{
    try{
      this.activeLocationKind="atr";this.selectedAtrDrive=target.drive;this.selectedAtrPath=target.directory||null;
      this.setAtr(await invoke("atr_import_host_files",{hostPaths:paths,destinationDirectory:target.directory,drive:target.drive}));
    }catch(error){window.alert(String(error));}
  }
  private armExplorerDrag(event:PointerEvent,row:HTMLElement,source:ExplorerDragSource):void{
    if(event.button!==0)return;
    this.dragGestureCleanup?.();
    const pointerId=event.pointerId,startX=event.clientX,startY=event.clientY;
    let started=false;
    const cleanup=()=>{
      window.removeEventListener("pointermove",move);
      window.removeEventListener("pointerup",finish);
      window.removeEventListener("pointercancel",cancel);
      row.classList.remove("drag-source");document.body.classList.remove("explorer-dragging");
      this.clearDropHighlight();
      try{if(row.hasPointerCapture(pointerId))row.releasePointerCapture(pointerId);}catch{}
      if(this.dragGestureCleanup===cleanup)this.dragGestureCleanup=null;
    };
    const move=(moveEvent:PointerEvent)=>{
      if(moveEvent.pointerId!==pointerId)return;
      if(!started&&Math.hypot(moveEvent.clientX-startX,moveEvent.clientY-startY)<6)return;
      if(!started){started=true;row.classList.add("drag-source");document.body.classList.add("explorer-dragging");}
      moveEvent.preventDefault();this.showDropHighlight(this.atrDropTargetAt(moveEvent.clientX,moveEvent.clientY));
    };
    const finish=(upEvent:PointerEvent)=>{
      if(upEvent.pointerId!==pointerId)return;
      const target=started?this.atrDropTargetAt(upEvent.clientX,upEvent.clientY):null;
      if(started){
        const suppress=(click:MouseEvent)=>{click.preventDefault();click.stopImmediatePropagation();};
        row.addEventListener("click",suppress,{capture:true,once:true});
        window.setTimeout(()=>row.removeEventListener("click",suppress,true),0);
      }
      cleanup();
      if(started&&target)void this.completeExplorerDrag(source,target);
    };
    const cancel=(cancelEvent:PointerEvent)=>{if(cancelEvent.pointerId===pointerId)cleanup();};
    this.dragGestureCleanup=cleanup;
    window.addEventListener("pointermove",move,{passive:false});
    window.addEventListener("pointerup",finish);
    window.addEventListener("pointercancel",cancel);
    try{row.setPointerCapture(pointerId);}catch{}
  }
  private async completeExplorerDrag(source:ExplorerDragSource,target:ExplorerDropTarget):Promise<void>{
    try{
      this.activeLocationKind="atr";this.selectedAtrDrive=target.drive;this.selectedAtrPath=target.directory||null;
      if(source.kind==="atr"){
        if(source.drive===target.drive&&this.parentAtrPath(source.entry.path)===target.directory)return;
        this.setAtr(await invoke("atr_copy_file",{sourceImageName:source.entry.path,sourceDrive:source.drive,destinationDirectory:target.directory,destinationDrive:target.drive}));
      }else{
        this.setAtr(await invoke("atr_import_host_files",{hostPaths:[source.path],destinationDirectory:target.directory,drive:target.drive}));
      }
    }catch(error){window.alert(String(error));}
  }
  private scrollCursorIntoView(): void { this.cellElements[this.cursor]?.scrollIntoView({block:"nearest",inline:"nearest"}); }
  private screenIndexAtPoint(clientX:number,clientY:number):number{
    const bounds=this.screen.getBoundingClientRect();
    const col=Math.max(0,Math.min(this.width-1,Math.floor((clientX-bounds.left)/(bounds.width/this.width))));
    const row=Math.max(0,Math.min(this.height-1,Math.floor((clientY-bounds.top)/(bounds.height/this.height))));
    return row*this.width+col;
  }
  private onScreenPointerDown(event:PointerEvent):void{
    if(event.button!==0||this.isBusy)return;
    const cell=(event.target as HTMLElement).closest<HTMLElement>(".cell");if(!cell)return;
    event.preventDefault();this.hideEditorContextMenu();
    const start=this.cellElements.indexOf(cell),previousCursor=this.cursor;
    if(start<0)return;
    if(event.shiftKey){
      if(!this.selectionActive)this.selectionAnchor=this.cursor;
      this.selectionFocus=start;this.selectionActive=true;
    }else{
      this.selectionActive=false;this.selectionAnchor=start;this.selectionFocus=start;
    }
    this.cursor=start;this.renderSelection();this.updateCell(previousCursor);this.updateCell(this.cursor);this.renderStatus();
    const pointerId=event.pointerId;
    const move=(moveEvent:PointerEvent)=>{
      if(moveEvent.pointerId!==pointerId||!(moveEvent.buttons&1))return;
      moveEvent.preventDefault();
      const index=this.screenIndexAtPoint(moveEvent.clientX,moveEvent.clientY);
      if(index===this.selectionFocus&&this.selectionActive)return;
      const oldCursor=this.cursor;this.selectionActive=true;this.selectionFocus=index;this.cursor=index;
      this.renderSelection();this.updateCell(oldCursor);this.updateCell(this.cursor);
    };
    const finish=(finishEvent:PointerEvent)=>{
      if(finishEvent.pointerId!==pointerId)return;
      window.removeEventListener("pointermove",move);window.removeEventListener("pointerup",finish);window.removeEventListener("pointercancel",finish);
      try{if(this.screen.hasPointerCapture(pointerId))this.screen.releasePointerCapture(pointerId);}catch{}
      this.hiddenInput.focus();
    };
    window.addEventListener("pointermove",move,{passive:false});window.addEventListener("pointerup",finish);window.addEventListener("pointercancel",finish);
    try{this.screen.setPointerCapture(pointerId);}catch{}
    this.hiddenInput.focus();
  }
  private showEditorContextMenu(event:MouseEvent):void{
    if(this.isBusy)return;
    const cell=(event.target as HTMLElement).closest<HTMLElement>(".cell");if(!cell)return;
    event.preventDefault();event.stopPropagation();this.hideAtrContextMenu();
    const index=this.cellElements.indexOf(cell);if(index<0)return;
    const previousCursor=this.cursor;this.editorContextIndex=index;this.cursor=index;
    if(!this.selectionContains(index)){this.selectionAnchor=index;this.selectionFocus=index;this.selectionActive=true;}
    this.renderSelection();this.updateCell(previousCursor);this.updateCell(index);
    const actions=[
      {action:"cut",label:"Cut",shortcut:"Ctrl+Shift+X",disabled:!this.selectionActive},
      {action:"copy",label:"Copy",shortcut:"Ctrl+Shift+C",disabled:!this.selectionActive},
      {action:"paste",label:"Paste",shortcut:"Ctrl+Shift+V",disabled:false},
      null,
      {action:"select",label:"Select Glyph",shortcut:"",disabled:false},
      {action:"selectAll",label:"Select All",shortcut:"Ctrl+Shift+A",disabled:false},
      null,
      {action:"inverse",label:"Inverse Selected Glyphs",shortcut:"",disabled:!this.selectionActive},
      null,
      {action:"deleteAfter",label:"Delete After Cursor",shortcut:"Ctrl+Del",disabled:this.editorContextIndex%this.width===this.width-1},
      {action:"deleteLine",label:"Delete Line",shortcut:"Ctrl+Shift+Del",disabled:false}
    ];
    this.editorContextMenu.replaceChildren();
    for(const item of actions){
      if(!item){const separator=document.createElement("div");separator.className="context-menu-separator";separator.role="separator";this.editorContextMenu.appendChild(separator);continue;}
      const button=document.createElement("button");button.type="button";button.role="menuitem";button.disabled=item.disabled;
      const label=document.createElement("span");label.textContent=item.label;button.appendChild(label);
      if(item.shortcut){const shortcut=document.createElement("kbd");shortcut.textContent=item.shortcut;button.appendChild(shortcut);}
      button.addEventListener("click",click=>{click.preventDefault();click.stopPropagation();this.hideEditorContextMenu();void this.runEditorContextAction(item.action);});
      this.editorContextMenu.appendChild(button);
    }
    this.editorContextMenu.hidden=false;
    const x=Math.max(6,Math.min(event.clientX,window.innerWidth-this.editorContextMenu.offsetWidth-6));
    const y=Math.max(6,Math.min(event.clientY,window.innerHeight-this.editorContextMenu.offsetHeight-6));
    this.editorContextMenu.style.left=`${x}px`;this.editorContextMenu.style.top=`${y}px`;
    this.editorContextMenu.querySelector<HTMLButtonElement>("button:not(:disabled)")?.focus();
  }
  private hideEditorContextMenu():void{if(this.editorContextMenu)this.editorContextMenu.hidden=true;}
  private async runEditorContextAction(action:string):Promise<void>{
    if(action==="cut")await this.copySelectionToSystem(true);
    else if(action==="copy")await this.copySelectionToSystem(false);
    else if(action==="paste")await this.pasteFromSystemClipboard();
    else if(action==="select"){const old=this.cursor;this.cursor=this.editorContextIndex;this.selectionAnchor=this.cursor;this.selectionFocus=this.cursor;this.selectionActive=true;this.renderSelection();this.updateCell(old);this.updateCell(this.cursor);}
    else if(action==="selectAll")this.selectAllCells();
    else if(action==="inverse")this.inverseSelectedCells();
    else if(action==="deleteAfter")this.deleteAfterCursor();
    else if(action==="deleteLine")this.deleteCurrentLine();
    this.hiddenInput.focus();
  }
  private plainClipboardCharacter(cell:Cell):string{
    const byte=cell.byte&0x7f;return byte>=0x20&&byte<=0x7e?String.fromCharCode(byte):" ";
  }
  private captureSelection():EditorClipboard|null{
    const bounds=this.selectionBounds();if(!bounds)return null;
    const cells:Cell[]=[],lines:string[]=[];
    for(let row=bounds.top;row<=bounds.bottom;row++){
      let line="";
      for(let col=bounds.left;col<=bounds.right;col++){
        const cell=this.cells[row*this.width+col]??blankCell();cells.push({...cell});line+=this.plainClipboardCharacter(cell);
      }
      lines.push(line);
    }
    return{width:bounds.width,height:bounds.height,cells,text:lines.join("\r\n")};
  }
  private onClipboardCopy(event:ClipboardEvent):void{
    const block=this.captureSelection();if(!block)return;
    event.preventDefault();this.internalClipboard=block;event.clipboardData?.setData("text/plain",block.text);
  }
  private onClipboardCut(event:ClipboardEvent):void{
    const block=this.captureSelection();if(!block)return;
    event.preventDefault();this.internalClipboard=block;event.clipboardData?.setData("text/plain",block.text);this.deleteSelectedCells();
  }
  private onClipboardPaste(event:ClipboardEvent):void{
    const text=event.clipboardData?.getData("text/plain")??"";if(!text&&!this.internalClipboard)return;
    event.preventDefault();
    const block=this.internalClipboard&&this.internalClipboard.text===text?this.internalClipboard:this.clipboardFromText(text);
    if(block)this.pasteClipboardBlock(block);
  }
  private async copySelectionToSystem(cut:boolean):Promise<void>{
    const block=this.captureSelection();if(!block)return;this.internalClipboard=block;
    try{await navigator.clipboard.writeText(block.text);}catch{}
    if(cut)this.deleteSelectedCells();
  }
  private async pasteFromSystemClipboard():Promise<void>{
    let text:string|null=null;try{text=await navigator.clipboard.readText();}catch{}
    const block=this.internalClipboard&&(text===null||this.internalClipboard.text===text)
      ?this.internalClipboard
      :text!==null?this.clipboardFromText(text):null;
    if(block)this.pasteClipboardBlock(block);
  }
  private clipboardFromText(text:string):EditorClipboard|null{
    const lines=text.replace(/\r\n/g,"\n").replace(/\r/g,"\n").replace(/\t/g,"    ").split("\n");
    const width=Math.max(0,...lines.map(line=>line.length));if(!width)return null;
    const cells:Cell[]=[];
    for(const line of lines)for(let col=0;col<width;col++){
      const code=line.charCodeAt(col);const byte=Number.isFinite(code)&&code<=0x7f?code:0x20;
      cells.push({byte,inverse:false,display:atariGlyphLabel(byte)});
    }
    return{width,height:lines.length,cells,text:lines.join("\r\n")};
  }
  private deleteSelectedCells():boolean{
    const bounds=this.selectionBounds();if(!bounds)return false;
    const indexes=this.selectedCellIndexes(bounds),destination=bounds.top*this.width+bounds.left;
    this.selectionActive=false;this.cursor=destination;this.selectionAnchor=destination;this.selectionFocus=destination;
    for(const index of indexes){this.cells[index]=blankCell();this.updateCell(index);}
    this.renderSelection();this.updateCell(destination);this.dirty=true;this.renderStatus();return true;
  }
  private pasteClipboardBlock(block:EditorClipboard):void{
    const selected=this.selectionBounds(),start=selected?selected.top*this.width+selected.left:this.cursor;
    if(selected)this.deleteSelectedCells();
    const startRow=Math.floor(start/this.width),startCol=start%this.width;
    const rows=Math.min(block.height,this.height-startRow),columns=Math.min(block.width,this.width-startCol);
    if(rows<=0||columns<=0)return;
    for(let row=0;row<rows;row++)for(let col=0;col<columns;col++){
      const source=block.cells[row*block.width+col]??blankCell(),index=(startRow+row)*this.width+startCol+col;
      this.cells[index]={byte:source.byte&0x7f,inverse:source.inverse,display:atariGlyphLabel(source.byte)};this.updateCell(index);
    }
    this.cursor=start;this.selectionAnchor=start;this.selectionFocus=(startRow+rows-1)*this.width+startCol+columns-1;this.selectionActive=true;
    this.dirty=true;this.renderSelection();this.updateCell(this.cursor);this.renderStatus();this.scrollCursorIntoView();
  }
  private inverseSelectedCells():void{
    const indexes=this.selectedCellIndexes();if(!indexes.length)return;
    for(const index of indexes){this.cells[index]={...this.cells[index],inverse:!this.cells[index].inverse};this.updateCell(index);}
    this.dirty=true;this.renderStatus();
  }
  private searchBytes(value:string):number[]|null{
    const bytes:number[]=[];
    for(const character of value){
      const code=character.codePointAt(0)??0;
      if(code>0x7f||character==="\r"||character==="\n")return null;
      bytes.push(code);
    }
    return bytes;
  }
  private foldedSearchByte(byte:number,matchCase:boolean):number{
    const base=byte&0x7f;
    return !matchCase&&base>=0x61&&base<=0x7a?base-0x20:base;
  }
  private cellsMatchAt(cells:Cell[],index:number,query:number[],matchCase:boolean):boolean{
    if(!query.length||index<0||index+query.length>cells.length||index%this.width+query.length>this.width)return false;
    return query.every((byte,offset)=>this.foldedSearchByte(cells[index+offset]?.byte??0x20,matchCase)===this.foldedSearchByte(byte,matchCase));
  }
  private selectSearchMatch(index:number,length:number):void{
    const previousCursor=this.cursor;
    this.cursor=index;this.selectionAnchor=index;this.selectionFocus=index+length-1;this.selectionActive=true;
    this.renderSelection();this.updateCell(previousCursor);this.updateCell(this.cursor);this.scrollCursorIntoView();
  }
  private findNextMatch(query:number[],matchCase:boolean,wrap:boolean):number|null{
    if(!query.length||query.length>this.width)return null;
    const bounds=this.selectionBounds();
    const start=bounds?bounds.bottom*this.width+bounds.right+1:this.cursor;
    const findBetween=(from:number,to:number):number|null=>{
      for(let index=from;index<to;index++)if(this.cellsMatchAt(this.cells,index,query,matchCase))return index;
      return null;
    };
    const match=findBetween(Math.min(start,this.cellCount),this.cellCount)??(wrap?findBetween(0,Math.min(start,this.cellCount)):null);
    if(match!==null)this.selectSearchMatch(match,query.length);
    return match;
  }
  private selectedCellsMatch(query:number[],matchCase:boolean):boolean{
    const bounds=this.selectionBounds();
    return Boolean(bounds&&bounds.height===1&&bounds.width===query.length&&this.cellsMatchAt(this.cells,bounds.top*this.width+bounds.left,query,matchCase));
  }
  private replacementCells(bytes:number[]):Cell[]{
    return bytes.map(byte=>({byte:byte&0x7f,inverse:this.inverse,display:atariGlyphLabel(byte)}));
  }
  private replaceSelectedMatch(query:number[],replacement:number[],matchCase:boolean):boolean{
    const bounds=this.selectionBounds();if(!bounds||!this.selectedCellsMatch(query,matchCase))return false;
    const rowStart=bounds.top*this.width,row=this.cells.slice(rowStart,rowStart+this.width);
    const replaced=[...row.slice(0,bounds.left),...this.replacementCells(replacement),...row.slice(bounds.left+query.length)];
    while(replaced.length<this.width)replaced.push(blankCell());
    this.cells.splice(rowStart,this.width,...replaced.slice(0,this.width));
    this.cursor=rowStart+Math.min(bounds.left,this.width-1);
    if(replacement.length){
      this.selectionAnchor=this.cursor;this.selectionFocus=rowStart+Math.min(this.width-1,bounds.left+replacement.length-1);this.selectionActive=true;
    }else{
      this.selectionAnchor=this.cursor;this.selectionFocus=this.cursor;this.selectionActive=false;
    }
    this.dirty=true;this.renderedSelection=new Set(this.selectedCellIndexes());
    this.updateCellRange(rowStart,rowStart+this.width);this.renderStatus();this.scrollCursorIntoView();return true;
  }
  private replaceAllMatches(query:number[],replacement:number[],matchCase:boolean):number{
    if(!query.length||query.length>this.width)return 0;
    let count=0;
    for(let rowIndex=0;rowIndex<this.height;rowIndex++){
      const rowStart=rowIndex*this.width,source=this.cells.slice(rowStart,rowStart+this.width),result:Cell[]=[];
      for(let column=0;column<this.width;){
        if(column+query.length<=this.width&&this.cellsMatchAt(source,column,query,matchCase)){
          result.push(...this.replacementCells(replacement));column+=query.length;count++;
        }else{
          result.push(source[column]??blankCell());column++;
        }
      }
      while(result.length<this.width)result.push(blankCell());
      this.cells.splice(rowStart,this.width,...result.slice(0,this.width));
    }
    if(count){
      this.clearEditorSelection();this.dirty=true;this.updateCellRange(0,this.cellCount);this.renderStatus();this.scrollCursorIntoView();
    }
    return count;
  }
  private showFindPanel(withReplace:boolean):void{
    const configure=(panel:HTMLElement):void=>{
      panel.classList.toggle("replace-mode",withReplace);
      panel.querySelector<HTMLElement>("[data-replace-row]")!.hidden=!withReplace;
      panel.querySelectorAll<HTMLElement>("[data-replace-only]").forEach(element=>element.hidden=!withReplace);
      panel.querySelector("[data-find-title]")!.textContent=withReplace?"Find and Replace":"Find";
      const input=panel.querySelector<HTMLInputElement>("[data-find-input]")!;input.focus();input.select();
    };
    if(this.findPanel){configure(this.findPanel);return;}
    const panel=document.createElement("section");panel.className="find-panel";panel.role="dialog";panel.setAttribute("aria-modal","false");
    panel.innerHTML=`
      <header><strong data-find-title>Find</strong><button type="button" data-find-close aria-label="Close search">×</button></header>
      <label>Find<input type="text" data-find-input autocomplete="off" spellcheck="false"></label>
      <label data-replace-row hidden>Replace with<input type="text" data-replace-input autocomplete="off" spellcheck="false"></label>
      <div class="find-options">
        <label><input type="checkbox" data-find-case> Match case</label>
        <label><input type="checkbox" data-find-wrap checked> Wrap</label>
      </div>
      <div class="find-actions">
        <button type="button" data-find-next>Find Next</button>
        <button type="button" data-replace-only data-replace-one hidden>Replace</button>
        <button type="button" data-replace-only data-replace-all hidden>Replace All</button>
      </div>
      <div class="find-status" data-find-status role="status">Search ignores inverse video and stays within each screen row.</div>`;
    document.body.appendChild(panel);this.findPanel=panel;
    const findInput=panel.querySelector<HTMLInputElement>("[data-find-input]")!;
    const replaceInput=panel.querySelector<HTMLInputElement>("[data-replace-input]")!;
    const matchCase=panel.querySelector<HTMLInputElement>("[data-find-case]")!;
    const wrap=panel.querySelector<HTMLInputElement>("[data-find-wrap]")!;
    const status=panel.querySelector<HTMLElement>("[data-find-status]")!;
    const selected=this.captureSelection();
    if(selected&&!selected.text.includes("\r\n")&&selected.text.trim())findInput.value=selected.text;
    const values=():{query:number[];replacement:number[]}|null=>{
      const query=this.searchBytes(findInput.value),replacement=this.searchBytes(replaceInput.value);
      if(query===null||replacement===null){status.textContent="Search and replacement text must use ATASCII/ASCII characters.";return null;}
      if(!query.length){status.textContent="Enter text to find.";return null;}
      if(query.length>this.width){status.textContent=`Search text cannot exceed ${this.width} columns.`;return null;}
      return{query,replacement};
    };
    const findNext=():boolean=>{
      const current=values();if(!current)return false;
      const match=this.findNextMatch(current.query,matchCase.checked,wrap.checked);
      status.textContent=match===null?"No match found.":`Match at row ${Math.floor(match/this.width)+1}, column ${match%this.width+1}.`;
      return match!==null;
    };
    panel.querySelector("[data-find-next]")!.addEventListener("click",findNext);
    panel.querySelector("[data-replace-one]")!.addEventListener("click",()=>{
      const current=values();if(!current)return;
      if(!this.selectedCellsMatch(current.query,matchCase.checked)){findNext();return;}
      this.replaceSelectedMatch(current.query,current.replacement,matchCase.checked);
      status.textContent="Replaced current match.";
      findNext();
    });
    panel.querySelector("[data-replace-all]")!.addEventListener("click",()=>{
      const current=values();if(!current)return;
      const count=this.replaceAllMatches(current.query,current.replacement,matchCase.checked);
      status.textContent=count?`Replaced ${count.toLocaleString()} match${count===1?"":"es"}.`:"No matches found.";
    });
    const close=():void=>{panel.remove();this.findPanel=null;this.hiddenInput.focus();};
    panel.querySelector("[data-find-close]")!.addEventListener("click",close);
    panel.addEventListener("keydown",event=>{
      if(event.key==="Escape"){event.preventDefault();close();}
      else if(event.key==="Enter"){event.preventDefault();findNext();}
    });
    configure(panel);
  }
  private onTextInput(): void {
    const value=this.hiddenInput.value;if(!value)return;
    if(this.selectionActive)this.deleteSelectedCells();
    const previousCursor=this.cursor;let start=this.cellCount,end=-1;
    for(const ch of value){const change=this.putChar(ch);start=Math.min(start,change.start);end=Math.max(end,change.end);}
    this.hiddenInput.value="";this.refreshEditorView(previousCursor,{start,end});
  }

  private onKeyDown(e: KeyboardEvent): void {
    if(this.isBusy){e.preventDefault();return;}
    if((e.ctrlKey||e.metaKey)&&e.key==="Delete"){
      e.preventDefault();e.stopPropagation();
      if(e.shiftKey)this.deleteCurrentLine();else this.deleteAfterCursor();
      return;
    }
    const controlByte=controlByteForKey(e);
    if(controlByte!==null){if(this.selectionActive)this.deleteSelectedCells();const previousCursor=this.cursor;e.preventDefault();e.stopPropagation();this.refreshEditorView(previousCursor,this.putByte(controlByte));return;}
    if(e.ctrlKey||e.metaKey)return;
    if((e.key==="Backspace"||e.key==="Delete")&&this.selectionActive){e.preventDefault();this.deleteSelectedCells();return;}
    if(e.key==="Escape"&&this.selectionActive){e.preventDefault();this.clearEditorSelection();return;}
    const previousCursor=this.cursor;let change:CellChange|undefined;
    let navigation=false;
    const row=Math.floor(this.cursor/this.width), col=this.cursor%this.width;
    switch(e.key){
      case"ArrowLeft":this.cursor=Math.max(0,this.cursor-1);navigation=true;break; case"ArrowRight":this.cursor=Math.min(this.cellCount-1,this.cursor+1);navigation=true;break;
      case"ArrowUp":this.cursor=Math.max(0,this.cursor-this.width);navigation=true;break; case"ArrowDown":this.cursor=Math.min(this.cellCount-1,this.cursor+this.width);navigation=true;break;
      case"Home":this.cursor=row*this.width;navigation=true;break; case"End":this.cursor=row*this.width+this.width-1;navigation=true;break;
      case"PageUp":this.cursor=Math.max(col,this.cursor-this.width*VIEW_ROWS);navigation=true;break; case"PageDown":this.cursor=Math.min(this.cellCount-1,this.cursor+this.width*VIEW_ROWS);navigation=true;break;
      case"Enter":this.cursor=Math.min(this.cellCount-1,(row+1)*this.width);navigation=true;break;
      case"Tab":this.cursor=Math.min(this.cellCount-1,row*this.width+Math.min(this.width-1,(Math.floor(col/8)+1)*8));navigation=true;break;
      case"Backspace":change=this.deleteBack();break; case"Delete":change=this.deleteAt();break; case"Insert":this.insertMode=!this.insertMode;break; case"Escape":this.inverse=false;break;
      default:return;
    }
    if(navigation){
      if(e.shiftKey){if(!this.selectionActive)this.selectionAnchor=previousCursor;this.selectionFocus=this.cursor;this.selectionActive=true;}
      else this.clearEditorSelection(false);
      this.renderSelection();
    }
    e.preventDefault();this.refreshEditorView(previousCursor,change);
  }

  private putChar(ch:string):CellChange{const code=ch.codePointAt(0)??0x20;return this.putByte(code<=0x7f?code:0x3f);}
  private putByte(byte:number):CellChange{const row=Math.floor(this.cursor/this.width),end=(row+1)*this.width,start=this.cursor;if(this.insertMode){for(let i=end-1;i>this.cursor;i--)this.cells[i]=this.cells[i-1];}this.cells[this.cursor]={byte:byte&0x7f,inverse:this.inverse,display:atariGlyphLabel(byte)};this.cursor=Math.min(this.cellCount-1,this.cursor+1);this.dirty=true;return{start,end:this.insertMode?end-1:start};}
  private deleteBack():CellChange|undefined{if(this.cursor>0){this.cursor--;return this.deleteAt();}return undefined;}
  private deleteAt():CellChange{const row=Math.floor(this.cursor/this.width),end=(row+1)*this.width;for(let i=this.cursor;i<end-1;i++)this.cells[i]=this.cells[i+1];this.cells[end-1]=blankCell();this.dirty=true;return{start:this.cursor,end:end-1};}
  private deleteAfterCursor():void{
    const rowEnd=(Math.floor(this.cursor/this.width)+1)*this.width;
    if(this.cursor+1>=rowEnd)return;
    this.clearEditorSelection();
    for(let index=this.cursor+1;index<rowEnd;index++)this.cells[index]=blankCell();
    this.dirty=true;this.updateCellRange(this.cursor+1,rowEnd);this.updateCell(this.cursor);this.renderStatus();
  }
  private deleteCurrentLine():void{
    const row=Math.floor(this.cursor/this.width),column=this.cursor%this.width,start=row*this.width;
    this.clearEditorSelection();
    this.cellElements[this.cursor]?.classList.remove("cursor");
    this.cells.splice(start,this.width);
    this.cells.push(...Array.from({length:this.width},blankCell));
    const recycled=this.cellElements.splice(start,this.width);
    this.cellElements.push(...recycled);
    const recycledRow=this.rowElements.splice(row,1)[0];
    if(recycledRow){this.rowElements.push(recycledRow);this.screen.appendChild(recycledRow);}
    this.cursor=Math.min(this.cellCount-1,row*this.width+column);
    this.selectionAnchor=this.cursor;this.selectionFocus=this.cursor;
    this.dirty=true;this.updateCellRange(this.cellCount-this.width,this.cellCount);this.updateCell(this.cursor);this.renderStatus();
  }

  private async command(cmd:string):Promise<void>{
    if(this.isBusy)return;
    const actions:Record<string,()=>void|Promise<void>>={
      new:()=>this.newFile(),open:()=>this.openFile(),openLocalFile:()=>this.openLocalFile(),openFolder:()=>this.openFolder(),save:()=>this.save(false),saveAs:()=>this.save(true),exportAscii:()=>this.exportAscii(),
      find:()=>this.showFindPanel(false),replace:()=>this.showFindPanel(true),
      columns40:()=>this.setColumns(40),columns80:()=>this.setColumns(80),inverse:()=>{this.inverse=!this.inverse;this.renderStatus();},insert:()=>{this.insertMode=!this.insertMode;this.renderStatus();},clear:()=>this.clear(),
      help:()=>this.showHelp(),helpShortcuts:()=>this.showHelp("shortcuts"),helpAtascii:()=>this.showHelp("atascii"),helpSupport:()=>this.showHelp("support"),
      helpUpdates:()=>this.checkForUpdates(),helpLicense:()=>this.showTextModal("License",LICENSE_TEXT.trimEnd()),helpAbout:()=>this.showAbout(),
      atrCreate:()=>this.atrCreate(),atrMount:()=>this.atrMount(),atrRefresh:()=>this.refreshAtr(true),atrOpenDocument:()=>this.atrOpenDocument(),atrWriteDocument:()=>this.atrWriteDocument(),atrAddHost:()=>this.atrAddHost(),atrExtract:()=>this.atrExtract(),atrDelete:()=>this.atrDelete(),atrMkdir:()=>this.atrMkdir(),atrInfo:()=>this.showAtrInfo(),atrClose:()=>this.atrClose(),
      basicDetokHost:()=>this.basicDetokHost(),basicTokHost:()=>this.basicTokHost(),basicListHost:()=>this.basicListHost(),basicDetokAtr:()=>this.basicDetokAtr(),basicTokAtr:()=>this.basicTokAtr(),basicListAtr:()=>this.basicListAtr()
    }; await actions[cmd]?.();if(!this.findPanel?.contains(document.activeElement))this.hiddenInput.focus();
  }

  private setColumns(next:number):void{if(next===this.width)return;const oldW=this.width,old=this.cells,newCells=Array.from({length:next*this.height},blankCell);for(let r=0;r<this.height;r++)for(let c=0;c<Math.min(oldW,next);c++)newCells[r*next+c]=old[r*oldW+c]??blankCell();const oldRow=Math.floor(this.cursor/oldW),oldCol=this.cursor%oldW;this.selectionActive=false;this.width=next;this.cells=newCells;this.cursor=oldRow*next+Math.min(oldCol,next-1);this.selectionAnchor=this.cursor;this.selectionFocus=this.cursor;this.dirty=true;this.render();}
  private clear():void{if(window.confirm("Clear the document?")){this.cells=Array.from({length:this.cellCount},blankCell);this.cursor=0;this.selectionActive=false;this.selectionAnchor=0;this.selectionFocus=0;this.dirty=true;this.render();}}
  private async newFile():Promise<void>{
    if(this.dirty&&!window.confirm("Discard unsaved changes?"))return;
    const location=this.activeStorageLocation(),defaultName=this.mode==="atascii"?"UNTITLED.ATA":"UNTITLED.TXT";
    if(location.kind==="atr"){
      if(!this.atr.drives.find(status=>status.drive===`D${location.drive}:`)?.mounted){window.alert(`Mount an ATR image in D${location.drive}: first.`);return;}
      const imagePath=await this.showAtrDocumentDialog("new",location,defaultName);if(!imagePath)return;
      this.path=null;this.atrDocument={drive:location.drive,path:imagePath};
    }else{
      const p=await save({title:"New Document",defaultPath:this.joinLocalPath(location.directory,defaultName)});if(!p)return;
      this.path=p;this.atrDocument=null;this.selectedLocalPath=p;await this.setLocalFolderForPath(p);
    }
    this.cells=Array.from({length:this.cellCount},blankCell);this.cursor=0;this.selectionActive=false;this.selectionAnchor=0;this.selectionFocus=0;this.dirty=true;this.render();
  }

  private request(path=""):SaveDocumentRequest{return{path,mode:this.mode,width:this.width,height:this.height,cells:this.cells,trimTrailingSpaces:true};}
  private basicRequest(path=""):SaveDocumentRequest{return{path,mode:"ascii",width:this.width,height:this.height,cells:this.cells,trimTrailingSpaces:true};}
  private listingRequest(path:string, mode:DocumentMode):SaveDocumentRequest{return{path,mode,width:this.width,height:this.height,cells:this.cells,trimTrailingSpaces:true};}
  private chooseListingMode(defaultMode:DocumentMode):DocumentMode|null{const answer=(prompt("Listing format: ATASCII or ASCII",defaultMode.toUpperCase())??"").trim().toLowerCase();if(!answer)return null;return answer.startsWith("at")?"atascii":"ascii";}
  private load(loaded:LoadedDocument,label?:string,atrDocument:AtrDocumentLocation|null=null):void{this.width=loaded.width===80?80:40;this.height=loaded.height;this.cells=loaded.cells;while(this.cells.length<this.cellCount)this.cells.push(blankCell());this.atrDocument=atrDocument;this.path=atrDocument?null:loaded.path;this.mode=loaded.mode;document.querySelector<HTMLSelectElement>("#mode")!.value=this.mode;this.cursor=0;this.selectionActive=false;this.selectionAnchor=0;this.selectionFocus=0;this.dirty=false;this.render();if(label)document.querySelector("#fileName")!.textContent=label;if(loaded.warnings.length)window.alert(loaded.warnings.join("\n"));}

  private async openLocalFile():Promise<void>{this.activeLocationKind="local";this.renderAtrTree();await this.openFile();}
  private async openFolder():Promise<void>{const p=await open({directory:true,multiple:false,defaultPath:this.selectedLocalDirectory()||undefined});if(!p||Array.isArray(p))return;try{this.activeLocationKind="local";this.selectedLocalPath=null;this.setAtr(await invoke("local_open_folder",{path:p}));}catch(e){window.alert(String(e));}}
  private async openFile():Promise<void>{
    if(this.dirty&&!window.confirm("Discard unsaved changes?"))return;
    const location=this.activeStorageLocation();
    if(location.kind==="atr"){
      if(!this.atr.drives.find(status=>status.drive===`D${location.drive}:`)?.mounted){window.alert(`Mount an ATR image in D${location.drive}: first.`);return;}
      const selected=this.selectedAtrEntry(),defaultName=selected&&!selected.isDirectory?selected.name:"";
      const imagePath=await this.showAtrDocumentDialog("open",location,defaultName);if(!imagePath)return;
      await this.openAtrTreeFile(imagePath,location.drive,false);return;
    }
    const p=await open({title:"Open From Local Folder",multiple:false,defaultPath:location.directory||undefined});if(!p)return;
    try{this.activeLocationKind="local";this.selectedLocalPath=p;this.load(await invoke("load_document",{path:p,mode:this.mode,width:this.width,height:this.height}));await this.setLocalFolderForPath(p);}catch(e){window.alert(String(e));}
  }
  private async save(_force:boolean):Promise<void>{
    const location=this.activeStorageLocation(),defaultName=this.currentDocumentName();
    if(location.kind==="atr"){
      if(!this.atr.drives.find(status=>status.drive===`D${location.drive}:`)?.mounted){window.alert(`Mount an ATR image in D${location.drive}: first.`);return;}
      const imagePath=await this.showAtrDocumentDialog("save",location,defaultName);if(!imagePath)return;
      try{
        this.selectedAtrDrive=location.drive;this.selectedAtrPath=imagePath;
        this.setAtr(await invoke("atr_write_document",{imageName:imagePath,request:this.request(),drive:location.drive,overwrite:true}));
        this.path=null;this.atrDocument={drive:location.drive,path:imagePath};this.dirty=false;this.renderStatus();
      }catch(e){window.alert(String(e));}
      return;
    }
    const p=await save({title:"Save To Local Folder",defaultPath:this.joinLocalPath(location.directory,defaultName)});if(!p)return;
    try{await invoke("save_document",{request:this.request(p)});this.path=p;this.atrDocument=null;this.selectedLocalPath=p;this.dirty=false;await this.setLocalFolderForPath(p);this.renderStatus();}catch(e){window.alert(String(e));}
  }
  private atrEntriesInDirectory(drive:number,directory:string):AtrTreeEntry[]{
    const tree=this.atr.drives.find(status=>status.drive===`D${drive}:`)?.tree??[];
    if(!directory)return tree;
    const entry=this.findAtrTreeEntry(tree,directory);
    return entry?.isDirectory?entry.children:[];
  }
  private validAtrFilename(value:string):string|null{
    const filename=value.trim().toUpperCase();
    return /^[A-Z0-9_]{1,8}(?:\.[A-Z0-9_]{1,3})?$/.test(filename)?filename:null;
  }
  private showAtrDocumentDialog(action:"new"|"open"|"save",location:{kind:"atr";drive:number;directory:string},defaultName:string):Promise<string|null>{
    return new Promise(resolve=>{
      const entries=this.atrEntriesInDirectory(location.drive,location.directory);
      const files=entries.filter(entry=>!entry.isDirectory);
      const backdrop=document.createElement("div");backdrop.className="modal-backdrop";
      backdrop.innerHTML=`<form class="modal atr-document-dialog"><h2></h2><p class="atr-dialog-location"></p><div class="atr-file-list" role="listbox"></div><label class="atr-filename-field">Filename<input name="filename" maxlength="12" autocomplete="off"></label><div class="atr-dialog-validation"></div><div class="format-actions"><button type="button" data-dialog-cancel>Cancel</button><button type="submit" class="primary" data-dialog-accept></button></div></form>`;
      const form=backdrop.querySelector<HTMLFormElement>("form")!;
      const title=form.querySelector("h2")!,locationLabel=form.querySelector<HTMLElement>(".atr-dialog-location")!;
      const list=form.querySelector<HTMLElement>(".atr-file-list")!,input=form.querySelector<HTMLInputElement>("input")!;
      const validation=form.querySelector<HTMLElement>(".atr-dialog-validation")!,accept=form.querySelector<HTMLButtonElement>("[data-dialog-accept]")!;
      const actionLabel=action==="new"?"New":action==="open"?"Open":"Save";
      title.textContent=`${actionLabel} — D${location.drive}:`;
      locationLabel.textContent=`Folder: D${location.drive}:${location.directory||"\\"}`;
      accept.textContent=actionLabel;
      if(files.length){
        for(const entry of files){
          const button=document.createElement("button");button.type="button";button.className="atr-file-choice";button.setAttribute("role","option");
          button.innerHTML="<span></span><small></small>";button.querySelector("span")!.textContent=entry.name;button.querySelector("small")!.textContent=`${entry.sizeBytes} bytes`;
          button.addEventListener("click",()=>{input.value=entry.name;update();input.focus();});
          if(action==="open")button.addEventListener("dblclick",()=>{input.value=entry.name;update();form.requestSubmit();});
          list.appendChild(button);
        }
      }else{const empty=document.createElement("div");empty.className="atr-file-empty";empty.textContent="(No files in this folder)";list.appendChild(empty);}
      input.value=defaultName;
      const update=()=>{
        input.value=input.value.toUpperCase().replace(/[^A-Z0-9_.]/g,"");
        const filename=this.validAtrFilename(input.value),match=filename?files.find(entry=>entry.name.toUpperCase()===filename):undefined;
        if(!filename)validation.textContent="Use an Atari 8.3 filename, for example NOTES.TXT.";
        else if(action==="open"&&!match)validation.textContent="Choose an existing file in this folder.";
        else if(action==="new"&&match)validation.textContent="That file already exists. Choose another name.";
        else if(action==="save"&&match)validation.textContent="The existing file will be replaced.";
        else validation.textContent="";
        accept.disabled=!filename||(action==="open"&&!match)||(action==="new"&&Boolean(match));
      };
      const finish=(result:string|null)=>{document.removeEventListener("keydown",onKeyDown);backdrop.remove();resolve(result);};
      const onKeyDown=(event:KeyboardEvent)=>{if(event.key==="Escape"){event.preventDefault();finish(null);}};
      form.addEventListener("submit",event=>{event.preventDefault();update();if(accept.disabled)return;const filename=this.validAtrFilename(input.value);if(!filename)return;finish(location.directory?`${location.directory}>${filename}`:filename);});
      input.addEventListener("input",update);
      form.querySelector("[data-dialog-cancel]")!.addEventListener("click",()=>finish(null));
      backdrop.addEventListener("click",event=>{if(event.target===backdrop)finish(null);});
      document.addEventListener("keydown",onKeyDown);document.body.appendChild(backdrop);update();input.select();
    });
  }
  private async exportAscii():Promise<void>{const p=await save({defaultPath:"EXPORT.TXT"});if(!p)return;const r=this.request(p);r.mode="ascii";await invoke("save_document",{request:r});await this.setLocalFolderForPath(p);}

  private requireAtr():boolean{if(this.selectedDriveStatus()?.mounted)return true;window.alert(`Mount an ATR image in D${this.selectedAtrDrive}: first.`);return false;}
  private async refreshAtr(show:boolean):Promise<void>{try{this.setAtr(await invoke("atr_status"));if(show&&this.atr.mounted)this.showAtrDirectory();}catch(e){if(show)window.alert(String(e));}}
  private async atrCreate():Promise<void>{
    const format=await this.showAtrFormatDialog();if(!format)return;
    const p=await save({defaultPath:"newdisk.atr",filters:[{name:"Atari ATR",extensions:["atr"]}]});if(!p)return;
    try{
      this.activeLocationKind="atr";this.selectedAtrDrive=format.drive;this.selectedAtrPath=null;
      this.setAtr(await invoke("atr_create",{request:{path:p,filesystem:format.filesystem,sectors:format.sectors,sectorSize:format.sectorSize,volumeLabel:format.volumeLabel,force:true,drive:format.drive}}));
    }catch(e){window.alert(String(e));}
  }
  private showAtrFormatDialog():Promise<AtrFormatConfig|null>{
    return new Promise(resolve=>{
      const backdrop=document.createElement("div");backdrop.className="modal-backdrop atr-format-backdrop";
      const driveOptions=[1,2,3,4].map(drive=>{
        const mounted=this.atr.drives.find(status=>status.drive===`D${drive}:`)?.mounted??false;
        return `<label class="format-radio compact"><input type="radio" name="drive" value="${drive}" ${drive===this.selectedAtrDrive?"checked":""}><span><strong>D${drive}:</strong><small>${mounted?"Mounted":"Empty"}</small></span></label>`;
      }).join("");
      backdrop.innerHTML=`
        <form class="modal atr-format-dialog" aria-labelledby="atrFormatTitle">
          <h2 id="atrFormatTitle">New ATR Disk Formatter</h2>
          <p class="format-lede">Configure the disk in one pass. The ATR destination is selected after formatting options are confirmed.</p>
          <div class="format-options">
            <fieldset>
              <legend>Target drive</legend>
              <div class="format-radio-grid drives">${driveOptions}</div>
              <small class="format-hint">Creating into a mounted slot replaces that slot's current mount; it does not alter the old ATR.</small>
            </fieldset>
            <fieldset>
              <legend>Filesystem</legend>
              <div class="format-radio-grid">
                <label class="format-radio"><input type="radio" name="filesystem" value="DOS2" checked><span><strong>Atari DOS 2</strong><small>Classic flat 8.3 directory</small></span></label>
                <label class="format-radio"><input type="radio" name="filesystem" value="SPARTA"><span><strong>SpartaDOS 2.x / SDX</strong><small>Volume label and subdirectories</small></span></label>
              </div>
            </fieldset>
            <fieldset>
              <legend>Disk geometry</legend>
              <div class="format-radio-grid geometry">
                <label class="format-radio"><input type="radio" name="geometry" value="90K" checked><span><strong>90K</strong><small>720 × 128 · Single density</small></span></label>
                <label class="format-radio"><input type="radio" name="geometry" value="130K"><span><strong>130K</strong><small>1040 × 128 · Enhanced</small></span></label>
                <label class="format-radio"><input type="radio" name="geometry" value="180K"><span><strong>180K</strong><small>720 × 256 · Double density</small></span></label>
                <label class="format-radio"><input type="radio" name="geometry" value="360K"><span><strong>360K</strong><small>1440 × 256 · DD, double-sided</small></span></label>
                <label class="format-radio"><input type="radio" name="geometry" value="16M"><span><strong>16M</strong><small>65,535 × 256 · Large partition</small></span></label>
                <label class="format-radio"><input type="radio" name="geometry" value="CUSTOM"><span><strong>Custom</strong><small>Choose sectors and bytes</small></span></label>
              </div>
              <div class="custom-geometry" hidden>
                <label>Sector count<input name="customSectors" type="number" min="368" max="1040" step="1" value="720" disabled></label>
                <label>Bytes / sector<select name="customSectorSize" disabled><option value="128">128</option><option value="256">256</option></select></label>
              </div>
              <small class="format-hint">360K and 16M use SpartaDOS. Double-density ATR images retain 128-byte boot sectors 1–3.</small>
            </fieldset>
            <fieldset>
              <legend>SpartaDOS volume</legend>
              <label class="volume-field">Volume label<input name="volumeLabel" type="text" maxlength="8" value="NEWDISK" autocomplete="off" spellcheck="false" disabled></label>
              <small class="format-hint">Up to 8 characters. Available when SpartaDOS is selected.</small>
            </fieldset>
          </div>
          <div class="format-validation" role="status"></div>
          <div class="format-actions"><button type="button" data-format-cancel>Cancel</button><button class="primary" type="submit" data-format-create>Choose File &amp; Create</button></div>
        </form>`;
      document.body.appendChild(backdrop);
      const form=backdrop.querySelector<HTMLFormElement>("form")!;
      const customSectors=form.elements.namedItem("customSectors") as HTMLInputElement;
      const customSectorSize=form.elements.namedItem("customSectorSize") as HTMLSelectElement;
      const customGeometry=form.querySelector<HTMLElement>(".custom-geometry")!;
      const volumeLabel=form.elements.namedItem("volumeLabel") as HTMLInputElement;
      const validation=form.querySelector<HTMLElement>(".format-validation")!;
      const createButton=form.querySelector<HTMLButtonElement>("[data-format-create]")!;
      const dos2Option=form.querySelector<HTMLInputElement>('input[name="filesystem"][value="DOS2"]')!;
      const spartaOption=form.querySelector<HTMLInputElement>('input[name="filesystem"][value="SPARTA"]')!;
      const presetGeometry:Record<string,{sectors:number;sectorSize:128|256;requiresSparta?:boolean}>={
        "90K":{sectors:720,sectorSize:128},
        "130K":{sectors:1040,sectorSize:128},
        "180K":{sectors:720,sectorSize:256},
        "360K":{sectors:1440,sectorSize:256,requiresSparta:true},
        "16M":{sectors:65535,sectorSize:256,requiresSparta:true}
      };
      let current:{drive:number;filesystem:"DOS2"|"SPARTA";sectors:number;sectorSize:128|256}|null=null;
      const radioValue=(name:string)=>new FormData(form).get(name)?.toString()??"";
      const update=()=>{
        const drive=Number(radioValue("drive"))||this.selectedAtrDrive;
        const geometry=radioValue("geometry");
        const preset=presetGeometry[geometry];
        dos2Option.disabled=preset?.requiresSparta??false;
        if(preset?.requiresSparta)spartaOption.checked=true;
        const filesystem=radioValue("filesystem")==="SPARTA"?"SPARTA":"DOS2";
        const isCustom=geometry==="CUSTOM";
        customGeometry.hidden=!isCustom;
        customSectors.disabled=!isCustom;customSectorSize.disabled=!isCustom;
        volumeLabel.disabled=filesystem!=="SPARTA";
        const sectors=preset?.sectors??Number(customSectors.value);
        const sectorSize=preset?.sectorSize??(Number(customSectorSize.value)===256?256:128);
        const minimum=filesystem==="SPARTA"?16:368,maximum=filesystem==="SPARTA"?65535:1040;
        customSectors.min=String(minimum);customSectors.max=String(maximum);
        const sectorError=isCustom&&(!Number.isInteger(sectors)||sectors<minimum||sectors>maximum)
          ? `${filesystem==="SPARTA"?"SpartaDOS":"Atari DOS 2"} requires ${minimum.toLocaleString()}–${maximum.toLocaleString()} sectors.`
          :"";
        const label=volumeLabel.value.trim();
        const volumeError=filesystem==="SPARTA"&&!/^[A-Za-z0-9][A-Za-z0-9 _-]{0,7}$/.test(label)
          ?"Use 1–8 letters, numbers, spaces, _ or -, beginning with a letter or number."
          :"";
        validation.textContent=sectorError||volumeError;
        createButton.disabled=Boolean(sectorError||volumeError);
        current={drive,filesystem,sectors,sectorSize};
      };
      let settled=false;
      const finish=(result:AtrFormatConfig|null)=>{
        if(settled)return;settled=true;
        document.removeEventListener("keydown",onKeyDown);
        backdrop.remove();resolve(result);
      };
      const onKeyDown=(event:KeyboardEvent)=>{if(event.key==="Escape"){event.preventDefault();finish(null);}};
      form.addEventListener("change",update);
      customSectors.addEventListener("input",update);
      volumeLabel.addEventListener("input",update);
      form.addEventListener("submit",event=>{
        event.preventDefault();update();
        if(createButton.disabled||!current)return;
        finish({drive:current.drive,filesystem:current.filesystem,sectors:current.sectors,sectorSize:current.sectorSize,volumeLabel:current.filesystem==="SPARTA"?volumeLabel.value.trim().toUpperCase():null});
      });
      form.querySelector("[data-format-cancel]")!.addEventListener("click",()=>finish(null));
      document.addEventListener("keydown",onKeyDown);
      update();
      form.querySelector<HTMLInputElement>('input[name="filesystem"]:checked')?.focus();
    });
  }
  private async atrMount():Promise<void>{const p=await open({multiple:false,filters:[{name:"Atari ATR",extensions:["atr"]}]});if(!p)return;try{this.activeLocationKind="atr";this.selectedAtrPath=null;this.setAtr(await invoke("atr_mount",{path:p,drive:this.selectedAtrDrive}));this.showAtrDirectory();}catch(e){window.alert(String(e));}}
  private async atrWriteDocument():Promise<void>{if(!this.requireAtr())return;const n=prompt(`Filename inside D${this.selectedAtrDrive}:`,this.defaultAtrPath(this.mode==="atascii"?"UNTITLED.ATA":"UNTITLED.TXT"));if(!n)return;this.setAtr(await invoke("atr_write_document",{imageName:n,request:this.request(),drive:this.selectedAtrDrive}));this.showAtrDirectory();}
  private async atrOpenDocument():Promise<void>{this.activeLocationKind="atr";this.renderAtrTree();await this.openFile();}
  private async atrAddHost():Promise<void>{if(!this.requireAtr())return;const p=await open({multiple:false});if(!p)return;const n=prompt("Destination filename",this.defaultAtrPath((p.split(/[\\/]/).pop()??"FILE").toUpperCase()));if(!n)return;this.setAtr(await invoke("atr_add_host_file",{hostPath:p,imageName:n,drive:this.selectedAtrDrive}));this.showAtrDirectory();}
  private async atrExtract():Promise<void>{if(!this.requireAtr())return;const n=prompt(`Filename inside D${this.selectedAtrDrive}:`);if(!n)return;const p=await save({defaultPath:n});if(!p)return;this.setAtr(await invoke("atr_extract_file",{imageName:n,hostPath:p,drive:this.selectedAtrDrive}));}
  private async atrDelete():Promise<void>{if(!this.requireAtr())return;const selected=this.selectedAtrEntry();if(selected){await this.deleteSelectedAtrEntry();return;}const n=prompt(`Filename to delete from D${this.selectedAtrDrive}:`);if(!n||!confirm(`Delete D${this.selectedAtrDrive}: ${n}?`))return;this.setAtr(await invoke("atr_delete_file",{imageName:n,drive:this.selectedAtrDrive}));this.showAtrDirectory();}
  private async atrMkdir():Promise<void>{if(!this.requireAtr())return;const n=prompt(`SpartaDOS directory path in D${this.selectedAtrDrive}:`,this.defaultAtrPath("NEWDIR"));if(!n)return;this.setAtr(await invoke("atr_mkdir",{path:n,drive:this.selectedAtrDrive}));this.collapsedAtrPaths.delete(`D${this.selectedAtrDrive}:${this.parentAtrPath(n)}`);this.showAtrDirectory();}
  private async atrClose():Promise<void>{this.selectedAtrPath=null;this.setAtr(await invoke("atr_close",{drive:this.selectedAtrDrive}));}
  private showAtrDirectory():void{if(this.requireAtr())this.showTextModal("ATR Directory",this.atr.entries.length?this.atr.entries.join("\n"):"(empty image)");}
  private showAtrInfo():void{if(this.requireAtr())this.showTextModal("ATR Image Information",[...this.atr.info,"","DIRECTORY",...this.atr.entries].join("\n"));}

  private async basicDetokHost():Promise<void>{if(this.dirty&&!confirm("Discard unsaved changes?"))return;const p=await open({multiple:false,filters:[{name:"Tokenized Atari BASIC",extensions:["bas"]}]});if(!p)return;try{this.load(await invoke("basic_detokenize_host",{path:p,width:this.width,height:this.height}),`${p.split(/[\\/]/).pop()} [DETOKENIZED]`);await this.setLocalFolderForPath(p);}catch(e){alert(String(e));}}
  private async basicTokHost():Promise<void>{const p=await save({defaultPath:"PROGRAM.BAS",filters:[{name:"Tokenized Atari BASIC",extensions:["bas"]}]});if(!p)return;try{await invoke("basic_tokenize_host",{request:{destination:p,document:this.basicRequest()}});await this.setLocalFolderForPath(p);alert(`Native tokenized BASIC saved:\n${p}`);}catch(e){alert(String(e));}}
  private async basicListHost():Promise<void>{const format=this.chooseListingMode("ascii");if(!format)return;const p=await save({defaultPath:format==="atascii"?"PROGRAM.LST":"PROGRAM.TXT",filters:[{name:"Detokenized BASIC Listing",extensions:["lst","bas","txt"]}]});if(!p)return;try{await invoke("basic_save_listing_host",{request:{destination:p,document:this.listingRequest(p,format)}});await this.setLocalFolderForPath(p);alert(`${format.toUpperCase()} detokenized listing saved:\n${p}`);}catch(e){alert(String(e));}}
  private async basicDetokAtr():Promise<void>{if(!this.requireAtr())return;const n=prompt("Tokenized BASIC filename in ATR","PROGRAM.BAS");if(!n)return;try{this.load(await invoke("basic_detokenize_atr",{imageName:n,width:this.width,height:this.height}),`${n} [ATR BASIC]`);}catch(e){alert(String(e));}}
  private async basicTokAtr():Promise<void>{if(!this.requireAtr())return;const n=prompt("Tokenized BASIC destination in ATR","PROGRAM.BAS");if(!n)return;try{this.setAtr(await invoke("basic_tokenize_to_atr",{imageName:n,document:this.basicRequest()}));alert(`${n} tokenized natively and saved to ATR.`);}catch(e){alert(String(e));}}
  private async basicListAtr():Promise<void>{if(!this.requireAtr())return;const format=this.chooseListingMode("atascii");if(!format)return;const n=prompt("Detokenized listing destination in ATR","PROGRAM.LST");if(!n)return;try{this.setAtr(await invoke("basic_save_listing_to_atr",{imageName:n,document:this.listingRequest("",format)}));alert(`${n} ${format.toUpperCase()} listing saved to ATR.`);}catch(e){alert(String(e));}}

  private showTextModal(title:string,text:string):void{const b=document.createElement("div");b.className="modal-backdrop";b.innerHTML=`<section class="modal atr-modal"><h2></h2><pre></pre><button>Close</button></section>`;b.querySelector("h2")!.textContent=title;b.querySelector("pre")!.textContent=text;b.querySelector("button")!.addEventListener("click",()=>b.remove());b.addEventListener("click",e=>{if(e.target===b)b.remove();});document.body.appendChild(b);}
  private async checkForUpdates():Promise<void>{
    document.querySelector(".update-backdrop")?.remove();
    const backdrop=document.createElement("div");backdrop.className="modal-backdrop update-backdrop";
    backdrop.innerHTML=`<section class="modal update-modal" role="dialog" aria-modal="true" aria-labelledby="updateTitle">
      <h2 id="updateTitle">Check for Updates</h2>
      <p class="update-status" role="status" aria-live="polite">Checking the published version…</p>
      <dl class="update-versions" hidden><dt>Installed</dt><dd data-update-current></dd><dt>Published</dt><dd data-update-latest></dd></dl>
      <p class="update-files" hidden></p>
      <div class="update-actions"><button type="button" data-update-close>Close</button><button type="button" data-update-exe hidden>Download Portable EXE</button><button type="button" class="primary" data-update-msi hidden>Download &amp; Install MSI</button><button type="button" class="primary" data-update-dmg hidden>Download macOS DMG</button></div>
    </section>`;
    const status=backdrop.querySelector<HTMLElement>(".update-status")!,versions=backdrop.querySelector<HTMLElement>(".update-versions")!,files=backdrop.querySelector<HTMLElement>(".update-files")!;
    const exeButton=backdrop.querySelector<HTMLButtonElement>("[data-update-exe]")!,msiButton=backdrop.querySelector<HTMLButtonElement>("[data-update-msi]")!,dmgButton=backdrop.querySelector<HTMLButtonElement>("[data-update-dmg]")!;
    const close=()=>{document.removeEventListener("keydown",onKeyDown);backdrop.remove();this.hiddenInput.focus();};
    const onKeyDown=(event:KeyboardEvent)=>{if(event.key==="Escape"&&!this.isBusy){event.preventDefault();close();}};
    backdrop.querySelector("[data-update-close]")!.addEventListener("click",close);
    backdrop.addEventListener("click",event=>{if(event.target===backdrop&&!this.isBusy)close();});
    document.addEventListener("keydown",onKeyDown);document.body.appendChild(backdrop);
    try{
      const info=await invoke<UpdateInfo>("check_for_updates");
      versions.hidden=false;
      versions.querySelector<HTMLElement>("[data-update-current]")!.textContent=info.currentVersion;
      versions.querySelector<HTMLElement>("[data-update-latest]")!.textContent=info.latestVersion;
      if(info.state==="current")status.textContent="QuarterMaster/M is up to date.";
      else if(info.state==="newer")status.textContent="This build is newer than the currently published release.";
      else{
        status.textContent=`QuarterMaster/M ${info.latestVersion} is available.`;
        files.hidden=false;files.textContent=`Portable: ${info.exeFile} · Installer: ${info.msiFile}`;
        if(info.platform==="macos"&&info.dmgFile){
          files.textContent=`macOS DMG: ${info.dmgFile}`;
          dmgButton.hidden=false;
        }else if(info.platform==="windows"&&info.exeFile&&info.msiFile){
          files.textContent=`Portable: ${info.exeFile} · Installer: ${info.msiFile}`;
          exeButton.hidden=false;msiButton.hidden=false;
        }else{
          files.textContent="No automatic update package is available for this platform.";
        }
        const setDownloading=(downloading:boolean)=>{exeButton.disabled=downloading;msiButton.disabled=downloading;dmgButton.disabled=downloading;};
        exeButton.addEventListener("click",async()=>{
          setDownloading(true);status.textContent="Downloading the portable update…";
          try{
            const result=await invoke<UpdateDownload>("download_portable_update");
            status.textContent=`Version ${result.version} downloaded beside this application. Close QuarterMaster/M, then run: ${result.path}`;
            exeButton.hidden=true;msiButton.hidden=true;
          }catch(error){status.textContent=`Update download failed: ${String(error)}`;setDownloading(false);}
        });
        msiButton.addEventListener("click",async()=>{
          setDownloading(true);status.textContent="Downloading the Windows Installer…";
          try{
            const result=await invoke<UpdateDownload>("download_and_install_update");
            status.textContent=`Version ${result.version} downloaded. Windows Installer has been launched from: ${result.path}`;
            exeButton.hidden=true;msiButton.hidden=true;
          }catch(error){status.textContent=`Installer download failed: ${String(error)}`;setDownloading(false);}
        });
        dmgButton.addEventListener("click",async()=>{
          setDownloading(true);status.textContent="Downloading the macOS disk image…";
          try{
            const result=await invoke<UpdateDownload>("download_macos_update");
            status.textContent=`Version ${result.version} downloaded and opened from: ${result.path}`;
            exeButton.hidden=true;msiButton.hidden=true;dmgButton.hidden=true;
          }catch(error){status.textContent=`macOS update download failed: ${String(error)}`;setDownloading(false);}
        });
      }
    }catch(error){status.textContent=`Could not check for updates: ${String(error)}`;}
  }
  private showAbout():void{this.showTextModal("About","Quartermaster/M\nVersion: "+APP_VERSION+"\n(C)2026 Rick Collette (megalith)");}
  private showHelp(section="start"):void{showHelpCenter(section,APP_VERSION);}
}

void new Editor().mount().catch(async error=>{
  console.error(error);
  try{await tauriInvoke("app_ready");}catch{}
  window.setTimeout(()=>window.alert(`QuarterMaster could not finish starting:\n${String(error)}`),0);
});
