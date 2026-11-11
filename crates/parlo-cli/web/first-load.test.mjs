import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import test from "node:test";
import vm from "node:vm";

const here = dirname(fileURLToPath(import.meta.url));
const island = readFileSync(join(here, "island.html"), "utf8").replace(/\r\n/g, "\n");
const locales = JSON.parse(readFileSync(join(here, "locales.json"), "utf8"));
const app = readFileSync(join(here, "app.js"), "utf8").replace(/\r\n/g, "\n");

const strings = Object.values(locales).find(
  (value) =>
    value &&
    typeof value === "object" &&
    !Array.isArray(value) &&
    Object.values(value).every(
      (item) => item && typeof item === "object" && !Array.isArray(item),
    ),
);

function sourceOf(name) {
  const plain = island.indexOf(`\nfunction ${name}(`);
  const asynchronous = island.indexOf(`\nasync function ${name}(`);
  const start = plain >= 0 ? plain : asynchronous;
  assert.ok(start >= 0, `the island defines ${name}`);
  const end = island.indexOf("\n}\n", start);
  assert.ok(end > start, `${name} closes`);
  return island.slice(start + 1, end + 3);
}

test("the language menu grows away from the island and stays inside the work area", async () => {
  for (const [y, height, expectedY, down] of [[700, 132, 460, false], [10, 132, 10, true], [250, 132, 10, false], [0, 132, 0, true], [0, 132, 0, false]]) {
    let position = { x: -1500, y };
    let size = { width: 560, height };
    const classes = new Set();
    const box = {
      ISLAND_HEIGHT: 132,
      ISLAND_OPEN_HEIGHT: 372,
      document: { body: { classList: {
        contains: (name) => classes.has(name),
        toggle: (name, value) => value ? classes.add(name) : classes.delete(name),
        remove: (name) => classes.delete(name),
      } } },
      window: { __TAURI__: {
        dpi: {
          PhysicalSize: class { constructor(width, height) { this.width = width; this.height = height; } },
          PhysicalPosition: class { constructor(x, top) { this.x = x; this.y = top; } },
        },
        window: { monitorFromPoint: async () => ({ workArea: { position: { x: -1920, y: 0 }, size: { width: 1920, height: y === 0 && !down ? 132 : 900 } } }) },
      } },
      tauriWindow: () => ({
        scaleFactor: async () => 1,
        outerSize: async () => size,
        outerPosition: async () => position,
        setSize: async (value) => { size = value; },
        setPosition: async (value) => { position = value; },
      }),
    };
    vm.runInNewContext(`${sourceOf("setIslandHeight")}\nthis.resize = setIslandHeight;`, box);
    await box.resize(true);
    assert.equal(position.y, expectedY);
    assert.equal(classes.has("menu-down"), down);
    assert.ok(position.y >= 0 && position.y + size.height <= (y === 0 && !down ? 132 : 900));
    await box.resize(false);
    assert.equal(position.y, y);
    assert.equal(size.height, 132);
    assert.equal(classes.has("menu-down"), false);
  }
});

test("rapid menu toggles are resized in order and the final state wins", async () => {
  const calls = [];
  let release;
  const wait = new Promise((resolve) => { release = resolve; });
  const box = {
    writeMenu: { hidden: true, querySelector: () => null, firstElementChild: null },
    writeEl: { setAttribute: (name, value) => calls.push([name, value]) },
    keepIsland() {},
    resizeIsland: async (open) => { calls.push(["resize", open]); if (open) await wait; },
  };
  vm.runInNewContext(`let menuResize = Promise.resolve();\n${sourceOf("openWriteMenu")}\nthis.open = openWriteMenu;`, box);
  box.open(true);
  await Promise.resolve();
  const done = box.open(false);
  release();
  await done;
  assert.equal(box.writeMenu.hidden, true);
  assert.deepEqual(calls.filter((entry) => entry[0] === "resize"), [["resize", true], ["resize", false]]);
});

test("recovery and menu resize requests are serialized even after failure", async () => {
  let release;
  const calls = [];
  const box = {
    setIslandHeight: async (open) => {
      calls.push(open);
      if (calls.length === 1) await new Promise((resolve) => { release = resolve; });
      if (calls.length === 2) throw new Error("resize failed");
    },
  };
  vm.runInNewContext(`let islandResize = Promise.resolve();\n${sourceOf("resizeIsland")}\nthis.resize = resizeIsland;`, box);
  const first = box.resize(true);
  await Promise.resolve();
  await Promise.resolve();
  const second = box.resize(false);
  const third = box.resize(true);
  release();
  await first;
  await assert.rejects(second, /resize failed/);
  await third;
  assert.deepEqual(calls, [true, false, true]);
});

test("disabling a shortcut keeps the displayed value if saving fails", async () => {
  const start = app.indexOf("async function saveShortcut(");
  const end = app.indexOf("\n}\n", start);
  assert.ok(start >= 0 && end > start);
  const input = { value: "Ctrl+Alt+F10" };
  const box = { token: "test", shortcutsNote: {}, fetch: async () => ({ ok: false, json: async () => ({ error: "Save failed" }) }) };
  vm.runInNewContext(`${app.slice(start, end + 3)}\nthis.save = saveShortcut;`, box);
  await box.save("copy_shortcut", input, "Copy shortcut disabled.", "");
  assert.equal(input.value, "Ctrl+Alt+F10");
  assert.equal(box.shortcutsNote.textContent, "Save failed");
  box.fetch = async () => ({ ok: true, json: async () => ({ copy_shortcut: "" }) });
  await box.save("copy_shortcut", input, "Copy shortcut disabled.", "");
  assert.equal(input.value, "");
});

test("the island carries the note hidden and keyed to a translation", () => {
  const tag = island.match(/<span[^>]*id="hint"[^>]*>/);
  assert.ok(tag, "the hint element exists");
  assert.match(tag[0], /role="status"/);
  assert.match(tag[0], /aria-live="polite"/);
  assert.match(tag[0], /data-i18n="island\.firstLoad"/);
  assert.match(tag[0], /hidden/);
});

test("the note is translated in Portuguese and English", () => {
  const english = strings.en["island.firstLoad"];
  const portuguese = strings.pt["island.firstLoad"];
  assert.equal(typeof english, "string");
  assert.equal(typeof portuguese, "string");
  assert.ok(english.length > 20 && portuguese.length > 20);
  assert.notEqual(portuguese, english);
});

test("applyText paints the active language into the note", () => {
  for (const language of ["pt", "en"]) {
    const note = {
      dataset: { i18n: "island.firstLoad" },
      textContent: "fallback",
      setAttribute() {},
    };
    const box = {
      strings: strings[language],
      document: {
        title: "",
        querySelectorAll: (selector) => (selector === "[data-i18n]" ? [note] : []),
      },
    };
    vm.runInNewContext(
      `${sourceOf("t")}\n${sourceOf("applyText")}\nthis.paint = applyText;`,
      box,
    );
    box.paint();
    assert.equal(note.textContent, strings[language]["island.firstLoad"], language);
  }
});

test("the note shows during the first dictation and stays hidden afterwards", () => {
  const box = {};
  vm.runInNewContext(`${sourceOf("paintFirstLoad")}\nthis.paint = paintFirstLoad;`, box);
  const hint = { hidden: true };
  box.paint(hint, false);
  assert.equal(hint.hidden, false);
  box.paint(hint, true);
  assert.equal(hint.hidden, true);
  box.paint(hint, true);
  assert.equal(hint.hidden, true);
  box.paint(null, false);
});

test("a refused write is reported instead of claiming it was written", () => {
  const stateEl = { textContent: "" };
  const lineEl = { textContent: "", classList: { toggle() {} } };
  const box = {
    strings: strings.pt,
    document: { body: { classList: { remove() {} } } },
    stateEl,
    lineEl,
    lastText: "hello",
    showRecovery() {},
    report() {},
  };
  vm.runInNewContext(
    `${sourceOf("t")}\n${sourceOf("setState")}\n${sourceOf("noteWriteFailure")}\nthis.note = noteWriteFailure;`,
    box,
  );
  box.note("the field refused the edit");
  assert.equal(box.stateEl.textContent, strings.pt["island.failed"]);
  assert.equal(box.lineEl.textContent, "the field refused the edit");
  box.note("");
  assert.equal(box.lineEl.textContent, strings.pt["island.dictationFailed"]);
  assert.match(sourceOf("stop"), /showRecovery\(text, false\)/);
});

test("the island paints the note when a dictation begins and when the first result lands", () => {
  assert.match(island, /paintFirstLoad\(hintEl, warmed\)/);
  assert.match(island, /paintFirstLoad\(hintEl, true\)/);
});

function stopHarness(overrides = {}) {
  const calls = [];
  const track = { stopped: false, stop() { this.stopped = true; } };
  const processor = { onaudioprocess() {} };
  const box = {
    busy: false,
    promoted: true,
    starting: false,
    commandMode: false,
    commandSelection: "",
    armed: false,
    session: 1,
    stream: { getTracks: () => [track] },
    processor,
    partials: null,
    partialTicket: 0,
    partialRunning: false,
    transcriptComplete: true,
    streamFinal: "final words",
    finalStale: false,
    lastText: "partial words",
    sentChunks: 1,
    chunks: [new Float32Array([1])],
    paused: false,
    tidy: false,
    queuePolishSegment() {},
    segmentQueue: Promise.resolve(),
    recoveryQueue: Promise.resolve(true),
    segmentController: null,
    segmentStart: 0,
    segmentUnsent: 0,
    segmentFailure: null,
    segmentWarning: "",
    segmentsWritten: 1,
    segmentText: "First.",
    writeSource: "en",
    token: "token",
    controller: null,
    warmed: false,
    hintEl: {},
    liveBase: "",
    liveConfirmed: "",
    liveGuess: "",
    document: { body: { classList: { remove() {} } } },
    performance,
    AbortController,
    clearInterval() {},
    t: (_key, fallback) => fallback,
    setState: (...args) => calls.push(["state", ...args]),
    report: (state) => calls.push(["report", state]),
    partialSettled: async () => true,
    release: () => ({ wav: "complete wav", seconds: 1 }),
    teardown() {},
    ticker: null,
    effectiveWrite: () => "same",
    paintFirstLoad() {},
    hideAfterDelay() {},
    ready() {},
    showRecovery: (...args) => calls.push(["recovery", ...args]),
    noteWriteFailure: (reason) => calls.push(["write-failed", reason]),
    writeIntoField: async (...args) => { calls.push(["insert", ...args]); return { written: true }; },
    fetch: async (url, options) => {
      calls.push(["fetch", url, options]);
      return { ok: true, json: async () => ({ text: "complete words" }) };
    },
    ...overrides,
  };
  vm.runInNewContext(`${sourceOf("writeFinal")}\n${sourceOf("stop")}\nthis.stop = stop;`, box);
  return { box, calls, track, processor };
}

test("a batch keeps every final segment and the following partial", () => {
  const box = {};
  vm.runInNewContext(
    `${sourceOf("combineWords")}\n${sourceOf("wordsOf")}\n${sourceOf("readLive")}\nthis.readLive = readLive;`,
    box,
  );
  const next = box.readLive("earlier", "", "", [
    { state: "corrected", text: "first corrected" },
    { state: "final", text: "first corrected" },
    { state: "final", text: "second" },
    { state: "partial", text: "third begins" },
  ]);
  assert.equal(next.base, "earlier first corrected second");
  assert.equal(next.text, "earlier first corrected second third begins");
  assert.equal(next.guess, "third begins");
});

test("release stops capture before waiting and reprocesses an unsent tail", async () => {
  const { box, calls, track, processor } = stopHarness({
    chunks: [new Float32Array([1]), new Float32Array([2])],
  });
  await box.stop();
  assert.equal(track.stopped, true);
  assert.equal(processor.onaudioprocess, null);
  const request = calls.find((entry) => entry[0] === "fetch" && entry[1] === "/dictate");
  assert.equal(request[2].body, "complete wav");
  assert.deepEqual(calls.filter((entry) => entry[0] === "insert"), [["insert", "complete words", undefined, "", undefined, false]]);
  assert.deepEqual(calls.find((entry) => entry[0] === "recovery"), ["recovery", "complete words", false]);
});

test("an unset write choice sends the spoken language on final audio", async () => {
  const { box, calls } = stopHarness({
    effectiveWrite: () => "",
    partialSettled: async () => false,
  });
  await box.stop();
  assert.equal(calls.find((entry) => entry[0] === "fetch")[2].headers["x-parlo-write"], "en");
});

test("a timed-out partial reprocesses all audio without starting an overlapping stream", async () => {
  const { box, calls } = stopHarness({ partialSettled: async () => false });
  await box.stop();
  assert.equal(calls.find((entry) => entry[0] === "fetch")[2].body, "complete wav");
  assert.equal(calls.filter((entry) => entry[0] === "fetch").length, 1);
});

test("polish finishes queued sentences and writes the final text once", async () => {
  const { box, calls } = stopHarness({ tidy: true });
  await box.stop();
  assert.deepEqual(calls.filter((entry) => entry[0] === "insert"), [["insert", "First.", undefined, "", 0, false]]);
  assert.equal(calls.some((entry) => entry[0] === "fetch"), false);
  assert.deepEqual(calls.find((entry) => entry[0] === "recovery"), ["recovery", "First.", false]);
  assert.equal(calls.some((entry) => entry[0] === "report" && entry[1] === "success"), false);
});

test("silence adapts to steady noise and closes only after speech", () => {
  const box = {};
  vm.runInNewContext(`${sourceOf("makeSilenceDetector")}\nthis.make = makeSilenceDetector;`, box);
  const detector = box.make();
  for (let i = 0; i < 20; i += 1) assert.equal(detector(0.015, 50), false);
  for (let i = 0; i < 8; i += 1) assert.equal(detector(0.06, 50), false);
  for (let i = 0; i < 15; i += 1) assert.equal(detector(0.015, 50), false);
  assert.equal(detector(0.015, 50), true);
  for (let i = 0; i < 20; i += 1) assert.equal(detector(0.015, 50), false);
});

test("silence detection can learn a quieter background while speech is active", () => {
  const box = {};
  vm.runInNewContext(`${sourceOf("makeSilenceDetector")}\nthis.make = makeSilenceDetector;`, box);
  const detector = box.make();
  for (let i = 0; i < 20; i += 1) detector(0.02, 50);
  for (let i = 0; i < 8; i += 1) detector(0.06, 50);
  for (let i = 0; i < 20; i += 1) detector(0.001, 50);
  let closed = false;
  for (let i = 0; i < 40; i += 1) closed = detector(0.015, 50) || closed;
  assert.equal(closed, false);
});

test("speech at the start is not learned as background noise", () => {
  const box = {};
  vm.runInNewContext(`${sourceOf("makeSilenceDetector")}\nthis.make = makeSilenceDetector;`, box);
  const detector = box.make();
  for (let i = 0; i < 80; i += 1) assert.equal(detector(0.03, 50), false);
  for (let i = 0; i < 15; i += 1) assert.equal(detector(0.008, 50), false);
  assert.equal(detector(0.008, 50), true);
});

test("continuous speech is not learned as background noise", () => {
  const box = {};
  vm.runInNewContext(`${sourceOf("makeSilenceDetector")}\nthis.make = makeSilenceDetector;`, box);
  const detector = box.make();
  for (let i = 0; i < 20; i += 1) detector(0.01, 50);
  for (let i = 0; i < 200; i += 1) assert.equal(detector(0.06, 50), false);
  for (let i = 0; i < 15; i += 1) assert.equal(detector(0.01, 50), false);
  assert.equal(detector(0.01, 50), true);
});

test("polished segments are processed and kept in order without insertion", async () => {
  const requests = [];
  const inserts = [];
  const box = {
    context: { sampleRate: 48000 },
    chunks: [new Float32Array([1]), new Float32Array([2])],
    segmentStart: 0,
    segmentSamples: 2,
    segmentFailure: null,
    segmentQueue: Promise.resolve(),
    segmentPending: 0,
    MAX_POLISH_PENDING: 8,
    segmentUnsent: 0,
    segmentsWritten: 0,
    segmentText: "",
    segmentWarning: "",
    segmentController: new AbortController(),
    saveRecovery() {},
    session: 1,
    token: "test",
    writeSource: "en",
    effectiveWrite: () => "",
    recording: (pieces) => ({ wav: pieces[0][0], seconds: 1 }),
    fetch: async (_url, options) => {
      requests.push(options);
      return { ok: true, json: async () => ({ text: options.body === 1 ? "First." : "Second." }) };
    },
    writeIntoField: async (text) => { inserts.push(text); return { written: true }; },
    warmed: false,
    hintEl: {},
    busy: false,
    paintFirstLoad() {},
    setState() {},
    t: (_key, fallback) => fallback,
  };
  vm.runInNewContext(`${sourceOf("queuePolishSegment")}\nthis.queue = queuePolishSegment;`, box);
  box.queue(1);
  box.queue(1);
  await box.segmentQueue;
  assert.deepEqual(inserts, []);
  assert.equal(box.segmentText, "First. Second.");
  assert.equal(requests[0].headers["x-parlo-clean"], "on");
  assert.equal(requests[0].headers["x-parlo-write"], "en");
  assert.equal(box.segmentPending, 0);
});

test("a silent final tail does not stop the following polished sentence", async () => {
  const inserts = [];
  const box = {
    context: { sampleRate: 48000 },
    chunks: [new Float32Array([1]), new Float32Array([2])],
    segmentStart: 0,
    segmentSamples: 2,
    segmentFailure: null,
    segmentQueue: Promise.resolve(),
    segmentPending: 0,
    MAX_POLISH_PENDING: 8,
    segmentUnsent: 0,
    segmentsWritten: 0,
    segmentText: "",
    segmentWarning: "",
    segmentController: new AbortController(),
    saveRecovery() {},
    session: 1,
    token: "test",
    writeSource: "en",
    effectiveWrite: () => "same",
    recording: (pieces) => ({ wav: pieces[0][0], seconds: 1 }),
    fetch: async (_url, options) => ({ ok: true, json: async () => ({ text: options.body === 1 ? "" : "Words." }) }),
    writeIntoField: async (text) => { inserts.push(text); return { written: true }; },
    warmed: false,
    hintEl: {},
    busy: false,
    paintFirstLoad() {},
    setState() {},
    t: (_key, fallback) => fallback,
  };
  vm.runInNewContext(`${sourceOf("queuePolishSegment")}\nthis.queue = queuePolishSegment;`, box);
  box.queue(1, true);
  box.queue(1);
  await box.segmentQueue;
  assert.deepEqual(inserts, []);
  assert.equal(box.segmentText, "Words.");
  assert.equal(box.segmentFailure, null);
});

test("release queues the last polished sentence and waits for it", async () => {
  const calls = [];
  const { box } = stopHarness({
    tidy: true,
    queuePolishSegment: (end, trailing) => calls.push([end, trailing]),
    segmentStart: 0,
  });
  await box.stop();
  assert.deepEqual(calls, [[1, true]]);
});

test("a full polish queue stops recording without discarding captured audio", () => {
  let stopped = false;
  const audio = new Float32Array([1]);
  const box = {
    context: { sampleRate: 48000 },
    chunks: [audio],
    segmentFailure: null,
    segmentPending: 8,
    MAX_POLISH_PENDING: 8,
    busy: false,
    stop() { stopped = true; },
  };
  vm.runInNewContext(`${sourceOf("queuePolishSegment")}\nthis.queue = queuePolishSegment;`, box);
  box.queue(1);
  assert.equal(stopped, true);
  assert.equal(box.chunks[0], audio);
  assert.equal(box.segmentFailure, null);
});

test("a failed transcription keeps earlier text and stops later work", async () => {
  const inserts = [];
  const box = {
    context: { sampleRate: 48000 },
    chunks: [new Float32Array([1]), new Float32Array([2])],
    segmentStart: 0,
    segmentSamples: 2,
    segmentFailure: null,
    segmentQueue: Promise.resolve(),
    segmentPending: 0,
    MAX_POLISH_PENDING: 8,
    segmentUnsent: 0,
    segmentsWritten: 0,
    segmentText: "",
    segmentWarning: "",
    segmentController: new AbortController(),
    saveRecovery() {},
    session: 1,
    token: "test",
    writeSource: "en",
    effectiveWrite: () => "same",
    recording: (pieces) => ({ wav: pieces[0][0], seconds: 1 }),
    fetch: async (_url, options) => options.body === 1
      ? { ok: true, json: async () => ({ text: "First." }) }
      : { ok: false, json: async () => ({ error: "Engine unavailable" }) },
    writeIntoField: async (text) => { inserts.push(text); return { written: false, reason: "Field refused" }; },
    warmed: false,
    hintEl: {},
    paintFirstLoad() {},
    setState() {},
    t: (_key, fallback) => fallback,
    busy: true,
  };
  vm.runInNewContext(`${sourceOf("queuePolishSegment")}\nthis.queue = queuePolishSegment;`, box);
  box.queue(1);
  box.queue(1);
  await box.segmentQueue;
  assert.deepEqual(inserts, []);
  assert.equal(box.segmentText, "First.");
  assert.equal(box.segmentFailure.message, "Engine unavailable");
  assert.equal(box.segmentPending, 0);
  assert.equal(box.segmentUnsent, 0);
});

test("partial text is display only and the final transcript is written once", async () => {
  const requested = [];
  const shown = [];
  const partialBox = {
    stream: {}, context: { sampleRate: 48000 }, paused: false, busy: false,
    partialRunning: false, transcriptComplete: true, streamFinal: "", finalStale: false,
    chunks: [new Float32Array([1])], sentChunks: 0, partialTicket: 0,
    liveBase: "", liveConfirmed: "", liveGuess: "", lastText: "", token: "token",
    localStorage: { getItem: () => "" }, WINDOW_SECONDS: 1, TARGET_RATE: 16000,
    encodeWav: () => "stream wav", downsample: (audio) => audio,
    saveRecovery() {}, setState: (...args) => shown.push(args), t: (_key, fallback) => fallback,
    fetch: async (url) => { requested.push(url); return { ok: true, json: async () => [{ state: "partial", text: "partial words" }] }; },
  };
  vm.runInNewContext(`${sourceOf("combineWords")}\n${sourceOf("wordsOf")}\n${sourceOf("readLive")}\n${sourceOf("sendPartial")}\nthis.sendPartial = sendPartial;`, partialBox);
  await partialBox.sendPartial(true);
  assert.equal(partialBox.lastText, "partial words");
  assert.deepEqual(shown.at(-1), ["Listening", "partial words", true]);
  assert.deepEqual(requested, ["/listen/stream"]);
  assert.doesNotMatch(island, /\blive:\s*true/);
  assert.doesNotMatch(island, /function writeLive/);

  const { box, calls } = stopHarness();
  await box.stop();
  assert.equal(box.lastText, "complete words");
  assert.deepEqual(calls.filter((entry) => entry[0] === "insert"), [["insert", "complete words", undefined, "", undefined, false]]);
  assert.deepEqual(calls.find((entry) => entry[0] === "recovery"), ["recovery", "complete words", false]);
  assert.equal(box.busy, false);
  assert.equal(calls.at(-1)[1], "ready-to-copy");
});

test("failed transcription offers the last partial text for recovery", async () => {
  const { box, calls } = stopHarness({
    transcriptComplete: false,
    fetch: async () => ({ ok: false, status: 500, json: async () => ({ error: "Engine unavailable" }) }),
  });
  await box.stop();
  assert.deepEqual(calls.find((entry) => entry[0] === "recovery"), ["recovery", "partial words", true]);
  assert.equal(calls.some((entry) => entry[0] === "insert"), false);
});

test("an HTTP success without an insertion confirmation is not reported as written", async () => {
  for (const [reply, expected] of [
    [{ ok: true, json: async () => ({}) }, false],
    [{ ok: true, json: async () => ({ inserted: false }) }, false],
    [{ ok: true, json: async () => ({ inserted: true }) }, true],
    [{ ok: false, json: async () => ({ error: "Field changed" }) }, false],
  ]) {
    const box = { token: "test", fetch: async () => reply };
    vm.runInNewContext(`${sourceOf("writeIntoField")}\nthis.write = writeIntoField;`, box);
    const outcome = await box.write("words", undefined, "", 0, false);
    assert.equal(outcome.written, expected);
    if (!expected) assert.ok(outcome.reason);
  }
});

test("a missing final capture keeps the latest recognized partial", async () => {
  const { box, calls } = stopHarness({ release: () => null });
  await box.stop();
  assert.deepEqual(calls.find((entry) => entry[0] === "recovery"), ["recovery", "partial words", true]);
  assert.equal(calls.some((entry) => entry[0] === "report" && entry[1] === "success"), false);
});

test("cancel keeps completed segments visible for copying", async () => {
  const calls = [];
  const box = {
    stream: { getTracks: () => [] }, busy: false, controller: null,
    segmentPending: 0, segmentsWritten: 1, segmentText: "First sentence.", lastText: "First sentence.", recoveryText: "", recoveryQueue: Promise.resolve(true),
    processor: null, segmentController: null,
    liveBase: "", liveConfirmed: "", liveGuess: "", session: 1, attempt: 0,
    document: { body: { classList: { remove() {} } } },
    discard() {}, ready() {},
    showRecovery: (...args) => calls.push(["recovery", ...args]),
    setState: (...args) => calls.push(["state", ...args]),
    report: (state) => calls.push(["report", state]),
    t: (_key, fallback) => fallback,
  };
  vm.runInNewContext(`${sourceOf("cancel")}\nthis.cancel = cancel;`, box);
  await box.cancel();
  assert.deepEqual(calls.find((entry) => entry[0] === "recovery"), ["recovery", "First sentence.", true]);
  assert.deepEqual(calls.at(-1), ["report", "error"]);
});

test("an uncopied result prevents a new recording and stays visible", async () => {
  const messages = [];
  const classes = new Set(["ready"]);
  const box = {
    recoveryText: "Your sentence",
    recoveryCopied: false,
    recoveryReady: true,
    copyRecoveryEl: { disabled: false },
    busy: false,
    promoted: false,
    stream: null,
    document: { body: { classList: {
      add: (name) => classes.add(name),
      remove: (...names) => names.forEach((name) => classes.delete(name)),
    } } },
    setState: (...args) => messages.push(args),
    discard() { throw new Error("the result was discarded"); },
  };
  vm.runInNewContext(`${sourceOf("ready")}\n${sourceOf("arm")}\nthis.ready = ready; this.arm = arm;`, box);
  box.ready();
  box.arm();
  assert.equal(classes.has("recovery"), true);
  assert.match(messages[0][1], /Copy the result/);
  assert.equal(box.recoveryText, "Your sentence");
});

test("cancelled partial text survives dismissal and is copied only on confirmation", async () => {
  const classes = new Set(["listening"]);
  const states = [];
  const requests = [];
  let reply = { ok: false, json: async () => ({ error: "Clipboard unavailable" }) };
  const box = {
    stream: { getTracks: () => [] }, busy: false, starting: false, controller: null,
    segmentPending: 0, segmentsWritten: 0, segmentText: "", lastText: "partial words",
    recoveryText: "", recoveryCopied: false, recoveryPartial: false,
    recoveryTextEl: {}, copyRecoveryEl: { disabled: false }, clearRecoveryEl: { dataset: {} },
    resizeIsland: async () => {}, saveRecovery() {}, recoveryQueue: Promise.resolve(true), starting: false,
    processor: null, segmentController: null,
    liveBase: "", liveConfirmed: "", liveGuess: "", session: 1, attempt: 0,
    token: "token", document: { body: { classList: {
      remove: (...names) => names.forEach((name) => classes.delete(name)),
      add: (name) => classes.add(name),
      toggle: (name, on) => on ? classes.add(name) : classes.delete(name),
    } } },
    discard() { box.stream = null; }, ready() {}, keepIsland() {}, openWriteMenu() {},
    hideIsland() { states.push(["hidden"]); },
    setState: (...args) => states.push(args), report() {}, t: (_key, fallback) => fallback,
    fetch: async (url, options) => { requests.push([url, JSON.parse(options.body)]); return reply; },
  };
  vm.runInNewContext(`${sourceOf("showRecovery")}\n${sourceOf("copyRecovery")}\n${sourceOf("cancel")}\n${sourceOf("dismissIsland")}\nthis.dismiss = dismissIsland; this.copy = copyRecovery;`, box);
  await box.dismiss();
  assert.equal(box.recoveryTextEl.textContent, "partial words");
  assert.equal(box.recoveryPartial, true);
  assert.equal(box.recoveryCopied, false);
  assert.equal(classes.has("recovery"), true);
  await box.copy();
  assert.equal(box.recoveryCopied, false);
  assert.equal(box.copyRecoveryEl.disabled, false);
  assert.equal(states.at(-1)[1], "Clipboard unavailable");
  reply = { ok: true, json: async () => ({ copied: false }) };
  await box.copy();
  assert.equal(box.recoveryCopied, false);
  reply = { ok: true, json: async () => { throw new Error("invalid response"); } };
  await box.copy();
  assert.equal(box.recoveryCopied, false);
  reply = { ok: true, json: async () => ({ copied: true }) };
  await box.copy();
  assert.equal(box.recoveryCopied, true);
  assert.deepEqual(requests.at(-1), ["/clipboard", { text: "partial words", keep: true }]);
  assert.match(states.at(-1)[1], /Partial text copied/);
  reply = { ok: false, json: async () => ({ error: "Clipboard unavailable" }) };
  await box.copy();
  assert.equal(box.recoveryCopied, false);
  assert.equal(box.recoveryText, "partial words");
});

test("a clipboard network failure leaves the result pending", async () => {
  const states = [];
  const box = {
    recoveryText: "words", recoveryCopied: false, recoveryPartial: false,
    copyRecoveryEl: { disabled: false }, clearRecoveryEl: { dataset: {} }, starting: false, token: "token",
    fetch: async () => { throw new Error("disconnected"); },
    setState: (...args) => states.push(args), t: (_key, fallback) => fallback,
  };
  vm.runInNewContext(`${sourceOf("copyRecovery")}\nthis.copy = copyRecovery;`, box);
  await box.copy();
  assert.equal(box.recoveryCopied, false);
  assert.equal(box.recoveryText, "words");
  assert.equal(box.copyRecoveryEl.disabled, false);
  assert.equal(states.at(-1)[1], "disconnected");
});

test("a pending copy cannot be overwritten by a new recording", async () => {
  let finish;
  const box = {
    recoveryText: "saved words", recoveryCopied: true, recoveryPartial: false,
    copyRecoveryEl: { disabled: false }, clearRecoveryEl: { dataset: {} }, recoveryReady: true, token: "token", busy: false, starting: false,
    fetch: () => new Promise((resolve) => { finish = resolve; }),
    setState() {}, t: (_key, fallback) => fallback,
  };
  vm.runInNewContext(`${sourceOf("copyRecovery")}\n${sourceOf("begin")}\nthis.copy = copyRecovery; this.begin = begin;`, box);
  const copying = box.copy();
  await box.begin();
  assert.equal(box.recoveryText, "saved words");
  assert.equal(box.copyRecoveryEl.disabled, true);
  finish({ ok: true, json: async () => ({ copied: true }) });
  await copying;
  assert.equal(box.copyRecoveryEl.disabled, false);
});

test("selecting the recovery text does not start dragging the window", () => {
  let dragged = false;
  const box = { window: { __TAURI__: { window: { getCurrentWindow: () => ({ startDragging: () => { dragged = true; } }) } } } };
  vm.runInNewContext(`${sourceOf("startDragging")}\nthis.drag = startDragging;`, box);
  box.drag({ button: 0, target: { closest: (selector) => selector.includes(".recovery-text") } });
  assert.equal(dragged, false);
});

test("saved recovery loads before recording and retains its full text", async () => {
  const classes = new Set(["ready"]);
  const messages = [];
  const box = {
    recoveryText: "", recoveryPartial: false, recoveryCopied: false, recoveryReady: false,
    recoveryTextEl: {}, copyRecoveryEl: {}, clearRecoveryEl: { dataset: {} }, token: "token",
    document: { body: { classList: {
      remove: (name) => classes.delete(name), add: (name) => classes.add(name),
    } } },
    resizeIsland: async () => {}, setState: (...args) => messages.push(args), report: (state) => messages.push(["report", state]),
    fetch: async () => ({ ok: true, json: async () => ({ text: "First\nSecond", partial: true }) }),
  };
  vm.runInNewContext(`${sourceOf("loadRecovery")}\nthis.load = loadRecovery;`, box);
  await box.load();
  assert.equal(box.recoveryTextEl.textContent, "First\nSecond");
  assert.equal(box.recoveryReady, true);
  assert.equal(box.recoveryPartial, true);
  assert.equal(classes.has("recovery"), true);
  assert.equal(box.copyRecoveryEl.hidden, false);
  assert.deepEqual(messages.at(-1), ["report", "ready-to-copy"]);
});

test("a recovery read failure is reported and blocks recording", async () => {
  const states = [];
  const box = {
    recoveryReady: false, token: "token",
    document: { body: { classList: { remove() {} } } },
    fetch: async () => ({ ok: false }),
    setState: (...args) => states.push(args), report: (state) => states.push(["report", state]),
  };
  vm.runInNewContext(`${sourceOf("loadRecovery")}\nthis.load = loadRecovery;`, box);
  await box.load();
  assert.equal(box.recoveryReady, false);
  assert.equal(states[0][0], "Recovery unavailable");
  assert.deepEqual(states.at(-1), ["report", "error"]);
});

test("new recording waits for recovery loading and cannot erase pending text", async () => {
  const states = [];
  const box = { recoveryReady: false, setState: (...args) => states.push(args), report: (state) => states.push(["report", state]) };
  vm.runInNewContext(`${sourceOf("begin")}\n${sourceOf("ready")}\nthis.begin = begin; this.ready = ready;`, box);
  await box.begin();
  box.ready();
  assert.equal(states[0][0], "Recovery unavailable");
  assert.equal(states.length, 1);
});

test("discard keeps the result if deleting saved recovery fails", async () => {
  const box = {
    recoveryText: "keep me", copyRecoveryEl: { disabled: false }, clearRecoveryEl: { disabled: false, dataset: { confirm: "yes" } },
    starting: false, saveRecovery: async () => false,
    showRecovery() { throw new Error("discarded unsaved text"); }, ready() {},
  };
  vm.runInNewContext(`${sourceOf("clearRecovery")}\nthis.clear = clearRecovery;`, box);
  await box.clear();
  assert.equal(box.recoveryText, "keep me");
  assert.equal(box.copyRecoveryEl.disabled, false);
});

test("recovery writes stay ordered and a failure prevents an unsafe clear", async () => {
  const writes = [];
  const messages = [];
  const box = {
    recoveryQueue: Promise.resolve(), token: "token", setState: (...args) => messages.push(args),
    fetch: async (_url, options) => {
      const payload = JSON.parse(options.body);
      writes.push(payload.text);
      return { ok: payload.text !== "" };
    },
  };
  vm.runInNewContext(`${sourceOf("saveRecovery")}\nthis.save = saveRecovery;`, box);
  const first = box.save("First", true);
  const second = box.save("First Second", false);
  assert.equal(await first, true);
  assert.equal(await second, true);
  assert.equal(await box.save("", false), false);
  assert.deepEqual(writes, ["First", "First Second", ""]);
  assert.equal(messages.at(-1)[0], "Save failed");
});

test("the tray action calls the same copy handler as the island button", () => {
  const hook = readFileSync(join(here, "../../parlo-app/src/island.rs"), "utf8");
  const tray = readFileSync(join(here, "../../parlo-app/src/presence.rs"), "utf8");
  assert.match(island, /copyRecoveryEl\.addEventListener\("click", copyRecovery\)/);
  assert.match(island, /window\.parloCopyRecovery = copyRecovery/);
  assert.match(hook, /window\.parloCopyRecovery && window\.parloCopyRecovery\(\)/);
  assert.match(tray, /"copy" => crate::island::copy_recovery\(app\)/);
  assert.match(tray, /"review" => crate::island::review_recovery\(app\)/);
  assert.match(hook, /window\.parloReviewRecovery && window\.parloReviewRecovery\(\)/);
});

test("a network failure leaves insertion unconfirmed", async () => {
  const box = { token: "test", fetch: async () => { throw new Error("disconnected"); } };
  vm.runInNewContext(`${sourceOf("writeIntoField")}\nthis.write = writeIntoField;`, box);
  const outcome = await box.write("words", undefined, "", 0, false);
  assert.equal(outcome.written, false);
  assert.match(outcome.reason, /Check the field/);
});

test("cancelling before the final dictation prevents insertion", async () => {
  let finishPartial;
  const waiting = new Promise((resolve) => { finishPartial = resolve; });
  const { box, calls } = stopHarness({ partialSettled: () => waiting });
  const stopped = box.stop();
  box.session += 1;
  finishPartial();
  await stopped;
  assert.equal(calls.some((entry) => entry[0] === "fetch"), false);
  assert.equal(calls.some((entry) => entry[0] === "insert"), false);
});
