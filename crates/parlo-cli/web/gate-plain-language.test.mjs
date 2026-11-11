import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import test from "node:test";

const here = dirname(fileURLToPath(import.meta.url));
const index = readFileSync(join(here, "index.html"), "utf8").replace(/\r\n/g, "\n");
const app = readFileSync(join(here, "app.js"), "utf8").replace(/\r\n/g, "\n");
const locales = JSON.parse(readFileSync(join(here, "locales.json"), "utf8"));

const KEYS = [
  "gate.continue",
  "gate.dictate.copy",
  "gate.dictate.copyFailed",
  "gate.dictate.copied",
  "gate.dictate.kicker",
  "gate.dictate.listening",
  "gate.dictate.note",
  "gate.dictate.nothingHeard",
  "gate.dictate.record",
  "gate.dictate.result",
  "gate.dictate.stop",
  "gate.dictate.title",
  "gate.dictate.tooShort",
  "gate.dictate.waiting",
  "gate.dictate.working",
  "gate.finish",
  "gate.from",
  "gate.languages.kicker",
  "gate.languages.note",
  "gate.languages.title",
  "gate.microphone.check",
  "gate.microphone.heard",
  "gate.microphone.idle",
  "gate.microphone.kicker",
  "gate.microphone.note",
  "gate.microphone.ok",
  "gate.microphone.title",
  "gate.microphone.waiting",
  "gate.name.label",
  "gate.name.note",
  "gate.name.short",
  "gate.name.title",
  "gate.pair.saving",
  "gate.review.done",
  "gate.review.notStarted",
  "gate.review.open",
  "gate.review.skipped",
  "gate.review.title",
  "gate.shortcut.cancelled",
  "gate.shortcut.discard",
  "gate.shortcut.hold",
  "gate.shortcut.kicker",
  "gate.shortcut.listening",
  "gate.shortcut.middle",
  "gate.shortcut.note2",
  "gate.shortcut.ok",
  "gate.shortcut.title",
  "gate.shortcut.try",
  "gate.skip",
  "gate.skipAll",
  "gate.step",
  "gate.style.kicker",
  "gate.style.note",
  "gate.style.title",
  "gate.swap",
  "gate.to",
  "gate.welcome",
  "text.Step 1 of 6",
  "welcome.hold",
  "welcome.install",
  "welcome.more",
  "welcome.rest",
  "welcome.setup",
  "welcome.title",
];

const STEP_NOTES = [
  "gate.name.note",
  "gate.languages.note",
  "gate.microphone.note",
  "gate.style.note",
  "gate.shortcut.note2",
  "gate.dictate.note",
];

test("every locale carries the gate words", () => {
  assert.equal(locales.locales.length, 21);
  for (const code of locales.locales) {
    for (const key of KEYS) {
      const value = locales.strings[code][key];
      assert.equal(typeof value, "string", `${code} misses ${key}`);
      assert.ok(value.trim().length > 0, `${code} leaves ${key} empty`);
    }
  }
  assert.notEqual(locales.strings.pt["gate.name.note"], locales.strings.en["gate.name.note"]);
  assert.notEqual(locales.strings.pt["gate.microphone.title"], locales.strings.en["gate.microphone.title"]);
  assert.notEqual(locales.strings.pt["gate.dictate.note"], locales.strings.en["gate.dictate.note"]);
});

test("each step note shows what comes next", () => {
  for (const key of STEP_NOTES) {
    assert.match(locales.strings.en[key], /Then:/, `en ${key} should end with Then:`);
    assert.match(locales.strings.pt[key], /Depois:/, `pt ${key} should end with Depois:`);
  }
});

test("the English gate uses plain words, no jargon", () => {
  for (const key of KEYS) {
    const value = locales.strings.en[key];
    assert.doesNotMatch(value, /desktop apps/i, `en ${key} still says desktop apps`);
    assert.doesNotMatch(value, /\bprogram\b/i, `en ${key} still says program`);
    assert.doesNotMatch(value, /\bPress\b/, `en ${key} still says Press`);
    assert.doesNotMatch(value, /bookmark/i, `en ${key} still says bookmark`);
  }
});

test("every gate key is wired into the page", () => {
  for (const key of KEYS) {
    const escaped = key.replace(/\./g, "\\.");
    const inMarkup = new RegExp(`data-i18n="${escaped}"`).test(index);
    const inApp = app.includes(`"${key}"`) || app.includes(`'${key}'`);
    assert.ok(inMarkup || inApp, `${key} is not used anywhere`);
  }
});
