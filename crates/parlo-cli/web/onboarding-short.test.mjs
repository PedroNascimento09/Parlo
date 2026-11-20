import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import test from "node:test";

const here = dirname(fileURLToPath(import.meta.url));
const index = readFileSync(join(here, "index.html"), "utf8").replace(/\r\n/g, "\n");
const app = readFileSync(join(here, "app.js"), "utf8").replace(/\r\n/g, "\n");

test("the guide has four steps and no explaining screens", () => {
  const steps = [...index.matchAll(/<div class="gate-step" id="(gate-[a-z]+)"/g)].map((match) => match[1]);
  assert.deepEqual(steps, ["gate-name", "gate-languages", "gate-microphone", "gate-dictate"]);
  assert.doesNotMatch(index, /id="gate-style"/);
  assert.doesNotMatch(index, /id="gate-shortcut"/);
  assert.match(app, /const GATE_STEPS = \["gate-name", "gate-languages", "gate-microphone", "gate-dictate"\];/);
});

test("the progress line counts four", () => {
  assert.match(index, /<p class="gate-progress" id="gate-progress">Step 1 of 4<\/p>/);
  assert.doesNotMatch(index, /Step 1 of 6/);
  assert.match(app, /paintGateProgress\(index, GATE_STEPS\.length\)/);
});

test("the microphone step never blocks the way forward", () => {
  const at = app.indexOf('gateMicCheck.addEventListener("click"');
  const caught = app.indexOf("} catch (failure) {", at);
  const close = app.indexOf("});", caught);
  assert.match(app.slice(caught, close), /gateMicNext\.disabled = false;/);
  assert.match(app, /gateMicNext\.addEventListener\("click", \(\) => gateStep\("gate-dictate"\)\);/);
  assert.match(app, /gateMicSkip\.addEventListener\("click", \(\) => gateStep\("gate-dictate"\)\);/);
});

test("the last step opens the app after a refusal", () => {
  const start = app.indexOf("async function startGateDictate()");
  const failed = app.indexOf("} catch (failure) {", start);
  assert.match(app.slice(failed, failed + 400), /gateFinish\.disabled = false;/);
  const finish = app.indexOf("async function finishGateDictate()");
  const refused = app.indexOf("} catch (failure) {", finish);
  assert.match(app.slice(refused, refused + 400), /gateFinish\.disabled = false;/);
  assert.match(app, /gateFinish\.addEventListener\("click", \(\) => closeGate\("done"\)\);/);
  assert.match(app, /navigate\("home"\);\n  paintHome\(\);/);
});

test("a finished guide never opens again", () => {
  assert.match(app, /function showNameGateIfNeeded\(\) \{\n  if \(onboardingState\(\)\) \{\n    return;\n  \}/);
  assert.match(app, /localStorage\.setItem\("parlo\.onboarding"/);
});
