// @vitest-environment jsdom

import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AtrStatus, Cell } from "./types";

const {invokeMock}=vi.hoisted(()=>({invokeMock:vi.fn()}));

vi.mock("@tauri-apps/api/core",()=>({invoke:invokeMock}));
vi.mock("@tauri-apps/plugin-dialog",()=>({open:vi.fn(),save:vi.fn()}));
vi.mock("@tauri-apps/api/webview",()=>({
  getCurrentWebview:()=>({onDragDropEvent:vi.fn().mockResolvedValue(()=>{})}),
}));

import { Editor } from "./main";

const statusFor=(imagePath:string):AtrStatus=>({
  mounted:true,
  activeDrive:"D1:",
  path:"/tmp/test.atr",
  filesystem:"SpartaDOS 2",
  entries:[imagePath],
  info:[],
  tree:[{name:imagePath,path:imagePath,isDirectory:false,sizeBytes:4,children:[]}],
  drives:[{
    drive:"D1:",
    mounted:true,
    active:true,
    path:"/tmp/test.atr",
    filesystem:"SpartaDOS 2",
    entries:[imagePath],
    info:[],
    tree:[{name:imagePath,path:imagePath,isDirectory:false,sizeBytes:4,children:[]}],
  }],
  localFolder:null,
  localTree:[],
});

type TestEditor={
  atr:AtrStatus;
  atrDocument:{drive:number;path:string}|null;
  activeLocationKind:"local"|"atr";
  selectedAtrDrive:number;
  selectedAtrPath:string|null;
  cells:Cell[];
  dirty:boolean;
  save(force:boolean):Promise<void>;
  setAtr(status:AtrStatus):void;
  renderStatus():void;
  showAtrSaveConfirmation(drive:number,imagePath:string):void;
};

const editorForAtr=(path="TEST.UDL"):TestEditor=>{
  const editor=new Editor() as unknown as TestEditor;
  editor.atr=statusFor(path);
  editor.atrDocument={drive:1,path};
  editor.activeLocationKind="atr";
  editor.selectedAtrDrive=1;
  editor.selectedAtrPath=path;
  editor.dirty=true;
  editor.setAtr=(status:AtrStatus)=>{editor.atr=status;};
  editor.renderStatus=()=>{};
  editor.showAtrSaveConfirmation=()=>{};
  return editor;
};

describe("ATR document saving",()=>{
  beforeEach(()=>{
    document.body.innerHTML="";
    invokeMock.mockReset();
    vi.stubGlobal("alert",vi.fn());
    HTMLElement.prototype.scrollIntoView=vi.fn();
    invokeMock.mockImplementation(async(command:string,args?:Record<string,unknown>)=>{
      expect(command).toBe("atr_write_document");
      return statusFor(String(args?.imageName));
    });
  });

  it("saves an opened ATR document directly back to its mounted image",async()=>{
    const editor=editorForAtr();

    await editor.save(false);

    expect(invokeMock).toHaveBeenCalledOnce();
    expect(invokeMock).toHaveBeenCalledWith("atr_write_document",expect.objectContaining({
      imageName:"TEST.UDL",
      drive:1,
      overwrite:true,
    }));
    expect(editor.dirty).toBe(false);
  });

  it("only sends rows with document data when saving",async()=>{
    const editor=editorForAtr();
    editor.cells[0]={byte:0x41,inverse:false,display:"A"};
    editor.cells[119]={byte:0x5a,inverse:false,display:"Z"};

    await editor.save(false);

    expect(invokeMock).toHaveBeenCalledWith("atr_write_document",expect.objectContaining({
      request:expect.objectContaining({
        width:40,
        height:3,
        cells:expect.arrayContaining([expect.objectContaining({byte:0x5a})]),
      }),
    }));
    const request=(invokeMock.mock.calls[0][1] as {request:{cells:Cell[]}}).request;
    expect(request.cells).toHaveLength(120);
  });

  it("saves an ATR document under a new 8.3 filename",async()=>{
    const editor=editorForAtr();
    const saving=editor.save(true);
    await vi.waitFor(()=>expect(document.querySelector("form.atr-document-dialog")).not.toBeNull());
    const input=document.querySelector<HTMLInputElement>('input[name="filename"]')!;
    input.value="TEST2.UDL";
    document.querySelector<HTMLFormElement>("form.atr-document-dialog")!
      .dispatchEvent(new Event("submit",{bubbles:true,cancelable:true}));

    await saving;

    expect(invokeMock).toHaveBeenCalledOnce();
    expect(invokeMock).toHaveBeenCalledWith("atr_write_document",expect.objectContaining({
      imageName:"TEST2.UDL",
      drive:1,
      overwrite:true,
    }));
    expect(editor.atrDocument).toEqual({drive:1,path:"TEST2.UDL"});
  });

  it("routes the real Save As button and form through Tauri IPC",async()=>{
    invokeMock.mockImplementation(async(command:string,args?:Record<string,unknown>)=>{
      if(command==="app_ready")return undefined;
      if(command==="atr_status")return statusFor("TEST.UDL");
      if(command==="atr_write_document")return statusFor(String(args?.imageName));
      throw new Error(`Unexpected command: ${command}`);
    });
    document.body.innerHTML='<div id="app"></div>';
    const editor=new Editor();
    await editor.mount();
    const state=editor as unknown as TestEditor;
    state.atr=statusFor("TEST.UDL");
    state.atrDocument={drive:1,path:"TEST.UDL"};
    state.activeLocationKind="atr";
    state.selectedAtrDrive=1;
    state.selectedAtrPath="TEST.UDL";

    document.querySelector<HTMLButtonElement>('[data-cmd="saveAs"]')!.click();
    await vi.waitFor(()=>expect(document.querySelector("form.atr-document-dialog")).not.toBeNull());
    const input=document.querySelector<HTMLInputElement>('input[name="filename"]')!;
    input.value="TEST2.UDL";
    document.querySelector<HTMLFormElement>("form.atr-document-dialog")!
      .dispatchEvent(new Event("submit",{bubbles:true,cancelable:true}));

    await vi.waitFor(()=>expect(invokeMock).toHaveBeenCalledWith(
      "atr_write_document",
      expect.objectContaining({imageName:"TEST2.UDL",drive:1,overwrite:true}),
    ));
  },15_000);
});
