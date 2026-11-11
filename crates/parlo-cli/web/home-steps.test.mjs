import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import test from "node:test";

const here = dirname(fileURLToPath(import.meta.url));
const index = readFileSync(join(here, "index.html"), "utf8").replace(/\r\n/g, "\n");
const app = readFileSync(join(here, "app.js"), "utf8").replace(/\r\n/g, "\n");
const css = readFileSync(join(here, "app.css"), "utf8").replace(/\r\n/g, "\n");
const locales = JSON.parse(readFileSync(join(here, "locales.json"), "utf8"));

const KEYS = [
  "home.stepsKicker",
  "home.gotIt",
  "home.step1",
  "home.step1Text",
  "home.step2",
  "home.step2Text",
  "home.step3",
  "home.step3Text",
];

test("the guide section renders three numbered steps before the grid", () => {
  const start = index.indexOf('id="home-steps"');
  const grid = index.indexOf('class="home-grid"');
  assert.ok(start >= 0, "the guide section exists");
  assert.ok(start < grid, "the guide sits above the grid");
  for (const key of KEYS) {
    assert.match(index, new RegExp(`data-i18n="${key.replace(/\./g, "\\.")}"`));
  }
  assert.equal((index.match(/home-step-num/g) || []).length, 3);
});

test("dismiss persists and hides the guide", () => {
  assert.match(app, /parlo\.homeStepsDismissed/);
  assert.match(app, /homeStepsDismiss\.addEventListener\("click"/);
  assert.match(app, /homeSteps\.hidden = true/);
});

test("the guide hides once the first dictation lands", () => {
  assert.match(app, /homeSteps\.hidden = stepsDismissed\(\) \|\| hasHistory/);
  assert.match(app, /historyEntries\.some/);
});

test("every locale carries the guide words", () => {
  assert.equal(locales.locales.length, 21);
  for (const code of locales.locales) {
    for (const key of KEYS) {
      const value = locales.strings[code][key];
      assert.equal(typeof value, "string", `${code} misses ${key}`);
      assert.ok(value.trim().length > 0, `${code} leaves ${key} empty`);
    }
  }
  assert.notEqual(locales.strings.pt["home.stepsKicker"], locales.strings.en["home.stepsKicker"]);
  assert.notEqual(locales.strings.pt["home.step1Text"], locales.strings.en["home.step1Text"]);
});

test("the guide carries responsive styles", () => {
  assert.match(css, /\.home-steps \{/);
  assert.match(css, /\.home-steps-list \{/);
  assert.match(css, /\.home-step-num \{/);
  assert.match(css, /\.home-steps\[hidden\] \{/);
});
