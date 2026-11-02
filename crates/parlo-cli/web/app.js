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
const stateEl = document.getElementById("status-state");
const uptimeEl = document.getElementById("status-uptime");
const micEl = document.getElementById("status-mic");
const latencyEl = document.getElementById("status-latency");
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
const settingsFrom = document.getElementById("settings-from");
const settingsTo = document.getElementById("settings-to");
const settingsStt = document.getElementById("settings-stt");
const settingsSwap = document.getElementById("settings-swap");
const settingsApply = document.getElementById("settings-apply");
const settingsUnload = document.getElementById("settings-unload");
const settingsNote = document.getElementById("settings-note");
const intelligenceMode = document.getElementById("intelligence-mode");
const intelligenceClean = document.getElementById("intelligence-clean");
const intelligenceProvider = document.getElementById("intelligence-provider");
const intelligenceModel = document.getElementById("intelligence-model");
const intelligenceModels = document.getElementById("intelligence-models");
const intelligenceAllow = document.getElementById("intelligence-allow");
const intelligenceBaseUrl = document.getElementById("intelligence-base-url");
const intelligenceLocalUrl = document.getElementById("intelligence-local-url");
const intelligenceLocalModel = document.getElementById("intelligence-local-model");
const intelligenceApply = document.getElementById("intelligence-apply");
const intelligenceNote = document.getElementById("intelligence-note");
const intelligenceKey = document.getElementById("intelligence-key");
const intelligenceKeySave = document.getElementById("intelligence-key-save");
const intelligenceKeyRemove = document.getElementById("intelligence-key-remove");
const intelligenceKeyNote = document.getElementById("intelligence-key-note");
const intelligenceTest = document.getElementById("intelligence-test");
const intelligenceTestNote = document.getElementById("intelligence-test-note");
const welcomeBox = document.getElementById("welcome");
const welcomeInstall = document.getElementById("welcome-install");
const modelsDictation = document.getElementById("models-dictation");
const welcomeStatus = document.getElementById("welcome-status");
const liveButton = document.getElementById("live");
const liveNote = document.getElementById("live-note");
const dictateButton = document.getElementById("dictate");
const dictateNote = document.getElementById("dictate-note");

const TARGET_RATE = 16000;
const MAX_SECONDS = 30;
const MIN_SECONDS = 0.25;
const LIVE_SECONDS = 2.5;
const LIVE_MIN_SECONDS = 0.6;
const LIVE_MAX_SECONDS = 8;
const LIVE_FAILURES = 3;

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
let welcomeTimer = null;
let modelsDownloading = false;
let chosenFile = null;
let dubbing = false;
let dubTimer = null;
let savedSettings = { from: "pt", to: "en", stt: "parakeet" };
let settingsBusy = false;
let live = false;
let dictating = false;
let liveTimer = null;
let liveLines = [];
let liveWindows = 0;
let liveFailures = 0;
let playbackDone = null;

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
      if (!recording && !live) {
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
  const done = playbackDone;
  playbackDone = null;
  if (done) {
    done();
  }
}

function playAndWait(blob) {
  return new Promise((resolve) => {
    stopPlayback();
    const url = URL.createObjectURL(blob);
    const audio = new Audio(url);
    player = audio;
    const done = () => {
      URL.revokeObjectURL(url);
      if (player === audio) {
        player = null;
      }
      if (playbackDone === done) {
        playbackDone = null;
      }
      resolve();
    };
    playbackDone = done;
    audio.addEventListener("ended", done, { once: true });
    audio.addEventListener("error", done, { once: true });
    audio.play().catch(() => {
      showError("The translation is ready but the browser blocked playback. Click the page once, then start live again.");
      done();
    });
  });
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
  if (live) {
    showError("Live is on. Stop it before holding the button for a single clip.");
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
  describeButton();

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

  if (dictating) {
    await dictateSpeech(merged, rate, seconds);
    return;
  }

  setProgress("Translating " + seconds.toFixed(1) + " s of speech.");
  try {
    const wav = encodeWav(downsample(merged, rate, TARGET_RATE), TARGET_RATE);
    const startedAt = performance.now();
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
    const waited = (performance.now() - startedAt) / 1000;
    showTranslation(response.headers.get("X-Parlo-Transcript"));
    latencyEl.textContent = waited.toFixed(1) + " s from speech to voice";
    setProgress("Playing the translation.");
    play(audio);
  } catch (failure) {
    setProgress("");
    showError(failure.message || "The translation failed. Check that the engine is running and try again.");
  }
}

function describeButton() {
  label.textContent = dictating ? "Hold to dictate" : "Hold to speak";
  sub.textContent = dictating
    ? "Release to get your words as text"
    : "Release to hear the translation";
}

async function dictateSpeech(merged, rate, seconds) {
  setProgress("Writing down " + seconds.toFixed(1) + " s of speech.");
  try {
    const wav = encodeWav(downsample(merged, rate, TARGET_RATE), TARGET_RATE);
    const startedAt = performance.now();
    const response = await fetch("/dictate", {
      method: "POST",
      headers: { "Content-Type": "audio/wav", Authorization: "Bearer " + token },
      body: wav,
    });
    if (response.status === 401) {
      throw new Error("This page lost its access token. Start parlo app again and open the link it prints.");
    }
    if (!response.ok) {
      const detail = await response.json().catch(() => ({}));
      throw new Error(detail.error || "Parlo answered with status " + response.status + ".");
    }
    const result = await response.json();
    const waited = (performance.now() - startedAt) / 1000;
    const text = (result.text || "").trim();
    if (!text) {
      setProgress("");
      showError("Nothing was recognised. Move closer to the microphone and try again.");
      return;
    }
    translation.textContent = text;
    translation.hidden = false;
    latencyEl.textContent = waited.toFixed(1) + " s from speech to text";
    let copied = false;
    try {
      await navigator.clipboard.writeText(text);
      copied = true;
    } catch (failure) {
      copied = false;
    }
    setProgress(
      copied
        ? "Copied to the clipboard. Paste it wherever you need it."
        : "The text is ready above. Copy it with the keyboard, the clipboard refused it.",
    );
  } catch (failure) {
    setProgress("");
    showError(
      failure.message || "The dictation failed. Check that the engine is running and try again.",
    );
  }
}

function describeLive(message) {
  liveNote.textContent =
    message ||
    (live
      ? "Listening continuously. Each window is translated and played in turn."
      : "Turns on continuous translation for a call. Everything stays on this machine.");
}

function appendLiveLine(encoded) {
  if (!encoded) {
    return;
  }
  try {
    const bytes = Uint8Array.from(atob(encoded), (character) => character.charCodeAt(0));
    const text = new TextDecoder().decode(bytes).trim();
    if (!text) {
      return;
    }
    liveLines.push(text);
    translation.textContent = liveLines.join(" ");
    translation.hidden = false;
  } catch (failure) {
    return;
  }
}

function takeWindow(minimum) {
  const rate = context ? context.sampleRate : 48000;
  const captured = chunks;
  const length = captured.reduce((total, chunk) => total + chunk.length, 0);
  if (!length || length < rate * minimum) {
    return null;
  }
  chunks = [];
  const merged = new Float32Array(length);
  let offset = 0;
  for (const chunk of captured) {
    merged.set(chunk, offset);
    offset += chunk.length;
  }
  const cap = Math.floor(rate * LIVE_MAX_SECONDS);
  const kept = merged.length > cap ? merged.subarray(merged.length - cap) : merged;
  return encodeWav(downsample(kept, rate, TARGET_RATE), TARGET_RATE);
}

async function translateWindow(wav) {
  const startedAt = performance.now();
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
  return { audio, transcript: response.headers.get("X-Parlo-Transcript"), waited: (performance.now() - startedAt) / 1000 };
}

async function liveStep() {
  liveTimer = null;
  if (!live) {
    return;
  }
  const wav = takeWindow(LIVE_SECONDS);
  if (!wav) {
    liveTimer = setTimeout(liveStep, 200);
    return;
  }
  try {
    const result = await translateWindow(wav);
    if (!live) {
      return;
    }
    liveFailures = 0;
    liveWindows += 1;
    latencyEl.textContent = result.waited.toFixed(1) + " s from speech to voice";
    appendLiveLine(result.transcript);
    setProgress(
      "Live, " +
        liveWindows +
        (liveWindows === 1 ? " window" : " windows") +
        " translated, each one played in turn.",
    );
    await playAndWait(result.audio);
  } catch (failure) {
    liveFailures += 1;
    if (liveFailures >= LIVE_FAILURES) {
      stopLive("Live stopped after " + LIVE_FAILURES + " failures in a row. " + (failure.message || ""));
      return;
    }
    showError(failure.message || "One window could not be translated. Parlo is still listening.");
  }
  if (live) {
    liveTimer = setTimeout(liveStep, 0);
  }
}

async function startLive() {
  if (live) {
    return;
  }
  if (dubbing) {
    showError("Wait for the file to finish dubbing before going live.");
    return;
  }
  if (!welcomeBox.hidden || modelsDownloading) {
    showError("Download the models first, then live mode works offline.");
    return;
  }
  if (recording) {
    showError("Release the record button first, then start live mode.");
    return;
  }
  if (dictating) {
    showError("Turn off dictation before going live.");
    return;
  }
  clearError();
  clearTranslation();
  stopPlayback();
  chunks = [];
  liveLines = [];
  liveWindows = 0;
  liveFailures = 0;
  try {
    await ensureAudio();
  } catch (failure) {
    micEl.textContent = "blocked";
    showError(describeMicrophoneError(failure));
    return;
  }
  live = true;
  liveButton.setAttribute("aria-pressed", "true");
  talk.disabled = true;
  micEl.textContent = "listening";
  describeLive();
  setProgress("Live, listening. The first translation arrives after a couple of seconds.");
  clearTimeout(liveTimer);
  liveTimer = setTimeout(liveStep, 200);
}

async function stopLive(message) {
  if (!live) {
    if (message) {
      showError(message);
    }
    return;
  }
  live = false;
  clearTimeout(liveTimer);
  liveTimer = null;
  liveButton.setAttribute("aria-pressed", "false");
  talk.disabled = false;
  micEl.textContent = "ready";
  describeLive();
  const rest = takeWindow(LIVE_MIN_SECONDS);
  chunks = [];
  if (!rest) {
    stopPlayback();
    setProgress("");
  } else {
    setProgress("Live stopped. Translating the last words now.");
    try {
      const result = await translateWindow(rest);
      appendLiveLine(result.transcript);
      setProgress("Live stopped after " + liveWindows + " windows.");
      play(result.audio);
    } catch (failure) {
      setProgress("");
      stopPlayback();
    }
  }
  if (message) {
    showError(message);
  }
}

liveButton.addEventListener("click", () => {
  if (live) {
    stopLive("");
  } else {
    startLive();
  }
});

window.addEventListener("beforeunload", () => {
  clearTimeout(liveTimer);
});

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

const STATE_WORDS = {
  idle: "ready",
  armed: "opening the microphone",
  listening: "listening",
  processing: "processing",
  inserting: "inserting",
  success: "inserted",
  error: "needs attention",
  cancelled: "cancelled",
};

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
  try {
    const response = await fetch("/state", { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    const current = await response.json();
    stateEl.textContent = STATE_WORDS[current.state] || "ready";
  } catch (failure) {
    stateEl.textContent = "unknown";
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

function setWelcomeStatus(message) {
  welcomeStatus.textContent = message || "";
  welcomeStatus.hidden = !message;
}

function startWelcomeClock() {
  stopWelcomeClock();
  const started = Date.now();
  const tick = () =>
    setWelcomeStatus(
      "Downloading models, " +
        formatClock((Date.now() - started) / 1000) +
        " elapsed. This can take a few minutes, and the page can stay open.",
    );
  tick();
  welcomeTimer = setInterval(tick, 1000);
}

function stopWelcomeClock() {
  clearInterval(welcomeTimer);
  welcomeTimer = null;
}

function reflectFirstRun(ready, dictation, downloading) {
  const firstRun = document.body.classList.contains("first-run");
  if (dictation) {
    if (firstRun) {
      stopWelcomeClock();
      setWelcomeStatus("");
      document.body.classList.remove("first-run");
      welcomeBox.hidden = true;
      setProgress(
        ready
          ? "The models are ready. Parlo works offline from now on."
          : "Dictation is ready. Translation and read aloud can be added later, one model at a time.",
      );
    }
    return;
  }
  if (!firstRun) {
    document.body.classList.add("first-run");
    welcomeBox.hidden = false;
  }
  if (downloading) {
    welcomeInstall.hidden = true;
    if (!welcomeTimer) {
      startWelcomeClock();
    }
    return;
  }
  stopWelcomeClock();
  welcomeInstall.hidden = false;
  welcomeInstall.disabled = modelsPolling;
  welcomeInstall.textContent = "Download the dictation model";
  setWelcomeStatus(
    "Dictation needs one download of about 640 MB. Translation and read aloud are separate models you add later, and everything works offline once it is here.",
  );
}

let modelState = { models: [], ready: false, dictation: false, downloading: false };

function describeModels(state) {
  modelState = state;
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

    const action = document.createElement("button");
    action.type = "button";
    action.className = "models-action";
    action.textContent = model.installed ? "Remove" : "Download";
    action.disabled = Boolean(state.downloading);
    action.addEventListener("click", () => modelAction(model, action));

    row.append(name, fact, action);
    modelsList.append(row);
  }
  modelsBox.hidden = false;

  if (state.ready) {
    modelsDownloading = false;
    modelsNote.textContent = "Everything the translator needs is on this machine, so you can speak now.";
    modelsInstall.hidden = true;
    reflectFirstRun(true, true, false);
    return true;
  }

  modelsDownloading = Boolean(state.downloading);
  if (modelsDownloading) {
    modelsNote.textContent =
      "Downloading models, this can take a few minutes on a normal connection. You can leave this page open.";
    modelsInstall.hidden = true;
    modelsDictation.hidden = true;
    reflectFirstRun(false, Boolean(state.dictation), true);
    return false;
  }

  const missing = (state.models || []).filter((model) => !model.installed);
  const byFeature = new Map();
  for (const model of missing) {
    const feature = model.feature || "A feature";
    if (!byFeature.has(feature)) {
      byFeature.set(feature, []);
    }
    byFeature.get(feature).push(model.name);
  }
  const sentences = Array.from(byFeature, ([feature, names]) => feature + " needs " + names.join(" and "));
  modelsNote.textContent = sentences.length
    ? "Missing: " + sentences.join("; ") + ". Each one can be downloaded on its own, and nothing else comes with it."
    : "Some models are missing. They are a one time download and stay on this machine.";
  modelsInstall.hidden = false;
  modelsDictation.hidden = missing.every((model) => model.installed) || !missing.some((model) => model.id === "parakeet");
  modelsInstall.disabled = modelsPolling;
  reflectFirstRun(false, Boolean(state.dictation), false);
  return false;
}

async function modelAction(model, button) {
  const path = (model.installed ? "/models/remove/" : "/models/download/") + model.id;
  button.disabled = true;
  try {
    const response = await fetch(path, {
      method: "POST",
      headers: { Authorization: "Bearer " + token },
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      modelsNote.textContent =
        detail.error || "That could not be done. Close Parlo, open it again and try once more.";
      button.disabled = false;
      return;
    }
    modelsNote.textContent = model.installed
      ? "Removed. The feature that needed it will say so before it is used."
      : "Downloading " + model.name + ". This page stays usable meanwhile.";
    await loadModels();
  } catch (failure) {
    modelsNote.textContent = "That could not be done. Check that Parlo is still running.";
    button.disabled = false;
  }
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
  stopWelcomeClock();
  welcomeInstall.disabled = false;
  welcomeInstall.textContent = "Download the dictation model";
}

async function downloadModels(path = "/models") {
  if (modelsPolling) {
    return;
  }
  modelsPolling = true;
  modelsDownloading = false;
  modelsInstall.disabled = true;
  modelsInstall.textContent = "Downloading…";
  welcomeInstall.disabled = true;
  welcomeInstall.textContent = "Downloading…";
  startWelcomeClock();
  modelsNote.textContent =
    path === "/models/dictation"
      ? "Downloading the recogniser. Dictation works as soon as it lands; translation and voices stay optional."
      : "Downloading the models. This takes a while on a slow connection; the page stays usable meanwhile.";
  try {
    const response = await fetch(path, {
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
      setWelcomeStatus(modelsNote.textContent);
      return;
    }
    modelsTimer = setInterval(async () => {
      const wasDownloading = modelsDownloading;
      const ready = await loadModels();
      const done = path === "/models/dictation" ? Boolean(modelState.dictation) : ready;
      if (done) {
        stopPollingModels();
        modelsNote.textContent = "The models are installed. If the translator does not answer, reload this page.";
        return;
      }
      if (wasDownloading && !modelsDownloading) {
        stopPollingModels();
        modelsNote.textContent =
          "The download stopped before the models were complete. Check the connection, then press Download models again.";
        setWelcomeStatus(
          "The download stopped before the models were complete. Check the connection, then press Download models again.",
        );
      }
    }, 5000);
  } catch (failure) {
    stopPollingModels();
    modelsNote.textContent = "The download could not start. Try again, or run parlo app once more.";
    setWelcomeStatus(modelsNote.textContent);
  }
}

modelsInstall.addEventListener("click", () => downloadModels());
welcomeInstall.addEventListener("click", () => downloadModels("/models/dictation"));
modelsDictation.addEventListener("click", () => downloadModels("/models/dictation"));

const LANGUAGE_NAMES = { pt: "Portuguese", en: "English" };
const settingsWrite = document.getElementById("settings-write");
const settingsShortcut = document.getElementById("settings-shortcut");
const settingsShortcutSave = document.getElementById("settings-shortcut-save");
const meetingTitle = document.getElementById("meeting-title");
const meetingText = document.getElementById("meeting-text");
const meetingImport = document.getElementById("meeting-import");
const meetingList = document.getElementById("meeting-list");
const meetingNote = document.getElementById("meetings-note");
const vocabularyKind = document.getElementById("vocabulary-kind");
const vocabularyWrong = document.getElementById("vocabulary-wrong");
const vocabularyRight = document.getElementById("vocabulary-right");
const vocabularyAdd = document.getElementById("vocabulary-add");
const vocabularyList = document.getElementById("vocabulary-list");
const vocabularyNote = document.getElementById("vocabulary-note");
const meetingLive = document.getElementById("meeting-live");
const meetingLiveNote = document.getElementById("meeting-live-note");
const catalogList = document.getElementById("catalog-list");
const catalogNote = document.getElementById("catalog-note");
const historyList = document.getElementById("history-list");
const historyNote = document.getElementById("history-note");
const historyClear = document.getElementById("history-clear");

function describePair(from, to) {
  return (LANGUAGE_NAMES[from] || from) + " to " + (LANGUAGE_NAMES[to] || to);
}

function settingsChanged() {
  return (
    settingsFrom.value !== savedSettings.from ||
    settingsTo.value !== savedSettings.to ||
    settingsStt.value !== savedSettings.stt ||
    settingsWrite.value !== savedSettings.write
  );
}

function refreshApply() {
  settingsApply.disabled = settingsBusy || !settingsChanged();
}

function reflectSettings(values) {
  settingsFrom.value = values.from || "pt";
  settingsTo.value = values.to || "en";
  settingsStt.value = values.stt || "parakeet";
  settingsWrite.value = values.write || "";
  settingsShortcut.value = values.shortcut || "Ctrl+Alt+D";
  savedSettings = {
    from: settingsFrom.value,
    to: settingsTo.value,
    stt: settingsStt.value,
    write: settingsWrite.value,
  };
  const pair = describePair(savedSettings.from, savedSettings.to);
  settingsNote.textContent = savedSettings.write
    ? pair + ", dictation writes " + (LANGUAGE_NAMES[savedSettings.write] || savedSettings.write)
    : pair + ", dictation keeps the language spoken";
  refreshApply();
}

async function saveShortcut() {
  const wanted = settingsShortcut.value.trim();
  try {
    const response = await fetch("/settings", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({ shortcut: wanted }),
    });
    const answer = await response.json().catch(() => ({}));
    if (!response.ok) {
      settingsNote.textContent = answer.error || "That shortcut cannot be used.";
      return;
    }
    settingsShortcut.value = answer.shortcut || wanted;
    settingsNote.textContent = "Saved. Restart Parlo so the new shortcut is registered.";
  } catch (failure) {
    settingsNote.textContent = "The shortcut could not be saved.";
  }
}

settingsShortcutSave.addEventListener("click", () => saveShortcut());

async function loadSettings() {
  try {
    const response = await fetch("/settings", { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    reflectSettings(await response.json());
  } catch (failure) {
    settingsNote.textContent = describePair(savedSettings.from, savedSettings.to);
  }
}

function keepPairValid(changed) {
  if (settingsFrom.value !== settingsTo.value) {
    return;
  }
  const other = settingsFrom.value === "pt" ? "en" : "pt";
  if (changed === "from") {
    settingsTo.value = other;
  } else {
    settingsFrom.value = other;
  }
}

function lockSettings(locked) {
  settingsBusy = locked;
  settingsFrom.disabled = locked;
  settingsTo.disabled = locked;
  settingsStt.disabled = locked;
  settingsWrite.disabled = locked;
  settingsSwap.disabled = locked;
  settingsApply.disabled = locked;
}

async function saveSettings() {
  if (settingsBusy || !settingsChanged()) {
    return;
  }
  if (recording) {
    showError("Release the record button before changing the languages.");
    return;
  }
  if (dubbing) {
    showError("Wait for the file to finish dubbing before changing the languages.");
    return;
  }
  if (live) {
    showError("Stop live mode before changing the languages.");
    return;
  }

  clearError();
  const previous = {
    from: savedSettings.from,
    to: savedSettings.to,
    stt: savedSettings.stt,
    write: savedSettings.write,
  };
  lockSettings(true);
  settingsApply.textContent = "Loading models…";
  settingsNote.textContent = "Parlo is loading the models again. This takes about twenty seconds.";

  try {
    const response = await fetch("/settings", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer " + token,
      },
      body: JSON.stringify({
        from: settingsFrom.value,
        to: settingsTo.value,
        stt: settingsStt.value,
        write: settingsWrite.value,
      }),
    });
    const detail = await response.json().catch(() => ({}));
    if (response.status === 401) {
      throw new Error("This page lost its access token. Start parlo app again and open the link it prints.");
    }
    if (!response.ok) {
      throw new Error(detail.error || "Parlo could not change the languages.");
    }
    reflectSettings(detail);
    settingsNote.textContent = settingsNote.textContent.replace(/, dictation.*$/, "") + ", ready.";
  } catch (failure) {
    reflectSettings(previous);
    showError(failure.message || "Parlo could not change the languages. Try again.");
  } finally {
    lockSettings(false);
    settingsApply.textContent = "Apply";
    refreshApply();
  }
}

settingsFrom.addEventListener("change", () => {
  keepPairValid("from");
  refreshApply();
});

settingsTo.addEventListener("change", () => {
  keepPairValid("to");
  refreshApply();
});

settingsStt.addEventListener("change", () => refreshApply());

settingsWrite.addEventListener("change", () => refreshApply());

settingsSwap.addEventListener("click", () => {
  if (settingsBusy) {
    return;
  }
  const from = settingsFrom.value;
  settingsFrom.value = settingsTo.value;
  settingsTo.value = from;
  refreshApply();
});

settingsApply.addEventListener("click", () => saveSettings());

settingsUnload.addEventListener("change", async () => {
  const minutes = Number(settingsUnload.value);
  try {
    const response = await fetch("/settings", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({ unload: minutes }),
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "That delay cannot be used.");
    }
    settingsNote.textContent =
      minutes === 0
        ? "The models stay loaded until Parlo closes."
        : "The models are freed after " + minutes + " minutes without use.";
  } catch (failure) {
    settingsNote.textContent = failure.message || "That delay cannot be used.";
  }
});

let intelligenceProviders = [];
let intelligenceStore = true;

function intelligenceHeaders() {
  return { "Content-Type": "application/json", Authorization: "Bearer " + token };
}

function intelligenceLabel(name) {
  const found = intelligenceProviders.find((provider) => provider.name === name);
  return found ? found.label : name;
}

function fillIntelligenceModels() {
  const chosen = intelligenceProviders.find(
    (provider) => provider.name === intelligenceProvider.value
  );
  intelligenceModels.replaceChildren();
  ((chosen && chosen.models) || []).forEach((model) => {
    const option = document.createElement("option");
    option.value = model;
    intelligenceModels.append(option);
  });
  if (chosen && !intelligenceModel.value && chosen.models.length) {
    intelligenceModel.value = chosen.models[0];
  }
}

function describeIntelligenceKey(names) {
  if (!names.length) {
    intelligenceKeyNote.textContent = "Choose a provider to save a key.";
    return;
  }
  const saved = names.filter((name) => name.key).map((name) => name.label);
  const missing = names.filter((name) => !name.key).map((name) => name.label);
  if (!intelligenceStore) {
    intelligenceKeyNote.textContent =
      "This computer has no credential store Parlo can use, so a key cannot be saved here.";
    return;
  }
  if (saved.length && missing.length) {
    intelligenceKeyNote.textContent =
      "A key is saved for " + saved.join(", ") + ". Nothing is saved for " + missing.join(", ") + ".";
    return;
  }
  intelligenceKeyNote.textContent = saved.length
    ? "A key is saved for " + saved.join(", ") + ". Parlo cannot show it again."
    : "No key is saved yet. Parlo never shows a key after saving it.";
}

function renderIntelligence(state) {
  const settings = state.settings || {};
  intelligenceProviders = state.providers || [];
  intelligenceStore = state.store !== false;
  intelligenceMode.value = settings.mode || "auto";
  intelligenceClean.value = settings.clean || "light";
  intelligenceProvider.replaceChildren();
  const none = document.createElement("option");
  none.value = "";
  none.textContent = "None";
  intelligenceProvider.append(none);
  intelligenceProviders.forEach((provider) => {
    const option = document.createElement("option");
    option.value = provider.name;
    option.textContent = provider.label;
    intelligenceProvider.append(option);
  });
  intelligenceProvider.value = settings.provider || "";
  intelligenceModel.value = settings.model || "";
  intelligenceAllow.checked = settings.allow_cloud === true;
  intelligenceBaseUrl.value = settings.base_url || "";
  intelligenceLocalUrl.value = settings.local_url || "";
  intelligenceLocalModel.value = settings.local_model || "";
  fillIntelligenceModels();
  describeIntelligenceKey(intelligenceProviders);
  intelligenceNote.textContent = state.error
    ? "Parlo could not read every setting: " + state.error
    : "Saved settings apply to the next thing you dictate.";
}

async function loadIntelligence() {
  try {
    const response = await fetch("/intelligence", { cache: "no-store" });
    if (!response.ok) {
      intelligenceNote.textContent = "The cleanup settings could not be read.";
      return;
    }
    renderIntelligence(await response.json());
  } catch (failure) {
    intelligenceNote.textContent = "The cleanup settings could not be read.";
  }
}

async function saveIntelligence() {
  const chosen = intelligenceProvider.value;
  intelligenceApply.disabled = true;
  intelligenceNote.textContent = "Saving.";
  try {
    const response = await fetch("/intelligence", {
      method: "POST",
      headers: intelligenceHeaders(),
      body: JSON.stringify({
        mode: intelligenceMode.value,
        clean: intelligenceClean.value,
        allow_cloud: intelligenceAllow.checked,
        provider: chosen,
        model: intelligenceModel.value.trim(),
        base_url: intelligenceBaseUrl.value.trim(),
        local_url: intelligenceLocalUrl.value.trim(),
        local_model: intelligenceLocalModel.value.trim(),
      }),
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "Parlo could not save these settings.");
    }
    renderIntelligence(detail);
    intelligenceNote.textContent = "Saved. It applies to the next thing you dictate.";
  } catch (failure) {
    intelligenceNote.textContent = failure.message || "Parlo could not save these settings.";
  } finally {
    intelligenceApply.disabled = false;
  }
}

async function saveIntelligenceKey() {
  const wanted = intelligenceKey.value.trim();
  if (!wanted) {
    intelligenceKeyNote.textContent = "Paste the key before saving it.";
    return;
  }
  if (!intelligenceProvider.value) {
    intelligenceKeyNote.textContent = "Choose a provider before saving a key.";
    return;
  }
  try {
    const response = await fetch("/intelligence/key", {
      method: "POST",
      headers: intelligenceHeaders(),
      body: JSON.stringify({ provider: intelligenceProvider.value, key: wanted }),
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "The key could not be saved.");
    }
    intelligenceKey.value = "";
    await loadIntelligence();
    intelligenceKeyNote.textContent =
      "Saved for " + intelligenceLabel(intelligenceProvider.value) + ". Parlo cannot show it again.";
  } catch (failure) {
    intelligenceKeyNote.textContent = failure.message || "The key could not be saved.";
  }
}

async function removeIntelligenceKey() {
  if (!intelligenceProvider.value) {
    intelligenceKeyNote.textContent = "Choose a provider before removing a key.";
    return;
  }
  try {
    const response = await fetch("/intelligence/key/delete", {
      method: "POST",
      headers: intelligenceHeaders(),
      body: JSON.stringify({ provider: intelligenceProvider.value }),
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "The key could not be removed.");
    }
    intelligenceKey.value = "";
    await loadIntelligence();
    intelligenceKeyNote.textContent =
      "The saved key for " + intelligenceLabel(intelligenceProvider.value) + " is gone.";
  } catch (failure) {
    intelligenceKeyNote.textContent = failure.message || "The key could not be removed.";
  }
}

async function testIntelligence() {
  intelligenceTest.disabled = true;
  intelligenceTestNote.textContent = "Sending one sentence, this can take a few seconds.";
  try {
    const response = await fetch("/intelligence/test", {
      method: "POST",
      headers: intelligenceHeaders(),
      body: "{}",
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "Parlo could not run the test.");
    }
    const engine = detail.engine === "cloud" || detail.engine === "local" ? detail.engine : "the rules";
    const model = detail.model ? " with " + detail.model : "";
    intelligenceTestNote.textContent = detail.fallback
      ? "Tested " + engine + model + ", which declined, so " + detail.reason + "."
      : "Written by " + engine + model + ": " + detail.text;
  } catch (failure) {
    intelligenceTestNote.textContent = failure.message || "Parlo could not run the test.";
  } finally {
    intelligenceTest.disabled = false;
  }
}

intelligenceProvider.addEventListener("change", () => fillIntelligenceModels());
intelligenceApply.addEventListener("click", () => saveIntelligence());
intelligenceKeySave.addEventListener("click", () => saveIntelligenceKey());
intelligenceKeyRemove.addEventListener("click", () => removeIntelligenceKey());
intelligenceTest.addEventListener("click", () => testIntelligence());

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
  if (live) {
    showError("Stop live mode before dubbing a file.");
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

dictateButton.addEventListener("click", () => {
  if (live) {
    showError("Stop live mode before dictating.");
    return;
  }
  dictating = !dictating;
  dictateButton.setAttribute("aria-pressed", dictating ? "true" : "false");
  dictateNote.textContent = dictating
    ? "Dictation is on. Hold the button, speak, and let go: your words arrive as text in the clipboard, with no translation and no voice."
    : "Holding the button writes down what you say instead of translating it. The text goes to your clipboard.";
  clearError();
  clearTranslation();
  describeButton();
});

talk.addEventListener("pointerdown", (event) => {
  event.preventDefault();
  talk.focus({ preventScroll: true });
  begin();
});

window.addEventListener("pointerup", () => finish());
window.addEventListener("pointercancel", () => finish());

function openSection() {
  const fragment = (location.hash || "").replace("#", "");
  if (fragment === "live") {
    startLive();
    return;
  }
  if (fragment === "settings") {
    const section = document.getElementById("settings");
    if (!section) {
      return;
    }
    section.scrollIntoView({ block: "center" });
    settingsFrom.focus();
  }
}

async function loadMeetings() {
  try {
    const response = await fetch("/meetings", {
      cache: "no-store",
      headers: { Authorization: "Bearer " + token },
    });
    if (!response.ok) {
      return;
    }
    renderMeetings(await response.json());
  } catch (failure) {
    meetingNote.textContent = "The meetings could not be read.";
  }
}

function renderMeetings(meetings) {
  meetingList.replaceChildren();
  if (!meetings.length) {
    meetingNote.textContent =
      "No transcript on this computer yet. Paste one above, with a speaker at the start of each line.";
    return;
  }
  for (const meeting of meetings) {
    const term = document.createElement("dt");
    const day = new Date((meeting.started_at || 0) * 1000);
    term.textContent =
      (meeting.title || "Untitled") +
      " · " +
      day.toLocaleDateString() +
      " · " +
      meeting.segments.length +
      " lines";
    const detail = document.createElement("dd");
    for (const speaker of meeting.speakers) {
      const chip = document.createElement("button");
      chip.type = "button";
      chip.className = "swap";
      chip.textContent = speaker;
      chip.title = "Name this speaker everywhere in this meeting";
      chip.addEventListener("click", () => nameSpeaker(meeting, speaker));
      detail.append(chip);
    }
    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "swap";
    remove.textContent = "Delete";
    remove.addEventListener("click", () => deleteMeeting(meeting.id));
    detail.append(remove);
    meetingList.append(term, detail);
  }
  meetingNote.textContent =
    meetings.length === 1
      ? "1 meeting kept on this computer."
      : meetings.length + " meetings kept on this computer.";
}

async function importMeeting() {
  const text = meetingText.value.trim();
  if (!text) {
    meetingNote.textContent = "Paste a transcript first.";
    return;
  }
  try {
    const response = await fetch("/meeting/import", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({ title: meetingTitle.value, text }),
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "The transcript was refused.");
    }
    meetingText.value = "";
    await loadMeetings();
  } catch (failure) {
    meetingNote.textContent = failure.message || "The transcript could not be imported.";
  }
}

async function nameSpeaker(meeting, speaker) {
  const chosen = window.prompt(
    "Name for " + speaker + "? Leave it empty to go back to the marker.",
    /^Speaker \d+$/.test(speaker) ? "" : speaker,
  );
  if (chosen === null) {
    return;
  }
  const line = meeting.segments.find((segment) => segment.speaker === speaker);
  try {
    const response = await fetch("/meeting/name", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({
        id: meeting.id,
        speaker,
        name: chosen,
        evidence: (line && line.text) || "",
      }),
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "The speaker could not be named.");
    }
    await loadMeetings();
  } catch (failure) {
    meetingNote.textContent = failure.message || "The speaker could not be named.";
  }
}

async function deleteMeeting(id) {
  try {
    const response = await fetch("/meeting/delete", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({ id }),
    });
    if (!response.ok) {
      throw new Error("status " + response.status);
    }
    await loadMeetings();
  } catch (failure) {
    meetingNote.textContent = "The meeting could not be deleted. Try again.";
  }
}

function vocabularyHeaders() {
  return { "Content-Type": "application/json", Authorization: "Bearer " + token };
}

function renderVocabulary(profile) {
  vocabularyList.replaceChildren();
  const rows = [];
  for (const [wrong, right] of Object.entries(profile.terms || {})) {
    rows.push({ kind: "term", wrong, right });
  }
  for (const [wrong, right] of Object.entries(profile.abbreviations || {})) {
    rows.push({ kind: "abbreviation", wrong, right });
  }
  if (!rows.length) {
    vocabularyNote.textContent =
      "Nothing here yet. A name the recogniser keeps getting wrong is the first thing worth adding.";
    return;
  }
  for (const row of rows) {
    const term = document.createElement("dt");
    term.textContent = (row.kind === "term" ? "Name: " : "Short form: ") + row.wrong;
    const detail = document.createElement("dd");
    const written = document.createElement("span");
    written.textContent = "becomes " + row.right;
    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "swap";
    remove.textContent = "Remove";
    remove.addEventListener("click", () => removeVocabulary(row));
    detail.append(written, remove);
    vocabularyList.append(term, detail);
  }
  const count = rows.length === 1 ? "1 word kept" : rows.length + " words kept";
  vocabularyNote.textContent = count + " on this computer.";
}

async function loadVocabulary() {
  try {
    const response = await fetch("/vocabulary", {
      cache: "no-store",
      headers: { Authorization: "Bearer " + token },
    });
    if (!response.ok) {
      vocabularyNote.textContent = "The words could not be read.";
      return;
    }
    renderVocabulary(await response.json());
  } catch (failure) {
    vocabularyNote.textContent = "The words could not be read.";
  }
}

async function addVocabulary() {
  const wrong = vocabularyWrong.value.trim();
  const right = vocabularyRight.value.trim();
  if (!wrong || !right) {
    vocabularyNote.textContent = "Both sides are needed: what Parlo hears and what it should write.";
    return;
  }
  try {
    const response = await fetch("/vocabulary", {
      method: "POST",
      headers: vocabularyHeaders(),
      body: JSON.stringify({ kind: vocabularyKind.value, wrong, right }),
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "That word could not be saved.");
    }
    vocabularyWrong.value = "";
    vocabularyRight.value = "";
    renderVocabulary(detail);
  } catch (failure) {
    vocabularyNote.textContent = failure.message || "That word could not be saved.";
  }
}

async function removeVocabulary(row) {
  try {
    const response = await fetch("/vocabulary/delete", {
      method: "POST",
      headers: vocabularyHeaders(),
      body: JSON.stringify({ kind: row.kind, wrong: row.wrong }),
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "That word could not be removed.");
    }
    renderVocabulary(detail);
  } catch (failure) {
    vocabularyNote.textContent = failure.message || "That word could not be removed.";
  }
}

vocabularyAdd.addEventListener("click", () => addVocabulary());
vocabularyRight.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    addVocabulary();
  }
});

const MEETING_SECONDS = 5;

let meetingRecording = false;
let meetingId = "";
let meetingStarted = 0;
let meetingTimer = null;
let meetingLines = 0;
let meetingFailures = 0;

function meetingStatus(last) {
  if (!meetingRecording) {
    meetingLiveNote.textContent =
      "Records the microphone into a meeting transcript, a few seconds behind. Sound from another application is not captured yet.";
    return;
  }
  const kept = meetingLines === 1 ? "1 line kept" : meetingLines + " lines kept";
  meetingLiveNote.textContent =
    "Recording, " + kept + ". " + (last ? "Last: " + last : "Speak normally.");
}

async function meetingSegment(wav) {
  const response = await fetch("/dictate", {
    method: "POST",
    headers: { "Content-Type": "audio/wav", Authorization: "Bearer " + token },
    body: wav,
  });
  if (!response.ok) {
    const detail = await response.json().catch(() => ({}));
    throw new Error(detail.error || "Parlo answered with status " + response.status + ".");
  }
  const result = await response.json();
  const text = (result.text || "").trim();
  if (!text) {
    return "";
  }
  const saved = await fetch("/meeting/segment", {
    method: "POST",
    headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
    body: JSON.stringify({
      id: meetingId,
      speaker: "Me",
      text,
      at_ms: Math.max(0, Math.round(performance.now() - meetingStarted)),
    }),
  });
  if (!saved.ok) {
    const detail = await saved.json().catch(() => ({}));
    throw new Error(detail.error || "The line could not be kept.");
  }
  return text;
}

async function meetingStep() {
  meetingTimer = null;
  if (!meetingRecording) {
    return;
  }
  const wav = takeWindow(MEETING_SECONDS);
  if (!wav) {
    meetingTimer = setTimeout(meetingStep, 300);
    return;
  }
  try {
    const text = await meetingSegment(wav);
    meetingFailures = 0;
    if (text) {
      meetingLines += 1;
    }
    meetingStatus(text);
  } catch (failure) {
    meetingFailures += 1;
    if (meetingFailures >= LIVE_FAILURES) {
      await stopMeeting(
        "Recording stopped after " + LIVE_FAILURES + " failures in a row. " + (failure.message || ""),
      );
      return;
    }
    showError(failure.message || "One window could not be written down. The meeting is still recording.");
  }
  if (meetingRecording) {
    meetingTimer = setTimeout(meetingStep, 0);
  }
}

async function startMeeting() {
  if (meetingRecording) {
    return;
  }
  if (live) {
    showError("Stop live mode before recording a meeting.");
    return;
  }
  if (recording || dictating) {
    showError("Finish the current dictation before recording a meeting.");
    return;
  }
  if (!welcomeBox.hidden || modelsDownloading) {
    showError("Download the models first; a meeting is written down on this machine.");
    return;
  }
  clearError();
  try {
    await ensureAudio();
  } catch (failure) {
    micEl.textContent = "blocked";
    showError(describeMicrophoneError(failure));
    return;
  }
  try {
    const response = await fetch("/meeting/start", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({ title: meetingTitle.value.trim() || "Meeting" }),
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "The meeting could not be started.");
    }
    meetingId = detail.id;
  } catch (failure) {
    showError(failure.message || "The meeting could not be started.");
    return;
  }
  meetingRecording = true;
  meetingStarted = performance.now();
  meetingLines = 0;
  meetingFailures = 0;
  chunks = [];
  meetingLive.setAttribute("aria-pressed", "true");
  micEl.textContent = "recording";
  meetingStatus("");
  clearTimeout(meetingTimer);
  meetingTimer = setTimeout(meetingStep, 200);
}

async function stopMeeting(message) {
  if (!meetingRecording) {
    if (message) {
      showError(message);
    }
    return;
  }
  meetingRecording = false;
  clearTimeout(meetingTimer);
  meetingTimer = null;
  meetingLive.setAttribute("aria-pressed", "false");
  micEl.textContent = "ready";
  const rest = takeWindow(LIVE_MIN_SECONDS);
  chunks = [];
  meetingStatus("");
  if (rest) {
    try {
      await meetingSegment(rest);
    } catch (failure) {
      showError(failure.message || "The last words could not be written down.");
    }
  }
  await loadMeetings();
  if (message) {
    showError(message);
  }
}

meetingLive.addEventListener("click", () => {
  if (meetingRecording) {
    stopMeeting("");
  } else {
    startMeeting();
  }
});

meetingImport.addEventListener("click", () => importMeeting());

const CATALOG_KINDS = {
  asr: "Speech recognition",
  mt: "Translation",
  tts: "Speech synthesis",
  voice: "Voice",
};

async function loadCatalog() {
  try {
    const response = await fetch("/catalog", { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    renderCatalog(await response.json());
  } catch (failure) {
    catalogNote.textContent = "The catalogue could not be read.";
  }
}

function renderCatalog(entries) {
  catalogList.replaceChildren();
  if (!entries.length) {
    catalogNote.textContent = "This build carries an empty catalogue.";
    return;
  }
  for (const entry of entries) {
    const term = document.createElement("dt");
    const kind = CATALOG_KINDS[entry.kind] || entry.kind;
    term.textContent = entry.id + (entry.version ? " · " + entry.version : "") + " · " + kind;
    const detail = document.createElement("dd");
    const verified = entry.status === "verified" ? "licence checked" : entry.status;
    detail.textContent = entry.license + " · " + verified;
    catalogList.append(term, detail);
  }
  catalogNote.textContent =
    entries.length +
    " components this build knows, with their licence and its state. Installing from here is not in this build yet.";
}

async function loadHistory() {
  try {
    const response = await fetch("/history", {
      cache: "no-store",
      headers: { Authorization: "Bearer " + token },
    });
    if (!response.ok) {
      return;
    }
    renderHistory(await response.json());
  } catch (failure) {
    historyNote.textContent = "The recent dictations could not be read.";
  }
}

function renderHistory(entries) {
  historyList.replaceChildren();
  if (!entries.length) {
    historyNote.textContent = "Nothing dictated yet. What you dictate and its translation will appear here.";
    historyClear.hidden = true;
    return;
  }
  for (const entry of entries) {
    const term = document.createElement("dt");
    const moment = new Date((entry.at || 0) * 1000);
    term.textContent =
      moment.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }) +
      " · " +
      (LANGUAGE_NAMES[entry.from] || entry.from) +
      " → " +
      (LANGUAGE_NAMES[entry.to] || entry.to);
    const detail = document.createElement("dd");
    const reuse = document.createElement("button");
    reuse.type = "button";
    reuse.className = "swap";
    reuse.textContent = entry.translation || entry.text;
    reuse.title = "Copy this text again";
    reuse.addEventListener("click", () => copyHistory(reuse.textContent));
    detail.append(reuse);
    historyList.append(term, detail);
  }
  historyNote.textContent =
    entries.length === 1
      ? "1 dictation kept on this computer."
      : entries.length + " dictations kept on this computer.";
  historyClear.hidden = false;
}

async function copyHistory(text) {
  try {
    await navigator.clipboard.writeText(text);
    historyNote.textContent = "Copied to the clipboard. Paste it where you need it.";
  } catch (failure) {
    historyNote.textContent = "The clipboard refused the text. Select it here and copy by hand.";
  }
}

async function clearHistory() {
  try {
    const response = await fetch("/history/clear", {
      method: "POST",
      headers: { Authorization: "Bearer " + token },
    });
    if (!response.ok) {
      throw new Error("status " + response.status);
    }
    renderHistory([]);
  } catch (failure) {
    historyNote.textContent = "The history could not be cleared. Try again.";
  }
}

historyClear.addEventListener("click", () => clearHistory());

window.addEventListener("hashchange", openSection);
window.addEventListener("load", () => {
  loadMeetings();
  loadCatalog();
  loadHistory();
  if (location.hash) {
    openSection();
  }
});

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
loadSettings();
loadIntelligence();
loadVocabulary();
describeLive();

const voiceForm = document.getElementById("voice-form");
const voiceList = document.getElementById("voice-list");
const voiceStatus = document.getElementById("voice-status");
const voiceOrigin = document.getElementById("voice-origin");
const voiceAuthorisationRow = document.getElementById("voice-authorisation-row");
const voiceAuthorisation = document.getElementById("voice-authorisation");

function voiceHeaders() {
  return { "Content-Type": "application/json", Authorization: "Bearer " + token };
}

function describeVoice(profile) {
  const source =
    profile.consent.origin === "third-party"
      ? "someone else's voice, with permission"
      : "your own voice";
  const state = profile.consent.revoked_at ? "authorisation revoked" : "authorised";
  return profile.language + " · " + source + " · " + state;
}

function voiceButton(text, handler) {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "voice-action";
  button.textContent = text;
  button.addEventListener("click", handler);
  return button;
}

function renderVoices(profiles) {
  voiceList.replaceChildren();
  if (!profiles.length) {
    voiceStatus.textContent = "No voice profiles on this computer yet.";
    return;
  }
  voiceStatus.textContent =
    profiles.length +
    (profiles.length === 1 ? " voice profile on this computer." : " voice profiles on this computer.");
  for (const profile of profiles) {
    const item = document.createElement("li");
    item.className = "voice-item";
    const title = document.createElement("p");
    title.className = "voice-item-title";
    title.textContent = profile.label;
    const meta = document.createElement("p");
    meta.className = "voice-item-meta";
    meta.textContent = describeVoice(profile) + " · since " + new Date(profile.created_at * 1000).toLocaleDateString();
    const actions = document.createElement("div");
    actions.className = "voice-item-actions";
    if (!profile.consent.revoked_at) {
      actions.append(
        voiceButton("Revoke", () =>
          voiceCall("/voice/revoke", profile, "Revoked. This voice will not be used again until you create and authorise a new profile.")
        )
      );
    }
    actions.append(
      voiceButton("Delete", () => {
        if (!window.confirm("Delete " + profile.label + " and everything made from it? This cannot be undone.")) {
          return;
        }
        voiceCall("/voice/delete", profile, "Deleted, with the sample and anything made from it.");
      })
    );
    item.append(title, meta, actions);
    voiceList.append(item);
  }
}

async function voiceCall(path, profile, done) {
  clearError();
  try {
    const response = await fetch(path, {
      method: "POST",
      headers: voiceHeaders(),
      body: JSON.stringify({ id: profile.id }),
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(payload.error || "The request failed.");
    }
    voiceStatus.textContent = done;
    await loadVoices();
  } catch (failure) {
    showError(failure.message);
  }
}

async function loadVoices() {
  try {
    const response = await fetch("/voice", { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    const payload = await response.json();
    renderVoices(payload.profiles || []);
  } catch (failure) {
    voiceStatus.textContent = "The voice list is not available right now.";
  }
}

voiceOrigin.addEventListener("change", () => {
  const thirdParty = voiceOrigin.value === "third-party";
  voiceAuthorisationRow.hidden = !thirdParty;
  voiceAuthorisation.required = thirdParty;
  if (!thirdParty) {
    voiceAuthorisation.checked = false;
  }
});

voiceForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  if (voiceOrigin.value === "third-party" && !voiceAuthorisation.checked) {
    showError("Tick the permission box before creating a profile for someone else's voice.");
    return;
  }
  clearError();
  const button = document.getElementById("voice-create");
  button.disabled = true;
  try {
    const response = await fetch("/voice", {
      method: "POST",
      headers: voiceHeaders(),
      body: JSON.stringify({
        label: document.getElementById("voice-label").value.trim(),
        language: document.getElementById("voice-language").value,
        purpose: document.getElementById("voice-purpose").value.trim(),
        scope: "translation and playback on this computer",
        origin: voiceOrigin.value,
        declared_authorisation: voiceAuthorisation.checked,
      }),
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(payload.error || "The profile could not be created.");
    }
    voiceStatus.textContent =
      "Created " + payload.label + ". The sample is recorded when the voice engine ships, and this profile is stored on this computer only.";
    voiceForm.reset();
    voiceAuthorisationRow.hidden = true;
    await loadVoices();
  } catch (failure) {
    showError(failure.message);
  } finally {
    button.disabled = false;
  }
});

loadVoices();
