const token = new URLSearchParams(location.search).get("t") || "";
const version = document.querySelector('meta[name="parlo-version"]').content;

const talk = document.getElementById("talk");
const label = talk.querySelector(".talk-label");
const sub = talk.querySelector(".talk-sub");
const meterFill = document.getElementById("meter-fill");
const progress = document.getElementById("progress");
const errorBox = document.getElementById("error");
const translation = document.getElementById("translation");
const updateBox = document.getElementById("update");
const updateVersion = document.getElementById("update-version");
const updateLink = document.getElementById("update-link");
const updateText = document.getElementById("update-text");
const installButton = document.getElementById("update-install");
const engineEl = document.getElementById("status-engine");
const uptimeEl = document.getElementById("status-uptime");
const micEl = document.getElementById("status-mic");
const versionEl = document.getElementById("status-version");
const modelsBox = document.getElementById("models");
const modelsList = document.getElementById("models-list");
const modelsNote = document.getElementById("models-note");
const modelsInstall = document.getElementById("models-install");
const dropZone = document.getElementById("drop");
const fileInput = document.getElementById("file");
const dubStart = document.getElementById("dub-start");
const dubFile = document.getElementById("dub-file");
const dubStatus = document.getElementById("dub-status");
const dubResult = document.getElementById("dub-result");
const dubDownload = document.getElementById("dub-download");
const dubCount = document.getElementById("dub-count");
const dubLines = document.getElementById("dub-lines");

const TARGET_RATE = 16000;
const MAX_SECONDS = 30;
const MIN_SECONDS = 0.25;

let media = null;
let context = null;
let analyser = null;
let processor = null;
let levels = null;
let meterFrame = null;
let recording = false;
let chunks = [];
let maxTimer = null;
let player = null;
let localUpdate = { available: false, latest: "", notes: "" };
let modelsTimer = null;
let modelsPolling = false;
let chosenFile = null;
let dubbing = false;
let dubTimer = null;

versionEl.textContent = version || "unknown";

function showError(message) {
  errorBox.textContent = message;
  errorBox.hidden = false;
}

function clearError() {
  errorBox.textContent = "";
  errorBox.hidden = true;
}

function setProgress(message) {
  progress.textContent = message || "";
  progress.hidden = !message;
}

function clearTranslation() {
  translation.textContent = "";
  translation.hidden = true;
}

function showTranslation(encoded) {
  if (!encoded) {
    clearTranslation();
    return;
  }
  try {
    const bytes = Uint8Array.from(atob(encoded), (character) => character.charCodeAt(0));
    const text = new TextDecoder().decode(bytes).trim();
    if (!text) {
      clearTranslation();
      return;
    }
    translation.textContent = text;
    translation.hidden = false;
  } catch (failure) {
    clearTranslation();
  }
}

function describeMicrophoneError(failure) {
  if (failure && failure.name === "NotAllowedError") {
    return "The browser blocked the microphone. Choose Allow for 127.0.0.1 in the address bar, then try again.";
  }
  if (failure && failure.name === "NotFoundError") {
    return "No microphone was found. Connect one and reload this page.";
  }
  if (failure && failure.name === "NotReadableError") {
    return "Another program is using the microphone. Close it and try again.";
  }
  return "The microphone could not be opened. Check the input device in your system settings.";
}

async function openMicrophone() {
  if (media) {
    return media;
  }
  if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
    throw new Error("This browser cannot open a microphone.");
  }
  media = await navigator.mediaDevices.getUserMedia({
    audio: { channelCount: 1, echoCancellation: true },
  });
  micEl.textContent = "ready";
  return media;
}

function drawMeter() {
  meterFrame = requestAnimationFrame(drawMeter);
  if (!analyser || !levels) {
    return;
  }
  analyser.getFloatTimeDomainData(levels);
  let sum = 0;
  for (let index = 0; index < levels.length; index += 1) {
    sum += levels[index] * levels[index];
  }
  const rms = Math.sqrt(sum / levels.length);
  const decibels = 20 * Math.log10(rms + 1e-6);
  const level = Math.min(1, Math.max(0, (decibels + 60) / 60));
  meterFill.style.width = (level * 100).toFixed(1) + "%";
}

async function ensureAudio() {
  const stream = await openMicrophone();
  if (!context) {
    const AudioContext = window.AudioContext || window.webkitAudioContext;
    context = new AudioContext();
    const source = context.createMediaStreamSource(stream);
    analyser = context.createAnalyser();
    analyser.fftSize = 2048;
    source.connect(analyser);
    processor = context.createScriptProcessor(4096, 1, 1);
    processor.onaudioprocess = (event) => {
      if (!recording) {
        return;
      }
      chunks.push(new Float32Array(event.inputBuffer.getChannelData(0)));
    };
    const silence = context.createGain();
    silence.gain.value = 0;
    source.connect(processor);
    processor.connect(silence);
    silence.connect(context.destination);
    levels = new Float32Array(analyser.fftSize);
    drawMeter();
  }
  if (context.state === "suspended") {
    await context.resume();
  }
  return context;
}

function downsample(input, fromRate, toRate) {
  if (fromRate === toRate) {
    return input;
  }
  const ratio = fromRate / toRate;
  const length = Math.floor(input.length / ratio);
  const output = new Float32Array(length);
  for (let index = 0; index < length; index += 1) {
    const position = index * ratio;
    const left = Math.floor(position);
    const right = Math.min(left + 1, input.length - 1);
    const weight = position - left;
    output[index] = input[left] * (1 - weight) + input[right] * weight;
  }
  return output;
}

function encodeWav(samples, sampleRate) {
  const buffer = new ArrayBuffer(44 + samples.length * 2);
  const view = new DataView(buffer);
  const ascii = (offset, text) => {
    for (let index = 0; index < text.length; index += 1) {
      view.setUint8(offset + index, text.charCodeAt(index));
    }
  };
  ascii(0, "RIFF");
  view.setUint32(4, 36 + samples.length * 2, true);
  ascii(8, "WAVE");
  ascii(12, "fmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, 1, true);
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * 2, true);
  view.setUint16(32, 2, true);
  view.setUint16(34, 16, true);
  ascii(36, "data");
  view.setUint32(40, samples.length * 2, true);
  let offset = 44;
  for (let index = 0; index < samples.length; index += 1) {
    const sample = Math.max(-1, Math.min(1, samples[index]));
    view.setInt16(offset, sample < 0 ? sample * 0x8000 : sample * 0x7fff, true);
    offset += 2;
  }
  return new Blob([view], { type: "audio/wav" });
}

function stopPlayback() {
  if (!player) {
    return;
  }
  player.pause();
  URL.revokeObjectURL(player.src);
  player = null;
}

function play(blob) {
  stopPlayback();
  const url = URL.createObjectURL(blob);
  player = new Audio(url);
  player.addEventListener(
    "ended",
    () => {
      URL.revokeObjectURL(url);
      player = null;
      setProgress("");
    },
    { once: true },
  );
  player.play().catch(() => {
    setProgress("");
    showError("The translation is ready but the browser blocked playback. Click the page, then try again.");
  });
}

async function begin() {
  if (recording) {
    return;
  }
  clearError();
  setProgress("");
  clearTranslation();
  stopPlayback();
  try {
    await ensureAudio();
  } catch (failure) {
    micEl.textContent = "blocked";
    showError(describeMicrophoneError(failure));
    return;
  }
  chunks = [];
  recording = true;
  micEl.textContent = "listening";
  talk.setAttribute("aria-pressed", "true");
  label.textContent = "Listening";
  sub.textContent = "Release to hear the translation";
  clearTimeout(maxTimer);
  maxTimer = setTimeout(() => finish(), MAX_SECONDS * 1000);
}

async function finish() {
  if (!recording) {
    return;
  }
  recording = false;
  micEl.textContent = "ready";
  clearTimeout(maxTimer);
  talk.setAttribute("aria-pressed", "false");
  label.textContent = "Hold to speak";
  sub.textContent = "Release to hear the translation";

  const captured = chunks;
  chunks = [];
  const rate = context ? context.sampleRate : 48000;
  const length = captured.reduce((total, chunk) => total + chunk.length, 0);
  const seconds = length / rate;
  if (seconds < MIN_SECONDS) {
    showError("Hold the button while you speak, then release it.");
    return;
  }

  const merged = new Float32Array(length);
  let offset = 0;
  for (const chunk of captured) {
    merged.set(chunk, offset);
    offset += chunk.length;
  }

  setProgress("Translating " + seconds.toFixed(1) + " s of speech.");
  try {
    const wav = encodeWav(downsample(merged, rate, TARGET_RATE), TARGET_RATE);
    const response = await fetch("/translate", {
      method: "POST",
      headers: { "Content-Type": "audio/wav", Authorization: "Bearer " + token },
      body: wav,
    });
    if (response.status === 401) {
      throw new Error("This page lost its access token. Start parlo app again and open the link it prints.");
    }
    if (!response.ok) {
      const detail = await response.json().catch(() => ({}));
      throw new Error(detail.error || "The translator answered with status " + response.status + ".");
    }
    const audio = await response.blob();
    showTranslation(response.headers.get("X-Parlo-Transcript"));
    setProgress("Playing the translation.");
    play(audio);
  } catch (failure) {
    setProgress("");
    showError(failure.message || "The translation failed. Check that the engine is running and try again.");
  }
}

function formatUptime(totalSeconds) {
  const seconds = Math.max(0, Math.floor(totalSeconds || 0));
  if (seconds < 60) {
    return seconds + " s";
  }
  if (seconds < 3600) {
    return Math.floor(seconds / 60) + " min";
  }
  return Math.floor(seconds / 3600) + " h " + Math.floor((seconds % 3600) / 60) + " min";
}

async function refreshHealth() {
  try {
    const response = await fetch("/health", { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    const health = await response.json();
    engineEl.textContent = health.engine || "unknown";
    uptimeEl.textContent = formatUptime(health.uptime_s);
  } catch (failure) {
    engineEl.textContent = "offline";
  }
}

function isNewer(candidate, current) {
  const parse = (value) =>
    String(value)
      .replace(/^v/i, "")
      .split(/[.+-]/)
      .map((part) => parseInt(part, 10))
      .filter((part) => !Number.isNaN(part));
  const left = parse(candidate);
  const right = parse(current);
  for (let index = 0; index < Math.max(left.length, right.length); index += 1) {
    const a = left[index] || 0;
    const b = right[index] || 0;
    if (a !== b) {
      return a > b;
    }
  }
  return false;
}

function showUpdate(candidate, notes) {
  updateVersion.textContent = candidate;
  if (notes) {
    updateLink.href = notes;
  }
  updateBox.hidden = false;
}

function allowInstall(allowed) {
  installButton.hidden = !allowed;
}

async function checkForUpdate() {
  const response = await fetch("https://api.github.com/repos/PedroNascimento09/Parlo/releases/latest", {
    headers: { Accept: "application/vnd.github+json" },
    cache: "no-store",
  });
  if (!response.ok) {
    return;
  }
  const release = await response.json();
  if (!release || !release.tag_name || !isNewer(release.tag_name, version)) {
    return;
  }
  if (!updateBox.hidden) {
    return;
  }
  showUpdate(release.tag_name.replace(/^v/i, ""), release.html_url);
}

async function checkLocalUpdate() {
  try {
    const response = await fetch("/update", { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    localUpdate = await response.json();
    if (!localUpdate.available) {
      allowInstall(false);
      return;
    }
    showUpdate(localUpdate.latest, localUpdate.notes);
    allowInstall(true);
  } catch (failure) {
    allowInstall(false);
  }
}

async function installUpdate() {
  if (installButton.disabled) {
    return;
  }
  installButton.disabled = true;
  installButton.textContent = "Installing…";
  clearError();
  updateBox.hidden = false;
  try {
    const response = await fetch("/update", {
      method: "POST",
      headers: { Authorization: "Bearer " + token },
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      updateText.textContent = detail.error || "The update could not be installed. Run parlo update in a terminal instead.";
      installButton.disabled = false;
      installButton.textContent = "Install update";
      return;
    }
    const installed = detail.version || localUpdate.latest || "";
    updateText.textContent = "Parlo " + installed + " is installed. Start parlo app again so the new version runs.";
    updateLink.hidden = true;
    installButton.textContent = "Installed";
  } catch (failure) {
    updateText.textContent = "The connection to Parlo closed, so the update may already be installed. Start parlo app again to check the version.";
    updateLink.hidden = true;
    installButton.textContent = "Start parlo app again";
  }
}

function formatSize(bytes) {
  const megabytes = Math.round((bytes || 0) / (1024 * 1024));
  if (megabytes < 1) {
    return "Missing";
  }
  return megabytes + " MB";
}

function describeModels(state) {
  modelsList.replaceChildren();
  for (const model of state.models || []) {
    const row = document.createElement("li");
    row.className = "models-row";

    const name = document.createElement("span");
    name.className = "models-name";
    name.textContent = model.name || "Model";

    const kind = document.createElement("span");
    kind.className = "models-kind";
    kind.textContent = model.kind || "";
    name.append(kind);

    const fact = document.createElement("span");
    fact.className = "models-fact" + (model.installed ? " ready" : "");
    fact.textContent = model.installed ? formatSize(model.bytes) : "Missing";

    row.append(name, fact);
    modelsList.append(row);
  }
  modelsBox.hidden = false;

  if (state.ready) {
    modelsNote.textContent = "Everything the translator needs is on this machine, so you can speak now.";
    modelsInstall.hidden = true;
    return true;
  }

  modelsNote.textContent = "Some models are missing. They are a one time download and stay on this machine.";
  modelsInstall.hidden = false;
  modelsInstall.disabled = modelsPolling;
  return false;
}

async function loadModels() {
  try {
    const response = await fetch("/models", { cache: "no-store" });
    if (!response.ok) {
      showModelsError("The model list could not be read. The translator still works if its models are already installed.");
      return false;
    }
    const state = await response.json();
    return describeModels(state);
  } catch (failure) {
    showModelsError("The model list could not be read. The translator still works if its models are already installed.");
    return false;
  }
}

function showModelsError(message) {
  modelsBox.hidden = false;
  modelsNote.textContent = message;
  modelsList.replaceChildren();
  modelsInstall.hidden = true;
}

function stopPollingModels() {
  modelsPolling = false;
  clearInterval(modelsTimer);
  modelsTimer = null;
  modelsInstall.disabled = false;
  modelsInstall.textContent = "Download models";
}

async function downloadModels() {
  if (modelsPolling) {
    return;
  }
  modelsPolling = true;
  modelsInstall.disabled = true;
  modelsInstall.textContent = "Downloading…";
  modelsNote.textContent = "Downloading the models. This takes a while on a slow connection; the page stays usable meanwhile.";
  try {
    const response = await fetch("/models", {
      method: "POST",
      headers: { Authorization: "Bearer " + token },
    });
    const detail = await response.json().catch(() => ({}));
    if (response.status === 409) {
      stopPollingModels();
      await loadModels();
      return;
    }
    if (!response.ok) {
      stopPollingModels();
      modelsNote.textContent = detail.error || "The download could not start. Try again, or run parlo app once more.";
      return;
    }
    modelsTimer = setInterval(async () => {
      const ready = await loadModels();
      if (ready) {
        stopPollingModels();
        modelsNote.textContent = "The models are installed. If the translator does not answer, reload this page.";
      }
    }, 5000);
  } catch (failure) {
    stopPollingModels();
    modelsNote.textContent = "The download could not start. Try again, or run parlo app once more.";
  }
}

modelsInstall.addEventListener("click", () => downloadModels());

function formatClock(seconds) {
  const whole = Math.max(0, Math.floor(seconds || 0));
  return Math.floor(whole / 60) + ":" + String(whole % 60).padStart(2, "0");
}

function describeBytes(bytes) {
  const megabytes = (bytes || 0) / (1024 * 1024);
  if (megabytes < 0.1) {
    return "less than 0.1 MB";
  }
  if (megabytes < 1024) {
    return megabytes.toFixed(megabytes < 10 ? 1 : 0) + " MB";
  }
  return (megabytes / 1024).toFixed(1) + " GB";
}

function setDubStatus(message) {
  dubStatus.textContent = message || "";
  dubStatus.hidden = !message;
}

function chooseFile(file) {
  if (!file || dubbing) {
    return;
  }
  chosenFile = file;
  dubFile.textContent = file.name + " · " + describeBytes(file.size);
  dubStart.disabled = false;
  dubResult.hidden = true;
  clearError();
  setDubStatus("");
}

function describeDubError(detail) {
  if (/ffmpeg/i.test(detail || "")) {
    return "This machine is missing ffmpeg, the tool Parlo uses to read video. Install ffmpeg, then try again.";
  }
  if (/no speech/i.test(detail || "")) {
    return "No speech was found in that file, so there was nothing to dub.";
  }
  return detail || "The file could not be dubbed. Check that the models are installed and try again.";
}

function showDub(result) {
  dubDownload.href = "/dub/" + encodeURIComponent(result.id);
  dubDownload.textContent = "Download " + (result.name || "the dubbed file");
  const lines = result.lines || [];
  dubCount.textContent =
    lines.length +
    (lines.length === 1 ? " line" : " lines") +
    " · " +
    formatClock(result.seconds) +
    " of audio";
  dubLines.replaceChildren();
  for (const line of lines) {
    const item = document.createElement("li");
    item.className = "dub-line";

    const time = document.createElement("span");
    time.className = "dub-time";
    time.textContent = formatClock(line.start);

    const body = document.createElement("div");
    const translated = document.createElement("p");
    translated.className = "dub-translated";
    translated.textContent = line.translated || "";
    const source = document.createElement("p");
    source.className = "dub-source";
    source.textContent = line.source || "";
    body.append(translated, source);

    item.append(time, body);
    dubLines.append(item);
  }
  dubResult.hidden = false;
}

async function startDub() {
  if (dubbing || !chosenFile) {
    return;
  }
  if (recording) {
    showError("Let go of the record button before dubbing a file.");
    return;
  }
  clearError();
  clearTranslation();
  stopPlayback();
  dubbing = true;
  dubStart.disabled = true;
  dubStart.textContent = "Dubbing…";
  dubResult.hidden = true;

  const started = Date.now();
  const tick = () =>
    setDubStatus(
      "Dubbing locally, " +
        formatClock((Date.now() - started) / 1000) +
        " elapsed. This can take a few minutes.",
    );
  tick();
  dubTimer = setInterval(tick, 1000);

  try {
    const response = await fetch("/dub", {
      method: "POST",
      headers: {
        "Content-Type": "application/octet-stream",
        "X-Parlo-Filename": encodeURIComponent(chosenFile.name),
        Authorization: "Bearer " + token,
      },
      body: chosenFile,
    });
    const detail = await response.json().catch(() => ({}));
    if (response.status === 401) {
      throw new Error("This page lost its access token. Start parlo app again and open the link it prints.");
    }
    if (!response.ok) {
      throw new Error(describeDubError(detail.error));
    }
    showDub(detail);
    setDubStatus("Dubbed in " + formatClock((Date.now() - started) / 1000) + ". Download it above, or keep using Parlo.");
  } catch (failure) {
    setDubStatus("");
    showError(failure.message || "The file could not be dubbed. Check that the models are installed and try again.");
  } finally {
    clearInterval(dubTimer);
    dubTimer = null;
    dubbing = false;
    dubStart.disabled = !chosenFile;
    dubStart.textContent = "Dub this file";
  }
}

dropZone.addEventListener("click", () => fileInput.click());

dropZone.addEventListener("keydown", (event) => {
  if (event.key === "Enter" || event.key === " ") {
    event.preventDefault();
    fileInput.click();
  }
});

fileInput.addEventListener("change", () => {
  chooseFile(fileInput.files && fileInput.files[0]);
  fileInput.value = "";
});

dubStart.addEventListener("click", () => startDub());

document.addEventListener("dragover", (event) => {
  if (!event.dataTransfer) {
    return;
  }
  event.preventDefault();
  dropZone.classList.add("over");
});

document.addEventListener("dragleave", (event) => {
  if (event.relatedTarget) {
    return;
  }
  dropZone.classList.remove("over");
});

document.addEventListener("drop", (event) => {
  event.preventDefault();
  dropZone.classList.remove("over");
  const file = event.dataTransfer && event.dataTransfer.files && event.dataTransfer.files[0];
  chooseFile(file);
  if (file) {
    startDub();
  }
});

talk.addEventListener("pointerdown", (event) => {
  event.preventDefault();
  talk.focus({ preventScroll: true });
  begin();
});

window.addEventListener("pointerup", () => finish());
window.addEventListener("pointercancel", () => finish());

talk.addEventListener("contextmenu", (event) => event.preventDefault());

talk.addEventListener("keydown", (event) => {
  if ((event.key === " " || event.key === "Enter") && !event.repeat) {
    event.preventDefault();
    begin();
  }
});

talk.addEventListener("keyup", (event) => {
  if (event.key === " " || event.key === "Enter") {
    event.preventDefault();
    finish();
  }
});

installButton.addEventListener("click", () => installUpdate());

ensureAudio().catch((failure) => {
  micEl.textContent = "blocked";
  showError(describeMicrophoneError(failure));
});
refreshHealth();
setInterval(refreshHealth, 5000);
checkLocalUpdate().then(() => checkForUpdate().catch(() => {}));
loadModels();
