const test = require("node:test");
const assert = require("node:assert/strict");
const { parseHash } = require("./routes.js");

test("a deep library link keeps its tab", () => {
  assert.deepEqual(parseHash("#library/models"), {
    route: "library",
    tab: "models",
    libraryTab: "models",
    settingsTab: "dictation",
    canonical: "library/models",
  });
});

test("a deep settings link keeps its tab", () => {
  const route = parseHash("#settings/languages");
  assert.equal(route.route, "settings");
  assert.equal(route.settingsTab, "languages");
  assert.equal(route.canonical, "settings/languages");
});

test("a bare library link falls back to history", () => {
  assert.equal(parseHash("#library").canonical, "library/history");
});

test("an unknown library tab falls back without touching settings", () => {
  const route = parseHash("#library/elsewhere");
  assert.equal(route.libraryTab, "history");
  assert.equal(route.canonical, "library/history");
});

test("the language tab is a deep link", () => {
  const route = parseHash("#settings/language");
  assert.equal(route.settingsTab, "language");
  assert.equal(route.canonical, "settings/language");
});

test("a bare settings link falls back to dictation", () => {
  assert.equal(parseHash("#settings").canonical, "settings/dictation");
});

test("an unknown route falls back to home", () => {
  const route = parseHash("#not-a-section");
  assert.equal(route.route, "home");
  assert.equal(route.canonical, "home");
});

test("an empty hash is home", () => {
  assert.equal(parseHash("").canonical, "home");
});

test("reloading a canonical link produces the same link", () => {
  for (const link of [
    "#home",
    "#meetings",
    "#library/batch",
    "#settings/language",
    "#settings/advanced",
  ]) {
    const once = parseHash(link).canonical;
    const twice = parseHash("#" + once).canonical;
    assert.equal(twice, once);
  }
});
