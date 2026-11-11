import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import test from "node:test";
import vm from "node:vm";

const here = dirname(fileURLToPath(import.meta.url));
const app = readFileSync(join(here, "app.js"), "utf8").replace(/\r\n/g, "\n");

const start = app.indexOf("\nfunction knownApp(");
assert.ok(start >= 0, "app.js defines knownApp");
const end = app.indexOf("\n}\n", start);
assert.ok(end > start, "knownApp closes");
const knownApp = vm.runInNewContext(`${app.slice(start + 1, end + 3)}; knownApp`);

test("a real program name passes through trimmed", () => {
  assert.equal(knownApp("Notepad"), "Notepad");
  assert.equal(knownApp("  Code.exe  "), "Code.exe");
});

test("the unknown placeholder is hidden, not shown", () => {
  assert.equal(knownApp("unknown"), "");
  assert.equal(knownApp("  unknown  "), "");
});

test("blank or missing names are hidden", () => {
  assert.equal(knownApp(""), "");
  assert.equal(knownApp(null), "");
  assert.equal(knownApp(undefined), "");
});
