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
  "home.lede",
  "home.hint",
  "text.Recent dictations",
  "text.Undo shortcut",
  "nav.library",
];

test("the landing copy is wired to the page and not stale", () => {
  assert.match(index, /data-i18n="home\.lede"/);
  assert.match(index, /data-i18n="home\.hint"/);
  assert.doesNotMatch(index, /copy the sentence from the bar/i);
  assert.doesNotMatch(index, /copy the result and paste/i);
  assert.equal(locales.strings.en["home.lede"], "Say it once. Parlo writes it where the cursor is, and nothing leaves this computer.");
  assert.equal(locales.strings.en["home.hint"], "Let go, and the sentence lands where the cursor is.");
});

test("the pair and mode line is visible", () => {
  assert.match(index, /<p class="stage-sub">/);
});

test("the last three dictations link to the library", () => {
  assert.match(index, /id="home-recent"/);
  assert.match(index, /data-i18n="text\.Recent dictations"/);
  assert.match(index, /href="#library\/history"/);
  assert.match(app, /homeRecentList\.replaceChildren\(\)/);
  assert.match(app, /homeRecent\.hidden = entries\.length === 0/);
  assert.match(app, /\.slice\(0, 3\)/);
});

test("the undo shortcut shows on the home card", () => {
  assert.match(index, /id="status-undo-shortcut"/);
  assert.match(app, /undoShortcutEl\.textContent = values\.undo/);
});

test("every locale carries the words the home uses", () => {
  assert.equal(locales.locales.length, 21);
  for (const code of locales.locales) {
    for (const key of KEYS) {
      const value = locales.strings[code][key];
      assert.equal(typeof value, "string", `${code} misses ${key}`);
      assert.ok(value.trim().length > 0, `${code} leaves ${key} empty`);
    }
  }
  assert.notEqual(locales.strings.pt["home.hint"], locales.strings.en["home.hint"]);
  assert.notEqual(locales.strings.pt["text.Recent dictations"], locales.strings.en["text.Recent dictations"]);
});

test("the recent list carries its own styles", () => {
  assert.match(css, /\.home-recent \{/);
  assert.match(css, /\.home-recent-list \{/);
  assert.match(css, /\.home-recent-item \{/);
  assert.match(css, /\.home-undo \{/);
});
