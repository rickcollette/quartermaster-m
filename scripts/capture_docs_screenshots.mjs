import { spawn } from "node:child_process";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
const outputDirectory = resolve(root, "docs/images");
const scratchDirectory = resolve(root, ".screenshots");
const chromeProfile = resolve(scratchDirectory, "chrome-profile");
const chromePath = "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe";
const devUrl = "http://127.0.0.1:1420";
const debugPort = 9333;

mkdirSync(outputDirectory, { recursive: true });
mkdirSync(scratchDirectory, { recursive: true });
rmSync(chromeProfile, { recursive: true, force: true });

const vite = spawn(
  process.execPath,
  [resolve(root, "node_modules/vite/bin/vite.js"), "--host", "127.0.0.1", "--port", "1420"],
  { cwd: root, windowsHide: true, stdio: "ignore" },
);

const chrome = spawn(
  chromePath,
  [
    "--headless=new",
    `--remote-debugging-port=${debugPort}`,
    `--user-data-dir=${chromeProfile}`,
    "--window-size=1260,900",
    "--force-device-scale-factor=1",
    "--hide-scrollbars",
    "--disable-features=Translate",
    "--no-first-run",
    "--no-default-browser-check",
    devUrl,
  ],
  { cwd: root, windowsHide: true, stdio: "ignore" },
);

const delay = milliseconds => new Promise(resolveDelay => setTimeout(resolveDelay, milliseconds));

async function waitFor(getter, description, attempts = 100) {
  let lastError;
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    try {
      const value = await getter();
      if (value) return value;
    } catch (error) {
      lastError = error;
    }
    await delay(100);
  }
  throw new Error(`Timed out waiting for ${description}${lastError ? `: ${lastError}` : ""}`);
}

async function connectCdp(webSocketUrl) {
  const socket = new WebSocket(webSocketUrl);
  await new Promise((resolveOpen, rejectOpen) => {
    socket.addEventListener("open", resolveOpen, { once: true });
    socket.addEventListener("error", rejectOpen, { once: true });
  });
  let sequence = 0;
  const pending = new Map();
  socket.addEventListener("message", event => {
    const message = JSON.parse(event.data);
    if (!message.id || !pending.has(message.id)) return;
    const { resolve: resolveCall, reject } = pending.get(message.id);
    pending.delete(message.id);
    if (message.error) reject(new Error(JSON.stringify(message.error)));
    else resolveCall(message.result);
  });
  return {
    call(method, params = {}) {
      const id = ++sequence;
      socket.send(JSON.stringify({ id, method, params }));
      return new Promise((resolveCall, reject) => pending.set(id, { resolve: resolveCall, reject }));
    },
    close() {
      socket.close();
    },
  };
}

const demoSources = {
  editor: {
    bytes: readFileSync(resolve(root, "docs/examples/quartermaster-command-deck.ata")).toString("base64"),
    fileName: "COMMAND.ATA",
    diskName: "COMMAND-DECK.ATR",
  },
  selection: {
    bytes: readFileSync(resolve(root, "docs/examples/quartermaster-screen-composer.ata")).toString("base64"),
    fileName: "COMPOSER.ATA",
    diskName: "SCREEN-LAB.ATR",
  },
};

function demoExpression(source) {
  return String.raw`
(async () => {
  const wait = ms => new Promise(resolve => setTimeout(resolve, ms));
  while (!document.querySelector("#keyboard") || document.querySelectorAll("#screen .cell").length < 1000) await wait(25);
  await document.fonts.ready;

  const keyboard = document.querySelector("#keyboard");
  const type = text => {
    keyboard.value = text;
    keyboard.dispatchEvent(new InputEvent("input", { bubbles: true, inputType: "insertText", data: text }));
  };
  const encoded = ${JSON.stringify(source.bytes)};
  const bytes = Uint8Array.from(atob(encoded), character => character.charCodeAt(0));
  const rows = [[]];
  for (const byte of bytes) {
    if (byte === 0x9b) rows.push([]);
    else rows[rows.length - 1].push(byte);
  }
  if (rows.length < 24) throw new Error("Screenshot ATASCII source must contain at least 24 rows");
  const screenshotRows = rows.slice(0, 24);
  let inverse = false;
  for (const sourceRow of screenshotRows) {
    const row = [...sourceRow.slice(0, 40)];
    while (row.length < 40) row.push(0x20);
    let run = "";
    for (const byte of row) {
      const nextInverse = (byte & 0x80) !== 0;
      if (nextInverse !== inverse) {
        if (run) type(run);
        run = "";
        document.querySelector("[data-cmd='inverse']").click();
        inverse = nextInverse;
      }
      run += String.fromCharCode(byte & 0x7f);
    }
    if (run) type(run);
  }
  if (inverse) document.querySelector("[data-cmd='inverse']").click();

  document.querySelector("#localRootName").textContent = "D:\\ATARI\\WORKSHOP";
  document.querySelector("#localTree").innerHTML = [
    '<button class="tree-root location-root" type="button">- WORKSHOP</button>',
    '<button class="tree-row" style="--depth:1" type="button"><span class="tree-icon">+</span><span>DIR</span><span class="tree-name">PROJECTS</span></button>',
    '<button class="tree-row file" style="--depth:1" type="button"><span></span><span>FILE</span><span class="tree-name">README.TXT</span></button>',
    '<button class="tree-row file" style="--depth:1" type="button"><span></span><span>FILE</span><span class="tree-name">${source.fileName}</span></button>',
  ].join("");
  document.querySelector("#atrTree").innerHTML = [
    '<button class="tree-row drive-root location-active" type="button">D1: ${source.diskName} [SPARTA2]</button>',
    '<button class="tree-row" style="--depth:1" type="button"><span class="tree-icon">-</span><span>DIR</span><span class="tree-name">ATASCII</span></button>',
    '<button class="tree-row" style="--depth:2" type="button"><span class="tree-icon">-</span><span>DIR</span><span class="tree-name">SCREENS</span></button>',
    '<button class="tree-row file selected" style="--depth:3" type="button"><span></span><span>FILE</span><span class="tree-name">${source.fileName}</span></button>',
    '<button class="tree-row file" style="--depth:3" type="button"><span></span><span>FILE</span><span class="tree-name">MENU.ATA</span></button>',
    '<button class="tree-row file" style="--depth:1" type="button"><span></span><span>FILE</span><span class="tree-name">DEMO.BAS</span></button>',
    '<button class="tree-row drive-root" type="button">D2: UTILITIES.ATR [DOS2]</button>',
    '<button class="tree-row drive-root" type="button">D3: NO ATR MOUNTED</button>',
    '<button class="tree-row drive-root" type="button">D4: NO ATR MOUNTED</button>',
  ].join("");
  document.querySelector("#atrDriveName").textContent = "D1: ${source.diskName}";
  document.querySelector("#atrSelection").textContent = "D1: ATASCII>SCREENS>${source.fileName}";
  document.querySelector("#atrMountLabel").textContent = "ATR: ${source.diskName} [SPARTADOS 2]";
  document.querySelector("#activeLocationLabel").textContent = "D1: ATASCII>SCREENS";
  const fileName = document.querySelector("#fileName");
  fileName.textContent = "${source.fileName} [D1:]";
  fileName.classList.add("dirty");
  document.querySelector("#position").textContent = "ROW 024 COL 40";
  document.querySelector("#modeStatus").textContent = "ATASCII NOR OVR";
  document.querySelector("#byteStatus").textContent = "$03";
  document.querySelector("#geometry").textContent = "40×357 / 24 ROW VIEW";
  document.querySelector(".workspace").scrollTop = 0;
  keyboard.focus();
  await wait(350);
  return { cells: document.querySelectorAll("#screen .cell").length, rows: screenshotRows.length };
})()
`;
}

const selectionExpression = String.raw`
(() => {
  const labels = {
    mount: document.querySelector("#atrMountLabel").textContent,
    active: document.querySelector("#activeLocationLabel").textContent,
    file: document.querySelector("#fileName").textContent,
  };
  const cells = document.querySelectorAll("#screen .cell");
  const start = cells[8 * 40 + 2];
  const end = cells[10 * 40 + 37];
  const a = start.getBoundingClientRect();
  const b = end.getBoundingClientRect();
  start.dispatchEvent(new PointerEvent("pointerdown", {
    bubbles: true, cancelable: true, button: 0, buttons: 1, pointerId: 7,
    clientX: a.left + a.width / 2, clientY: a.top + a.height / 2
  }));
  window.dispatchEvent(new PointerEvent("pointermove", {
    bubbles: true, cancelable: true, button: 0, buttons: 1, pointerId: 7,
    clientX: b.left + b.width / 2, clientY: b.top + b.height / 2
  }));
  window.dispatchEvent(new PointerEvent("pointerup", {
    bubbles: true, cancelable: true, button: 0, buttons: 0, pointerId: 7,
    clientX: b.left + b.width / 2, clientY: b.top + b.height / 2
  }));
  const selected = cells[9 * 40 + 10];
  const p = selected.getBoundingClientRect();
  selected.dispatchEvent(new MouseEvent("contextmenu", {
    bubbles: true, cancelable: true, button: 2,
    clientX: p.left + p.width / 2, clientY: p.top + p.height / 2
  }));
  document.querySelector("#atrMountLabel").textContent = labels.mount;
  document.querySelector("#activeLocationLabel").textContent = labels.active;
  document.querySelector("#fileName").textContent = labels.file;
  return document.querySelectorAll("#screen .cell.selected").length;
})()
`;

try {
  await waitFor(async () => (await fetch(devUrl)).ok, "Vite");
  const target = await waitFor(async () => {
    const targets = await (await fetch(`http://127.0.0.1:${debugPort}/json`)).json();
    return targets.find(candidate => candidate.type === "page" && candidate.url.startsWith(devUrl));
  }, "headless Chrome page");
  const cdp = await connectCdp(target.webSocketDebuggerUrl);
  try {
    await cdp.call("Page.enable");
    await cdp.call("Runtime.enable");
    await cdp.call("Emulation.setDeviceMetricsOverride", {
      width: 1260,
      height: 900,
      deviceScaleFactor: 1,
      mobile: false,
    });
    await waitFor(async () => {
      const result = await cdp.call("Runtime.evaluate", {
        expression: "Boolean(document.querySelector('#screen'))",
        returnByValue: true,
      });
      return result.result.value;
    }, "QuarterMaster/M UI");
    const demo = await cdp.call("Runtime.evaluate", {
      expression: demoExpression(demoSources.editor),
      awaitPromise: true,
      returnByValue: true,
    });
    if (demo.exceptionDetails) throw new Error(demo.exceptionDetails.text);

    const first = await cdp.call("Page.captureScreenshot", {
      format: "png",
      fromSurface: true,
      captureBeyondViewport: false,
    });
    writeFileSync(resolve(outputDirectory, "quartermaster-editor.png"), Buffer.from(first.data, "base64"));

    await cdp.call("Page.reload", { ignoreCache: true });
    await waitFor(async () => {
      const result = await cdp.call("Runtime.evaluate", {
        expression: "document.querySelectorAll('#screen .cell').length >= 1000",
        returnByValue: true,
      });
      return result.result.value;
    }, "reloaded QuarterMaster/M UI");
    const composer = await cdp.call("Runtime.evaluate", {
      expression: demoExpression(demoSources.selection),
      awaitPromise: true,
      returnByValue: true,
    });
    if (composer.exceptionDetails) throw new Error(composer.exceptionDetails.text);

    const selection = await cdp.call("Runtime.evaluate", {
      expression: selectionExpression,
      returnByValue: true,
    });
    if (selection.exceptionDetails) throw new Error(selection.exceptionDetails.text);
    await delay(150);
    const second = await cdp.call("Page.captureScreenshot", {
      format: "png",
      fromSurface: true,
      captureBeyondViewport: false,
    });
    writeFileSync(resolve(outputDirectory, "quartermaster-selection.png"), Buffer.from(second.data, "base64"));
  } finally {
    cdp.close();
  }

  for (const filename of ["quartermaster-editor.png", "quartermaster-selection.png"]) {
    const path = resolve(outputDirectory, filename);
    console.log(`Captured ${path} (${readFileSync(path).length} bytes)`);
  }
} finally {
  chrome.kill();
  vite.kill();
}
