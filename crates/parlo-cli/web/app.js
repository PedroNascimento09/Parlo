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
const updateRestart = document.getElementById("update-restart");
const progressBox = document.getElementById("update-progress");
const progressFill = document.getElementById("update-fill");
const progressNote = document.getElementById("update-note");
const engineEl = document.getElementById("status-engine");
const stateEl = document.getElementById("status-state");
const shortcutEl = document.getElementById("status-shortcut");
const languagesEl = document.getElementById("status-languages");
const modeEl = document.getElementById("status-mode");
const micEl = document.getElementById("status-mic");
const latencyEl = document.getElementById("status-latency");
const versionEl = document.getElementById("status-version");
const modelsBox = document.getElementById("models");
const modelsList = document.getElementById("models-list");
const modelsCancel = document.getElementById("models-cancel");
const featureTranslation = document.getElementById("feature-translation");
const featureVoice = document.getElementById("feature-voice");
const featureCaptions = document.getElementById("feature-captions");
const featuresNote = document.getElementById("features-note");
const modelsFolder = document.getElementById("models-folder");
const modelsMove = document.getElementById("models-move");
const modelsMoveNote = document.getElementById("models-move-note");
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
const settingsManageMemory = document.getElementById("settings-manage-memory");
const memoryNote = document.getElementById("memory-note");
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
const cleanupApply = document.getElementById("cleanup-apply");
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

const TARGET_RATE = 16000;
const MAX_SECONDS = 30;
const MIN_SECONDS = 0.25;
const LIVE_MAX_SECONDS = 8;
const LIVE_FAILURES = 3;

let media = null;
let context = null;
let analyser = null;
let processor = null;
let levels = null;
let meterFrame = null;
let recording = false;
let sampling = false;
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
let currentShortcut = "Ctrl+Alt+D";
let settingsBusy = false;
let playbackDone = null;

const versionLabel = version ? "v" + version : "";
versionEl.textContent = versionLabel;

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
    return "Windows blocked the microphone. Allow Parlo in Settings, Privacy, Microphone, then try again.";
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
      if (!recording && !sampling && (!meetingRecording || meetingPaused)) {
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

function mergedVoiceChunks() {
  const total = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
  const merged = new Float32Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    merged.set(chunk, offset);
    offset += chunk.length;
  }
  return merged;
}

async function recordVoiceSample(profile, button) {
  if (sampling) {
    return;
  }
  clearError();
  try {
    const audio = await ensureAudio();
    chunks.length = 0;
    sampling = true;
    const started = Date.now();
    const tick = setInterval(() => {
      button.textContent = "Recording " + Math.max(0, 10 - Math.round((Date.now() - started) / 1000)) + " s";
    }, 250);
    button.textContent = "Recording 10 s";
    await new Promise((done) => setTimeout(done, 10000));
    sampling = false;
    clearInterval(tick);
    button.textContent = "Saving the sample";
    const samples = downsample(mergedVoiceChunks(), audio.sampleRate, 16000);
    const response = await fetch("/voice/sample", {
      method: "POST",
      headers: { "Content-Type": "audio/wav", Authorization: "Bearer " + token, "X-Parlo-Voice": profile.id },
      body: encodeWav(samples, 16000),
    });
    const answer = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(answer.error || "The sample was not saved.");
    }
    voiceStatus.textContent = "Sample saved. Listen to it before you use this voice.";
    await loadVoices();
  } catch (failure) {
    sampling = false;
    showError(failure.message);
  }
}

async function playVoiceSample(profile) {
  clearError();
  try {
    const response = await fetch("/voice/sample/play", {
      method: "POST",
      headers: voiceHeaders(),
      body: JSON.stringify({ id: profile.id }),
    });
    if (!response.ok) {
      const answer = await response.json().catch(() => ({}));
      throw new Error(answer.error || "The sample could not be read.");
    }
    play(await response.blob());
  } catch (failure) {
    showError(failure.message);
  }
}

async function exportVoice(profile) {
  clearError();
  try {
    const response = await fetch("/voice/export", {
      method: "POST",
      headers: voiceHeaders(),
      body: JSON.stringify({ id: profile.id }),
    });
    if (!response.ok) {
      const answer = await response.json().catch(() => ({}));
      throw new Error(answer.error || "The profile could not be exported.");
    }
    const blob = await response.blob();
    const link = document.createElement("a");
    link.href = URL.createObjectURL(blob);
    link.download = "parlo-voice-" + profile.id + ".txt";
    link.click();
    setTimeout(() => URL.revokeObjectURL(link.href), 5000);
    voiceStatus.textContent = "Exported. The file marks the voice as synthetic and names its origin.";
  } catch (failure) {
    showError(failure.message);
  }
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
      showError("The translation is ready but the browser blocked playback. Click the page once, then try again.");
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
  sub.textContent = "Release when you are done";
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
  label.textContent = "Try it once";
  sub.textContent = "Hold, speak, release";

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
    const startedAt = performance.now();
    const response = await fetch("/translate", {
      method: "POST",
      headers: { "Content-Type": "audio/wav", Authorization: "Bearer " + token },
      body: wav,
    });
    if (response.status === 401) {
      throw new Error("Parlo lost the connection to its engine. Close Parlo and open it again.");
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


const STATE_WORDS = {
  idle: "Ready to dictate",
  armed: "Opening the microphone",
  listening: "Listening",
  processing: "Writing it down",
  inserting: "Typing into your app",
  success: "Done",
  error: "Needs attention",
  cancelled: "Thrown away",
};

async function refreshHealth() {
  try {
    const response = await fetch("/health", { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    const health = await response.json();
    engineEl.textContent = health.engine || "unknown";
    languagesEl.textContent = describePair(health.from, health.to);
  } catch (failure) {
    engineEl.textContent = "not answering";
  }
  try {
    const response = await fetch("/state", { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    const current = await response.json();
    stateEl.textContent = STATE_WORDS[current.state] || "ready";
    const stage = stateEl.closest(".stage");
    if (stage) {
      stage.dataset.state = current.state || "idle";
    }
  } catch (failure) {
    stateEl.textContent = "checking";
  }
  try {
    const response = await fetch("/settings", { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    const values = await response.json();
    currentShortcut = values.shortcut || "not set";
    shortcutEl.textContent = currentShortcut;
    modeEl.textContent = values.write
      ? "writing in " + (LANGUAGE_NAMES[values.write] || values.write)
      : "translating";
  } catch (failure) {
    shortcutEl.textContent = "unknown";
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
    const build = localUpdate.build !== false;
    allowInstall(build);
    if (!build) {
      updateText.textContent =
        "Release " +
        localUpdate.latest +
        " has no build for this computer yet. See what changed; the download appears here when it is ready.";
    }
  } catch (failure) {
    allowInstall(false);
  }
}

const aboutVersion = document.getElementById("about-version");
const aboutCheck = document.getElementById("about-check");
const aboutNote = document.getElementById("about-note");

aboutVersion.textContent = version || "unknown";

aboutCheck.addEventListener("click", async () => {
  aboutCheck.disabled = true;
  aboutNote.textContent = "Asking GitHub for the latest release.";
  await checkLocalUpdate();
  await checkForUpdate().catch(() => {});
  aboutNote.textContent = localUpdate.available
    ? "Release " + localUpdate.latest + " is available. The box at the top of Home installs it."
    : "This is the latest release.";
  aboutCheck.disabled = false;
});

function describeUpdatePhase(state) {
  if (state.phase === "downloading") {
    return "Downloading the new version: " + (state.percent || 0) + "%.";
  }
  if (state.phase === "unpacking") {
    return "Unpacking the new version.";
  }
  if (state.phase === "checking") {
    return "Checking the new program is there.";
  }
  if (state.phase === "ready") {
    return "Ready. Parlo closes and starts the new version.";
  }
  return "Waiting for the release.";
}

async function pollUpdateProgress() {
  try {
    const response = await fetch("/update/progress", { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    const state = await response.json();
    progressFill.style.width = (state.percent || 0) + "%";
    progressNote.textContent = describeUpdatePhase(state);
  } catch (failure) {
    progressNote.textContent = "Parlo is closing to replace itself.";
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
  progressBox.hidden = false;
  progressFill.style.width = "0%";
  progressNote.textContent = "Starting the download.";
  const poll = setInterval(pollUpdateProgress, 400);
  try {
    const response = await fetch("/update", {
      method: "POST",
      headers: { Authorization: "Bearer " + token },
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      updateText.textContent = detail.error || "The update could not be installed. Run parlo update in a terminal instead.";
      progressNote.textContent = "";
      installButton.disabled = false;
      installButton.textContent = "Install update";
      return;
    }
    progressFill.style.width = "100%";
    const installed = detail.version || localUpdate.latest || "";
    updateText.textContent =
      "Parlo " + installed + " is unpacked. The window closes and the new version starts on its own.";
    progressNote.textContent = "If Parlo does not come back by itself, press Restart now.";
    installButton.hidden = true;
    updateRestart.hidden = false;
  } catch (failure) {
    updateText.textContent = "Parlo is closing to replace itself, which is what should happen. The new version starts on its own.";
    progressNote.textContent = "If Parlo does not come back by itself, open it again.";
  } finally {
    clearInterval(poll);
  }
}

updateRestart.addEventListener("click", async () => {
  updateRestart.disabled = true;
  try {
    await fetch("/update/restart", {
      method: "POST",
      headers: { Authorization: "Bearer " + token },
    });
    progressNote.textContent = "Parlo is starting again.";
  } catch (failure) {
    progressNote.textContent = "Parlo did not answer. Open it again from the Start menu.";
  }
});

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

async function downloadChosenModels(units) {
  if (!units.length) {
    return;
  }
  welcomeInstall.disabled = true;
  setWelcomeStatus("Downloading the models you chose. This can take a few minutes.");
  try {
    const response = await fetch("/models/download", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({ units }),
    });
    if (!response.ok) {
      const detail = await response.json().catch(() => ({}));
      throw new Error(detail.error || "The download could not start.");
    }
    startWelcomeClock();
  } catch (failure) {
    setWelcomeStatus(failure.message || "The download could not start.");
    welcomeInstall.disabled = false;
  }
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
        (ready
          ? "The models are ready. Parlo works offline from now on."
          : "Dictation is ready. Translation and read aloud can be added later, one model at a time.") +
          " Hold " + currentShortcut + " and speak; the text lands where the cursor is.",
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
  const parakeet = catalogEntries.find((item) => item.unit === "parakeet");
  welcomeInstall.textContent =
    parakeet && parakeet.size_mb
      ? "Download the dictation model · " + parakeet.size_mb + " MB"
      : "Download the dictation model";
}

let modelState = { models: [], ready: false, dictation: false, downloading: false };

function describeIdle(seconds) {
  if (seconds < 60) {
    return Math.max(0, Math.round(seconds)) + " s";
  }
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) {
    return minutes + " min";
  }
  const hours = Math.round(minutes / 60);
  return hours + (hours === 1 ? " hour" : " hours");
}

function describeLifecycle(model) {
  if (!model.installed) {
    return "";
  }
  const parts = [];
  if (model.state === "loaded" && typeof model.idle_seconds === "number") {
    parts.push("idle for " + describeIdle(model.idle_seconds));
  }
  if (model.policy === 0) {
    parts.push("kept loaded until you unload it");
    return parts.join(", ");
  }
  if (typeof model.policy !== "number") {
    return parts.join(", ");
  }
  const delay = "freed after " + describeIdle(model.policy * 60) + " without use";
  parts.push(model.policy_origin === "model" ? delay : delay + ", from the global setting");
  return parts.join(", ");
}

function describeModels(state) {
  modelState = state;
  modelsCancel.hidden = !state.downloading;
  if (state.folder && !modelsFolder.dataset.filled) {
    modelsFolder.value = state.folder;
    modelsFolder.dataset.filled = "1";
  }
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

    const reason = document.createElement("span");
    reason.className = "models-kind models-reason";
    reason.textContent = model.reason || "";
    name.append(reason);

    if (model.state === "loaded" || model.state === "installed" || model.state === "in use") {
      const badge = document.createElement("span");
      badge.className = "models-state";
      badge.dataset.state = model.state;
      badge.textContent =
        model.state === "loaded"
          ? "Loaded"
          : model.state === "in use"
            ? "In use"
            : "Installed · unloaded";
      name.append(badge);
    }

    const lifecycle = describeLifecycle(model);
    if (lifecycle) {
      const policy = document.createElement("span");
      policy.className = "models-kind models-policy-note";
      policy.textContent = lifecycle;
      name.append(policy);
    }

    if (model.quality) {
      const quality = document.createElement("span");
      quality.className = "models-kind models-quality";
      quality.textContent = model.quality;
      name.append(quality);
    }

    const fact = document.createElement("span");
    fact.className = "models-fact" + (model.installed ? " ready" : "");
    fact.textContent = model.installed ? formatSize(model.bytes) : "Missing";

    const actions = document.createElement("div");
    actions.className = "models-actions";

    if (model.installed) {
      const policy = document.createElement("select");
      policy.className = "field-input models-policy";
      policy.setAttribute("aria-label", "Free " + model.name + " after");
      [
        ["", "Follows the global setting"],
        ["10", "Free after 10 min"],
        ["15", "Free after 15 min"],
        ["30", "Free after 30 min"],
        ["60", "Free after 1 hour"],
        ["120", "Free after 2 hours"],
        ["240", "Free after 4 hours"],
        ["0", "Keep loaded"],
      ].forEach(([value, label]) => {
        const option = document.createElement("option");
        option.value = value;
        option.textContent = label;
        policy.append(option);
      });
      policy.value = model.policy_origin === "model" ? String(model.policy) : "";
      policy.disabled = Boolean(state.downloading);
      policy.addEventListener("change", () => savePolicy(model, policy));
      actions.append(policy);

      const held = document.createElement("button");
      held.type = "button";
      held.className = "models-action";
      held.textContent = model.state === "loaded" ? "Unload now" : "Load";
      held.disabled = Boolean(state.downloading) || model.state === "in use";
      held.title = model.state === "in use" ? "In use. Unload it when the task finishes." : "";
      held.addEventListener("click", () => modelStateAction(model, held));
      actions.append(held);
    }

    if (model.previous) {
      const restore = document.createElement("button");
      restore.type = "button";
      restore.className = "models-action";
      restore.textContent = "Restore";
      restore.disabled = Boolean(state.downloading);
      restore.addEventListener("click", () => rollbackModel(model, restore));
      actions.append(restore);
    }

    const action = document.createElement("button");
    action.type = "button";
    action.className = "models-action";
    action.textContent = model.installed ? "Remove" : "Download";
    action.disabled = Boolean(state.downloading);
    action.addEventListener("click", () => modelAction(model, action));
    actions.append(action);

    row.append(name, fact, actions);
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
      ? detail.message || "Removed."
      : "Downloading " + model.name + ". This page stays usable meanwhile.";
    await loadModels();
    if (model.installed) {
      await loadFeatures();
    }
  } catch (failure) {
    modelsNote.textContent = "That could not be done. Check that Parlo is still running.";
    button.disabled = false;
  }
}

async function modelStateAction(model, button) {
  const loading = model.state !== "loaded";
  button.disabled = true;
  try {
    const response = await fetch("/models/" + (loading ? "load/" : "unload/") + model.id, {
      method: "POST",
      headers: { Authorization: "Bearer " + token },
    });
    const detail = await response.json().catch(() => ({}));
    modelsNote.textContent =
      detail.message || detail.error || "That could not be done. Try again in a moment.";
    if (response.ok) {
      await loadModels();
    } else {
      button.disabled = false;
    }
  } catch (failure) {
    modelsNote.textContent = "That could not be done. Check that Parlo is still running.";
    button.disabled = false;
  }
}

async function savePolicy(model, select) {
  select.disabled = true;
  const minutes = select.value === "" ? null : Number(select.value);
  try {
    const response = await fetch("/models/policy", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({ id: model.id, minutes }),
    });
    const detail = await response.json().catch(() => ({}));
    modelsNote.textContent =
      detail.message || detail.error || "That setting could not be saved.";
    if (response.ok) {
      await loadModels();
    }
  } catch (failure) {
    modelsNote.textContent =
      "That setting could not be saved. Check that Parlo is still running.";
  } finally {
    select.disabled = false;
  }
}

let movePlan = null;

async function moveModels() {
  modelsMove.disabled = true;
  try {
    const response = await fetch("/models/relocate", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({ path: modelsFolder.value, confirm: Boolean(movePlan) }),
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      modelsMoveNote.textContent = detail.error || "The models could not be moved.";
      movePlan = null;
      modelsMove.textContent = "Move the models";
      return;
    }
    modelsMoveNote.textContent = detail.message || "The models were moved.";
    if (!movePlan) {
      movePlan = detail;
      modelsMove.textContent = detail.adopt
        ? "Use that copy"
        : "Copy " + Math.round((detail.needed || 0) / (1024 * 1024)) + " MB";
      return;
    }
    movePlan = null;
    modelsMove.textContent = "Move the models";
    modelsFolder.dataset.filled = "1";
    await loadModels();
  } catch (failure) {
    modelsMoveNote.textContent = "The models could not be moved. Check that Parlo is still running.";
    movePlan = null;
    modelsMove.textContent = "Move the models";
  } finally {
    modelsMove.disabled = false;
  }
}

async function rollbackModel(model, button) {
  button.disabled = true;
  try {
    const response = await fetch("/models/rollback/" + encodeURIComponent(model.id), {
      method: "POST",
      headers: { Authorization: "Bearer " + token },
    });
    const detail = await response.json().catch(() => ({}));
    modelsNote.textContent =
      detail.message || detail.error || "That could not be done.";
    await loadModels();
  } catch (failure) {
    modelsNote.textContent = "That could not be done. Check that Parlo is still running.";
  } finally {
    button.disabled = false;
  }
}

modelsMove.addEventListener("click", () => moveModels());

async function cancelDownload() {
  modelsCancel.disabled = true;
  try {
    const response = await fetch("/models/cancel", {
      method: "POST",
      headers: { Authorization: "Bearer " + token },
    });
    const detail = await response.json().catch(() => ({}));
    modelsNote.textContent =
      detail.message || detail.error || "The download could not be stopped.";
    await loadModels();
  } catch (failure) {
    modelsNote.textContent = "The download could not be stopped. Check that Parlo is still running.";
  } finally {
    modelsCancel.disabled = false;
  }
}

modelsCancel.addEventListener("click", () => cancelDownload());

async function loadModels() {
  try {
    const response = await fetch("/models", { cache: "no-store" });
    if (!response.ok) {
      showModelsError("The model list could not be read. The translator still works if its models are already installed.");
      return false;
    }
    const state = await response.json();
    describeHomeModels(state);
    return describeModels(state);
  } catch (failure) {
    showModelsError("The model list could not be read. The translator still works if its models are already installed.");
    return false;
  }
}

function formatFree(bytes) {
  if (!bytes) {
    return "";
  }
  if (bytes >= 1024 * 1024 * 1024) {
    return (bytes / (1024 * 1024 * 1024)).toFixed(0) + " GB free";
  }
  return formatSize(bytes) + " free";
}

function describeHomeModels(state) {
  const installed = (state.models || []).filter((model) => model.installed);
  const bytes = installed.reduce((total, model) => total + (model.bytes || 0), 0);
  const free = formatFree(state.free_bytes);
  const talking =
    installed.length === 0
      ? "none installed yet"
      : installed.length + " installed · " + formatSize(bytes);
  statusModels.textContent = free ? talking + " · " + free : talking;
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
welcomeInstall.addEventListener("click", () => downloadChosenModels(["parakeet"]));

document.getElementById("welcome-more").addEventListener("click", () => {
  libraryTab = "models";
  showLibraryTab("models");
});
modelsDictation.addEventListener("click", () => downloadModels("/models/dictation"));

const LANGUAGE_NAMES = { pt: "Portuguese", en: "English" };
const settingsWrite = document.getElementById("settings-write");
const settingsShortcut = document.getElementById("settings-shortcut");
const settingsShortcutSave = document.getElementById("settings-shortcut-save");
const settingsHandsFree = document.getElementById("settings-hands-free");
const settingsHandsFreeSave = document.getElementById("settings-hands-free-save");
const settingsCaptions = document.getElementById("settings-captions");
const settingsCaptionsSave = document.getElementById("settings-captions-save");
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
const meetingPause = document.getElementById("meeting-pause");
const meetingNotesRow = document.getElementById("meeting-notes-row");
const meetingNotes = document.getElementById("meeting-notes");
const meetingNoteAdd = document.getElementById("meeting-note-add");
const meetingExport = document.getElementById("meeting-export");
const meetingSearch = document.getElementById("meeting-search");
const meetingSummary = document.getElementById("meeting-summary");
const meetingSummaryText = document.getElementById("meeting-summary-text");
const meetingMatches = document.getElementById("meeting-matches");
const meetingQuestion = document.getElementById("meeting-question");
const meetingAsk = document.getElementById("meeting-ask");
const meetingAskAll = document.getElementById("meeting-ask-all");
const meetingAnswer = document.getElementById("meeting-answer");
const meetingCitations = document.getElementById("meeting-citations");
const callBanner = document.getElementById("call-banner");
const callText = document.getElementById("call-text");
const callRecord = document.getElementById("call-record");
const callDismiss = document.getElementById("call-dismiss");
const callsIgnored = new Set();

async function checkCall() {
  if (!callBanner || meetingLive.getAttribute("aria-pressed") === "true") {
    return;
  }
  try {
    const response = await fetch("/call", {
      cache: "no-store",
      headers: { Authorization: "Bearer " + token },
    });
    if (!response.ok) {
      callBanner.hidden = true;
      return;
    }
    const call = await response.json();
    if (!call.app || callsIgnored.has(call.app)) {
      callBanner.hidden = true;
      return;
    }
    callText.dataset.app = call.app;
    callText.textContent = call.app + " is open on this computer. Record this meeting?";
    callBanner.hidden = false;
  } catch (failure) {
    callBanner.hidden = true;
  }
}

callRecord.addEventListener("click", () => {
  callBanner.hidden = true;
  meetingLive.click();
});

callDismiss.addEventListener("click", () => {
  callsIgnored.add(callText.dataset.app || "");
  callBanner.hidden = true;
});

window.setInterval(checkCall, 20000);
checkCall();
const catalogList = document.getElementById("catalog-list");
const catalogNote = document.getElementById("catalog-note");
const catalogTabs = document.getElementById("catalog-tabs");
const CATALOG_CATEGORIES = [
  ["dictation", "Dictation"],
  ["voice", "Voice"],
  ["translation", "Translation"],
  ["text", "Text"],
];
let catalogTab = "dictation";
let catalogLast = [];

const historyList = document.getElementById("history-list");
const historyNote = document.getElementById("history-note");
const historySearch = document.getElementById("history-search");
const historyApp = document.getElementById("history-app");
const historyMode = document.getElementById("history-mode");
const historyFavourite = document.getElementById("history-favourite");
const recentAll = document.getElementById("recent-all");
const statusModels = document.getElementById("status-models");
let historyEntries = [];
const historyClear = document.getElementById("history-clear");
const historyFilters = document.getElementById("history-filters");
const recentList = document.getElementById("recent-list");
const recentEmpty = document.getElementById("recent-empty");
const recentSection = document.getElementById("recent");

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
  settingsManageMemory.checked = values.manage_memory !== false;
  settingsUnload.value = String(typeof values.unload === "number" ? values.unload : 30);
  settingsUnload.disabled = !settingsManageMemory.checked;
  settingsShortcut.value = values.shortcut || "Ctrl+Alt+D";
  currentShortcut = settingsShortcut.value;
  settingsHandsFree.value = values.hands_free || "Ctrl+Alt+Shift+D";
  settingsCaptions.value = values.captions || "Ctrl+Alt+C";
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

async function saveShortcut(field, input, sentence) {
  const wanted = input.value.trim();
  const body = {};
  body[field] = wanted;
  try {
    const response = await fetch("/settings", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify(body),
    });
    const answer = await response.json().catch(() => ({}));
    if (!response.ok) {
      settingsNote.textContent = answer.error || "That shortcut cannot be used.";
      return;
    }
    input.value = answer[field] || wanted;
    settingsNote.textContent = "Saved. " + sentence;
  } catch (failure) {
    settingsNote.textContent = "The shortcut could not be saved.";
  }
}

settingsShortcutSave.addEventListener("click", () =>
  saveShortcut(
    "shortcut",
    settingsShortcut,
    "Restart Parlo so the new dictation shortcut is registered.",
  ),
);
settingsHandsFreeSave.addEventListener("click", () =>
  saveShortcut(
    "hands_free",
    settingsHandsFree,
    "Restart Parlo so the new hands free shortcut is registered.",
  ),
);
settingsCaptionsSave.addEventListener("click", () =>
  saveShortcut(
    "captions",
    settingsCaptions,
    "Restart Parlo so the new captions shortcut is registered.",
  ),
);

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
      throw new Error("Parlo lost the connection to its engine. Close Parlo and open it again.");
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

settingsManageMemory.addEventListener("change", async () => {
  const enabled = settingsManageMemory.checked;
  settingsUnload.disabled = !enabled;
  try {
    const response = await fetch("/settings", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({ manage_memory: enabled }),
    });
    if (!response.ok) {
      throw new Error("That setting cannot be used.");
    }
    memoryNote.textContent = enabled
      ? "A model is freed after the delay beside it and loaded again the next time it is needed."
      : "No model is freed on its own. The files stay on disk and each model is loaded when it is needed.";
  } catch (failure) {
    settingsManageMemory.checked = !enabled;
    settingsUnload.disabled = enabled;
    memoryNote.textContent = failure.message || "That setting cannot be used.";
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
  cleanupApply.disabled = true;
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
    cleanupApply.disabled = false;
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
cleanupApply.addEventListener("click", () => saveIntelligence());
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
      throw new Error("Parlo lost the connection to its engine. Close Parlo and open it again.");
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

function openSection() {
  const fragment = (location.hash || "").replace("#", "");
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
    const day = new Date(meeting.started_at || 0);
    term.textContent =
      (meeting.title || "Untitled") +
      " · " +
      day.toLocaleDateString() +
      " · " +
      meeting.segments.length +
      " lines";
    const detail = document.createElement("dd");
    for (const speaker of meeting.speakers.filter((name) => name !== "Note")) {
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
    const download = document.createElement("button");
    download.type = "button";
    download.className = "swap";
    download.textContent = "Download .md";
    download.title = "Save this meeting as a Markdown file";
    download.addEventListener("click", () => downloadMeeting(meeting));
    detail.append(download);
    for (const note of (meeting.segments || []).filter((segment) => segment.speaker === "Note")) {
      const line = document.createElement("p");
      line.className = "meeting-note";
      line.textContent = "Note · " + meetingClock(note.at_ms) + " · " + note.text;
      detail.append(line);
    }
    for (const action of meeting.actions || []) {
      const line = document.createElement("p");
      line.className = "meeting-note";
      line.textContent =
        "Action · " +
        action.owner +
        (action.due ? " · " + action.due : "") +
        " · " +
        action.text;
      detail.append(line);
    }
    for (const decision of meeting.decisions || []) {
      const line = document.createElement("p");
      line.className = "meeting-note";
      line.textContent =
        "Decision · " +
        decision.speaker +
        " · " +
        decision.text +
        (decision.supersedes === null ? "" : " (replaces the one at " + meetingClock(decision.supersedes) + ")");
      detail.append(line);
    }
    meetingList.append(term, detail);
  }
  meetingNote.textContent =
    meetings.length === 1
      ? "1 meeting kept on this computer."
      : meetings.length + " meetings kept on this computer.";
}

async function downloadMeeting(meeting) {
  try {
    const response = await fetch("/meetings/" + encodeURIComponent(meeting.id) + "/markdown", {
      cache: "no-store",
      headers: { Authorization: "Bearer " + token },
    });
    if (!response.ok) {
      throw new Error("status " + response.status);
    }
    const blob = await response.blob();
    const named = /filename="([^"]+)"/.exec(response.headers.get("content-disposition") || "");
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = named ? named[1] : "meeting.md";
    link.click();
    URL.revokeObjectURL(url);
    meetingNote.textContent = "Saved " + link.download + " in your downloads folder.";
  } catch (failure) {
    meetingNote.textContent = "The meeting could not be saved as a file. Try again.";
  }
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

function meetingClock(atMs) {
  const seconds = Math.floor((atMs || 0) / 1000);
  return Math.floor(seconds / 60) + ":" + String(seconds % 60).padStart(2, "0");
}

function renderAnswer(answer) {
  meetingCitations.replaceChildren();
  meetingAnswer.textContent = answer.answer || "No answer.";
  for (const citation of answer.citations || []) {
    const term = document.createElement("dt");
    term.textContent =
      (citation.title || "Untitled") + " · " + citation.speaker + " · " + meetingClock(citation.at_ms);
    const detail = document.createElement("dd");
    detail.textContent = citation.text;
    meetingCitations.append(term, detail);
  }
}

async function askMeetings(every) {
  const question = meetingQuestion.value.trim();
  if (!question) {
    meetingAnswer.textContent = "Ask a question first.";
    return;
  }
  meetingAnswer.textContent = "Reading the stored lines…";
  meetingCitations.replaceChildren();
  try {
    const response = await fetch(every ? "/memory/ask" : "/meeting/ask", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({ question }),
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "The question was refused.");
    }
    renderAnswer(detail);
  } catch (failure) {
    meetingAnswer.textContent = failure.message || "The question could not be answered.";
  }
}

meetingAsk.addEventListener("click", () => askMeetings(false));
meetingAskAll.addEventListener("click", () => askMeetings(true));
meetingQuestion.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    askMeetings(false);
  }
});

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
let meetingPaused = false;
let meetingPausedAt = 0;
let meetingPausedTotal = 0;
let meetingTimer = null;
let meetingLines = 0;
let meetingFailures = 0;

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && meetingRecording) {
    event.preventDefault();
    stopMeeting("Recording stopped. The lines already kept stay in the meeting.");
  }
});

function meetingElapsed() {
  const reached = meetingPaused ? meetingPausedAt : performance.now();
  return Math.max(0, Math.round(reached - meetingStarted - meetingPausedTotal));
}

function meetingStatus(last) {
  if (meetingRecording && meetingPaused) {
    const kept = meetingLines === 1 ? "1 line kept" : meetingLines + " lines kept";
    meetingLiveNote.textContent =
      "Paused at " +
      meetingClock(meetingElapsed()) +
      ", " +
      kept +
      ". Nothing is written down until you resume.";
    return;
  }
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
      at_ms: meetingElapsed(),
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
  if (meetingPaused) {
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
  if (recording) {
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
  meetingPaused = false;
  meetingPausedAt = 0;
  meetingPausedTotal = 0;
  meetingPause.textContent = "Pause";
  meetingPause.setAttribute("aria-pressed", "false");
  meetingPause.disabled = false;
  meetingNotesRow.hidden = false;
  meetingNotes.value = "";
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

function pauseMeeting() {
  if (!meetingRecording || meetingPaused) {
    return;
  }
  clearTimeout(meetingTimer);
  meetingTimer = null;
  meetingPaused = true;
  meetingPausedAt = performance.now();
  chunks = [];
  meetingPause.textContent = "Resume";
  meetingPause.setAttribute("aria-pressed", "true");
  meetingStatus("");
}

function resumeMeeting() {
  if (!meetingRecording || !meetingPaused) {
    return;
  }
  meetingPausedTotal += performance.now() - meetingPausedAt;
  meetingPaused = false;
  meetingPausedAt = 0;
  chunks = [];
  meetingPause.textContent = "Pause";
  meetingPause.setAttribute("aria-pressed", "false");
  meetingStatus("");
  clearTimeout(meetingTimer);
  meetingTimer = setTimeout(meetingStep, 200);
}

async function addMeetingNote() {
  const text = meetingNotes.value.trim();
  if (!meetingRecording) {
    return;
  }
  if (!text) {
    meetingLiveNote.textContent = "Write the note before adding it.";
    return;
  }
  const at = meetingElapsed();
  try {
    const response = await fetch("/meeting/segment", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({ id: meetingId, speaker: "Note", text, at_ms: at }),
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "The note could not be kept.");
    }
    meetingNotes.value = "";
    await loadMeetings();
    meetingLiveNote.textContent = "Note kept at " + meetingClock(at) + ".";
  } catch (failure) {
    meetingLiveNote.textContent = failure.message || "The note could not be kept.";
  }
}

async function stopMeeting(message) {
  if (!meetingRecording) {
    if (message) {
      showError(message);
    }
    return;
  }
  meetingRecording = false;
  meetingPaused = false;
  meetingPausedAt = 0;
  meetingPause.textContent = "Pause";
  meetingPause.setAttribute("aria-pressed", "false");
  meetingPause.disabled = true;
  meetingNotesRow.hidden = true;
  meetingNotes.value = "";
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
  meetingPausedTotal = 0;
  if (message) {
    showError(message);
  }
}

function meetingsAsMarkdown(meetings) {
  const lines = [];
  for (const meeting of meetings) {
    const day = new Date(meeting.started_at || 0).toLocaleString();
    lines.push("# " + (meeting.title || "Untitled"), "", day, "");
    for (const segment of meeting.segments || []) {
      const total = Math.floor((segment.at_ms || 0) / 1000);
      const stamp =
        String(Math.floor(total / 60)).padStart(2, "0") +
        ":" +
        String(total % 60).padStart(2, "0");
      const who = segment.name ? segment.name.name : segment.speaker;
      lines.push("- `" + stamp + "` **" + who + "**: " + segment.text);
    }
    lines.push("");
  }
  return lines.join("\n").trim() + "\n";
}

async function exportMeetings() {
  try {
    const response = await fetch("/meetings", { cache: "no-store" });
    if (!response.ok) {
      throw new Error("status " + response.status);
    }
    const meetings = await response.json();
    if (!meetings.length) {
      meetingNote.textContent = "There is nothing to export yet.";
      return;
    }
    await navigator.clipboard.writeText(meetingsAsMarkdown(meetings));
    meetingNote.textContent =
      meetings.length === 1
        ? "Copied 1 meeting as Markdown."
        : "Copied " + meetings.length + " meetings as Markdown.";
  } catch (failure) {
    meetingNote.textContent = "The meetings could not be exported. Try again.";
  }
}

meetingExport.addEventListener("click", () => exportMeetings());

async function searchMeetings() {
  const needle = meetingSearch.value.trim().toLowerCase();
  meetingMatches.replaceChildren();
  if (needle.length < 2) {
    return;
  }
  let meetings = [];
  try {
    const response = await fetch("/meetings", { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    meetings = await response.json();
  } catch (failure) {
    return;
  }
  const matches = [];
  for (const meeting of meetings) {
    for (const segment of meeting.segments || []) {
      if (!(segment.text || "").toLowerCase().includes(needle)) {
        continue;
      }
      const who = segment.name ? segment.name.name : segment.speaker;
      const total = Math.floor((segment.at_ms || 0) / 1000);
      const stamp =
        String(Math.floor(total / 60)).padStart(2, "0") + ":" + String(total % 60).padStart(2, "0");
      matches.push((meeting.title || "Untitled") + " · " + stamp + " · " + who + ": " + segment.text);
      if (matches.length >= 40) {
        break;
      }
    }
    if (matches.length >= 40) {
      break;
    }
  }
  if (!matches.length) {
    matches.push("No line in any transcript carries that word.");
  }
  for (const line of matches) {
    const row = document.createElement("p");
    row.textContent = line;
    meetingMatches.append(row);
  }
}

meetingSearch.addEventListener("input", searchMeetings);

async function summariseMeeting() {
  meetingSummaryText.textContent = "Reading the transcript.";
  try {
    const response = await fetch("/meeting/summary", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({}),
    });
    const answer = await response.json().catch(() => ({}));
    if (!response.ok) {
      meetingSummaryText.textContent = answer.error || "The summary failed.";
      return;
    }
    meetingSummaryText.textContent = answer.summary || "The model answered with nothing.";
  } catch (failure) {
    meetingSummaryText.textContent = "The model could not be reached.";
  }
}

meetingSummary.addEventListener("click", () => summariseMeeting());



meetingLive.addEventListener("click", () => {
  if (meetingRecording) {
    stopMeeting("");
  } else {
    startMeeting();
  }
});

meetingImport.addEventListener("click", () => importMeeting());

meetingPause.addEventListener("click", () => {
  if (meetingPaused) {
    resumeMeeting();
  } else {
    pauseMeeting();
  }
});

meetingNoteAdd.addEventListener("click", () => addMeetingNote());
meetingNotes.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    addMeetingNote();
  }
});

const CATALOG_KINDS = {
  asr: "Speech recognition",
  mt: "Translation",
  tts: "Speech synthesis",
  voice: "Voice",
};

let catalogEntries = [];

async function loadCatalog() {
  try {
    const response = await fetch("/catalog", { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    catalogEntries = await response.json();
    renderCatalog(catalogEntries);
  } catch (failure) {
    catalogNote.textContent = "The catalogue could not be read.";
  }
}

async function downloadCatalogModel(entry, button) {
  button.disabled = true;
  button.textContent = "Starting…";
  try {
    const response = await fetch("/models/download/" + encodeURIComponent(entry.unit), {
      method: "POST",
      headers: { Authorization: "Bearer " + token },
    });
    if (!response.ok) {
      const detail = await response.json().catch(() => ({}));
      throw new Error(detail.error || "The download could not start.");
    }
    catalogNote.textContent =
      "Downloading " + (entry.name || entry.id) + ". It appears in the model list when it arrives.";
    setTimeout(loadCatalog, 4000);
  } catch (failure) {
    catalogNote.textContent = failure.message || "The download could not start.";
    button.disabled = false;
    button.textContent = "Download";
  }
}

function catalogBar(label, score, value) {
  const row = document.createElement("div");
  row.className = "metric";
  const term = document.createElement("span");
  term.textContent = label;
  const track = document.createElement("span");
  track.className = "track";
  const fill = document.createElement("i");
  fill.className = "fill";
  fill.style.width = Math.max(4, Math.min(100, Math.round(score))) + "%";
  track.append(fill);
  const readout = document.createElement("span");
  readout.textContent = value;
  row.append(term, track, readout);
  return row;
}

function benchmarkBest(items, metric, pick) {
  const values = items
    .filter((item) => item.bench && item.bench.metric === metric)
    .map(pick)
    .filter((value) => typeof value === "number" && value > 0);
  return values.length ? { low: Math.min(...values), high: Math.max(...values) } : null;
}

function benchmarkValue(bench) {
  if (bench.metric === "WER") {
    return bench.value + "% wrong words";
  }
  if (bench.metric === "MMLU") {
    return bench.value + "% on MMLU";
  }
  return String(bench.value);
}

function catalogLanguages(entry) {
  const languages = Array.isArray(entry.languages) ? entry.languages : [];
  if (!languages.length) {
    return null;
  }
  const row = document.createElement("div");
  row.className = "model-langs";
  const label = document.createElement("span");
  label.textContent =
    languages.length > 1
      ? languages[0].toUpperCase() + " + " + (languages.length - 1) + " more"
      : languages[0].toUpperCase();
  row.append(label);
  if (languages.length > 1) {
    const more = document.createElement("button");
    more.type = "button";
    more.className = "langs-more";
    more.textContent = "info";
    more.setAttribute("aria-expanded", "false");
    const list = document.createElement("ul");
    list.className = "langs-list";
    list.hidden = true;
    for (const language of languages) {
      const item = document.createElement("li");
      item.textContent = language;
      list.append(item);
    }
    more.addEventListener("click", () => {
      list.hidden = !list.hidden;
      more.setAttribute("aria-expanded", String(!list.hidden));
    });
    row.append(more, list);
  }
  return row;
}

function catalogCard(entry, items) {
  const card = document.createElement("article");
  card.className = "model-card";

  const head = document.createElement("div");
  head.className = "model-card-head";
  const name = document.createElement("h3");
  const base = entry.name || entry.id;
  const version = (entry.version || "").trim();
  name.textContent =
    version && !base.toLowerCase().includes(version.toLowerCase()) ? base + " " + version : base;
  head.append(name);
  if (entry.pair) {
    const pair = document.createElement("span");
    pair.className = "pair-chip";
    pair.textContent = entry.pair.toUpperCase();
    head.append(pair);
  }
  const licence = document.createElement("span");
  licence.className = "model-licence";
  licence.textContent = entry.license || "licence unknown";
  head.append(licence);

  const purpose = document.createElement("p");
  purpose.className = "model-purpose";
  purpose.textContent = entry.purpose || CATALOG_KINDS[entry.kind] || entry.kind;
  card.append(head, purpose);

  const facts = document.createElement("dl");
  facts.className = "model-facts";
  const fact = (term, value) => {
    if (!value) {
      return;
    }
    const label = document.createElement("dt");
    label.textContent = term;
    const detail = document.createElement("dd");
    detail.textContent = value;
    facts.append(label, detail);
  };
  fact("Size", entry.size_mb ? entry.size_mb + " MB" : "");
  fact("Memory", entry.memory_mb ? entry.memory_mb + " MB" : "");
  if (entry.rtf) {
    fact("Measured", entry.rtf.toFixed(2) + "× realtime here");
  }
  if (entry.latency_ms) {
    fact("Measured", entry.latency_ms + " ms a sentence here");
  }
  if (facts.childElementCount) {
    card.append(facts);
  }

  const languages = catalogLanguages(entry);
  if (languages) {
    card.append(languages);
  }

  const bars = document.createElement("div");
  bars.className = "model-bars";
  const bench = entry.bench;
  if (bench && typeof bench.value === "number") {
    const scale = benchmarkBest(items, bench.metric, (item) => item.bench.value);
    if (scale) {
      const score =
        bench.better === "lower" ? (100 * scale.low) / bench.value : (100 * bench.value) / scale.high;
      bars.append(catalogBar("Accuracy", score, benchmarkValue(bench)));
    }
    if (typeof bench.speed === "number") {
      const speeds = benchmarkBest(items, bench.metric, (item) => item.bench.speed);
      if (speeds) {
        bars.append(
          catalogBar("Speed", (100 * bench.speed) / speeds.high, bench.speed + "× relative"),
        );
      }
    }
  }
  if (bars.childElementCount) {
    const source = document.createElement("p");
    source.className = "bench-source";
    source.textContent = bench.source;
    bars.append(source);
    card.append(bars);
  } else {
    const unmeasured = document.createElement("p");
    unmeasured.className = "model-note";
    unmeasured.textContent = "No published figure for this one yet.";
    card.append(unmeasured);
  }

  if (entry.note) {
    const note = document.createElement("p");
    note.className = "model-note";
    note.textContent = entry.note;
    card.append(note);
  }

  const installed =
    Boolean(entry.unit) &&
    modelState.models.some((model) => model.id === entry.unit && model.installed);
  const action = document.createElement("button");
  action.type = "button";
  action.className = "models-action";
  if (installed) {
    action.textContent = "On disk";
    action.disabled = true;
  } else if (entry.unit) {
    action.textContent = "Download";
    action.addEventListener("click", () => downloadCatalogModel(entry, action));
  } else {
    action.textContent = "Not in this build";
    action.disabled = true;
  }
  card.append(action);
  return card;
}

function renderCatalog(entries) {
  catalogLast = entries;
  catalogTabs.replaceChildren();
  catalogList.replaceChildren();
  if (!entries.length) {
    catalogNote.textContent = "This build carries an empty catalogue.";
    return;
  }
  const sections = new Map();
  for (const entry of entries) {
    const key = entry.category || "other";
    if (!sections.has(key)) {
      sections.set(key, []);
    }
    sections.get(key).push(entry);
  }
  for (const items of sections.values()) {
    items.sort(
      (left, right) =>
        Number(Boolean(right.bench)) - Number(Boolean(left.bench)) ||
        (left.name || left.id).localeCompare(right.name || right.id),
    );
  }
  const known = CATALOG_CATEGORIES.filter(([key]) => sections.has(key));
  if (!sections.has(catalogTab)) {
    catalogTab = known.length ? known[0][0] : "other";
  }
  for (const [key, label] of known) {
    const tab = document.createElement("button");
    tab.type = "button";
    tab.className = "catalog-tab";
    tab.setAttribute("role", "tab");
    tab.setAttribute("aria-selected", String(key === catalogTab));
    tab.textContent = label + " · " + sections.get(key).length;
    tab.addEventListener("click", () => {
      catalogTab = key;
      renderCatalog(entries);
    });
    catalogTabs.append(tab);
  }
  const items = sections.get(catalogTab) || [];
  for (const entry of items) {
    catalogList.append(catalogCard(entry, items));
  }
  const label = (CATALOG_CATEGORIES.find(([key]) => key === catalogTab) || [catalogTab, catalogTab])[1];
  catalogNote.textContent =
    items.length +
    " in " +
    label +
    ", " +
    entries.length +
    " in this build. A bar compares published figures with the best in the same category; the line under it names where the figure comes from.";
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
    historyEntries = await response.json();
    fillHistoryApps();
    renderHistory();
  } catch (failure) {
    historyNote.textContent = "The recent dictations could not be read.";
  }
}

function fillHistoryApps() {
  const seen = new Set();
  for (const entry of historyEntries) {
    const app = (entry.app || "").trim();
    if (app) {
      seen.add(app);
    }
  }
  const chosen = historyApp.value;
  historyApp.replaceChildren();
  const any = document.createElement("option");
  any.value = "";
  any.textContent = "Any program";
  historyApp.append(any);
  for (const app of [...seen].sort()) {
    const option = document.createElement("option");
    option.value = app;
    option.textContent = app;
    historyApp.append(option);
  }
  historyApp.value = seen.has(chosen) ? chosen : "";
}

function matchesHistory(entry) {
  const app = historyApp.value;
  const mode = historyMode.value;
  const needle = historySearch.value.trim().toLowerCase();
  if (app && (entry.app || "") !== app) {
    return false;
  }
  if (mode && (entry.mode || "") !== mode) {
    return false;
  }
  if (historyFavourite.checked && !entry.favourite) {
    return false;
  }
  if (needle) {
    const haystack = ((entry.text || "") + " " + (entry.translation || "")).toLowerCase();
    if (!haystack.includes(needle)) {
      return false;
    }
  }
  return true;
}

function renderRecent() {
  recentList.replaceChildren();
  const shown = historyEntries.slice(0, 5);
  recentEmpty.hidden = shown.length > 0;
  recentSection.hidden = shown.length === 0;
  for (const entry of shown) {
    const item = document.createElement("li");
    const text = document.createElement("p");
    text.className = "recent-text";
    text.textContent = entry.translation || entry.text;
    const moment = new Date((entry.at || 0) * 1000);
    const where = document.createElement("p");
    where.className = "recent-meta";
    where.textContent = [
      moment.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
      (LANGUAGE_NAMES[entry.from] || entry.from) + " to " + (LANGUAGE_NAMES[entry.to] || entry.to),
      entry.app,
    ]
      .filter(Boolean)
      .join(" · ");
    item.append(text, where);
    recentList.append(item);
  }
}

function renderHistory() {
  renderRecent();
  historyList.replaceChildren();
  if (!historyEntries.length) {
    historyFilters.hidden = true;
    historyNote.textContent =
      "Nothing dictated yet. Press Ctrl+Alt+D, speak, and the sentence appears here with the program it landed in.";
    historyClear.hidden = true;
    return;
  }
  const shown = historyEntries.filter(matchesHistory);
  for (const entry of shown) {
    const term = document.createElement("dt");
    const moment = new Date((entry.at || 0) * 1000);
    const where = [entry.app, entry.mode].filter(Boolean).join(" · ");
    term.textContent =
      moment.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }) +
      " · " +
      (LANGUAGE_NAMES[entry.from] || entry.from) +
      " → " +
      (LANGUAGE_NAMES[entry.to] || entry.to) +
      (entry.seconds > 0 ? " · " + entry.seconds.toFixed(1) + " s" : "") +
      (where ? " · " + where : "");
    const detail = document.createElement("dd");
    const reuse = document.createElement("button");
    reuse.type = "button";
    reuse.className = "swap";
    reuse.textContent = entry.translation || entry.text;
    reuse.title = "Copy this text again";
    reuse.addEventListener("click", () => copyHistory(reuse.textContent));
    const actions = document.createElement("div");
    actions.className = "history-actions";
    const star = document.createElement("button");
    star.type = "button";
    star.className = "swap";
    star.textContent = entry.favourite ? "Favourite ✓" : "Favourite";
    star.setAttribute("aria-pressed", String(Boolean(entry.favourite)));
    star.title = "Mark this dictation so the favourite filter keeps it";
    star.addEventListener("click", () => setFavourite(entry));
    const correct = document.createElement("button");
    correct.type = "button";
    correct.className = "swap";
    correct.textContent = "Correct";
    correct.title = "Replace the sentence Parlo wrote down";
    correct.addEventListener("click", () => editHistory(entry));
    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "swap";
    remove.textContent = "Delete";
    remove.title = "Remove this dictation from this computer";
    remove.addEventListener("click", () => deleteHistory(entry));
    actions.append(star, correct, remove);
    detail.append(reuse, actions);
    const card = document.createElement("div");
    card.className = "history-entry";
    card.append(term, detail);
    historyList.append(card);
  }
  historyNote.textContent =
    shown.length === historyEntries.length
      ? historyEntries.length === 1
        ? "1 dictation kept on this computer."
        : historyEntries.length + " dictations kept on this computer."
      : shown.length + " of " + historyEntries.length + " dictations match.";
  historyFilters.hidden = false;
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

async function historyPost(path, body) {
  const response = await fetch(path, {
    method: "POST",
    headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
    body: JSON.stringify(body),
  });
  const detail = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(detail.error || "The dictation could not be changed.");
  }
  return detail;
}

async function setFavourite(entry) {
  try {
    await historyPost("/history/favourite", { id: entry.id, value: !entry.favourite });
    await loadHistory();
  } catch (failure) {
    historyNote.textContent = failure.message;
  }
}

async function editHistory(entry) {
  const chosen = window.prompt("Correct what Parlo wrote down.", entry.text || "");
  if (chosen === null) {
    return;
  }
  try {
    await historyPost("/history/edit", { id: entry.id, text: chosen });
    await loadHistory();
    historyNote.textContent = "Corrected on this computer. Nothing was sent anywhere.";
  } catch (failure) {
    historyNote.textContent = failure.message;
  }
}

async function deleteHistory(entry) {
  if (!window.confirm("Delete this dictation from this computer?")) {
    return;
  }
  try {
    await historyPost("/history/delete", { id: entry.id });
    await loadHistory();
  } catch (failure) {
    historyNote.textContent = failure.message;
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
    historyEntries = [];
    renderHistory();
  } catch (failure) {
    historyNote.textContent = "The history could not be cleared. Try again.";
  }
}

const onboarding = document.getElementById("onboarding");
const onboardingTitle = document.getElementById("onboarding-title");
const onboardingNote = document.getElementById("onboarding-note");
const onboardingResult = document.getElementById("onboarding-result");
const onboardingNext = document.getElementById("onboarding-next");
const onboardingSkip = document.getElementById("onboarding-skip");
const onboardingPair = document.getElementById("onboarding-pair");
const onboardingFrom = document.getElementById("onboarding-from");
const onboardingTo = document.getElementById("onboarding-to");
const onboardingSwap = document.getElementById("onboarding-swap");
const onboardingPairSave = document.getElementById("onboarding-pair-save");
const onboardingMic = document.getElementById("onboarding-mic");
const onboardingMicAllow = document.getElementById("onboarding-mic-allow");
const onboardingMicNote = document.getElementById("onboarding-mic-note");
const onboardingKeys = document.getElementById("onboarding-keys");
const onboardingShortcut = document.getElementById("onboarding-shortcut");
const onboardingKeySave = document.getElementById("onboarding-key-save");
const onboardingKeyNote = document.getElementById("onboarding-key-note");

const ONBOARDING_TITLES = [
  "Two languages",
  "The microphone",
  "The shortcut",
  "Say one sentence",
  "Ready",
];

let onboardingTimer = null;
let onboardingNudge = null;
let onboardingStage = 0;
let onboardingKnown = -1;
let onboardingChosen = "Ctrl+Alt+D";

function showOnboardingStage(step) {
  onboardingStage = step;
  localStorage.setItem("parlo.onboarding.stage", String(step));
  onboarding.hidden = false;
  onboardingTitle.textContent = ONBOARDING_TITLES[step - 1];
  onboardingPair.hidden = step !== 1;
  onboardingMic.hidden = step !== 2;
  onboardingKeys.hidden = step !== 3;
  onboardingNext.hidden = step !== 5;
  if (step !== 4) {
    clearInterval(onboardingTimer);
    onboardingTimer = null;
  }
  if (step === 1) {
    onboardingNote.textContent =
      "Parlo hears one language and speaks the other. Changing the pair later loads the models again.";
  }
  if (step === 2) {
    onboardingNote.textContent =
      "The microphone opens only while you hold the shortcut, and the sound never leaves this machine.";
  }
  if (step === 3) {
    onboardingNote.textContent = "Press the keys you want to use for dictation.";
  }
  if (step === 4) {
    onboardingNote.textContent =
      "Hold " + onboardingChosen + ", say a sentence, then let go. Escape throws the sentence away.";
    onboardingKnown = -1;
    readHistory().then((entries) => {
      onboardingKnown = entries.length;
    });
    clearInterval(onboardingTimer);
    clearTimeout(onboardingNudge);
    onboardingTimer = setInterval(tickOnboarding, 400);
    onboardingNudge = setTimeout(() => {
      if (onboardingStage !== 4) {
        return;
      }
      onboardingNote.textContent =
        "Nothing arrived. Allow the microphone in Windows settings, Privacy and security, Microphone, then hold " +
        onboardingChosen +
        " and speak again.";
    }, 25000);
  }
  if (step === 5) {
    onboardingNote.textContent =
      "Home says Ready. Press " + onboardingChosen + " anywhere and speak; Escape throws the sentence away.";
  }
}

function finishOnboarding() {
  clearInterval(onboardingTimer);
  clearTimeout(onboardingNudge);
  onboardingTimer = null;
  localStorage.setItem("parlo.onboarded", "1");
  localStorage.removeItem("parlo.onboarding.stage");
  onboarding.hidden = true;
}

async function readState() {
  try {
    const response = await fetch("/state", { cache: "no-store" });
    if (!response.ok) {
      return "";
    }
    const current = await response.json();
    return current.state || "";
  } catch (failure) {
    return "";
  }
}

async function readHistory() {
  try {
    const response = await fetch("/history", {
      cache: "no-store",
      headers: { Authorization: "Bearer " + token },
    });
    if (!response.ok) {
      return [];
    }
    return await response.json();
  } catch (failure) {
    return [];
  }
}

async function tickOnboarding() {
  if (onboardingStage !== 4 || onboardingKnown < 0) {
    return;
  }
  const state = await readState();
  if (state === "listening") {
    onboardingNote.textContent = "Keep talking, then let go of the key.";
    return;
  }
  if (!["success", "error", "cancelled"].includes(state)) {
    return;
  }
  const entries = await readHistory();
  if (entries.length <= onboardingKnown) {
    onboardingNote.textContent =
      "Nothing was written. Hold the key, speak a full sentence, then let go.";
    return;
  }
  const latest = entries[0];
  const text = latest.translation || latest.text || "";
  onboardingResult.textContent =
    "“" + text + "”" + (latest.app ? " landed in " + latest.app : "");
  showOnboardingStage(5);
}

async function saveOnboardingPair() {
  onboardingPairSave.disabled = true;
  onboardingNote.textContent = "Loading the models. This takes about twenty seconds.";
  try {
    const response = await fetch("/settings", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({ from: onboardingFrom.value, to: onboardingTo.value }),
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "That pair cannot be used.");
    }
    onboardingChosen = detail.shortcut || onboardingChosen;
    showOnboardingStage(2);
  } catch (failure) {
    onboardingNote.textContent = failure.message || "That pair cannot be used.";
  } finally {
    onboardingPairSave.disabled = false;
  }
}

async function saveOnboardingShortcut() {
  const wanted = onboardingShortcut.value.trim();
  if (!wanted) {
    onboardingKeyNote.textContent = "Press a combination first.";
    return;
  }
  onboardingKeySave.disabled = true;
  try {
    const response = await fetch("/settings", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({ shortcut: wanted }),
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "That combination cannot be used.");
    }
    onboardingChosen = detail.shortcut || wanted;
    onboardingResult.textContent = "";
    showOnboardingStage(4);
  } catch (failure) {
    onboardingKeyNote.textContent = failure.message || "That combination cannot be used.";
  } finally {
    onboardingKeySave.disabled = false;
  }
}

async function startOnboarding() {
  if (localStorage.getItem("parlo.onboarded") === "1") {
    return;
  }
  try {
    const response = await fetch("/settings", { cache: "no-store" });
    if (response.ok) {
      const values = await response.json();
      onboardingFrom.value = values.from || "pt";
      onboardingTo.value = values.to || "en";
      onboardingShortcut.value = values.shortcut || "Ctrl+Alt+D";
      onboardingChosen = values.shortcut || "Ctrl+Alt+D";
    }
  } catch (failure) {
    onboardingShortcut.value = "Ctrl+Alt+D";
  }
  onboardingResult.textContent = "";
  const saved = Number(localStorage.getItem("parlo.onboarding.stage") || "1");
  showOnboardingStage(saved === 5 ? 4 : Math.min(4, Math.max(1, saved)));
}

onboardingSwap.addEventListener("click", () => {
  const from = onboardingFrom.value;
  onboardingFrom.value = onboardingTo.value;
  onboardingTo.value = from;
});

onboardingPairSave.addEventListener("click", () => saveOnboardingPair());

onboardingKeySave.addEventListener("click", () => saveOnboardingShortcut());

onboardingShortcut.addEventListener("keydown", (event) => {
  event.preventDefault();
  const refused = ["Control", "Alt", "Shift", "Meta", "Escape", "Tab"];
  const named = { " ": "Space", Spacebar: "Space", Space: "Space" };
  const parts = [];
  if (event.ctrlKey) {
    parts.push("Ctrl");
  }
  if (event.altKey) {
    parts.push("Alt");
  }
  if (event.shiftKey) {
    parts.push("Shift");
  }
  if (event.metaKey) {
    parts.push("Win");
  }
  if (refused.includes(event.key)) {
    onboardingKeyNote.textContent = "Add a letter, a digit or Space after the modifiers.";
    return;
  }
  if (parts.length === 0) {
    onboardingKeyNote.textContent =
      "A key on its own is refused: it would swallow typing everywhere.";
    return;
  }
  const key = named[event.key] || event.key.toUpperCase();
  onboardingShortcut.value = parts.concat([key]).join("+");
  onboardingKeyNote.textContent = "Press Save when this is the combination you want.";
});

onboardingMicAllow.addEventListener("click", async () => {
  onboardingMicNote.textContent = "Asking Windows for the microphone.";
  try {
    await openMicrophone();
    onboardingMicNote.textContent = "The microphone is ready.";
    showOnboardingStage(3);
  } catch (failure) {
    onboardingMicNote.textContent = describeMicrophoneError(failure);
  }
});

onboardingSkip.addEventListener("click", () => finishOnboarding());
onboardingNext.addEventListener("click", () => finishOnboarding());

historyClear.addEventListener("click", () => clearHistory());
historySearch.addEventListener("input", () => renderHistory());
historyApp.addEventListener("change", () => renderHistory());
historyMode.addEventListener("change", () => renderHistory());
historyFavourite.addEventListener("change", () => renderHistory());
recentAll.addEventListener("click", () => {
  libraryTab = "history";
  showLibraryTab(libraryTab);
});

window.addEventListener("hashchange", openSection);
window.addEventListener("load", () => {
  loadMeetings();
  loadCatalog();
  loadHistory();
  startOnboarding();
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

const ROUTES = {
  home: [
    "update",
    "onboarding",
    "welcome",
    "progress",
    "error",
    "translation",
    "status-list",
    "recent",
  ],
  library: ["library-tabs", "vocabulary", "catalog", "history", "models", "dub"],
  meetings: ["meetings"],
  voice: ["voice"],
  settings: [
    "settings-tabs",
    "settings-foot",
    "settings",
    "languages",
    "shortcuts",
    "intelligence",
    "provider",
    "privacy",
    "about",
    "licences",
  ],
};

function showRoute(name) {
  const route = ROUTES[name] ? name : "home";
  for (const [key, ids] of Object.entries(ROUTES)) {
    for (const id of ids) {
      const element = document.getElementById(id);
      if (element) {
        element.classList.toggle("away", key !== route);
      }
    }
  }
  for (const link of document.querySelectorAll(".rail a")) {
    if (link.dataset.route === route) {
      link.setAttribute("aria-current", "page");
    } else {
      link.removeAttribute("aria-current");
    }
  }
  showLibraryTab(libraryTab);
  showSettingsTab(settingsTab);
}

function routeNow() {
  const route = (location.hash || "#home").slice(1);
  localStorage.setItem("parlo.route", route);
  showRoute(route);
}

const ROUTE_KEYS = { 1: "home", 2: "meetings", 3: "voice", 4: "library", 5: "settings" };

document.addEventListener("keydown", (event) => {
  if (!event.altKey || event.ctrlKey || event.metaKey) {
    return;
  }
  const target = event.target;
  if (target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA")) {
    return;
  }
  if (event.key === "ArrowLeft") {
    event.preventDefault();
    window.history.back();
    return;
  }
  const route = ROUTE_KEYS[event.key];
  if (!route) {
    return;
  }
  event.preventDefault();
  if (location.hash === "#" + route) {
    showRoute(route);
  } else {
    location.hash = "#" + route;
  }
});

const LIBRARY_TABS = {
  history: ["history"],
  models: ["models", "catalog"],
  vocabulary: ["vocabulary"],
  files: ["dub"],
};

let libraryTab = "history";

function showLibraryTab(name) {
  const chosen = LIBRARY_TABS[name] ? name : "history";
  for (const [key, ids] of Object.entries(LIBRARY_TABS)) {
    for (const id of ids) {
      const element = document.getElementById(id);
      if (element) {
        element.classList.toggle("tab-away", key !== chosen);
      }
    }
  }
  for (const button of document.querySelectorAll("#library-tabs .tab")) {
    button.setAttribute("aria-selected", String(button.dataset.tab === chosen));
  }
}

for (const button of document.querySelectorAll("#library-tabs .tab")) {
  button.addEventListener("click", () => {
    libraryTab = button.dataset.tab;
    showLibraryTab(libraryTab);
  });
}

const SETTINGS_TABS = {
  dictation: ["settings"],
  languages: ["languages"],
  shortcuts: ["shortcuts"],
  advanced: ["intelligence", "provider", "privacy", "about"],
};

let settingsTab = "dictation";

function showSettingsTab(name) {
  const chosen = SETTINGS_TABS[name] ? name : "dictation";
  for (const [key, ids] of Object.entries(SETTINGS_TABS)) {
    for (const id of ids) {
      const element = document.getElementById(id);
      if (element) {
        element.classList.toggle("tab-away", key !== chosen);
      }
    }
  }
  for (const button of document.querySelectorAll("#settings-tabs .tab")) {
    button.setAttribute("aria-selected", String(button.dataset.tab === chosen));
  }
}

for (const button of document.querySelectorAll("#settings-tabs .tab")) {
  button.addEventListener("click", () => {
    settingsTab = button.dataset.tab;
    showSettingsTab(settingsTab);
  });
}

window.addEventListener("hashchange", routeNow);
if (!location.hash) {
  const remembered = localStorage.getItem("parlo.route");
  if (remembered) {
    location.hash = "#" + remembered;
  }
}
routeNow();
checkLocalUpdate().then(() => checkForUpdate().catch(() => {}));
loadModels();
loadSettings();
loadIntelligence();
loadVocabulary();

const voiceForm = document.getElementById("voice-form");
const voiceList = document.getElementById("voice-list");
const voiceStatus = document.getElementById("voice-status");
const voiceOrigin = document.getElementById("voice-origin");
const voiceAuthorisationRow = document.getElementById("voice-authorisation-row");
const voiceAuthorisation = document.getElementById("voice-authorisation");
const voiceEncrypt = document.getElementById("voice-encrypt");

function voiceHeaders() {
  return { "Content-Type": "application/json", Authorization: "Bearer " + token };
}

function describeVoice(profile) {
  const source =
    profile.consent.origin === "third-party"
      ? "someone else's voice, with permission"
      : "your own voice";
  const state = profile.consent.revoked_at ? "authorisation revoked" : "authorised";
  const seal = profile.encrypted ? " · sealed" : "";
  return profile.language + " · " + source + " · " + state + seal;
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
    const hasSample = (profile.artefacts || []).includes("sample.wav");
    if (hasSample) {
      actions.append(voiceButton("Play sample", () => playVoiceSample(profile)));
    } else {
      actions.append(
        voiceButton("Record 10 s", (event) => recordVoiceSample(profile, event.currentTarget))
      );
    }
    actions.append(voiceButton("Export", () => exportVoice(profile)));
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
    voiceStatus.textContent = payload.message || done;
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


async function loadFeatures() {
  try {
    const response = await fetch("/features", { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    const features = await response.json();
    featureTranslation.checked = features.translation !== false;
    featureVoice.checked = features.voice !== false;
    featureCaptions.checked = features.captions !== false;
  } catch (failure) {
    featuresNote.textContent = "";
  }
}

async function saveFeatures() {
  featuresNote.textContent = "Saving.";
  try {
    const response = await fetch("/features", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({
        translation: featureTranslation.checked,
        voice: featureVoice.checked,
        captions: featureCaptions.checked,
      }),
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      featuresNote.textContent = detail.error || "The switch was not saved.";
      await loadFeatures();
      return;
    }
    const off = [];
    if (!detail.translation) off.push("translation");
    if (!detail.voice) off.push("read aloud");
    if (!detail.captions) off.push("captions");
    featuresNote.textContent = off.length
      ? "Off: " + off.join(", ") + ". The models stay on this computer."
      : "Everything is on again.";
  } catch (failure) {
    featuresNote.textContent = "The switch was not saved. Check that Parlo is still running.";
  }
}

for (const input of [featureTranslation, featureVoice, featureCaptions]) {
  input.addEventListener("change", () => saveFeatures());
}
loadFeatures();
