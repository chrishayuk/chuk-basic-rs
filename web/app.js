// The playground page: editor, transcript, input line.
//
// The interesting parts are elsewhere: wasi.js is the WASI host and
// session.js runs programs and implements the re-run trick for INPUT.

import { DEFAULT_MAX_STEPS, Session, randomSeed, runProgram } from "./session.js";

const WASM_URL = new URL("./chuk-basic.wasm", import.meta.url);
const EXAMPLES_URL = new URL("./examples/", import.meta.url);

const $ = (id) => document.getElementById(id);
const editor = $("editor");
const examples = $("examples");
const seedBox = $("seed");
const capsBox = $("caps");
const runButton = $("run");
const stopButton = $("stop");
const statusLine = $("status");
const output = $("output");
const inputForm = $("input-form");
const inputBox = $("input");
const sendButton = $("send");

class Stopped extends Error {}

/**
 * Runs programs in a worker, so the page stays alive and Stop can work.
 * Falls back to running in the page if the worker cannot be started.
 */
function createRunner(module) {
  let worker = null;
  let pending = null;
  let nextId = 1;
  let useWorker = typeof Worker === "function";

  const runLocal = (request) =>
    new Promise((resolve, reject) => {
      // let the browser paint "Running..." first
      setTimeout(() => {
        try {
          resolve(runProgram(module, request));
        } catch (error) {
          reject(error);
        }
      }, 0);
    });

  function fallBack() {
    useWorker = false;
    if (worker) worker.terminate();
    worker = null;
    if (pending) {
      const { request, resolve, reject } = pending;
      pending = null;
      runLocal(request).then(resolve, reject);
    }
  }

  function spawn() {
    const w = new Worker(new URL("./worker.js", import.meta.url), { type: "module" });
    w.onmessage = (event) => {
      if (!pending || event.data.id !== pending.id) return;
      const { resolve, reject } = pending;
      pending = null;
      if (event.data.error) reject(new Error(event.data.error));
      else resolve(event.data.result);
    };
    w.onerror = (event) => {
      event.preventDefault();
      console.warn("worker failed; running programs in the page instead", event.message || "");
      fallBack();
    };
    w.postMessage({ module });
    return w;
  }

  return {
    run(request) {
      if (useWorker && worker === null) {
        try {
          worker = spawn();
        } catch (error) {
          console.warn("no module workers; running programs in the page instead", error);
          useWorker = false;
        }
      }
      if (!useWorker) return runLocal(request);
      return new Promise((resolve, reject) => {
        pending = { id: nextId++, request, resolve, reject };
        worker.postMessage({ id: pending.id, request });
      });
    },
    /** Abandon the run in progress, if any. */
    stop() {
      if (pending) {
        // the only way to stop running wasm is to end its thread
        worker.terminate();
        worker = null;
        pending.reject(new Stopped());
        pending = null;
      }
    },
  };
}

let runner = null;
let session = null;
let loadedExample = "";
/** The arithmetic each example was written for (file -> dialect). */
const dialects = new Map();
/** The arithmetic to run the program in the editor with. */
let dialect = "microsoft";
let runningTimer = 0;

function setStatus(text, kind = "") {
  statusLine.textContent = text;
  statusLine.className = kind;
}

function setControls() {
  const ready = runner !== null;
  const waiting = session !== null && session.waiting;
  const busy = session !== null && session.busy;
  runButton.disabled = !ready;
  stopButton.disabled = !(waiting || busy);
  inputBox.disabled = !waiting;
  sendButton.disabled = !waiting;
  inputBox.placeholder = waiting ? "Type an answer" : "";
}

function span(className, text) {
  const element = document.createElement("span");
  element.className = className;
  element.textContent = text;
  return element;
}

/** Draw the transcript, then whatever comes after it. */
function renderTranscript(segments, ...after) {
  const nodes = segments.map((segment) =>
    segment.type === "in" ? span("typed", segment.text) : document.createTextNode(segment.text),
  );
  const endsMidLine = segments.length > 0 && !segments[segments.length - 1].text.endsWith("\n");
  // messages start on a line of their own; the cursor sits after the prompt
  if (endsMidLine && after.length > 0 && after[0].className !== "cursor") {
    nodes.push(document.createTextNode("\n"));
  }
  output.replaceChildren(...nodes, ...after);
  output.scrollTop = output.scrollHeight;
  output.scrollLeft = 0;
}

function seedNote(s) {
  return `seed ${s.seed}`;
}

function showResult(s) {
  const result = s.result;
  const steps = (s.maxSteps ?? DEFAULT_MAX_STEPS).toLocaleString("en-US");
  switch (result.status) {
    case "waiting":
      renderTranscript(result.segments, span("cursor", ""));
      setStatus(`Waiting for input · ${seedNote(s)}`, "waiting");
      break;
    case "done":
      renderTranscript(result.segments);
      setStatus(`Finished · ${seedNote(s)}`);
      break;
    case "step-limit":
      renderTranscript(
        result.segments,
        span("error", `Stopped after ${steps} steps - is the program in an endless loop?\n`),
        span("note", `(${result.message})\n`),
      );
      setStatus("Stopped: step limit", "error");
      break;
    case "output-limit":
      renderTranscript(
        result.segments,
        span("error", "Stopped: the program printed too much - is it in an endless loop?\n"),
      );
      setStatus("Stopped: output limit", "error");
      break;
    case "crash":
      renderTranscript(result.segments, span("error", `The interpreter crashed: ${result.message}\n`));
      setStatus("Crashed", "error");
      break;
    default:
      renderTranscript(result.segments, span("error", `${result.message || "The program failed"}\n`));
      setStatus(`Error · ${seedNote(s)}`, "error");
  }
}

/** Run one step of a session (its start, or a line of input) and show it. */
async function drive(s, step) {
  clearTimeout(runningTimer);
  runningTimer = setTimeout(() => setStatus("Running..."), 150);
  const promise = step();
  setControls();
  try {
    await promise;
  } catch (error) {
    if (s !== session) return; // stopped, or replaced by a newer run
    clearTimeout(runningTimer);
    session = null;
    renderTranscript([], span("error", `Could not run the program: ${error.message}\n`));
    setStatus("Failed", "error");
    setControls();
    return;
  }
  if (s !== session) return;
  clearTimeout(runningTimer);
  showResult(s);
  setControls();
  if (s.waiting) inputBox.focus({ preventScroll: true });
}

function run() {
  if (runner === null) return;
  const seedText = seedBox.value.trim();
  if (seedText !== "" && !/^\d{1,15}$/.test(seedText)) {
    setStatus("The seed must be a whole number", "error");
    seedBox.focus();
    return;
  }
  runner.stop();
  inputBox.value = "";
  // a new session: no input yet, and a new seed unless one was asked for
  session = new Session(runner.run, {
    source: editor.value,
    seed: seedText === "" ? randomSeed() : seedText,
    dialect,
  });
  renderTranscript([]);
  const s = session;
  drive(s, () => s.start());
}

function stop() {
  if (session === null) return;
  clearTimeout(runningTimer);
  runner.stop();
  const shown = session.result ? session.result.segments.slice() : [];
  if (session.busy && session.result) {
    // the line that was being run when Stop was pressed
    for (const line of session.inputs.slice(session.result.linesRead)) {
      shown.push({ type: "in", text: line + "\n" });
    }
  }
  session = null;
  renderTranscript(shown, span("note", "Stopped.\n"));
  setStatus("Stopped");
  setControls();
}

function submitLine(event) {
  event.preventDefault();
  if (session === null || !session.waiting) return;
  const line = capsBox.checked ? inputBox.value.toUpperCase() : inputBox.value;
  inputBox.value = "";
  // show the line at once; the re-run will draw the same thing and carry on
  renderTranscript([...session.result.segments, { type: "in", text: line + "\n" }]);
  const s = session;
  drive(s, () => s.submit(line));
}

async function fetchText(url) {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`${response.status} ${response.statusText}`);
  return response.text();
}

async function loadExample(file) {
  try {
    const text = await fetchText(new URL(file, EXAMPLES_URL));
    if (examples.value !== file) return; // another one was chosen meanwhile
    if (session !== null) stop();
    editor.value = text;
    editor.scrollTop = 0;
    loadedExample = text;
    // the 1964 manual's programs expect Dartmouth's arithmetic
    dialect = dialects.get(file) || "microsoft";
    renderTranscript([]);
    if (runner !== null) setStatus("Ready - press Run");
  } catch (error) {
    setStatus(`Could not load ${file}: ${error.message}`, "error");
  }
}

async function loadExamples() {
  try {
    const list = JSON.parse(await fetchText(new URL("index.json", EXAMPLES_URL)));
    const groups = new Map();
    for (const example of list) {
      const option = new Option(example.title, example.file);
      dialects.set(example.file, example.dialect);
      const name = example.group || "Examples";
      if (!groups.has(name)) {
        const group = document.createElement("optgroup");
        group.label = name;
        groups.set(name, group);
      }
      groups.get(name).append(option);
    }
    examples.replaceChildren(...groups.values());
    const wanted = new URLSearchParams(location.search).get("example");
    const chosen = list.find((example) => example.file.replace(/\.bas$/, "") === wanted);
    if (chosen) {
      examples.value = chosen.file;
      await loadExample(chosen.file);
    }
    if (list.length > 0 && editor.value === "") await loadExample(list[0].file);
  } catch (error) {
    examples.replaceChildren(new Option("(no examples)", ""));
    console.warn("could not load the examples", error);
  }
}

async function loadInterpreter() {
  try {
    let module;
    try {
      module = await WebAssembly.compileStreaming(fetch(WASM_URL));
    } catch {
      // a server that does not send application/wasm
      const response = await fetch(WASM_URL);
      if (!response.ok) throw new Error(`${response.status} ${response.statusText}`);
      module = await WebAssembly.compile(await response.arrayBuffer());
    }
    runner = createRunner(module);
    setStatus("Ready - press Run");
  } catch (error) {
    setStatus(`Could not load chuk-basic.wasm (${error.message}). Run web/build.sh.`, "error");
  }
  setControls();
}

let previousExample = "";
examples.addEventListener("focus", () => (previousExample = examples.value));
examples.addEventListener("change", () => {
  if (examples.value === "") return;
  if (editor.value !== loadedExample && editor.value.trim() !== "") {
    if (!window.confirm("Replace the program in the editor? Your changes will be lost.")) {
      examples.value = previousExample;
      return;
    }
  }
  previousExample = examples.value;
  loadExample(examples.value);
});
runButton.addEventListener("click", run);
stopButton.addEventListener("click", stop);
inputForm.addEventListener("submit", submitLine);
document.addEventListener("keydown", (event) => {
  if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
    event.preventDefault();
    run();
  }
});
// clicking the transcript puts the cursor in the input line, as in a terminal
output.addEventListener("click", () => {
  if (!inputBox.disabled && String(window.getSelection()) === "") {
    inputBox.focus({ preventScroll: true });
  }
});

setControls();
loadExamples();
loadInterpreter();
