import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import test from "node:test";

const here = dirname(fileURLToPath(import.meta.url));
const index = readFileSync(join(here, "index.html"), "utf8").replace(/\r\n/g, "\n");
const app = readFileSync(join(here, "app.js"), "utf8").replace(/\r\n/g, "\n");
const css = readFileSync(join(here, "app.css"), "utf8").replace(/\r\n/g, "\n");
const localesMeta = JSON.parse(readFileSync(join(here, "locales/meta.json"), "utf8"));
const locales = {
  ...localesMeta,
  strings: Object.fromEntries(
    localesMeta.locales.map((code) => [code, JSON.parse(readFileSync(join(here, `locales/${code}.json`), "utf8"))]),
  ),
};

test("the home card dictates without the global shortcut", () => {
  const start = index.indexOf('class="home-primary"');
  const end = index.indexOf('class="home-grid"');
  assert.ok(start >= 0 && end > start, "the dictation card sits above the grid");
  const card = index.slice(start, end);
  assert.match(card, /id="home-take-record"/);
  assert.match(card, /data-i18n="home\.takeStart"/);
  assert.match(card, /id="home-take-text"/);
  assert.match(app, /async function startHomeTake/);
  assert.match(app, /async function finishHomeTake/);
  assert.match(app, /navigator\.mediaDevices\.getUserMedia/);
  assert.match(app, /fetch\("\/dictate"/);
  assert.match(css, /\.home-take-record \{/);
  assert.match(css, /\.home-take-meter\[hidden\]/);
});

test("leaving the home screen releases the microphone", () => {
  assert.match(app, /if \(parsed\.route !== "home"\) \{\n    stopHomeMicCheck\(\);\n    releaseHomeTake\(\);/);
});

test("the duplicate first sentence guide is gone", () => {
  assert.doesNotMatch(index, /id="home-steps"/);
  assert.doesNotMatch(app, /homeSteps/);
  assert.doesNotMatch(css, /\.home-steps/);
});

test("every locale carries the words the new control uses", () => {
  assert.equal(locales.locales.length, 21);
  for (const code of locales.locales) {
    for (const key of ["home.takeStart", "home.takeHint"]) {
      const value = locales.strings[code][key];
      assert.equal(typeof value, "string", `${code} misses ${key}`);
      assert.ok(value.trim().length > 0, `${code} leaves ${key} empty`);
    }
  }
  assert.equal(locales.strings.en["home.takeStart"], "Record a sentence");
  assert.notEqual(locales.strings.pt["home.takeHint"], locales.strings.en["home.takeHint"]);
});
