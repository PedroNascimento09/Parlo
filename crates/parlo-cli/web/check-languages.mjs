import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const read = (name) => readFileSync(join(here, name), "utf8");
const table = JSON.parse(read("languages.json"));
const shared = read("languages.js");

const writeTargets = new Function(`${shared}; return writeTargets;`)();

if (!read("island.html").includes('src="languages.js"') || !read("island.html").includes("writeTargets(")) {
  throw new Error("the island does not load the shared list");
}
if (read("index.html").includes('id="settings-write"') || read("app.js").includes("writeTargets(")) {
  throw new Error("the app offers a second write list");
}

const names = table.names;
const paired = table.pairs;
const spoken = [...new Set(paired.map(([from]) => from))].sort();
if (spoken.length === 0) {
  throw new Error("the language table names no spoken language");
}

for (const from of spoken) {
  const targets = writeTargets(paired, names, from);
  if (targets.length === 0) {
    throw new Error(`${from} offers no language to write in`);
  }
  if (new Set(targets).size !== targets.length) {
    throw new Error(`${from} offers the same language twice`);
  }
  for (const target of targets) {
    if (!names[target]) {
      throw new Error(`${from}->${target} has no name`);
    }
  }
  console.log(`${from} -> ${targets.map((code) => names[code]).join(", ")}`);
}
