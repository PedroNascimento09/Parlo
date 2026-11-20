import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import test from "node:test";

const here = dirname(fileURLToPath(import.meta.url));
const index = readFileSync(join(here, "index.html"), "utf8").replace(/\r\n/g, "\n");
const app = readFileSync(join(here, "app.js"), "utf8").replace(/\r\n/g, "\n");

test("the rail carries four destinations", () => {
  const nav = index.slice(index.indexOf('<nav class="rail"'), index.indexOf("</nav>"));
  const routes = [...nav.matchAll(/data-route="([a-z]+)"/g)].map((match) => match[1]);
  assert.deepEqual(routes, ["home", "meetings", "library", "settings"]);
});

test("the two secondary screens stay one click from settings", () => {
  assert.match(index, /<a class="home-link" href="#voice"/);
  assert.match(index, /<a class="home-link" href="#insights"/);
});

test("the number keys follow the rail", () => {
  assert.match(app, /const ROUTE_KEYS = \{ 1: "home", 2: "meetings", 3: "library", 4: "settings" \};/);
  for (const [number, route] of [
    ["1", "home"],
    ["2", "meetings"],
    ["3", "library"],
    ["4", "settings"],
  ]) {
    assert.match(index, new RegExp(`data-route="${route}" aria-keyshortcuts="Alt\\+${number}"`));
  }
});
