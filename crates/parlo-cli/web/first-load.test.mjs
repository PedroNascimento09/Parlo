import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import test from "node:test";
import vm from "node:vm";

const here = dirname(fileURLToPath(import.meta.url));
const island = readFileSync(join(here, "island.html"), "utf8").replace(/\r\n/g, "\n");
const locales = JSON.parse(readFileSync(join(here, "locales.json"), "utf8"));

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
  const start = island.indexOf(`\nfunction ${name}(`);
  assert.ok(start >= 0, `the island defines ${name}`);
  const end = island.indexOf("\n}\n", start);
  assert.ok(end > start, `${name} closes`);
  return island.slice(start + 1, end + 3);
}

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
  assert.match(island, /noteWriteFailure\(outcome\.reason\)/);
});

test("the island paints the note when a dictation begins and when the first result lands", () => {
  assert.match(island, /paintFirstLoad\(hintEl, warmed\)/);
  assert.match(island, /paintFirstLoad\(hintEl, true\)/);
});
