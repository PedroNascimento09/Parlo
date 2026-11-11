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
  "gate.microphone.check",
  "gate.microphone.waiting",
  "gate.microphone.heard",
  "gate.microphone.ok",
  "gate.microphone.idle",
  "gate.dictate.stop",
  "text.This browser cannot open a microphone.",
  "text.Windows blocked the microphone. Allow Parlo in Settings, Privacy, Microphone, then try again.",
  "text.No microphone was found. Connect one and reload this page.",
  "text.Another program is using the microphone. Close it and try again.",
  "text.The microphone could not be opened. Check the input device in your system settings.",
];

test("the readiness panel carries the check button, note and live bar", () => {
  assert.match(index, /id="home-mic-check"/);
  assert.match(index, /id="home-mic-note"/);
  assert.match(index, /id="home-mic-meter"/);
  assert.match(index, /data-i18n="gate\.microphone\.check"/);
  assert.match(index, /data-i18n="gate\.microphone\.idle"/);
  assert.match(index, /aria-live="polite"/);
  assert.doesNotMatch(index, /<dd id="home-mic">not checked<\/dd>\s*<\/dl>\s*<\/section>/);
});

test("the check reads the microphone and only then says ready", () => {
  const at = app.indexOf('homeMicCheck.addEventListener("click"');
  assert.ok(at > 0, "the home mic check handler exists");
  const handler = app.slice(at, app.indexOf("} catch (failure) {", at));
  assert.match(handler, /navigator\.mediaDevices\.getUserMedia/);
  assert.match(app, /localStorage\.setItem\("parlo\.mic", "ready"\)/);
  const granted = app.indexOf('localStorage.setItem("parlo.mic", "ready")');
  const caught = app.indexOf("} catch (failure) {", at);
  assert.ok(granted > at && granted < caught, "ready is recorded only after the stream opens");
  assert.match(app, /homeMic\.textContent = statusWord\("ready"\)/);
});

test("each state is honest: waiting, open, heard, blocked, absent, busy and unsupported", () => {
  assert.match(app, /homeMicNote\.textContent = t\("gate\.microphone\.waiting"/);
  assert.match(app, /homeMicNote\.textContent = t\("gate\.microphone\.ok"/);
  assert.match(app, /homeMicNote\.textContent = t\("text\.This browser cannot open a microphone\."/);
  assert.match(app, /failure\.name === "NotAllowedError"/);
  assert.match(app, /failure\.name === "NotFoundError"/);
  assert.match(app, /failure\.name === "NotReadableError"/);
  assert.match(app, /homeMicNote\.textContent = describeMicrophoneError\(failure\)/);
});

test("microphone errors speak the visitor language", () => {
  const at = app.indexOf("function describeMicrophoneError(failure)");
  const end = app.indexOf("\n}\n", at) + 3;
  const fn = app.slice(at, end);
  assert.match(fn, /t\("text\.Windows blocked the microphone\./);
  assert.match(fn, /t\("text\.No microphone was found\./);
  assert.match(fn, /t\("text\.Another program is using the microphone\./);
  assert.match(fn, /t\("text\.The microphone could not be opened\./);
  assert.doesNotMatch(fn, /^\s*return "/m);
});

test("the bar moves with the live input and stops cleanly", () => {
  assert.match(app, /function drawHomeMicMeter\(\)/);
  assert.match(app, /homeMicAnalyser\.getFloatTimeDomainData\(homeMicLevels\)/);
  assert.match(app, /homeMicMeter\.style\.width = \(level \* 100\)\.toFixed\(1\) \+ "%"/);
  assert.match(app, /level > 0\.08/);
  assert.match(app, /function stopHomeMicCheck\(\)/);
  assert.match(app, /homeMicStream\.getTracks\(\)\.forEach\(\(track\) => track\.stop\(\)\)/);
  assert.match(app, /clearTimeout\(homeMicTimer\)/);
  assert.match(app, /homeMicTimer = setTimeout\(\(\) => \{\s*stopHomeMicCheck\(\);/);
  assert.match(app, /if \(parsed\.route !== "home"\) \{\s*stopHomeMicCheck\(\);/);
});

test("every locale carries the words the mic check uses", () => {
  assert.equal(locales.locales.length, 21);
  for (const code of locales.locales) {
    for (const key of KEYS) {
      const value = locales.strings[code][key];
      assert.equal(typeof value, "string", `${code} misses ${key}`);
      assert.ok(value.trim().length > 0, `${code} leaves ${key} empty`);
    }
  }
  assert.notEqual(locales.strings.pt["gate.microphone.check"], locales.strings.en["gate.microphone.check"]);
  assert.notEqual(locales.strings.pt["gate.microphone.idle"], locales.strings.en["gate.microphone.idle"]);
});

test("the check block carries its own styles", () => {
  assert.match(css, /\.home-mic-check \{/);
  assert.match(css, /\.home-mic-note \{/);
  assert.match(css, /\.home-mic-check-button \{/);
});
