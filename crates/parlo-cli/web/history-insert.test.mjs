import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import test from "node:test";
import vm from "node:vm";

const here = dirname(fileURLToPath(import.meta.url));
const app = readFileSync(join(here, "app.js"), "utf8").replace(/\r\n/g, "\n");

function sourceOf(name) {
  const start = app.indexOf(`\nasync function ${name}(`);
  assert.ok(start >= 0, `the app defines ${name}`);
  const end = app.indexOf("\n}\n", start);
  assert.ok(end > start, `${name} closes`);
  return app.slice(start + 1, end + 3);
}

function harness(historyPostImpl) {
  const calls = [];
  const note = { textContent: "" };
  const box = {
    historyNote: note,
    t: (key, fallback) => fallback,
    historyPost: async (path, body) => {
      calls.push([path, body]);
      return historyPostImpl(path, body);
    },
  };
  vm.runInNewContext(
    `${sourceOf("insertHistory")}\nthis.insertHistory = insertHistory;`,
    box,
  );
  return { calls, note, insertHistory: box.insertHistory };
}

test("reinserting posts the dictation id and confirms the text landed", async () => {
  const { calls, note, insertHistory } = harness(async () => ({ inserted: true }));
  const button = { disabled: false };
  await insertHistory({ id: "h1-0" }, button);
  assert.deepEqual(
    calls.map((call) => JSON.stringify(call)),
    [JSON.stringify(["/history/reinsert", { id: "h1-0" }])],
  );
  assert.equal(
    note.textContent,
    "The dictation is in your field. Undo removes it.",
  );
  assert.equal(button.disabled, false);
});

test("reinserting shows the server refusal instead of failing silently", async () => {
  const refusal = "Parlo has no window to write in. Copy the text and paste it yourself.";
  const { note, insertHistory } = harness(async () => {
    throw new Error(refusal);
  });
  const button = { disabled: false };
  await insertHistory({ id: "h1-0" }, button);
  assert.equal(note.textContent, refusal);
  assert.equal(button.disabled, false);
});

test("the history card offers the insert action beside copy", () => {
  assert.ok(app.includes('t("text.Insert", "Insert")'), "the insert label is translated");
  assert.ok(
    app.includes("insertHistory(entry, put)"),
    "the card wires the insert button to insertHistory",
  );
});
