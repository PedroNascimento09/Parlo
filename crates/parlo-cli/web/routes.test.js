const test = require("node:test");
const assert = require("node:assert/strict");
const { parseHash } = require("./routes.js");

test("a deep library link keeps its tab", () => {
  assert.deepEqual(parseHash("#library/batch"), {
    route: "library",
    tab: "batch",
    libraryTab: "batch",
    settingsTab: "dictation",
    canonical: "library/batch",
  });
});

test("a deep settings link keeps its tab", () => {
  const route = parseHash("#settings/languages");
  assert.equal(route.route, "settings");
  assert.equal(route.settingsTab, "languages");
  assert.equal(route.canonical, "settings/languages");
});

test("a bare library link falls back to meetings", () => {
  assert.equal(parseHash("#library").canonical, "library/meetings");
});

test("an unknown library tab falls back without touching settings", () => {
  const route = parseHash("#library/elsewhere");
  assert.equal(route.libraryTab, "meetings");
  assert.equal(route.canonical, "library/meetings");
});

test("the old meetings link opens the media workspace", () => {
  const route = parseHash("#meetings");
  assert.equal(route.route, "library");
  assert.equal(route.libraryTab, "meetings");
  assert.equal(route.canonical, "library/meetings");
});

test("a bare settings link falls back to dictation", () => {
  assert.equal(parseHash("#settings").canonical, "settings/dictation");
});

test("the models tab is a settings link", () => {
  const route = parseHash("#settings/models");
  assert.equal(route.settingsTab, "models");
  assert.equal(route.libraryTab, "meetings");
  assert.equal(route.canonical, "settings/models");
});

test("an unknown route falls back to home", () => {
  const route = parseHash("#not-a-section");
  assert.equal(route.route, "home");
  assert.equal(route.canonical, "home");
});

test("an empty hash is home", () => {
  assert.equal(parseHash("").canonical, "home");
});

test("the library no longer offers a models or words tab", () => {
  for (const tab of ["models", "vocabulary", "language"]) {
    assert.equal(parseHash("#library/" + tab).libraryTab, "meetings");
  }
});

test("reloading a canonical link produces the same link", () => {
  for (const link of [
    "#home",
    "#meetings",
    "#library/batch",
    "#library/stories",
    "#settings/models",
    "#settings/languages",
    "#settings/advanced",
  ]) {
    const once = parseHash(link).canonical;
    const twice = parseHash("#" + once).canonical;
    assert.equal(twice, once);
  }
});
