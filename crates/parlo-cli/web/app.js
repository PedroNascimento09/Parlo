const token = new URLSearchParams(location.search).get("t") || "";
const versionMeta = document.querySelector('meta[name="parlo-version"]').content;
const version = versionMeta.startsWith("__") ? "" : versionMeta;
const masthead = document.querySelector(".masthead");

if (masthead) {
  const syncMasthead = () => {
    document.documentElement.style.setProperty(
      "--masthead-height",
      `${masthead.offsetHeight}px`,
    );
  };
  syncMasthead();
  if (typeof ResizeObserver === "function") {
    new ResizeObserver(syncMasthead).observe(masthead);
  }
  window.addEventListener("resize", syncMasthead);
}

const talk = document.getElementById("talk");
const label = talk.querySelector(".talk-label");
const sub = talk.querySelector(".talk-sub");
const meterFill = document.getElementById("meter-fill");
const progress = document.getElementById("progress");
const errorBox = document.getElementById("error");
const errorText = document.getElementById("error-text");
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
const shortcutEl = document.getElementById("status-shortcut");
const insightsKey = document.getElementById("insights-key");
const languagesEl = document.getElementById("status-languages");
const modeEl = document.getElementById("status-mode");
const micEl = document.getElementById("status-mic");
const latencyEl = document.getElementById("status-latency");
const versionEl = document.getElementById("status-version");
const modelsBox = document.getElementById("models");
const homeEngine = document.getElementById("home-engine");
const homeModel = document.getElementById("home-model");
const homeMic = document.getElementById("home-mic");
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
const dubSeparation = document.getElementById("dub-separation");
const dubLines = document.getElementById("dub-lines");
const dubExports = document.getElementById("dub-exports");
const dubQc = document.getElementById("dub-qc");
const dubIssues = document.getElementById("dub-issues");
const dubPrev = document.getElementById("dub-prev");
const dubNext = document.getElementById("dub-next");
const dubImport = document.getElementById("dub-import");
const dubSubtitles = document.getElementById("dub-subtitles");
const dubProjectLabel = document.getElementById("dub-project");
const dubProjectList = document.getElementById("dub-project-list");
const dubProjectsNote = document.getElementById("dub-projects-note");
const dubProjectsRefresh = document.getElementById("dub-projects-refresh");
const dubSave = document.getElementById("dub-save");
const dubCast = document.getElementById("dub-cast");
const dubCastRows = document.getElementById("dub-cast-rows");
const dubLanguage = document.getElementById("dub-language");
const dubTarget = document.getElementById("dub-target");
const dubTiming = document.getElementById("dub-timing");
const dubBackground = document.getElementById("dub-background");
const dubSpeed = document.getElementById("dub-speed");
const dubFastest = document.getElementById("dub-fastest");
const dubSlowest = document.getElementById("dub-slowest");
const dubUrlForm = document.getElementById("dub-url-form");
const dubUrl = document.getElementById("dub-url");
const dubUrlStart = document.getElementById("dub-url-start");
const dubCookies = document.getElementById("dub-cookies");
const dubClear = document.getElementById("dub-clear");
const batchFrom = document.getElementById("batch-from");
const batchTargets = document.getElementById("batch-targets");
const batchTiming = document.getElementById("batch-timing");
const batchBackground = document.getElementById("batch-background");
const batchChoose = document.getElementById("batch-choose");
const batchFiles = document.getElementById("batch-files");
const batchCreate = document.getElementById("batch-create");
const batchSelected = document.getElementById("batch-selected");
const batchStatus = document.getElementById("batch-status");
const batchList = document.getElementById("batch-list");
const watchPath = document.getElementById("watch-path");
const watchInterval = document.getElementById("watch-interval");
const watchAdd = document.getElementById("watch-add");
const watchStatus = document.getElementById("watch-status");
const watchList = document.getElementById("watch-list");
const settingsFrom = document.getElementById("settings-from");
const settingsTo = document.getElementById("settings-to");
const settingsStt = document.getElementById("settings-stt");
const settingsSwap = document.getElementById("settings-swap");
const settingsApply = document.getElementById("settings-apply");
const settingsUnload = document.getElementById("settings-unload");
const settingsManageMemory = document.getElementById("settings-manage-memory");
const settingsCompute = document.getElementById("settings-compute");
const settingsChannel = document.getElementById("settings-channel");
const channelNote = document.getElementById("channel-note");
const settingsShare = document.getElementById("settings-share");
const settingsSharePin = document.getElementById("settings-share-pin");
const settingsShareSave = document.getElementById("settings-share-save");
const shareNote = document.getElementById("share-note");
const settingsLocale = document.getElementById("settings-locale");
const settingsLocaleSave = document.getElementById("settings-locale-save");
const localeNote = document.getElementById("locale-note");
const computeNote = document.getElementById("compute-note");
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
let inputSource = null;
let inputKind = "";
let systemStream = null;
let meterFrame = null;
let recording = false;
let sampling = false;
let chunks = [];
let maxTimer = null;
let player = null;
let localUpdate = { available: false, latest: "", notes: "" };
let updateSequence = 0;
let modelsTimer = null;
let modelsPolling = false;
let welcomeTimer = null;
let modelsDownloading = false;
let chosenFile = null;
let dubbing = false;
let dubTimer = null;
let dubProject = null;
let issueCursor = -1;
let savedSettings = { from: "pt", to: "en", stt: "parakeet" };
let currentShortcut = "Ctrl+Alt+D";
let settingsBusy = false;
let playbackDone = null;

const versionLabel = version ? "v" + version : "";
versionEl.textContent = versionLabel;

function showError(message) {
  errorText.textContent = message;
  errorBox.hidden = false;
}

function clearError() {
  errorText.textContent = "";
  errorBox.hidden = true;
}

document.getElementById("error-close").addEventListener("click", clearError);

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

function describeSystemAudioError(failure) {
  if (failure && (failure.name === "NotAllowedError" || failure.name === "AbortError")) {
    return "The screen share was cancelled. Press Record a meeting again and tick Share audio.";
  }
  if (failure && failure.name === "NotSupportedError") {
    return "This browser cannot share sound. Use the microphone instead.";
  }
  return (failure && failure.message) || "The computer's sound could not be opened. Use the microphone instead.";
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
  micEl.textContent = statusWord("ready");
  return media;
}

function releaseMicrophone() {
  if (recording || sampling || meetingRecording) {
    return;
  }
  if (meterFrame) {
    cancelAnimationFrame(meterFrame);
    meterFrame = null;
  }
  if (inputSource) {
    inputSource.disconnect();
    inputSource = null;
  }
  inputKind = "";
  if (media) {
    media.getTracks().forEach((track) => track.stop());
    media = null;
  }
  if (context && context.state !== "closed") {
    context.close().catch(() => {});
  }
  context = null;
  analyser = null;
  processor = null;
  levels = null;
  meterFill.style.width = "0%";
  micEl.textContent = statusWord("not opened yet");
}

function drawMeter() {
  if (!analyser || !levels) {
    meterFrame = null;
    return;
  }
  meterFrame = requestAnimationFrame(drawMeter);
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

function ensureGraph() {
  if (context) {
    return;
  }
  const AudioContext = window.AudioContext || window.webkitAudioContext;
  context = new AudioContext();
  analyser = context.createAnalyser();
  analyser.fftSize = 2048;
  processor = context.createScriptProcessor(4096, 1, 1);
  processor.onaudioprocess = (event) => {
    if (!recording && !sampling && (!meetingRecording || meetingPaused)) {
      return;
    }
    chunks.push(new Float32Array(event.inputBuffer.getChannelData(0)));
  };
  const silence = context.createGain();
  silence.gain.value = 0;
  processor.connect(silence);
  silence.connect(context.destination);
  levels = new Float32Array(analyser.fftSize);
  if (!meterFrame) {
    drawMeter();
  }
}

function attach(stream, kind) {
  if (inputSource) {
    inputSource.disconnect();
  }
  inputSource = context.createMediaStreamSource(stream);
  inputSource.connect(analyser);
  inputSource.connect(processor);
  inputKind = kind;
}

async function ensureAudio() {
  ensureGraph();
  const stream = await openMicrophone();
  if (inputKind !== "mic") {
    attach(stream, "mic");
  }
  if (context.state === "suspended") {
    await context.resume();
  }
  return context;
}

async function ensureSystemAudio() {
  ensureGraph();
  if (!navigator.mediaDevices || !navigator.mediaDevices.getDisplayMedia) {
    throw new Error("This browser cannot capture the computer's sound.");
  }
  const display = await navigator.mediaDevices.getDisplayMedia({ video: true, audio: true });
  const audio = display.getAudioTracks();
  display.getVideoTracks().forEach((track) => track.stop());
  if (!audio.length) {
    display.getTracks().forEach((track) => track.stop());
    throw new Error("No sound was shared. Choose a tab or window and tick Share audio.");
  }
  if (systemStream) {
    systemStream.getTracks().forEach((track) => track.stop());
  }
  systemStream = new MediaStream(audio);
  audio[0].addEventListener("ended", () => {
    if (meetingRecording) {
      stopMeeting("The shared sound stopped. The lines already kept stay in the meeting.");
    }
  });
  attach(systemStream, "system");
  if (context.state === "suspended") {
    await context.resume();
  }
  return context;
}

function restoreMicrophone() {
  if (systemStream) {
    systemStream.getTracks().forEach((track) => track.stop());
    systemStream = null;
  }
  if (!context || !media || inputKind === "mic") {
    return;
  }
  attach(media, "mic");
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
  if (meetingRecording) {
    showError(
      t("studio.busyMeeting", "Finish the meeting before recording a voice sample."),
    );
    return;
  }
  clearError();
  try {
    const audio = await ensureAudio();
    chunks.length = 0;
    sampling = true;
    const started = Date.now();
    const tick = setInterval(() => {
      button.textContent = t("studio.recordingLeft", "{count} s left", {
        count: Math.max(0, 10 - Math.round((Date.now() - started) / 1000)),
      });
    }, 250);
    button.textContent = t("studio.recording", "Recording 10 s");
    await new Promise((done) => setTimeout(done, 10000));
    sampling = false;
    clearInterval(tick);
    button.textContent = t("studio.saving", "Saving the sample");
    const samples = downsample(mergedVoiceChunks(), audio.sampleRate, 16000);
    releaseMicrophone();
    const response = await fetch("/voice/sample", {
      method: "POST",
      headers: { "Content-Type": "audio/wav", Authorization: "Bearer " + token, "X-Parlo-Voice": profile.id },
      body: encodeWav(samples, 16000),
    });
    const answer = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(answer.error || t("studio.sampleError", "The sample was not saved."));
    }
    voiceStatus.textContent = t(
      "studio.sampleSaved",
      "Sample saved. Listen to it before you use this voice.",
    );
    await loadVoices();
  } catch (failure) {
    sampling = false;
    releaseMicrophone();
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
      throw new Error(answer.error || t("studio.readError", "The sample could not be read."));
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
      throw new Error(answer.error || t("studio.exportError", "The profile could not be exported."));
    }
    const blob = await response.blob();
    const link = document.createElement("a");
    link.href = URL.createObjectURL(blob);
    link.download = "parlo-voice-" + profile.id + ".txt";
    link.click();
    setTimeout(() => URL.revokeObjectURL(link.href), 5000);
    voiceStatus.textContent = t(
      "studio.exported",
      "Exported. The file marks the voice as synthetic and names its origin.",
    );
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

function play(blob, blockedMessage = "The translation is ready but the browser blocked playback. Click the page, then try again.") {
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
    showError(blockedMessage);
  });
}

async function speakText(text) {
  const clean = (text || "").trim();
  if (!clean) {
    return;
  }
  if (!featureVoice.checked) {
    showError("Read aloud is switched off in Settings.");
    return;
  }
  clearError();
  try {
    const response = await fetch("/v1/audio/speech", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({ input: clean, response_format: "wav" }),
    });
    if (!response.ok) {
      const detail = await response.json().catch(() => ({}));
      const message =
        detail && detail.error && detail.error.message
          ? detail.error.message
          : "The voice could not be generated.";
      throw new Error(message);
    }
    play(
      await response.blob(),
      "The voice is ready but the browser blocked playback. Click the page, then try again.",
    );
  } catch (failure) {
    showError(failure.message || "The voice could not be generated.");
  }
}

function listenButton(text, className = "swap speaker-chip") {
  const button = document.createElement("button");
  button.type = "button";
  button.className = className;
  button.textContent = t("text.Listen", "Listen");
  button.title = t("text.Hear this line aloud", "Hear this line aloud");
  button.addEventListener("click", async () => {
    const label = button.textContent;
    button.disabled = true;
    button.textContent = t("text.Reading", "Reading");
    try {
      await speakText(text);
    } finally {
      button.disabled = false;
      button.textContent = label;
    }
  });
  return button;
}

async function begin() {
  if (recording) {
    return;
  }
  if (meetingRecording) {
    showError("Finish the meeting before dictating.");
    return;
  }
  clearError();
  setProgress("");
  clearTranslation();
  stopPlayback();
  try {
    await ensureAudio();
  } catch (failure) {
    releaseMicrophone();
    micEl.textContent = statusWord("blocked");
    showError(describeMicrophoneError(failure));
    return;
  }
  chunks = [];
  recording = true;
  micEl.textContent = statusWord("listening");
  talk.setAttribute("aria-pressed", "true");
  label.textContent = t("text.Listening", "Listening");
  sub.textContent = t("text.Release when you are done", "Release when you are done");
  clearTimeout(maxTimer);
  maxTimer = setTimeout(() => finish(), MAX_SECONDS * 1000);
}

async function finish() {
  if (!recording) {
    return;
  }
  recording = false;
  micEl.textContent = statusWord("ready");
  clearTimeout(maxTimer);
  talk.setAttribute("aria-pressed", "false");
  label.textContent = t("text.Speak a translation", "Speak a translation");
  sub.textContent = t("text.Hold while you speak; the answer plays aloud.", "Hold while you speak; the answer plays aloud.");

  const captured = chunks;
  chunks = [];
  const rate = context ? context.sampleRate : 48000;
  const length = captured.reduce((total, chunk) => total + chunk.length, 0);
  const seconds = length / rate;
  if (seconds < MIN_SECONDS) {
    releaseMicrophone();
    showError("Hold the button while you speak, then release it.");
    return;
  }

  const merged = new Float32Array(length);
  let offset = 0;
  for (const chunk of captured) {
    merged.set(chunk, offset);
    offset += chunk.length;
  }
  releaseMicrophone();

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



async function refreshHealth() {
  try {
    const response = await fetch("/health", { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    const health = await response.json();
    engineEl.textContent = statusWord(health.engine || "unknown");
    if (homeEngine) {
      homeEngine.textContent = statusWord(health.engine || "unknown");
    }
    languagesEl.textContent = describePair(health.from, health.to);
  } catch (failure) {
    engineEl.textContent = statusWord("not answering");
    if (homeEngine) {
      homeEngine.textContent = statusWord("not answering");
    }
  }
  try {
    const response = await fetch("/settings", {
      cache: "no-store",
      headers: { Authorization: "Bearer " + token },
    });
    if (!response.ok) {
      return;
    }
    const values = await response.json();
    currentShortcut = values.shortcut || "not set";
    shortcutEl.textContent = currentShortcut;
    insightsKey.textContent = currentShortcut;
    const translating = values.write
      ? values.write !== values.from
      : values.translation && values.from !== values.to;
    const written = values.write || (translating ? values.to : values.from);
    modeEl.textContent = translating
      ? t("mode.translating", "translating to {language}", { language: languageName(written) })
      : t("mode.writing", "writing in {language}", { language: languageName(written) });
  } catch (failure) {
    shortcutEl.textContent = statusWord("unknown");
  }
  if (homeMic) {
    homeMic.textContent = statusWord(
      localStorage.getItem("parlo.mic") === "ready" ? "ready" : "not checked",
    );
  }
}

function showUpdate(candidate, notes) {
  updateVersion.textContent = candidate;
  if (notes) {
    updateLink.href = notes;
  }
  updateBox.hidden = false;
}

function allowInstall(allowed) {
  const focused = document.activeElement === installButton;
  installButton.hidden = !allowed;
  if (!allowed && focused && !updateBox.hidden) {
    updateLink.focus();
  }
}

async function checkLocalUpdate() {
  const sequence = ++updateSequence;
  try {
    const response = await fetch("/update", { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    const detail = await response.json();
    if (sequence !== updateSequence) {
      return;
    }
    localUpdate = detail;
    if (!localUpdate.available) {
      if (updateBox.hidden) {
        allowInstall(false);
      }
      return;
    }
    showUpdate(localUpdate.latest, localUpdate.notes);
    const build = localUpdate.build !== false;
    allowInstall(build);
    if (!build) {
      updateText.textContent = t(
        "update.noBuild",
        "Release {version} has no build for this computer yet. See what changed; the download appears here when it is ready.",
        { version: localUpdate.latest },
      );
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
  aboutNote.textContent = t("about.asking", "Asking GitHub for the latest release.");
  await checkLocalUpdate();
  aboutNote.textContent = localUpdate.available
    ? t(
        "about.available",
        "Release {version} is available. The box at the top of Home installs it.",
        { version: localUpdate.latest },
      )
    : t("about.latest", "This is the latest release.");
  aboutCheck.disabled = false;
});

function describeUpdatePhase(state) {
  if (state.phase === "downloading") {
    return t("update.downloading", "Downloading the new version: {percent}%.", {
      percent: state.percent || 0,
    });
  }
  if (state.phase === "unpacking") {
    return t("update.unpacking", "Unpacking the new version.");
  }
  if (state.phase === "checking") {
    return t("update.checking", "Checking the new program is there.");
  }
  if (state.phase === "ready") {
    return t("update.ready", "Ready. Parlo closes and starts the new version.");
  }
  return t("update.waiting", "Waiting for the release.");
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
    progressNote.textContent = t("update.closingShort", "Parlo is closing to replace itself.");
  }
}

async function installUpdate() {
  if (installButton.disabled) {
    return;
  }
  installButton.disabled = true;
  installButton.textContent = t("update.installing", "Installing…");
  clearError();
  updateBox.hidden = false;
  progressBox.hidden = false;
  progressFill.style.width = "0%";
  progressNote.textContent = t("update.starting", "Starting the download.");
  const poll = setInterval(pollUpdateProgress, 400);
  try {
    const response = await fetch("/update", {
      method: "POST",
      headers: { Authorization: "Bearer " + token },
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      updateText.textContent =
        detail.error ||
        t(
          "update.failed",
          "The update could not be installed. Run parlo update in a terminal instead.",
        );
      progressNote.textContent = "";
      installButton.disabled = false;
      installButton.textContent = t("update.install", "Install update");
      return;
    }
    progressFill.style.width = "100%";
    const installed = detail.version || localUpdate.latest || "";
    updateText.textContent = t(
      "update.unpacked",
      "Parlo {version} is unpacked. The window closes and the new version starts on its own.",
      { version: installed },
    );
    progressNote.textContent = t(
      "update.restartHint",
      "If Parlo does not come back by itself, press Restart now.",
    );
    const installFocused = document.activeElement === installButton;
    installButton.hidden = true;
    updateRestart.hidden = false;
    if (installFocused) {
      updateRestart.focus();
    }
  } catch (failure) {
    updateText.textContent = t(
      "update.closing",
      "Parlo is closing to replace itself, which is what should happen. The new version starts on its own.",
    );
    progressNote.textContent = t(
      "update.reopenHint",
      "If Parlo does not come back by itself, open it again.",
    );
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
    progressNote.textContent = t("update.startingAgain", "Parlo is starting again.");
  } catch (failure) {
    progressNote.textContent = t(
      "update.noAnswer",
      "Parlo did not answer. Open it again from the Start menu.",
    );
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
  const welcomeProgress = document.getElementById("welcome-progress");
  if (welcomeProgress) {
    welcomeProgress.hidden = !downloading;
    const fill = document.getElementById("welcome-fill");
    const models = modelState.models || [];
    const onDisk = models.filter((model) => model.installed).length;
    if (fill) {
      fill.style.width = (models.length ? Math.round((onDisk / models.length) * 100) : 0) + "%";
    }
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
    kind.textContent = MODEL_KIND_KEYS[model.kind]
      ? t(MODEL_KIND_KEYS[model.kind], model.kind)
      : model.kind || "";
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

    if (model.licence) {
      const licence = document.createElement("span");
      licence.className = "models-kind models-licence";
      licence.textContent = model.licence;
      name.append(licence);
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
      policy.setAttribute(
        "aria-label",
        t("models.freeAfter", "Free {name} after", { name: model.name }),
      );
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

  const modelsProgress = document.getElementById("models-progress");
  if (modelsProgress) {
    const models = state.models || [];
    const onDisk = models.filter((model) => model.installed).length;
    modelsProgress.hidden = !state.downloading;
    const fill = document.getElementById("models-fill");
    const note = document.getElementById("models-progress-note");
    if (fill) {
      fill.style.width = (models.length ? Math.round((onDisk / models.length) * 100) : 0) + "%";
    }
    if (note) {
      note.textContent = state.downloading
        ? onDisk + " of " + models.length + " models on disk. This page updates by itself."
        : "";
    }
  }

  const allModels = state.models || [];
  const dictationReady = allModels.some(
    (model) => model.installed && model.kind === "recogniser",
  );
  const missing = allModels.filter((model) => !model.installed);

  if (!missing.length) {
    modelsDownloading = false;
    modelsNote.textContent = "Every model this build offers is on this machine.";
    modelsInstall.hidden = true;
    reflectFirstRun(false, dictationReady, false);
    return true;
  }

  modelsDownloading = Boolean(state.downloading);
  if (modelsDownloading) {
    modelsNote.textContent =
      "Downloading models, this can take a few minutes on a normal connection. You can leave this page open.";
    modelsInstall.hidden = true;
    modelsDictation.hidden = true;
    reflectFirstRun(false, dictationReady, true);
    return false;
  }
  const byFeature = new Map();
  for (const model of missing) {
    const feature = t("text." + (model.feature || "A feature"), model.feature || "A feature");
    if (!byFeature.has(feature)) {
      byFeature.set(feature, []);
    }
    byFeature.get(feature).push(t("text." + model.name, model.name));
  }
  const joiner = " " + t("text.and", "and") + " ";
  const sentences = Array.from(
    byFeature,
    ([feature, names]) =>
      feature + " " + t("text.needs", "needs") + " " + names.join(joiner),
  );
  modelsNote.textContent = sentences.length
    ? "Missing: " + sentences.join("; ") + ". Each one downloads on its own."
    : "Some models are missing. They are a one time download and stay on this machine.";
  modelsInstall.hidden = false;
  modelsDictation.hidden = !missing.some((model) => model.id === "parakeet");
  modelsInstall.disabled = modelsPolling;
  reflectFirstRun(false, dictationReady, false);
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
    loadStudioVoices();
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
    return t("models.freeGb", "{size} GB free", {
      size: (bytes / (1024 * 1024 * 1024)).toFixed(0),
    });
  }
  return t("models.free", "{size} free", { size: formatSize(bytes) });
}

function describeHomeModels(state) {
  const installed = (state.models || []).filter((model) => model.installed);
  const bytes = installed.reduce((total, model) => total + (model.bytes || 0), 0);
  const free = formatFree(state.free_bytes);
  const talking =
    installed.length === 0
      ? t("models.noneInstalled", "none installed yet")
      : t("models.installed", "{count} installed · {size}", {
          count: installed.length,
          size: formatSize(bytes),
        });
  statusModels.textContent = free ? talking + " · " + free : talking;
  if (homeModel) {
    const recogniser = (state.models || []).find(
      (model) => model.installed && model.kind === "recogniser",
    );
    homeModel.textContent = recogniser
      ? recogniser.name || recogniser.id
      : t("common.notInstalled", "not installed");
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
welcomeInstall.addEventListener("click", () => downloadChosenModels(["parakeet"]));

modelsDictation.addEventListener("click", () => downloadModels("/models/dictation"));

const LANGUAGE_NAMES = {
  pt: "Portuguese",
  en: "English",
  es: "Spanish",
  fr: "French",
  it: "Italian",
  nl: "Dutch",
  de: "German",
  ru: "Russian",
};
const TIMING_NAMES = {
  natural: "Natural timing",
  concise: "Concise",
  "smart-fit": "Smart Fit",
  stretch: "Stretch",
  "lip-sync": "Lip sync",
};
const settingsShortcut = document.getElementById("settings-shortcut");
const settingsShortcutSave = document.getElementById("settings-shortcut-save");
const settingsHandsFree = document.getElementById("settings-hands-free");
const settingsHandsFreeSave = document.getElementById("settings-hands-free-save");
const settingsCaptions = document.getElementById("settings-captions");
const settingsCaptionsSave = document.getElementById("settings-captions-save");
const settingsUndo = document.getElementById("settings-undo");
const settingsUndoSave = document.getElementById("settings-undo-save");
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
const meetingSystem = document.getElementById("meeting-system");
const meetingSpeakers = document.getElementById("meeting-speakers");
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
  ["speakers", "Meetings"],
  ["text", "Text"],
];
let catalogTab = "dictation";
let catalogLast = [];

const historyList = document.getElementById("history-list");
const homeGreeting = document.getElementById("home-greeting");
const homeDate = document.getElementById("home-date");
const homeLast = document.getElementById("home-last");
const homeLastText = document.getElementById("home-last-text");
const homeLastWhen = document.getElementById("home-last-when");
const homeLastApp = document.getElementById("home-last-app");
const homeLastDot = document.getElementById("home-last-dot");
const homeLastLabel = document.getElementById("home-last-label");

function storedName() {
  return (localStorage.getItem("parlo.name") || "").trim();
}

function paintHome() {
  const name = storedName();
  const locale = document.documentElement.lang || "en";
  if (homeGreeting) {
    const hour = new Date().getHours();
    const base =
      hour < 5
        ? t("home.night", "Still awake?")
        : hour < 12
          ? t("home.morning", "Good morning")
          : hour < 19
            ? t("home.afternoon", "Good afternoon")
            : t("home.evening", "Good evening");
    homeGreeting.textContent = name
      ? t("home.greetingName", "{greeting}, {name}", { greeting: base, name })
      : base;
  }
  if (homeDate) {
    homeDate.textContent = new Intl.DateTimeFormat(locale, {
      weekday: "long",
      day: "numeric",
      month: "long",
    }).format(new Date());
  }
  if (homeLast) {
    const entry = historyEntries.find((item) => (item.translation || item.text || "").trim());
    if (entry) {
      homeLastLabel.textContent = t("home.last", "Last thing you said");
      homeLastText.textContent = (entry.translation || entry.text).trim();
      homeLastWhen.textContent = new Intl.DateTimeFormat(locale, {
        day: "numeric",
        month: "short",
        hour: "2-digit",
        minute: "2-digit",
      }).format(new Date((entry.at || 0) * 1000));
      const app = (entry.app || "").trim();
      homeLastApp.textContent = app;
      homeLastApp.hidden = !app;
      homeLastDot.hidden = !app;
      homeLast.classList.remove("empty");
      homeLast.hidden = false;
    } else {
      homeLastLabel.textContent = t("home.nothing", "Nothing yet");
      homeLastText.textContent = t(
        "home.nothingText",
        "Hold the shortcut, say a sentence, and the last one lands here.",
      );
      homeLastWhen.textContent = "";
      homeLastApp.textContent = "";
      homeLastApp.hidden = true;
      homeLastDot.hidden = true;
      homeLast.classList.add("empty");
      homeLast.hidden = false;
    }
  }
}

const historyNote = document.getElementById("history-note");
const historySearch = document.getElementById("history-search");
const historyApp = document.getElementById("history-app");
const historyFavourite = document.getElementById("history-favourite");
const statusModels = document.getElementById("status-models");
let historyEntries = [];
const historyClear = document.getElementById("history-clear");
const historyFilters = document.getElementById("history-filters");
const historyTools = document.getElementById("history-tools");

function languageName(code) {
  return t("language." + code, LANGUAGE_NAMES[code] || code.toUpperCase());
}

function stateLabel(value) {
  return t("state." + value, value);
}

function batchStateLabel(value) {
  return stateLabel(value);
}

function uiLocale() {
  return document.documentElement.lang || "en";
}

function describePair(from, to) {
  return t("settings.pair", "{from} to {to}", {
    from: languageName(from),
    to: languageName(to),
  });
}

function settingsChanged() {
  return (
    settingsFrom.value !== savedSettings.from ||
    settingsTo.value !== savedSettings.to ||
    settingsStt.value !== savedSettings.stt
  );
}

function refreshApply() {
  settingsApply.disabled = settingsBusy || !settingsChanged();
}

function reflectSettings(values) {
  settingsFrom.value = values.from || "en";
  settingsTo.value = values.to || "pt";
  settingsStt.value = values.stt || "parakeet";
  settingsManageMemory.checked = values.manage_memory !== false;
  settingsUnload.value = String(typeof values.unload === "number" ? values.unload : 30);
  settingsUnload.disabled = !settingsManageMemory.checked;
  settingsShortcut.value = values.shortcut || "Ctrl+Alt+D";
  currentShortcut = settingsShortcut.value;
  settingsHandsFree.value = values.hands_free || "Ctrl+Alt+Shift+D";
  settingsCaptions.value = values.captions || "Ctrl+Alt+C";
  if (settingsUndo) {
    settingsUndo.value = values.undo || "Ctrl+Alt+Z";
  }
  savedSettings = {
    from: settingsFrom.value,
    to: settingsTo.value,
    stt: settingsStt.value,
    write: values.write || "",
  };
  const pair = describePair(savedSettings.from, savedSettings.to);
  settingsNote.textContent = savedSettings.write
    ? t("settings.writes", "{pair}, dictation writes {language}", {
        pair,
        language: languageName(savedSettings.write),
      })
    : t("settings.keeps", "{pair}, dictation keeps the language spoken", { pair });
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
    "The old combination stops working.",
  ),
);
settingsHandsFreeSave.addEventListener("click", () =>
  saveShortcut(
    "hands_free",
    settingsHandsFree,
    "The old combination stops working.",
  ),
);
settingsCaptionsSave.addEventListener("click", () =>
  saveShortcut(
    "captions",
    settingsCaptions,
    "The old combination stops working.",
  ),
);
if (settingsUndoSave) {
  settingsUndoSave.addEventListener("click", () =>
    saveShortcut(
      "undo",
      settingsUndo,
      "The old combination stops working.",
    ),
  );
}





function captureShortcut(input, note) {
  input.addEventListener("focus", () => input.classList.add("capturing"));
  input.addEventListener("blur", () => input.classList.remove("capturing"));
  input.addEventListener("keydown", (event) => {
    if (event.key === "Tab") {
      return;
    }
    event.preventDefault();
    const modifiers = [];
    if (event.ctrlKey) modifiers.push("Ctrl");
    if (event.altKey) modifiers.push("Alt");
    if (event.shiftKey) modifiers.push("Shift");
    if (event.metaKey) modifiers.push("Win");
    if (["Control", "Alt", "Shift", "Meta", "Escape", "Tab"].includes(event.key)) {
      note.textContent = "Keep the modifiers held and press a letter, a digit or Space.";
      return;
    }
    if (!modifiers.length) {
      note.textContent = "A key on its own is refused: it would swallow typing everywhere.";
      return;
    }
    const named = { " ": "Space", Spacebar: "Space" };
    input.value = modifiers.concat([named[event.key] || event.key.toUpperCase()]).join("+");
    note.textContent = "Press the button beside it to save this combination.";
  });
}

for (const input of [settingsShortcut, settingsHandsFree, settingsCaptions, settingsUndo]) {
  if (input) {
    captureShortcut(input, settingsNote);
  }
}

let localeStrings = {};
function t(key, fallback, values = {}) {
  let text = localeStrings[key] || fallback || key;
  for (const [name, value] of Object.entries(values)) {
    text = text.replaceAll("{" + name + "}", String(value));
  }
  return text;
}

function statusWord(value) {
  const keys = {
    ready: "common.ready",
    unknown: "common.unknown",
    "not answering": "common.notAnswering",
    "not checked": "common.notChecked",
    "not installed": "common.notInstalled",
    "not set": "common.notSet",
    "not opened yet": "common.notOpened",
    blocked: "common.blocked",
    listening: "common.listening",
    starting: "common.starting",
    checking: "common.checking",
  };
  return keys[value] ? t(keys[value], value) : value;
}

function paintGateProgress(index, total) {
  const progress = document.getElementById("gate-progress");
  if (!progress) {
    return;
  }
  const template = localeStrings["gate.step"] || "Step {current} of {total}";
  progress.textContent = template
    .replace("{current}", String(index + 1))
    .replace("{total}", String(total));
}

async function applyLocale() {
  try {
    const response = await fetch("/locale", { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    const table = await response.json();
    localeStrings = table.strings || {};
    const code = table.locale || "en";
    document.documentElement.lang = code;
    document.documentElement.dir = code === "ar" ? "rtl" : "ltr";
    document.documentElement.dataset.locale = code;
    if (settingsLocale) {
      settingsLocale.replaceChildren(
        ...(table.available || []).map((entry) => {
          const option = document.createElement("option");
          option.value = entry.code;
          option.textContent = entry.name;
          return option;
        }),
      );
      settingsLocale.value = table.locale || "en";
    }
    document.querySelectorAll("[data-i18n]").forEach((node) => {
      const value = table.strings && table.strings[node.dataset.i18n];
      if (value) {
        node.textContent = value;
      }
    });
    document.querySelectorAll("[data-i18n-placeholder]").forEach((node) => {
      const value = table.strings && table.strings[node.dataset.i18nPlaceholder];
      if (value) {
        node.placeholder = value;
      }
    });
    document.querySelectorAll("[data-i18n-aria-label]").forEach((node) => {
      const value = table.strings && table.strings[node.dataset.i18nAriaLabel];
      if (value) {
        node.setAttribute("aria-label", value);
      }
    });
    const steps = Array.from(document.querySelectorAll(".gate-step"));
    const visible = steps.findIndex((step) => !step.hidden);
    if (visible !== -1) {
      paintGateProgress(visible, steps.length);
    }
    if (modelState.models.length) {
      describeHomeModels(modelState);
      if (modelsList) {
        describeModels(modelState);
      }
    }
    if (catalogLast.length && catalogList) {
      renderCatalog(catalogLast);
    }
    if (settingsLocale && settingsLocale.value) {
      showRecommendation(settingsLocale.value);
    }
    paintHome();
    paintStyleControls();
    refreshHealth();
    if (document.getElementById("voice-list")) {
      loadVoices();
      loadPacks();
      loadStudioVoices();
    }
  } catch (failure) {
    return;
  }
}

function loadEverything() {
  if (currentRoute === "home") {
    loadHistory();
  }
  loadVocabulary();
  loadIntelligence();
  loadCatalog();
}

async function loadSettings() {
  try {
    const response = await fetch("/settings", {
      cache: "no-store",
      headers: { Authorization: "Bearer " + token },
    });
    if (!response.ok) {
      return;
    }
    const values = await response.json();
    if (values.name) {
      localStorage.setItem("parlo.name", values.name);
      paintHome();
      paintSettingsName();
    }
    if (settingsCompute) {
      settingsCompute.value = values.compute === "cpu" ? "cpu" : "auto";
    }
    if (settingsChannel) {
      settingsChannel.value = values.channel === "preview" ? "preview" : "stable";
    }
    if (settingsShare) {
      settingsShare.checked = values.share === true;
      settingsSharePin.value = values.share_pin || "";
      shareNote.textContent = values.share
        ? "Sharing on port " + (values.share_port || 8788) + ". Every request needs the PIN. Restart Parlo after changing it."
        : "Off by default. Every request on the network port needs the PIN. Restart Parlo after changing it.";
    }
    reflectSettings(values);
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
  const pairChanged =
    settingsFrom.value !== savedSettings.from || settingsTo.value !== savedSettings.to;
  const reloads = settingsStt.value !== savedSettings.stt;
  lockSettings(true);
  settingsApply.textContent = reloads ? "Loading models…" : "Switching…";
  settingsNote.textContent = reloads
    ? "Parlo is loading the models again. This takes about twenty seconds."
    : pairChanged
      ? "Parlo keeps the models it already has and changes the pair in place."
      : "Parlo saves that in place, without touching the models.";

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
        write: savedSettings.write,
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

if (settingsCompute) {
  settingsCompute.addEventListener("change", async () => {
    try {
      const response = await fetch("/settings", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
        body: JSON.stringify({ compute: settingsCompute.value }),
      });
      const detail = await response.json().catch(() => ({}));
      if (!response.ok) {
        throw new Error(detail.error || "That setting cannot be used.");
      }
      computeNote.textContent = settingsCompute.value === "cpu"
        ? "Parlo uses the processor only. The graphics card is left to other programs."
        : "Parlo uses the graphics card when this machine has one.";
    } catch (failure) {
      showError(failure.message);
    }
  });
}

if (settingsChannel) {
  settingsChannel.addEventListener("change", async () => {
    try {
      const response = await fetch("/settings", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
        body: JSON.stringify({ channel: settingsChannel.value }),
      });
      const detail = await response.json().catch(() => ({}));
      if (!response.ok) {
        throw new Error(detail.error || "That update channel cannot be used.");
      }
      channelNote.textContent = settingsChannel.value === "preview"
        ? "Preview builds arrive sooner and can break. Stable releases still replace them when they are newer."
        : "Only tested releases arrive. A preview build installed earlier still updates to the next stable.";
      updateBox.hidden = true;
      await checkLocalUpdate();
    } catch (failure) {
      showError(failure.message);
    }
  });
}

if (settingsShareSave) {
  settingsShareSave.addEventListener("click", async () => {
    const body = { share: settingsShare.checked };
    const pin = settingsSharePin.value.trim();
    if (pin) {
      body.share_pin = pin;
    }
    try {
      const response = await fetch("/settings", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
        body: JSON.stringify(body),
      });
      const detail = await response.json().catch(() => ({}));
      if (!response.ok) {
        throw new Error(detail.error || "That sharing setting cannot be used.");
      }
      shareNote.textContent =
        "Saved. Restart Parlo so the network port opens or closes.";
    } catch (failure) {
      showError(failure.message);
    }
  });
}

if (settingsLocaleSave) {
  settingsLocaleSave.addEventListener("click", async () => {
    try {
      const response = await fetch("/settings", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
        body: JSON.stringify({ locale: settingsLocale.value }),
      });
      const detail = await response.json().catch(() => ({}));
      if (!response.ok) {
        throw new Error(detail.error || t("settings.languageError", "That language cannot be used."));
      }
      location.reload();
    } catch (failure) {
      showError(failure.message);
    }
  });
}

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
    const engine =
      detail.engine === "cloud" || detail.engine === "local"
        ? detail.engine
        : t("settings.theRules", "the rules");
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
  dubSave.hidden = true;
  dubCast.hidden = true;
  if (!dubProject) {
    dubProjectLabel.textContent = "";
    dubClear.hidden = true;
    dubLanguage.hidden = true;
  }
  dubDownload.hidden = false;
  dubDownload.href = "/dub/" + encodeURIComponent(result.id);
  dubDownload.textContent = t("dub.downloadNamed", "Download {name}", {
    name: result.name || t("dub.theDubbedFile", "the dubbed file"),
  });
  const lines = result.lines || [];
  dubExports.replaceChildren();
  for (const item of result.exports || []) {
    if (!item.url) {
      continue;
    }
    const link = document.createElement("a");
    link.className = "dub-export";
    link.href = item.url;
    link.download = item.name || "";
    link.textContent = item.label || item.format;
    dubExports.append(link);
  }
  dubExports.hidden = dubExports.childElementCount === 0;
  if (result.separation && result.separation !== "off") {
    dubSeparation.hidden = false;
    dubSeparation.textContent =
      (result.separation === "ducking" ? "Degraded: " : "") +
      (result.separation_note || "The background was kept.");
  } else {
    dubSeparation.hidden = true;
  }
  const flagged = lines.filter((line) => (line.issues || []).length > 0);
  issueCursor = -1;
  dubQc.hidden = false;
  dubIssues.textContent =
    flagged.length === 0
      ? "No issues"
      : flagged.length + (flagged.length === 1 ? " line needs" : " lines need") + " a look";
  dubPrev.disabled = flagged.length === 0;
  dubNext.disabled = flagged.length === 0;
  dubCount.textContent =
    lines.length +
    (lines.length === 1 ? " line" : " lines") +
    " · " +
    formatClock(result.seconds) +
    " of audio" +
    (result.to ? " · " + languageName(result.to) : "") +
    (result.timing && result.timing !== "natural"
      ? " · " + t("timing." + result.timing, TIMING_NAMES[result.timing] || result.timing)
      : "");
  dubLines.replaceChildren();
  for (const line of lines) {
    const item = document.createElement("li");
    item.className = "dub-line" + ((line.issues || []).length ? " issue" : "");
    item.dataset.line = line.id || "";

    const time = document.createElement("span");
    time.className = "dub-time";
    time.textContent = formatClock(line.start);

    const body = document.createElement("div");
    const translated = document.createElement("p");
    translated.className = "dub-translated";
    translated.textContent =
      line.translated ||
      (line.failed ? "This line failed." : line.skipped ? "This line is skipped." : "");
    if (line.translated) {
      const listen = listenButton(line.translated);
      listen.title = "Hear this dubbed line aloud";
      translated.append(document.createTextNode(" "), listen);
    }
    const source = document.createElement("p");
    source.className = "dub-source";
    source.textContent = [line.voice, line.source].filter(Boolean).join(" · ");
    body.append(translated, source);

    if ((line.issues || []).length) {
      const labels = document.createElement("ul");
      labels.className = "dub-issues";
      for (const issue of line.issues) {
        const badge = document.createElement("li");
        badge.className = "dub-issue";
        badge.textContent = issue;
        labels.append(badge);
      }
      body.append(labels);
    }
    if (line.failed || line.skipped) {
      const reason = document.createElement("p");
      reason.className = "dub-source";
      reason.textContent = line.failed
        ? "Failed: " + (line.failure || t("dub.lineFailed", "the engine could not render it"))
        : "Skipped. Retry it to put the line back.";
      body.append(reason);
    }
    if ((line.issues || []).length || line.failed || line.skipped) {
      const actions = document.createElement("div");
      actions.className = "dub-actions";
      if (line.id) {
        const retry = document.createElement("button");
        retry.type = "button";
        retry.className = "swap";
        retry.textContent = line.skipped ? "Put back" : "Retry";
        retry.addEventListener("click", () =>
          lineAction("retry", result.id, line.id, retry),
        );
        actions.append(retry);
        if (!line.skipped) {
          const skip = document.createElement("button");
          skip.type = "button";
          skip.className = "swap";
          skip.textContent = "Skip";
          skip.addEventListener("click", () =>
            lineAction("skip", result.id, line.id, skip),
          );
          actions.append(skip);
        }
      }
      body.append(actions);
    }

    item.append(time, body);
    dubLines.append(item);
  }
  dubResult.hidden = false;
}

async function lineAction(action, dubId, lineId, button) {
  if (!dubId || !lineId) {
    setDubStatus("This line cannot be changed.");
    return;
  }
  button.disabled = true;
  setDubStatus(action === "retry" ? "Retrying this line…" : "Skipping this line…");
  try {
    const response = await fetch(
      "/dub/" + encodeURIComponent(dubId) + "/" + action,
      {
        method: "POST",
        headers: {
          Authorization: "Bearer " + token,
          "X-Parlo-Line": lineId,
        },
      },
    );
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "That line could not be changed.");
    }
    showDub(detail);
    setDubStatus(action === "retry" ? "Line retried." : "Line skipped.");
  } catch (failure) {
    setDubStatus("");
    showError(failure.message || "That line could not be changed.");
  } finally {
    button.disabled = false;
  }
}

function jumpIssue(direction) {
  const items = Array.from(dubLines.querySelectorAll(".dub-line.issue"));
  if (items.length === 0) {
    setDubStatus("There are no issues to jump to.");
    return;
  }
  issueCursor = (issueCursor + direction + items.length) % items.length;
  const item = items[issueCursor];
  item.scrollIntoView({ block: "center", behavior: "smooth" });
  const focus = item.querySelector("button");
  if (focus) {
    focus.focus({ preventScroll: true });
  }
}

dubPrev.addEventListener("click", () => jumpIssue(-1));
dubNext.addEventListener("click", () => jumpIssue(1));

function fillDubTargets(project, chosen) {
  const codes = [];
  for (const code of [
    project.to,
    ...(project.targets || []),
    ...(project.languages || []),
  ]) {
    if (code && !codes.includes(code)) {
      codes.push(code);
    }
  }
  dubTarget.replaceChildren();
  for (const code of codes) {
    const option = document.createElement("option");
    option.value = code;
    option.textContent = languageName(code);
    dubTarget.append(option);
  }
  dubTarget.value = codes.includes(chosen) ? chosen : project.to;
  dubLanguage.hidden = codes.length === 0;
}

function showProject(project) {
  dubProject = project;
  dubQc.hidden = true;
  dubExports.hidden = true;
  fillDubTargets(project, dubTarget.value);
  dubDownload.hidden = true;
  dubClear.hidden = false;
  dubSave.hidden = false;
  const lines = project.segments || [];
  dubProjectLabel.textContent =
    (project.title || t("dub.importedSubtitles", "Imported subtitles")) +
    " · " +
    t(lines.length === 1 ? "dub.cueOne" : "dub.cueMany", "{count} cues", {
      count: lines.length,
    });
  const edited = lines.some((line) => line.state === "edited");
  dubCount.textContent =
    t(lines.length === 1 ? "dub.lineOne" : "dub.lineMany", "{count} lines", {
      count: lines.length,
    }) + (edited ? t("dub.edited", " · edited") : t("dub.imported", " · imported"));
  dubLines.replaceChildren();
  const field = (labelText, input) => {
    const label = document.createElement("label");
    label.className = "dub-field";
    const title = document.createElement("span");
    title.textContent = labelText;
    label.append(title, input);
    return label;
  };
  const voices = Array.isArray(project.voices) ? project.voices : [];
  const cast = project.cast || {};
  const speakers = [];
  for (const line of lines) {
    const name = (line.speaker || "").trim();
    if (name && !speakers.includes(name)) {
      speakers.push(name);
    }
  }
  const voiceSelect = (selected, defaultLabel) => {
    const select = document.createElement("select");
    select.className = "dub-input dub-voice";
    const fallback = document.createElement("option");
    fallback.value = "";
    fallback.textContent = defaultLabel;
    select.append(fallback);
    for (const voice of voices) {
      const option = document.createElement("option");
      option.value = voice.id;
      option.textContent = voice.name + (voice.installed ? "" : " · not installed");
      select.append(option);
    }
    select.value = voices.some((voice) => voice.id === selected) ? selected : "";
    return select;
  };
  dubCastRows.replaceChildren();
  dubCast.hidden = speakers.length === 0;
  for (const speaker of speakers) {
    const row = document.createElement("div");
    row.className = "dub-cast-row";
    row.dataset.speaker = speaker;
    const name = document.createElement("span");
    name.className = "dub-cast-name";
    name.textContent = speaker;
    const select = voiceSelect(cast[speaker], "No voice yet");
    select.classList.add("dub-cast-voice");
    row.append(name, select);
    dubCastRows.append(row);
  }
  for (const line of lines) {
    const item = document.createElement("li");
    item.className = "dub-line dub-edit-line";
    item.dataset.segment = line.id;

    const body = document.createElement("div");
    body.className = "dub-edit";
    const times = document.createElement("div");
    times.className = "dub-times";
    const start = document.createElement("input");
    start.className = "dub-input dub-start";
    start.type = "number";
    start.step = "0.001";
    start.min = "0";
    start.required = true;
    start.value = ((line.start_ms || 0) / 1000).toFixed(3);
    const end = document.createElement("input");
    end.className = "dub-input dub-end";
    end.type = "number";
    end.step = "0.001";
    end.min = "0";
    end.required = true;
    end.value = ((line.end_ms || 0) / 1000).toFixed(3);
    times.append(field("Start (s)", start), field("End (s)", end));
    const speaker = document.createElement("input");
    speaker.className = "dub-input dub-speaker";
    speaker.type = "text";
    speaker.maxLength = 120;
    speaker.placeholder = "Optional";
    speaker.value = line.speaker || "";
    const pace = document.createElement("select");
    pace.className = "dub-input dub-pace";
    for (const [value, label] of [
      ["0.9", "Slower"],
      ["1", "Natural"],
      ["1.1", "Faster"],
    ]) {
      const option = document.createElement("option");
      option.value = value;
      option.textContent = label;
      pace.append(option);
    }
    pace.value = String(line.pace || 1);
    const text = document.createElement("textarea");
    text.className = "dub-input dub-text";
    text.rows = 2;
    text.required = true;
    text.value = line.source_text || "";
    body.append(
      times,
      field("Speaker", speaker),
      field("Voice", voiceSelect(line.voice_id, "Speaker voice")),
      field("Pace", pace),
      field("Text", text)
    );
    const revisions = Array.isArray(line.revisions) ? line.revisions : [];
    if (revisions.length) {
      const history = document.createElement("ol");
      history.className = "dub-history";
      revisions.forEach((text, index) => {
        const stored = document.createElement("li");
        stored.textContent = "Version " + (index + 1) + ": " + text;
        history.append(stored);
      });
      const current = document.createElement("p");
      current.className = "dub-original";
      current.textContent = "Current: " + (line.source_text || "");
      body.append(history, current);
    }

    item.append(body);
    dubLines.append(item);
  }
  dubResult.hidden = false;
}

const DUB_STATUS = {
  done: "dub.ready",
  interrupted: "dub.interrupted",
  failed: "dub.failed",
  draft: "dub.notRendered",
  invalid: "dub.invalid",
};

function dubProjectStatus(value) {
  return t(DUB_STATUS[value] || "", value);
}

function projectSummary(project) {
  const parts = [];
  if (project.segments) {
    parts.push(
      t(project.segments === 1 ? "dub.lineOne" : "dub.lineMany", "{count} lines", {
        count: project.segments,
      }),
    );
  }
  if (project.seconds) {
    parts.push(t("dub.seconds", "{count} s", { count: Math.round(project.seconds) }));
  }
  if (project.from && project.to) {
    parts.push(describePair(project.from, project.to));
  }
  if (project.updated_at) {
    parts.push(new Date(project.updated_at * 1000).toLocaleString(uiLocale()));
  }
  return parts.join(" · ");
}

async function openProject(id) {
  const response = await fetch("/dub/project/" + encodeURIComponent(id), {
    headers: { Authorization: "Bearer " + token },
  });
  const detail = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(detail.error || "The project could not be opened.");
  }
  showProject(detail);
  document.getElementById("dub").scrollIntoView({ block: "start" });
}

async function recoverProject(id, button) {
  const label = button.textContent;
  button.disabled = true;
  button.textContent = "Recovering…";
  setDubStatus("Recovering the project from the saved source.");
  try {
    const response = await fetch("/dub/project/" + encodeURIComponent(id) + "/recover", {
      method: "POST",
      headers: { Authorization: "Bearer " + token },
    });
    const result = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(result.error || "The project could not be recovered.");
    }
    await openProject(id);
    showDub(result);
    setDubStatus("Recovered. The exports are ready again.");
    await loadDubProjects();
  } catch (failure) {
    setDubStatus("");
    showError(failure.message || "The project could not be recovered.");
  } finally {
    button.disabled = false;
    button.textContent = label;
  }
}

async function removeProject(project) {
  if (
    !window.confirm(
      t("dub.removeConfirm", "Remove {title} and its exports from this computer?", {
        title: project.title || t("dub.thisProject", "this project"),
      }),
    )
  ) {
    return;
  }
  const response = await fetch("/dub/project/" + encodeURIComponent(project.id), {
    method: "DELETE",
    headers: { Authorization: "Bearer " + token },
  });
  const detail = await response.json().catch(() => ({}));
  if (!response.ok) {
    showError(detail.error || "The project could not be removed.");
    return;
  }
  if (dubProject && dubProject.id === project.id) {
    dubProject = null;
    dubProjectLabel.textContent = "";
    dubResult.hidden = true;
  }
  await loadDubProjects();
}

function renderDubProjects(projects) {
  const items = projects.map((project) => {
    const body = [];
    if (project.error) {
      body.push(mediaMessage(project.error, "error"));
    }
    const actions = [];
    if (project.status !== "invalid") {
      const open = document.createElement("button");
      open.type = "button";
      open.className = "swap";
      open.textContent = t("dub.open", "Open");
      open.addEventListener("click", () => {
        openProject(project.id).catch((failure) =>
          setNote(dubProjectsNote, failure.message, "error"),
        );
      });
      actions.push(open);
    }
    if (project.status === "interrupted" || project.status === "failed") {
      const recover = document.createElement("button");
      recover.type = "button";
      recover.className = "install";
      recover.textContent = t("dub.recover", "Recover");
      recover.addEventListener("click", () => recoverProject(project.id, recover));
      actions.push(recover);
    }
    if (project.exports && project.exports.length) {
      const download = document.createElement("a");
      download.className = "swap";
      download.href = project.exports[0].url;
      download.textContent = t("dub.download", "Download");
      actions.push(download);
    }
    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "swap";
    remove.textContent = t("text.Remove", "Remove");
    remove.addEventListener("click", () => removeProject(project));
    actions.push(remove);
    return mediaCard({
      title: project.title || t("dub.untitledProject", "Untitled project"),
      meta: projectSummary(project),
      state: dubProjectStatus(project.status),
      tone:
        project.status === "failed" || project.status === "invalid" ? "failed" : project.status,
      body,
      actions,
    });
  });
  renderMedia(dubProjectList, items, {
    kicker: t("library.tabs.files", "Files"),
    title: t(
      "dub.noProjects",
      "No projects yet. Import subtitles or dub a file to start one.",
    ),
  });
}


async function loadDubProjects() {
  if (!dubProjectList) {
    return;
  }
  try {
    const response = await fetch("/dub/projects", {
      headers: { Authorization: "Bearer " + token },
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(payload.error || "The projects could not be listed.");
    }
    renderDubProjects(Array.isArray(payload.projects) ? payload.projects : []);
    dubProjectsNote.textContent = "Finished and interrupted renders stay here after a restart.";
  } catch (failure) {
    setNote(dubProjectsNote, failure.message || "The projects could not be listed.", "error");
  }
}

if (dubProjectsRefresh) {
  dubProjectsRefresh.addEventListener("click", () => loadDubProjects());
}
loadDubProjects();

let batchSelection = [];
let batchTimer = null;

function setBatchStatus(message) {
  batchStatus.textContent = message || "";
  batchStatus.hidden = !message;
}

function fillBatchTargets(preferred) {
  const checked = preferred || Array.from(batchTargets.querySelectorAll("input:checked")).map((input) => input.value);
  const from = batchFrom.value;
  batchTargets.replaceChildren();
  for (const code of Object.keys(LANGUAGE_NAMES)) {
    if (code === from) {
      continue;
    }
    const label = document.createElement("label");
    label.className = "check";
    const input = document.createElement("input");
    input.type = "checkbox";
    input.value = code;
    input.checked = checked.includes(code);
    const text = document.createElement("span");
    text.textContent = languageName(code);
    label.append(input, text);
    batchTargets.append(label);
  }
}

function fillBatchSource() {
  const codes = Object.keys(LANGUAGE_NAMES);
  batchFrom.replaceChildren();
  for (const code of codes) {
    const option = document.createElement("option");
    option.value = code;
    option.textContent = languageName(code);
    batchFrom.append(option);
  }
  batchFrom.value = codes.includes(savedSettings.from) ? savedSettings.from : codes[0];
  fillBatchTargets(codes.includes(savedSettings.to) ? [savedSettings.to] : []);
}

function selectedBatchTargets() {
  return Array.from(batchTargets.querySelectorAll("input:checked")).map((input) => input.value);
}

function batchSummary(batch) {
  const done = batch.items.filter((item) => item.state === "done").length;
  const failed = batch.items.filter((item) => item.state === "failed").length;
  const failure = failed ? t("batch.failedPart", " · {count} failed", { count: failed }) : "";
  if (batch.story) {
    const title = batch.items[0] ? batch.items[0].title : t("batch.story", "Story");
    return t("batch.storySummary", "{title} · story · {done} done", { title, done }) + failure;
  }
  const targets = batch.targets.map(languageName).join(", ");
  return (
    t(batch.items.length === 1 ? "batch.summaryOne" : "batch.summaryMany", "", {
      count: batch.items.length,
      from: languageName(batch.from),
      targets,
      done,
    }) + failure
  );
}

function renderBatches(batches) {
  const items = batches.map((batch) => {
    const done = batch.items.filter((item) => item.state === "done").length;
    const active = batch.state === "queued" || batch.state === "running";
    const body = [];
    if (active) {
      body.push(
        mediaMessage(
          t(
            done === 1 ? "text.{count} file ready" : "text.{count} files ready",
            "",
            { count: done },
          ),
          "running",
          batch.items.length ? (100 * done) / batch.items.length : 0,
        ),
      );
    }
    if (batch.items.length) {
      const rows = document.createElement("div");
      rows.className = "media-list";
      for (const item of batch.items) {
        const extra = [];
        if (item.state === "done") {
          const link = document.createElement("a");
          link.className = "swap";
          link.href =
            "/batch/" + encodeURIComponent(batch.id) + "/items/" + encodeURIComponent(item.id);
          link.textContent = t("text.Download", "Download");
          extra.push(link);
        }
        if (item.state === "failed") {
          extra.push(
            mediaMessage(item.error || t("text.This line failed.", "the render failed"), "error"),
          );
          const retry = document.createElement("button");
          retry.type = "button";
          retry.className = "swap";
          retry.textContent = t("text.Retry", "Retry");
          retry.addEventListener("click", () => retryBatchItem(batch.id, item.id, retry));
          extra.push(retry);
        }
        rows.append(
          mediaRow({
            text: item.title + " → " + languageName(item.to),
            state: stateLabel(item.state === "running" ? "rendering" : item.state),
            extra,
          }),
        );
      }
      body.push(rows);
    }
    const actions = [];
    if (batch.state === "done" || batch.state === "failed") {
      const report = document.createElement("a");
      report.className = "swap";
      report.href = "/batch/" + encodeURIComponent(batch.id) + "/report";
      report.textContent = t("text.Download report", "Download report");
      actions.push(report);
    }
    if (batch.state === "draft" && batch.items.length) {
      const start = document.createElement("button");
      start.type = "button";
      start.className = "install";
      start.textContent = t("text.Start queue", "Start queue");
      start.addEventListener("click", () => startBatch(batch.id, start));
      actions.push(start);
    }
    return mediaCard({
      title: batch.story
        ? t("batch.storyRender", "Story render")
        : t("batch.title", "Batch {id}", { id: batch.id.slice(-6) }),
      meta: batchSummary(batch),
      state: batchStateLabel(batch.state),
      tone: batch.state === "failed" ? "failed" : batch.state,
      body,
      actions,
    });
  });
  renderMedia(batchList, items, {
    kicker: t("library.tabs.batch", "Batch"),
    title: t("text.No batch yet. Choose files and at least one language.", "No batch yet."),
  });
  setNote(batchStatus, "");
}


async function loadBatches() {
  try {
    const response = await fetch("/batch", {
      headers: { Authorization: "Bearer " + token },
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(payload.error || "The batch queue could not be listed.");
    }
    const batches = Array.isArray(payload.batches) ? payload.batches : [];
    renderBatches(batches);
    if (batches.some((batch) => batch.state === "queued" || batch.state === "running")) {
      clearTimeout(batchTimer);
      batchTimer = setTimeout(loadBatches, 1500);
    }
  } catch (failure) {
    setBatchStatus(failure.message || "The batch queue could not be listed.");
  }
}

async function startBatch(batchId, button) {
  button.disabled = true;
  try {
    const response = await fetch("/batch/" + encodeURIComponent(batchId) + "/start", {
      method: "POST",
      headers: { Authorization: "Bearer " + token },
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "This batch could not start.");
    }
    await loadBatches();
  } catch (failure) {
    setBatchStatus(failure.message || "This batch could not start.");
    button.disabled = false;
  }
}

async function retryBatchItem(batchId, itemId, button) {
  button.disabled = true;
  try {
    const response = await fetch(
      "/batch/" + encodeURIComponent(batchId) + "/retry/" + encodeURIComponent(itemId),
      { method: "POST", headers: { Authorization: "Bearer " + token } },
    );
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "This render could not be retried.");
    }
    await loadBatches();
  } catch (failure) {
    setBatchStatus(failure.message || "This render could not be retried.");
    button.disabled = false;
  }
}

async function createBatch() {
  if (!batchSelection.length) {
    return;
  }
  const targets = selectedBatchTargets();
  if (!targets.length) {
    setBatchStatus("Choose at least one language.");
    return;
  }
  batchCreate.disabled = true;
  try {
    const response = await fetch("/batch", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer " + token,
      },
      body: JSON.stringify({
        from: batchFrom.value,
        targets,
        timing: batchTiming.value,
        keep_background: batchBackground.checked,
      }),
    });
    const created = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(created.error || "The batch could not be created.");
    }
    for (let index = 0; index < batchSelection.length; index += 1) {
      const file = batchSelection[index];
      setBatchStatus("Adding " + (index + 1) + " of " + batchSelection.length + " files.");
      const upload = await fetch("/batch/" + encodeURIComponent(created.id) + "/files", {
        method: "POST",
        headers: {
          "X-Parlo-Filename": encodeURIComponent(file.name),
          Authorization: "Bearer " + token,
        },
        body: file,
      });
      const detail = await upload.json().catch(() => ({}));
      if (!upload.ok) {
        throw new Error(detail.error || file.name + " could not be added.");
      }
    }
    const started = await fetch("/batch/" + encodeURIComponent(created.id) + "/start", {
      method: "POST",
      headers: { Authorization: "Bearer " + token },
    });
    const detail = await started.json().catch(() => ({}));
    if (!started.ok) {
      throw new Error(detail.error || "The batch could not start.");
    }
    setBatchStatus("Added " + batchSelection.length + " files. The queue runs one render at a time.");
    batchSelection = [];
    batchSelected.textContent = "";
    batchFiles.value = "";
    batchCreate.disabled = true;
    await loadBatches();
  } catch (failure) {
    setBatchStatus(failure.message || "The batch could not be created.");
    batchCreate.disabled = !batchSelection.length;
  }
}

batchChoose.addEventListener("click", () => batchFiles.click());
batchFiles.addEventListener("change", () => {
  batchSelection = Array.from(batchFiles.files || []);
  batchSelected.textContent = batchSelection.length
    ? batchSelection.length + (batchSelection.length === 1 ? " file ready" : " files ready")
    : "";
  batchCreate.disabled = batchSelection.length === 0;
});
batchFrom.addEventListener("change", () => fillBatchTargets());
batchCreate.addEventListener("click", () => createBatch());
fillBatchSource();
loadBatches();

const storyList = document.getElementById("story-list");
const storyTitle = document.getElementById("story-title");
const storyAuthor = document.getElementById("story-author");
const storyYear = document.getElementById("story-year");
const storySeries = document.getElementById("story-series");
const storyComment = document.getElementById("story-comment");
const storyLanguage = document.getElementById("story-language");
const storyCharacters = document.getElementById("story-characters");
const storyChapters = document.getElementById("story-chapters");
const storyAddCharacter = document.getElementById("story-add-character");
const storyAddChapter = document.getElementById("story-add-chapter");
const storySave = document.getElementById("story-save");
const storyRender = document.getElementById("story-render");
const storyNew = document.getElementById("story-new");
const storyStatus = document.getElementById("story-status");
const storyState = document.getElementById("story-state");
const storyQueue = document.getElementById("story-queue");
const storyAudiobook = document.getElementById("story-audiobook");

let storyVoices = [];
let storyCurrent = null;
let storyTimer = null;
let storyCounter = 0;

function setStoryStatus(message) {
  storyStatus.textContent = message || "";
  storyStatus.hidden = !message;
}

function storyIdentifier(prefix) {
  storyCounter += 1;
  return prefix + Date.now().toString(36) + storyCounter.toString(36);
}

function fillStoryLanguages() {
  storyLanguage.replaceChildren();
  for (const code of Object.keys(LANGUAGE_NAMES)) {
    const option = document.createElement("option");
    option.value = code;
    option.textContent = languageName(code);
    storyLanguage.append(option);
  }
  storyLanguage.value = LANGUAGE_NAMES[savedSettings.from] ? savedSettings.from : "pt";
}

function storyVoiceOptions(selected) {
  const select = document.createElement("select");
  select.className = "field-input story-voice";
  for (const voice of storyVoices) {
    const option = document.createElement("option");
    option.value = voice.id;
    option.textContent =
      voice.label + (voice.installed ? "" : " · " + t("common.notInstalled", "not installed"));
    select.append(option);
  }
  select.value = storyVoices.some((voice) => voice.id === selected)
    ? selected
    : storyVoices[0]?.id || "";
  return select;
}

function storyCharacterRow(character) {
  const row = document.createElement("div");
  row.className = "story-character-row";
  row.dataset.character = character.id;
  const name = document.createElement("input");
  name.className = "field-input story-name";
  name.type = "text";
  name.maxLength = 40;
  name.placeholder = t("text.Character name", "Character name");
  name.value = character.name || "";
  name.setAttribute("aria-label", t("text.Character name", "Character name"));
  const voice = storyVoiceOptions(character.voice);      voice.setAttribute(
        "aria-label",
        t("studio.voiceFor", "Voice for {name}", {
          name: character.name || t("studio.thisCharacter", "this character"),
        }),
      );
  const remove = document.createElement("button");
  remove.type = "button";
  remove.className = "swap";
  remove.textContent = t("text.Remove", "Remove");
  remove.addEventListener("click", () => {
    row.remove();
    refreshStorySpeakers();
  });
  row.append(name, voice, remove);
  return row;
}

function storyLineRow(line) {
  const row = document.createElement("div");
  row.className = "story-line-row";
  row.dataset.line = line.id;
  const speaker = document.createElement("select");
  speaker.className = "field-input story-speaker";
  speaker.dataset.selected = line.speaker || "";
  const text = document.createElement("input");
  text.className = "field-input story-text";
  text.type = "text";
  text.maxLength = 1000;
  text.placeholder = t("text.Line of dialogue", "Line of dialogue");
  text.value = line.text || "";
  text.setAttribute("aria-label", t("text.Line of dialogue", "Line of dialogue"));
  const pace = document.createElement("select");
  pace.className = "field-input story-pace";
  for (const [value, label] of [
    ["0.9", t("text.Slower", "Slower")],
    ["1", t("text.Natural", "Natural")],
    ["1.1", t("text.Faster", "Faster")],
  ]) {
    const option = document.createElement("option");
    option.value = value;
    option.textContent = label;
    pace.append(option);
  }
  pace.value = String(line.pace || 1);
  const state = document.createElement("span");
  state.className = "story-line-state";
  state.textContent = stateLabel(line.state || "pending");
  const audio = document.createElement("a");
  audio.className = "story-line-audio";
  audio.textContent = t("text.Play line", "Play line");
  audio.hidden = !line.out || !storyCurrent;
  if (storyCurrent && line.out) {
    audio.href = "/stories/" + encodeURIComponent(storyCurrent.id) + "/lines/" + encodeURIComponent(line.id);
  }
  const remove = document.createElement("button");
  remove.type = "button";
  remove.className = "swap";
  remove.textContent = "Remove";
  remove.addEventListener("click", () => row.remove());
  row.append(speaker, text, pace, state, audio, remove);
  return row;
}

function refreshStorySpeakers() {
  const characters = Array.from(storyCharacters.querySelectorAll("[data-character]"));
  for (const select of storyChapters.querySelectorAll(".story-speaker")) {
    const chosen = select.value || select.dataset.selected || "";
    select.replaceChildren();
    for (const row of characters) {
      const name = row.querySelector(".story-name").value.trim() || "Unnamed";
      const option = document.createElement("option");
      option.value = row.dataset.character;
      option.textContent = name;
      select.append(option);
    }
    select.value = characters.some((row) => row.dataset.character === chosen)
      ? chosen
      : characters[0]?.dataset.character || "";
    select.dataset.selected = select.value;
  }
}

function storyChapterCard(chapter) {
  const card = document.createElement("article");
  card.className = "story-chapter";
  card.dataset.chapter = chapter.id;
  const head = document.createElement("div");
  head.className = "story-chapter-head";
  const title = document.createElement("input");
  title.className = "field-input story-chapter-title";
  title.type = "text";
  title.maxLength = 80;
  title.placeholder = "Chapter title";
  title.value = chapter.title || "";
  const remove = document.createElement("button");
  remove.type = "button";
  remove.className = "swap";
  remove.textContent = "Remove chapter";
  remove.addEventListener("click", () => card.remove());
  const add = document.createElement("button");
  add.type = "button";
  add.className = "swap";
  add.textContent = "Add line";
  const lines = document.createElement("div");
  lines.className = "story-lines";
  add.addEventListener("click", () => {
    const row = storyLineRow({
      id: storyIdentifier("l"),
      speaker: storyCharacters.querySelector("[data-character]")?.dataset.character || "",
      text: "",
      pace: 1,
      state: "pending",
    });
    lines.append(row);
    refreshStorySpeakers();
  });
  head.append(title, add, remove);
  for (const line of chapter.lines || []) {
    lines.append(storyLineRow(line));
  }
  card.append(head, lines);
  return card;
}

function fillStoryEditor(story) {
  storyCurrent = story && story.id ? story : null;
  storyTitle.value = story?.title || "";
  storyAuthor.value = story?.author || "";
  storyYear.value = story?.year || "";
  storySeries.value = story?.series || "";
  storyComment.value = story?.comment || "";
  storyLanguage.value = story?.language || storyLanguage.value;
  storyCharacters.replaceChildren();
  const characters = story?.characters?.length
    ? story.characters
    : [
        { id: "c1", name: "", voice: storyVoices[0]?.id || "" },
        { id: "c2", name: "", voice: storyVoices[1]?.id || storyVoices[0]?.id || "" },
        { id: "c3", name: "", voice: storyVoices[2]?.id || storyVoices[0]?.id || "" },
      ];
  for (const character of characters) {
    storyCharacters.append(storyCharacterRow(character));
  }
  storyChapters.replaceChildren();
  const chapters = story?.chapters?.length
    ? story.chapters
    : [
        {
          id: "ch1",
          title: "Chapter 1",
          lines: [
            { id: "l1", speaker: characters[0]?.id || "c1", text: "", pace: 1, state: "pending" },
          ],
        },
      ];
  for (const chapter of chapters) {
    storyChapters.append(storyChapterCard(chapter));
  }
  refreshStorySpeakers();
  updateStoryState(story);
}

function updateStoryState(story) {
  storyState.replaceChildren();
  if (!story) {
    storyState.hidden = true;
    storyQueue.hidden = true;
    storyAudiobook.hidden = true;
    return;
  }
  storyState.hidden = false;
  const rows = [
    ["State", story.state || "draft"],
    ["Lines rendered", String(story.rendered || 0)],
    ["Lines reused", String(story.cached || 0)],
    ["Lines failed", String(story.failed || 0)],
  ];
  for (const [label, value] of rows) {
    const term = document.createElement("dt");
    term.textContent = label;
    const detail = document.createElement("dd");
    detail.textContent = value;
    storyState.append(term, detail);
  }
  storyQueue.hidden = !story.batch;
  if (story.batch) {
    storyQueue.textContent = "Batch " + story.batch.slice(-6) + " in the queue";
  }
  storyAudiobook.hidden = story.state !== "done" || !story.batch;
  if (!storyAudiobook.hidden) {
    storyAudiobook.href = "/stories/" + encodeURIComponent(story.id) + "/audiobook";
  }
  const lines = [];
  for (const chapter of story.chapters || []) {
    for (const line of chapter.lines || []) {
      lines.push(line);
    }
  }
  for (const line of lines) {
    const row = storyChapters.querySelector('[data-line="' + CSS.escape(line.id) + '"]');
    if (!row) {
      continue;
    }
    const state = row.querySelector(".story-line-state");
    state.textContent = stateLabel(line.state || "pending");
    state.classList.toggle("failed", line.state === "failed");
    const audio = row.querySelector(".story-line-audio");
    audio.hidden = !line.out;
    if (line.out) {
      audio.href = "/stories/" + encodeURIComponent(story.id) + "/lines/" + encodeURIComponent(line.id);
    }
    if (line.error) {
      state.title = line.error;
    }
  }
}

function collectStory() {
  return {
    title: storyTitle.value.trim(),
    author: storyAuthor.value.trim() || null,
    year: storyYear.value ? Number(storyYear.value) : null,
    series: storySeries.value.trim() || null,
    comment: storyComment.value.trim() || null,
    language: storyLanguage.value,
    characters: Array.from(storyCharacters.querySelectorAll("[data-character]")).map((row) => ({
      id: row.dataset.character,
      name: row.querySelector(".story-name").value.trim(),
      voice: row.querySelector(".story-voice").value,
    })),
    chapters: Array.from(storyChapters.querySelectorAll("[data-chapter]")).map((card) => ({
      id: card.dataset.chapter,
      title: card.querySelector(".story-chapter-title").value.trim(),
      lines: Array.from(card.querySelectorAll("[data-line]")).map((row) => ({
        id: row.dataset.line,
        speaker: row.querySelector(".story-speaker").value,
        text: row.querySelector(".story-text").value.trim(),
        pace: Number(row.querySelector(".story-pace").value),
      })),
    })),
  };
}

async function saveStory() {
  const story = collectStory();
  const path = storyCurrent ? "/stories/" + encodeURIComponent(storyCurrent.id) : "/stories";
  const response = await fetch(path, {
    method: storyCurrent ? "PUT" : "POST",
    headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
    body: JSON.stringify(story),
  });
  const saved = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(saved.error || "This story could not be saved.");
  }
  storyCurrent = saved;
  fillStoryEditor(saved);
  await loadStories();
  return saved;
}

function renderStoryCards(stories) {
  const items = stories.map((story) => {
    const open = document.createElement("button");
    open.type = "button";
    open.className = "swap";
    open.textContent = t("text.Open", "Open");
    open.addEventListener("click", () => loadStory(story.id));
    const characters = story.characters.length || 0;
    const chapters = story.chapters.length || 0;
    return mediaCard({
      title: story.title,
      meta: [
        t(characters === 1 ? "text.{count} word" : "text.{count} words", "", { count: characters }),
        t(
          chapters === 1 ? "text.{count} line of audio" : "text.{count} lines of audio",
          "",
          { count: chapters },
        ),
      ].join(" · "),
      state: stateLabel(story.state || "draft"),
      tone: story.state === "failed" ? "failed" : story.state || "draft",
      actions: [open],
    });
  });
  renderMedia(storyList, items, {
    kicker: t("library.tabs.stories", "Stories"),
    title: t("text.No story yet. Write one and give its characters voices.", "No story yet."),
  });
}


async function loadStories() {
  try {
    const response = await fetch("/stories", {
      headers: { Authorization: "Bearer " + token },
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(payload.error || "The stories could not be listed.");
    }
    renderStoryCards(Array.isArray(payload.stories) ? payload.stories : []);
  } catch (failure) {
    setStoryStatus(failure.message || "The stories could not be listed.");
  }
}

async function loadStory(id) {
  try {
    const response = await fetch("/stories/" + encodeURIComponent(id), {
      headers: { Authorization: "Bearer " + token },
    });
    const story = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(story.error || "This story could not be opened.");
    }
    fillStoryEditor(story);
    await loadStories();
    if (story.state === "rendering") {
      pollStory(story.id);
    }
  } catch (failure) {
    setStoryStatus(failure.message || "This story could not be opened.");
  }
}

function pollStory(id) {
  clearTimeout(storyTimer);
  storyTimer = setTimeout(async () => {
    try {
      const response = await fetch("/stories/" + encodeURIComponent(id), {
        headers: { Authorization: "Bearer " + token },
      });
      const story = await response.json().catch(() => ({}));
      if (response.ok && storyCurrent && storyCurrent.id === story.id) {
        storyCurrent = story;
        updateStoryState(story);
        if (story.state === "rendering") {
          pollStory(id);
        } else if (story.state === "done") {
          setStoryStatus("The story is ready. Download it from the queue.");
          await loadStories();
        } else if (story.state === "failed") {
          setStoryStatus(story.error || "The story stopped. Retry it from the queue.");
          await loadStories();
        }
      }
    } catch (failure) {
      setStoryStatus(failure.message || "The story state could not be read.");
    }
  }, 1500);
}

async function renderStory() {
  storyRender.disabled = true;
  setStoryStatus("Saving the story before its render.");
  try {
    const saved = await saveStory();
    const response = await fetch("/stories/" + encodeURIComponent(saved.id) + "/render", {
      method: "POST",
      headers: { Authorization: "Bearer " + token },
    });
    const story = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(story.error || "This story could not start.");
    }
    storyCurrent = story;
    updateStoryState(story);
    setStoryStatus("Rendering in the queue. Finished lines are kept if you stop it.");
    await loadBatches();
    await loadStories();
    pollStory(story.id);
  } catch (failure) {
    setStoryStatus(failure.message || "This story could not start.");
  } finally {
    storyRender.disabled = false;
  }
}

async function loadStoryVoices() {
  const fallback = [
    { id: "pt_PT-tugão-medium", label: "Portuguese Portugal · tugão", installed: false },
    { id: "pt_BR-cadu-medium", label: "Portuguese Brazil · cadu", installed: false },
    { id: "pt_BR-faber-medium", label: "Portuguese Brazil · faber", installed: false },
  ];
  try {
    const response = await fetch("/stories/voices", {
      headers: { Authorization: "Bearer " + token },
    });
    const payload = await response.json().catch(() => ({}));
    storyVoices = response.ok && Array.isArray(payload.voices) ? payload.voices : fallback;
  } catch {
    storyVoices = fallback;
  }
  fillStoryLanguages();
  fillStoryEditor(null);
  await loadStories();
}

storyAddCharacter.addEventListener("click", () => {
  storyCharacters.append(
    storyCharacterRow({ id: storyIdentifier("c"), name: "", voice: storyVoices[0]?.id || "" }),
  );
  refreshStorySpeakers();
});
storyAddChapter.addEventListener("click", () => {
  storyChapters.append(
    storyChapterCard({
      id: storyIdentifier("ch"),
      title: "Chapter " + (storyChapters.children.length + 1),
      lines: [
        {
          id: storyIdentifier("l"),
          speaker: storyCharacters.querySelector("[data-character]")?.dataset.character || "",
          text: "",
          pace: 1,
          state: "pending",
        },
      ],
    }),
  );
  refreshStorySpeakers();
});
storyNew.addEventListener("click", () => {
  storyCurrent = null;
  fillStoryEditor(null);
  setStoryStatus("");
  loadStories();
});
storySave.addEventListener("click", async () => {
  setStoryStatus("Saving the story.");
  try {
    await saveStory();
    setStoryStatus("Saved. Render it to hear each character.");
  } catch (failure) {
    setStoryStatus(failure.message || "This story could not be saved.");
  }
});
storyRender.addEventListener("click", () => renderStory());
loadStoryVoices();

let watchTimer = null;
let watchIngested = 0;

function setWatchStatus(message) {
  watchStatus.textContent = message || "";
  watchStatus.hidden = !message;
}

function watchSummary(folder) {
  const targets = folder.targets
    .map(languageName)
    .join(", ");
  return (
    languageName(folder.from) +
    " to " +
    targets +
    " · every " +
    folder.interval_seconds +
    "s · " +
    folder.ingested +
    (folder.ingested === 1 ? " file ingested" : " files ingested") +
    (folder.pending ? " · " + folder.pending + " settling" : "")
  );
}

function renderWatch(folders) {
  watchList.replaceChildren();
  if (!folders.length) {
    const empty = document.createElement("p");
    empty.className = "batch-empty";
    empty.textContent = "No folder is watched yet.";
    watchList.append(empty);
    return;
  }
  for (const folder of folders) {
    const card = document.createElement("article");
    card.className = "watch-card";
    const head = document.createElement("div");
    head.className = "watch-card-head";
    const path = document.createElement("h4");
    path.textContent = folder.path;
    const state = document.createElement("span");
    state.className = "watch-state";
    state.textContent = stateLabel(folder.state);
    head.append(path, state);
    const summary = document.createElement("p");
    summary.className = "batch-summary";
    summary.textContent = watchSummary(folder);
    card.append(head, summary);
    if (folder.last_error) {
      const error = document.createElement("p");
      error.className = "batch-error";
      error.textContent = folder.last_error;
      card.append(error);
    }
    const actions = document.createElement("div");
    actions.className = "watch-card-actions";
    const scan = document.createElement("button");
    scan.type = "button";
    scan.className = "swap";
    scan.textContent = "Scan now";
    scan.addEventListener("click", () => scanWatch(folder.id, scan));
    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "swap";
    remove.textContent = "Remove";
    remove.addEventListener("click", () => removeWatch(folder.id, remove));
    actions.append(scan, remove);
    card.append(actions);
    watchList.append(card);
  }
}

async function loadWatch() {
  try {
    const response = await fetch("/watch", {
      headers: { Authorization: "Bearer " + token },
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(payload.error || "The watched folders could not be listed.");
    }
    const folders = Array.isArray(payload.folders) ? payload.folders : [];
    renderWatch(folders);
    const ingested = folders.reduce((total, folder) => total + folder.ingested, 0);
    if (ingested !== watchIngested) {
      watchIngested = ingested;
      await loadBatches();
    }
    clearTimeout(watchTimer);
    if (folders.length) {
      watchTimer = setTimeout(loadWatch, 3000);
    }
  } catch (failure) {
    setWatchStatus(failure.message || "The watched folders could not be listed.");
  }
}

async function addWatch() {
  const path = watchPath.value.trim();
  if (!path) {
    setWatchStatus("Choose the folder to watch.");
    watchPath.focus();
    return;
  }
  const targets = selectedBatchTargets();
  if (!targets.length) {
    setWatchStatus("Choose at least one language.");
    return;
  }
  watchAdd.disabled = true;
  try {
    const response = await fetch("/watch", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer " + token,
      },
      body: JSON.stringify({
        path,
        from: batchFrom.value,
        targets,
        timing: batchTiming.value,
        interval_seconds: Number(watchInterval.value) || 2,
        keep_background: batchBackground.checked,
      }),
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "This folder could not be watched.");
    }
    watchPath.value = "";
    setWatchStatus("Watching " + detail.path + ". Drop a media file into it.");
    await loadWatch();
  } catch (failure) {
    setWatchStatus(failure.message || "This folder could not be watched.");
  } finally {
    watchAdd.disabled = false;
  }
}

async function scanWatch(id, button) {
  button.disabled = true;
  try {
    const response = await fetch("/watch/" + encodeURIComponent(id) + "/scan", {
      method: "POST",
      headers: { Authorization: "Bearer " + token },
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "This folder could not be scanned.");
    }
    await loadWatch();
    await loadBatches();
  } catch (failure) {
    setWatchStatus(failure.message || "This folder could not be scanned.");
  } finally {
    button.disabled = false;
  }
}

async function removeWatch(id, button) {
  button.disabled = true;
  try {
    const response = await fetch("/watch/" + encodeURIComponent(id) + "/remove", {
      method: "POST",
      headers: { Authorization: "Bearer " + token },
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "This folder could not be removed.");
    }
    setWatchStatus("The folder is no longer watched. Its queue keeps every file already ingested.");
    await loadWatch();
  } catch (failure) {
    setWatchStatus(failure.message || "This folder could not be removed.");
  } finally {
    button.disabled = false;
  }
}

watchAdd.addEventListener("click", () => addWatch());
loadWatch();

function editedProjectSegments() {
  return Array.from(dubLines.querySelectorAll("[data-segment]")).map((row) => {
    const start = Number(row.querySelector(".dub-start").value);
    const end = Number(row.querySelector(".dub-end").value);
    const sourceText = row.querySelector(".dub-text").value;
    if (!Number.isFinite(start) || !Number.isFinite(end) || start < 0 || end <= start) {
      throw new Error("Each line needs a start before its end.");
    }
    if (!sourceText.trim()) {
      throw new Error("Each line needs text.");
    }
    return {
      id: row.dataset.segment,
      start_ms: Math.round(start * 1000),
      end_ms: Math.round(end * 1000),
      speaker: row.querySelector(".dub-speaker").value.trim() || null,
      voice_id: row.querySelector(".dub-voice").value || null,
      pace: Number(row.querySelector(".dub-pace").value),
      source_text: sourceText,
    };
  });
}

function editedProjectCast() {
  const cast = {};
  for (const row of dubCastRows.querySelectorAll("[data-speaker]")) {
    const voice = row.querySelector(".dub-cast-voice").value;
    if (voice) {
      cast[row.dataset.speaker] = voice;
    }
  }
  return cast;
}

async function saveProject() {
  if (!dubProject || dubbing) {
    return;
  }
  clearError();
  let segments;
  try {
    segments = editedProjectSegments();
  } catch (failure) {
    showError(failure.message);
    return;
  }
  dubSave.disabled = true;
  setDubStatus("Saving the edited lines on this computer.");
  try {
    const response = await fetch("/dub/project/" + encodeURIComponent(dubProject.id), {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer " + token,
      },
      body: JSON.stringify({ cast: editedProjectCast(), segments }),
    });
    const detail = await response.json().catch(() => ({}));
    if (response.status === 401) {
      throw new Error("Parlo lost the connection to its engine. Close Parlo and open it again.");
    }
    if (!response.ok) {
      throw new Error(detail.error || "The edited lines could not be saved.");
    }
    showProject(detail);
    setDubStatus("Saved. The next dub uses these lines.");
  } catch (failure) {
    setDubStatus("");
    showError(failure.message || "The edited lines could not be saved.");
  } finally {
    dubSave.disabled = false;
  }
}

async function refreshDubProject(language) {
  if (!dubProject) {
    return;
  }
  const id = dubProject.id;
  try {
    const response = await fetch("/dub/project/" + encodeURIComponent(id), {
      headers: { Authorization: "Bearer " + token },
    });
    if (!response.ok) {
      return;
    }
    const project = await response.json();
    if (!dubProject || dubProject.id !== id) {
      return;
    }
    dubProject = project;
    fillDubTargets(project, language);
  } catch {
    return;
  }
}

function describeLinkProblem(value) {
  let parsed;
  try {
    parsed = new URL(value);
  } catch {
    return "Paste the whole link, starting with http or https.";
  }
  if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
    return "Only http and https links can be dubbed.";
  }
  return "";
}

function timingHeaders() {
  const mode = dubTiming.value;
  const headers = { "X-Parlo-Timing": mode };
  if (dubBackground.checked) {
    headers["X-Parlo-Background"] = "1";
  }
  if (mode === "smart-fit") {
    const fastest = Number(dubFastest.value);
    const slowest = Number(dubSlowest.value);
    if (!(fastest >= 1 && fastest <= 4)) {
      throw new Error("Smart Fit needs a fastest speed between 1 and 4.");
    }
    if (!(slowest >= 0.25 && slowest <= 1)) {
      throw new Error("Smart Fit needs a slowest speed between 0.25 and 1.");
    }
    headers["X-Parlo-Fastest"] = String(fastest);
    headers["X-Parlo-Slowest"] = String(slowest);
  }
  return headers;
}

async function startDubUrl(value) {
  const url = (value || "").trim();
  if (dubbing) {
    return;
  }
  if (recording) {
    showError("Let go of the record button before dubbing a link.");
    return;
  }
  const problem = describeLinkProblem(url);
  if (problem) {
    showError(problem);
    return;
  }
  clearError();
  clearTranslation();
  stopPlayback();
  dubbing = true;
  dubUrlStart.disabled = true;
  dubStart.disabled = true;
  dubUrlStart.textContent = "Dubbing…";
  dubResult.hidden = true;
  const started = Date.now();
  const tick = () =>
    setDubStatus(
      "Downloading and dubbing the link locally, " +
        formatClock((Date.now() - started) / 1000) +
        " elapsed.",
    );
  tick();
  dubTimer = setInterval(tick, 1000);
  try {
    const headers = {
      "X-Parlo-Url": url,
      Authorization: "Bearer " + token,
    };
    const cookies = dubCookies ? dubCookies.value.trim() : "";
    if (cookies) {
      headers["X-Parlo-Cookies"] = encodeURIComponent(cookies);
    }
    if (dubProject) {
      headers["X-Parlo-Project"] = dubProject.id;
      if (dubTarget.value) {
        headers["X-Parlo-To"] = dubTarget.value;
      }
    }
    Object.assign(headers, timingHeaders());
    const response = await fetch("/dub", { method: "POST", headers });
    const detail = await response.json().catch(() => ({}));
    if (response.status === 401) {
      throw new Error("Parlo lost the connection to its engine. Close Parlo and open it again.");
    }
    if (!response.ok) {
      throw new Error(detail.error || "The link could not be dubbed.");
    }
    showDub(detail);
    if (dubProject && detail.project_id) {
      refreshDubProject(detail.to);
    }
    setDubStatus("Dubbed from the link in " + formatClock((Date.now() - started) / 1000) + ". Download it above, or keep using Parlo.");
  } catch (failure) {
    setDubStatus("");
    showError(failure.message || "The link could not be dubbed. Check that it points at a video or audio file and try again.");
  } finally {
    clearInterval(dubTimer);
    dubTimer = null;
    dubbing = false;
    dubUrlStart.disabled = false;
    dubUrlStart.textContent = "Dub this link";
    dubStart.disabled = !chosenFile;
  }
}

async function importSubtitles(file) {
  if (!file || dubbing) {
    return;
  }
  clearError();
  dubImport.disabled = true;
  dubStart.disabled = true;
  setDubStatus("Reading " + file.name + " on this computer.");
  try {
    const response = await fetch("/dub/import", {
      method: "POST",
      headers: {
        "Content-Type": "text/plain; charset=utf-8",
        "X-Parlo-Filename": encodeURIComponent(file.name),
        "X-Parlo-From": savedSettings.from,
        "X-Parlo-To": savedSettings.to,
        Authorization: "Bearer " + token,
      },
      body: file,
    });
    const detail = await response.json().catch(() => ({}));
    if (response.status === 401) {
      throw new Error("Parlo lost the connection to its engine. Close Parlo and open it again.");
    }
    if (!response.ok) {
      throw new Error(detail.error || "The subtitle file could not be imported.");
    }
    const reload = await fetch("/dub/project/" + encodeURIComponent(detail.id), {
      headers: { Authorization: "Bearer " + token },
    });
    const project = await reload.json().catch(() => ({}));
    if (!reload.ok) {
      throw new Error(project.error || "The imported project could not be reloaded.");
    }
    showProject(project);
    setDubStatus(
      "Imported " +
        (project.segments || []).length +
        " cues ready. Choose a file, then press Dub this file.",
    );
  } catch (failure) {
    setDubStatus("");
    showError(failure.message || "The subtitle file could not be imported.");
  } finally {
    dubImport.disabled = false;
    dubStart.disabled = !chosenFile;
  }
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
    const headers = {
      "Content-Type": "application/octet-stream",
      "X-Parlo-Filename": encodeURIComponent(chosenFile.name),
      Authorization: "Bearer " + token,
    };
    if (dubProject) {
      headers["X-Parlo-Project"] = dubProject.id;
      if (dubTarget.value) {
        headers["X-Parlo-To"] = dubTarget.value;
      }
    }
    Object.assign(headers, timingHeaders());
    const response = await fetch("/dub", {
      method: "POST",
      headers,
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
    if (dubProject && detail.project_id) {
      refreshDubProject(detail.to);
    }
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

dubUrlForm.addEventListener("submit", (event) => {
  event.preventDefault();
  startDubUrl(dubUrl.value);
});

dubTiming.addEventListener("change", () => {
  dubSpeed.hidden = dubTiming.value !== "smart-fit";
});

dubSave.addEventListener("click", () => saveProject());

dubImport.addEventListener("click", () => dubSubtitles.click());

dubSubtitles.addEventListener("change", () => {
  const file = dubSubtitles.files && dubSubtitles.files[0];
  importSubtitles(file);
  dubSubtitles.value = "";
});

dubClear.addEventListener("click", () => {
  dubProject = null;
  dubProjectLabel.textContent = "";
  dubClear.hidden = true;
  dubSave.hidden = true;
  dubCast.hidden = true;
  dubResult.hidden = true;
  dubLanguage.hidden = true;
  setDubStatus("Imported subtitles removed. The next dub will use speech recognition.");
});

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
    setNote(meetingNote, "The meetings could not be read.", "error");
  }
}

function mediaTemplate(id) {
  const template = document.getElementById(id);
  return template ? template.content.firstElementChild.cloneNode(true) : null;
}

function mediaCard({ title, meta, state, tone, body, actions }) {
  const node = mediaTemplate("media-card");
  node.dataset.tone = tone || "open";
  node.querySelector(".media-card-title").textContent = title;
  const metaNode = node.querySelector(".media-card-meta");
  if (meta) {
    metaNode.textContent = meta;
  } else {
    metaNode.remove();
  }
  const stateNode = node.querySelector(".media-card-state");
  if (state) {
    stateNode.textContent = state;
  } else {
    stateNode.remove();
  }
  const bodyNode = node.querySelector(".media-card-body");
  for (const part of body || []) {
    if (part) {
      bodyNode.append(part);
    }
  }
  if (!bodyNode.childElementCount) {
    bodyNode.remove();
  }
  const actionNode = node.querySelector(".media-card-actions");
  for (const action of actions || []) {
    if (action) {
      actionNode.append(action);
    }
  }
  if (!actionNode.childElementCount) {
    actionNode.remove();
  }
  return node;
}

function mediaEmpty({ kicker, title, text, hint }) {
  const node = mediaTemplate("media-empty");
  const put = (selector, value) => {
    const target = node.querySelector(selector);
    if (value) {
      target.textContent = value;
    } else {
      target.remove();
    }
  };
  put(".empty-kicker", kicker);
  put(".empty-title", title);
  put(".empty-text", text);
  put(".empty-hint", hint);
  if (!hint) {
    node.querySelector(".empty-actions").remove();
  }
  return node;
}

function mediaMessage(text, kind, percent) {
  const node = mediaTemplate("media-message");
  node.dataset.kind = kind || "info";
  node.querySelector(".media-message-text").textContent = text;
  if (typeof percent === "number") {
    const track = node.querySelector(".media-message-track");
    const share = Math.max(0, Math.min(100, Math.round(percent)));
    track.hidden = false;
    track.setAttribute("role", "progressbar");
    track.setAttribute("aria-valuemin", "0");
    track.setAttribute("aria-valuemax", "100");
    track.setAttribute("aria-valuenow", String(share));
    node.querySelector(".media-message-fill").style.width = share + "%";
  }
  return node;
}

function mediaRow({ text, state, extra }) {
  const node = mediaTemplate("media-row");
  node.querySelector(".media-row-text").textContent = text;
  const stateNode = node.querySelector(".media-row-state");
  if (state) {
    stateNode.textContent = state;
  } else {
    stateNode.remove();
  }
  const slot = node.querySelector(".media-row-extra");
  for (const part of extra || []) {
    if (part) {
      slot.append(part);
    }
  }
  if (!slot.childElementCount) {
    slot.remove();
  }
  return node;
}

function renderMedia(host, items, copy) {
  host.replaceChildren();
  if (!items.length) {
    if (copy) {
      host.append(mediaEmpty(copy));
    }
    return false;
  }
  host.append(...items);
  return true;
}

function setNote(node, text, kind) {
  if (!node) {
    return;
  }
  node.dataset.kind = kind || "info";
  node.textContent = text || "";
  node.hidden = !text;
}

function renderMeetings(meetings) {
  const heading = meetingList.previousElementSibling;
  const ask = document.getElementById("meetings").querySelector("details.more");
  if (heading && heading.classList.contains("meeting-group")) {
    heading.hidden = !meetings.length;
  }
  if (ask) {
    ask.hidden = !meetings.length;
  }
  const items = meetings.map((meeting) => {
    const body = [];
    const spoken = (meeting.segments || []).filter((segment) => segment.speaker !== "Note");
    if (spoken.length) {
      const lines = document.createElement("ol");
      lines.className = "meeting-lines";
      for (const segment of spoken) {
        const line = document.createElement("li");
        line.className = "meeting-line";
        const when = document.createElement("span");
        when.className = "meeting-when";
        when.textContent = meetingClock(segment.at_ms);
        const who = document.createElement("span");
        who.className = segment.named ? "meeting-who named" : "meeting-who";
        who.textContent = segment.spoken_by || segment.speaker;
        const said = document.createElement("p");
        said.className = "meeting-said";
        said.textContent = segment.text;
        said.append(document.createTextNode(" "), listenButton(segment.text, "meeting-listen"));
        line.append(when, who, said);
        lines.append(line);
      }
      body.push(lines);
    }
    const speakers = (meeting.speakers || []).filter((name) => name !== "Note");
    if (speakers.length) {
      const row = document.createElement("div");
      row.className = "meeting-row";
      const label = document.createElement("span");
      label.className = "meeting-label";
      label.textContent = t("text.Speakers", "Speakers");
      row.append(label);
      for (const speaker of speakers) {
        const chip = document.createElement("button");
        chip.type = "button";
        chip.className = "swap speaker-chip";
        chip.textContent = speaker;
        chip.title = t(
          "text.Name this speaker everywhere in this meeting",
          "Name this speaker everywhere in this meeting",
        );
        chip.addEventListener("click", () => nameSpeaker(meeting, speaker));
        row.append(chip);
      }
      body.push(row);
    }
    const summary = document.createElement("ul");
    summary.className = "meeting-summary";
    for (const decision of meeting.decisions || []) {
      const line = document.createElement("li");
      line.textContent = t("text.Agreed · {speaker} · {text}", "Agreed · {speaker} · {text}", {
        speaker: decision.speaker,
        text: decision.text,
      });
      summary.append(line);
    }
    for (const action of meeting.actions || []) {
      const line = document.createElement("li");
      line.textContent = action.due
        ? t("text.To do · {owner} · {due} · {text}", "To do · {owner} · {due} · {text}", action)
        : t("text.To do · {owner} · {text}", "To do · {owner} · {text}", action);
      summary.append(line);
    }
    if (summary.childElementCount) {
      body.push(summary);
    }
    for (const note of (meeting.segments || []).filter((segment) => segment.speaker === "Note")) {
      const line = document.createElement("p");
      line.className = "meeting-note";
      line.textContent = t("text.Note · {time} · {text}", "Note · {time} · {text}", {
        time: meetingClock(note.at_ms),
        text: note.text,
      });
      body.push(line);
    }
    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "swap";
    remove.textContent = t("text.Delete", "Delete");
    remove.addEventListener("click", () => deleteMeeting(meeting.id));
    const download = document.createElement("button");
    download.type = "button";
    download.className = "swap";
    download.textContent = t("text.Save this meeting as a Markdown file", "Save as Markdown");
    download.addEventListener("click", () => downloadMeeting(meeting));
    const day = new Date(meeting.started_at || 0);
    const lines = (meeting.segments || []).length;
    return mediaCard({
      title: meeting.title || t("text.Untitled", "Untitled"),
      meta: [
        day.toLocaleDateString(),
        lines
          ? t(
              lines === 1 ? "text.{count} line of audio" : "text.{count} lines of audio",
              "",
              { count: lines },
            )
          : "",
      ]
        .filter(Boolean)
        .join(" · "),
      body,
      actions: [remove, download],
    });
  });
  renderMedia(meetingList, items, {
    kicker: t("text.No transcript yet", "No transcript yet"),
    title: t("text.A meeting becomes text, line by line", "A meeting becomes text, line by line"),
    text: t(
      "text.Give it a title and press Record a meeting. Every line lands here with the time it was said, and with speaker separation installed, who said it. The transcript never leaves this computer.",
      "Give it a title and press Record a meeting. The transcript never leaves this computer.",
    ),
    hint: t(
      "text.Or import a transcript you already have.",
      "Or import a transcript you already have.",
    ),
  });
  setNote(
    meetingNote,
    meetings.length
      ? t(
          meetings.length === 1
            ? "text.{count} meeting kept on this computer."
            : "text.{count} meetings kept on this computer.",
          "",
          { count: meetings.length },
        )
      : "",
  );
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
    setNote(meetingNote, "The meeting could not be saved as a file. Try again.", "error");
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
    setNote(meetingNote, failure.message || "The transcript could not be imported.", "error");
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
    setNote(meetingNote, failure.message || "The speaker could not be named.", "error");
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
    setNote(meetingNote, "The meeting could not be deleted. Try again.", "error");
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
  for (const [wrong, right] of Object.entries(profile.snippets || {})) {
    rows.push({ kind: "snippet", wrong, right });
  }
  if (!rows.length) {
    vocabularyNote.textContent =
      "Nothing here yet. A snippet is the quickest win: say \"my linkedin\" and Parlo writes your link.";
    return;
  }
  for (const row of rows) {
    const term = document.createElement("dt");
    const label =
      row.kind === "term" ? "Name: " : row.kind === "snippet" ? "Snippet: " : "Short form: ";
    term.textContent = label + row.wrong;
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
  const count = rows.length === 1 ? "1 entry kept" : rows.length + " entries kept";
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
let meetingHint = null;

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

function meetingSource() {
  return meetingSystem.checked ? "the computer's sound" : "the microphone";
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
      "Records " + meetingSource() + " into a transcript, a few seconds behind. With Speaker separation installed in Model library, every line says who spoke.";
    return;
  }
  const kept = meetingLines === 1 ? "1 line kept" : meetingLines + " lines kept";
  const voices = meetingHint ? "listening for " + meetingHint + " voices, " : "";
  meetingLiveNote.textContent =
    "Recording " + meetingSource() + ", " + voices + kept + ". " + (last ? "Last: " + last : "Speak normally.");
}

async function meetingSegment(wav) {
  const response = await fetch("/meeting/window", {
    method: "POST",
    headers: {
      "Content-Type": "audio/wav",
      Authorization: "Bearer " + token,
      "x-parlo-meeting": meetingId,
      "x-parlo-at": String(meetingElapsed()),
    },
    body: wav,
  });
  if (!response.ok) {
    const detail = await response.json().catch(() => ({}));
    throw new Error(detail.error || "Parlo answered with status " + response.status + ".");
  }
  const result = await response.json();
  return (result.text || "").trim();
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
      await loadMeetings();
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
  if (sampling) {
    showError("Finish the voice sample before recording a meeting.");
    return;
  }
  if (!welcomeBox.hidden || modelsDownloading) {
    showError("Download the models first; a meeting is written down on this machine.");
    return;
  }
  clearError();
  const useSystem = Boolean(meetingSystem.checked);
  try {
    if (useSystem) {
      await ensureSystemAudio();
    } else {
      await ensureAudio();
    }
  } catch (failure) {
    micEl.textContent = useSystem ? "not used" : "blocked";
    showError(useSystem ? describeSystemAudioError(failure) : describeMicrophoneError(failure));
    return;
  }
  const request = { title: meetingTitle.value.trim() || "Meeting" };
  const speakers = Number.parseInt(meetingSpeakers.value, 10);
  if (Number.isInteger(speakers)) {
    request.speakers = speakers;
  }
  try {
    const response = await fetch("/meeting/start", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify(request),
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "The meeting could not be started.");
    }
    meetingId = detail.id;
  } catch (failure) {
    showError(failure.message || "The meeting could not be started.");
    restoreMicrophone();
    return;
  }
  meetingRecording = true;
  meetingHint = Number.isInteger(speakers) ? speakers : null;
  meetingPaused = false;
  meetingPausedAt = 0;
  meetingPausedTotal = 0;
  meetingPause.textContent = "Pause";
  meetingPause.setAttribute("aria-pressed", "false");
  meetingPause.disabled = false;
  meetingSystem.disabled = true;
  meetingNotesRow.hidden = false;
  meetingNotes.value = "";
  meetingStarted = performance.now();
  meetingLines = 0;
  meetingFailures = 0;
  chunks = [];
  meetingLive.setAttribute("aria-pressed", "true");
  micEl.textContent = useSystem ? "not used" : "recording";
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
  meetingHint = null;
  meetingPaused = false;
  meetingPausedAt = 0;
  meetingPause.textContent = "Pause";
  meetingPause.setAttribute("aria-pressed", "false");
  meetingPause.disabled = true;
  meetingSystem.disabled = false;
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
  restoreMicrophone();
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
    setNote(meetingNote, "The meetings could not be exported. Try again.", "error");
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
    const produced =
      answer.by === "rules"
        ? "the local rules"
        : "the local model" + (answer.model ? ", " + answer.model : "");
    meetingSummaryText.textContent =
      (answer.summary || "The answer was empty.") + " — summarised by " + produced;
    if (answer.note) {
      meetingSummaryText.textContent += " · the model could not be used: " + answer.note;
    }
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

meetingSystem.addEventListener("change", () => {
  if (meetingRecording) {
    return;
  }
  meetingLiveNote.textContent = meetingSystem.checked
    ? "Records the computer's sound. Press Record a meeting, choose a tab or window, and tick Share audio."
    : "Records the microphone into a transcript, a few seconds behind. With Speaker separation installed, every line says who spoke.";
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

const MODEL_KIND_KEYS = {
  asr: "catalog.kind.asr",
  translation: "catalog.kind.mt",
  tts: "catalog.kind.tts",
  voice: "catalog.kind.voice",
  speaker: "catalog.kind.speaker",
};

let catalogEntries = [];
let recommendLanguage = (document.documentElement.lang || "en").split("-")[0];
let machine = null;

function languageLabel(code) {
  try {
    const names = new Intl.DisplayNames([document.documentElement.lang || "en"], { type: "language" });
    return names.of(code) || code.toUpperCase();
  } catch (failure) {
    return code.toUpperCase();
  }
}

function renderCatalogRecommendation(entries) {
  const host = document.getElementById("catalog-recommend");
  if (!host) {
    return;
  }
  host.replaceChildren();
  host.setAttribute("role", "region");
  host.setAttribute("aria-labelledby", "catalog-recommend-title");
  const recognisers = entries.filter(
    (entry) => entry.kind === "asr" && entry.unit && Array.isArray(entry.languages) && entry.languages.length,
  );
  if (!recognisers.length) {
    host.hidden = true;
    return;
  }
  const languages = [...new Set(recognisers.flatMap((entry) => entry.languages))].sort();
  if (!languages.includes(recommendLanguage)) {
    recommendLanguage = languages.includes("en") ? "en" : languages[0];
  }
  const ranked = recognisers
    .filter((entry) => entry.languages.includes(recommendLanguage))
    .sort(
      (left, right) =>
        (left.bench?.value ?? Number.MAX_SAFE_INTEGER) -
          (right.bench?.value ?? Number.MAX_SAFE_INTEGER) ||
        (left.size_mb ?? Number.MAX_SAFE_INTEGER) - (right.size_mb ?? Number.MAX_SAFE_INTEGER),
    );
  const pick = ranked[0];
  if (!pick) {
    host.hidden = true;
    return;
  }
  const title = document.createElement("h3");
  title.id = "catalog-recommend-title";
  title.textContent = t("catalog.recommendTitle", "Parlo recommends");
  const body = document.createElement("p");
  body.className = "catalog-recommend-body";
  body.textContent = t(
    "catalog.recommendBody",
    "For {language}, {name} fits best: one download covers {count} languages, so you do not need a separate model for each.",
    {
      language: languageLabel(recommendLanguage),
      name: pick.name || pick.id,
      count: pick.languages.length,
    },
  );
  const choose = document.createElement("label");
  choose.className = "catalog-recommend-choose";
  choose.textContent = t("text.Language", "Language");
  const select = document.createElement("select");
  select.className = "field-input";
  select.id = "catalog-recommend-language";
  select.setAttribute("aria-label", t("text.Language", "Language"));
  for (const code of languages) {
    const option = document.createElement("option");
    option.value = code;
    option.textContent = languageLabel(code);
    option.selected = code === recommendLanguage;
    select.append(option);
  }
  select.addEventListener("change", () => {
    recommendLanguage = select.value;
    renderCatalogRecommendation(entries);
  });
  choose.append(select);
  const action = document.createElement("button");
  action.type = "button";
  action.className = "install";
  const installed = modelState.models.some((model) => model.id === pick.unit && model.installed);
  if (installed) {
    action.textContent = t("text.On disk", "On disk");
    action.disabled = true;
  } else {
    action.textContent = t("text.Download", "Download");
    action.addEventListener("click", () => downloadCatalogModel(pick, action));
  }
  action.setAttribute("aria-label", `${pick.name || pick.id}: ${action.textContent}`);
  host.append(title, body, choose, action);
  const translator = entries.find(
    (entry) => entry.kind === "mt" && Array.isArray(entry.languages) && entry.languages.length > 2,
  );
  if (translator) {
    const note = document.createElement("p");
    note.className = "catalog-recommend-body";
    note.textContent =
      t("catalog.kind.mt", "Translation") +
      ": " +
      t("catalog.coverage", "One download covers {count} languages", {
        count: translator.languages.length,
      });
    const multi = document.createElement("button");
    multi.type = "button";
    multi.className = "install";
    const onDisk = modelState.models.some((model) => model.id === translator.unit && model.installed);
    if (onDisk) {
      multi.textContent = t("text.On disk", "On disk");
      multi.disabled = true;
    } else {
      multi.textContent = t("text.Download", "Download");
      multi.addEventListener("click", () => downloadCatalogModel(translator, multi));
    }
    multi.setAttribute("aria-label", `${translator.name || translator.id}: ${multi.textContent}`);
    host.append(note, multi);
  }
  host.hidden = false;
}

async function loadHardware() {
  try {
    const response = await fetch("/hardware", { cache: "no-store" });
    if (response.ok) {
      machine = await response.json();
    }
  } catch (failure) {
    machine = null;
  }
}

function parloPick(entry, items) {
  if (!entry.bench || typeof entry.bench.value !== "number") {
    return false;
  }
  const memory = machine && machine.memory_mb ? machine.memory_mb : 0;
  const weight = (item) => item.memory_mb || (item.size_mb ? item.size_mb * 2.2 : 0);
  const ranked = items
    .filter((item) => item.bench && typeof item.bench.value === "number")
    .sort((left, right) =>
      left.bench.better === "lower"
        ? left.bench.value - right.bench.value
        : right.bench.value - left.bench.value,
    );
  if (!ranked.length) {
    return false;
  }
  const affordable = ranked.filter((item) => !memory || weight(item) <= memory * 0.6);
  const picks = new Set([ranked[0], affordable[0]].filter(Boolean).map((item) => item.id));
  return picks.has(entry.id);
}

function fitBadge(entry) {
  if (!machine || !machine.memory_mb) {
    return null;
  }
  const gigabyte = 1024;
  const disk = Math.round((machine.free_disk_mb || 0) / gigabyte);
  const badge = document.createElement("span");
  badge.className = "fit";
  const need = entry.memory_mb;
  const download = entry.size_mb ? (entry.size_mb / gigabyte).toFixed(1) + " GB" : "";
  if (!need) {
    if (!download) {
      return null;
    }
    badge.dataset.state = "unknown";
    badge.textContent = "Downloads " + download + " · " + disk + " GB free";
    badge.title =
      "The publisher does not say how much memory this one needs, so Parlo only checks the download against this disk. It does not claim the model will run well here.";
    return badge;
  }
  const memory = Math.round(machine.memory_mb / gigabyte);
  const needed = (need / gigabyte).toFixed(1);
  let state = "green";
  let reason =
    "Fits here: it needs about " + needed + " GB of memory and this machine has " + memory + " GB.";
  if (need > machine.memory_mb * 0.9) {
    state = "red";
    reason =
      "Too heavy: it needs about " + needed + " GB of memory and this machine has " + memory + " GB.";
  } else if (need > machine.memory_mb * 0.55) {
    state = "yellow";
    reason =
      "Tight: it needs about " +
      needed +
      " GB of memory and this machine has " +
      memory +
      " GB, so the rest of the system will feel it.";
  }
  if (entry.size_mb && machine.free_disk_mb) {
    if (entry.size_mb > machine.free_disk_mb * 0.9) {
      state = "red";
      reason = "No room: the download is " + entry.size_mb + " MB and " + disk + " GB are free.";
    } else if (entry.size_mb > machine.free_disk_mb * 0.5 && state !== "red") {
      state = "yellow";
      reason =
        "Tight on disk: the download is " + entry.size_mb + " MB and " + disk + " GB are free.";
    }
  }
  badge.dataset.state = state;
  badge.textContent = "Runs here";
  badge.title = reason;
  return badge;
}
async function loadCatalog() {
  try {
    const response = await fetch("/catalog", { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    catalogEntries = await response.json();
    await loadHardware();
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
    more.textContent = t("text.Languages", "Languages");
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
  const squash = (text) => text.toLowerCase().replace(/[\s-]+/g, "");
  name.textContent = version && !squash(base).includes(squash(version)) ? base + " " + version : base;
  head.append(name);
  if (entry.pair) {
    const pair = document.createElement("span");
    pair.className = "pair-chip";
    pair.textContent = entry.pair.toUpperCase();
    head.append(pair);
  }
  const fit = fitBadge(entry);
  if (fit) {
    head.append(fit);
  }
  if (parloPick(entry, items)) {
    const pick = document.createElement("span");
    pick.className = "pick";
    pick.textContent = t("catalog.pick", "Parlo pick");
    head.append(pick);
  }
  if (Array.isArray(entry.languages) && entry.languages.length > 2) {
    const coverage = document.createElement("span");
    coverage.className = "model-coverage";
    coverage.textContent = t("catalog.coverage", "One download covers {count} languages", {
      count: entry.languages.length,
    });
    head.append(coverage);
  }
  const licence = document.createElement("span");
  licence.className = "model-licence";
  licence.textContent = entry.license || "licence unknown";
  head.append(licence);

  const purpose = document.createElement("p");
  purpose.className = "model-purpose";
  purpose.textContent =
    entry.kind === "mt"
      ? t("catalog.kind.mt", "Translation")
      : entry.purpose || t("catalog.kind." + entry.kind, CATALOG_KINDS[entry.kind] || entry.kind);
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

  if (!entry.pair) {
    const languages = catalogLanguages(entry);
    if (languages) {
      card.append(languages);
    }
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
  for (const [key, rawLabel] of known) {
    const label = t("text." + rawLabel, rawLabel);
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
  renderCatalogRecommendation(entries);
  const items = sections.get(catalogTab) || [];
  for (const entry of items) {
    catalogList.append(catalogCard(entry, items));
  }
  const rawLabel = (CATALOG_CATEGORIES.find(([key]) => key === catalogTab) || [catalogTab, catalogTab])[1];
  catalogNote.textContent = t(
    "catalog.note",
    "{count} in {label}, {total} in this build. A bar compares published figures with the best in the same category; the line under it names where the figure comes from.",
    { count: items.length, label: t("text." + rawLabel, rawLabel), total: entries.length },
  );
}

function historyStamp(entries) {
  return entries
    .map((entry) =>
      [entry.id, entry.at, entry.favourite ? 1 : 0, (entry.text || "").length, (entry.translation || "").length].join(":"),
    )
    .join("|");
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
    const next = await response.json();
    if (historyStamp(next) === historyStamp(historyEntries)) {
      return;
    }
    historyEntries = next;
    fillHistoryApps();
    renderHistory();
  } catch (failure) {
    setNote(historyNote, "The recent dictations could not be read.", "error");
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
  const needle = historySearch.value.trim().toLowerCase();
  if (app && (entry.app || "") !== app) {
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

function paintInsights() {
  const wordsEl = document.getElementById("insights-words");
  const empty = historyEntries.length === 0;
  for (const node of document.querySelectorAll("#insights h3.meeting-group")) {
    node.hidden = empty;
  }
  for (const id of ["insights-apps", "insights-top-words", "insights-days", "insights-days-note", "insights-empty"]) {
    const node = document.getElementById(id);
    if (node) {
      node.hidden = id === "insights-empty" ? !empty : empty;
    }
  }
  if (!wordsEl) {
    return;
  }
  const textOf = (entry) => entry.translation || entry.text || "";
  const countOf = (entry) => textOf(entry).split(/\s+/).filter(Boolean).length;
  const words = historyEntries.reduce((total, entry) => total + countOf(entry), 0);
  const seconds = historyEntries.reduce((total, entry) => total + (entry.seconds || 0), 0);
  wordsEl.textContent = String(words);
  document.getElementById("insights-minutes").textContent = (seconds / 60).toFixed(1);
  document.getElementById("insights-count").textContent = String(historyEntries.length);
  document.getElementById("insights-rate").textContent =
    seconds > 0 && words > 0 ? String(Math.round(words / (seconds / 60))) : "–";

  const apps = new Map();
  for (const entry of historyEntries) {
    const total = countOf(entry);
    if (!total) {
      continue;
    }
    const raw = (entry.app || "").trim();
    const name = raw && raw !== "unknown" ? raw : "Another app";
    apps.set(name, (apps.get(name) || 0) + total);
  }
  const ranked = [...apps.entries()].sort((left, right) => right[1] - left[1]).slice(0, 4);
  const best = ranked.length ? ranked[0][1] : 0;
  const list = document.getElementById("insights-apps");
  list.replaceChildren();
  if (!ranked.length) {
    const empty = document.createElement("li");
    empty.className = "insights-empty";
    empty.textContent = "No words written yet, so there is nowhere to point at.";
    list.append(empty);
  }
  for (const [name, total] of ranked) {
    const item = document.createElement("li");
    const label = document.createElement("span");
    label.className = "insights-app";
    label.textContent = name;
    const track = document.createElement("span");
    track.className = "insights-track";
    const fill = document.createElement("span");
    fill.className = "insights-fill";
    fill.style.width = Math.max(4, Math.round((100 * total) / best)) + "%";
    track.append(fill);
    const value = document.createElement("span");
    value.className = "insights-value";
    value.textContent = total + (total === 1 ? " word" : " words");
    item.append(label, track, value);
    list.append(item);
  }

  const surface = new Map();
  const skip = new Set([
    "the", "and", "that", "this", "with", "for", "you", "your", "are", "was", "were", "have",
    "has", "had", "but", "not", "from", "they", "them", "then", "than", "what", "when", "who",
    "how", "all", "can", "our", "out", "one", "two", "get", "got", "just", "very", "also",
    "into", "over", "about", "para", "porque", "como", "mais", "muito", "isso", "esta", "este",
    "não", "que", "com", "uma", "dos", "das", "por", "mas", "foi", "ser", "tem", "aos", "pelo",
    "pela", "aqui", "ainda", "depois",
  ]);
  for (const entry of historyEntries) {
    for (const raw of textOf(entry).toLowerCase().split(/[^\p{L}\p{N}]+/u)) {
      if (raw.length < 4 || skip.has(raw)) {
        continue;
      }
      surface.set(raw, (surface.get(raw) || 0) + 1);
    }
  }
  const top = [...surface.entries()].sort((left, right) => right[1] - left[1]).slice(0, 6);
  const wordList = document.getElementById("insights-top-words");
  wordList.replaceChildren();
  if (!top.length) {
    const empty = document.createElement("li");
    empty.className = "insights-empty";
    empty.textContent = "Nothing written yet, so there are no favourites.";
    wordList.append(empty);
  }
  for (const [word, hits] of top) {
    const item = document.createElement("li");
    const label = document.createElement("span");
    label.className = "insights-word";
    label.textContent = word;
    const count = document.createElement("span");
    count.className = "insights-hits";
    count.textContent = hits + "×";
    item.append(label, count);
    wordList.append(item);
  }

  const days = document.getElementById("insights-days");
  days.replaceChildren();
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const perDay = new Map();
  for (const entry of historyEntries) {
    const when = new Date((entry.at || 0) * 1000);
    when.setHours(0, 0, 0, 0);
    perDay.set(when.getTime(), (perDay.get(when.getTime()) || 0) + 1);
  }
  const peak = Math.max(1, ...perDay.values());
  let active = 0;
  for (let back = 27; back >= 0; back -= 1) {
    const day = new Date(today.getTime() - back * 86400000);
    const hits = perDay.get(day.getTime()) || 0;
    if (hits) {
      active += 1;
    }
    const cell = document.createElement("span");
    cell.className = "insights-day";
    cell.dataset.level = hits === 0 ? "0" : hits >= peak * 0.66 ? "3" : hits >= peak * 0.33 ? "2" : "1";
    cell.title = day.toLocaleDateString() + ": " + hits + (hits === 1 ? " dictation" : " dictations");
    days.append(cell);
  }
  document.getElementById("insights-days-note").textContent =
    active === 0
      ? "No dictation in the last four weeks."
      : active + (active === 1 ? " day" : " days") + " with at least one dictation in the last four weeks.";
}

function renderHistory() {
  paintHome();
  paintInsights();
  paintPrograms();
  const shown = historyEntries.filter(matchesHistory);
  const items = shown.map((entry) => {
    const moment = new Date((entry.at || 0) * 1000);
    const when = [
      moment.toLocaleDateString([], { day: "numeric", month: "short" }),
      moment.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }),
    ];
    if (entry.from && entry.to && entry.from !== entry.to) {
      when.push(languageName(entry.from) + " → " + languageName(entry.to));
    }
    if (entry.seconds > 0) {
      when.push(entry.seconds.toFixed(1) + " s");
    }
    const program = (entry.app || "").trim();
    if (program && program !== "unknown") {
      when.push(program);
    }
    const words = document.createElement("button");
    words.type = "button";
    words.className = "history-text";
    words.textContent = entry.text;
    words.title = t("text.Copy this dictation again", "Copy this dictation again");
    words.addEventListener("click", () => copyHistory(entry.text));
    const body = [words];
    if (entry.translation && entry.translation !== entry.text) {
      const translation = document.createElement("p");
      translation.className = "history-translation";
      translation.textContent = entry.translation;
      body.push(translation);
    }
    const listen = listenButton(entry.translation || entry.text, "swap");
    listen.title = t("text.Hear this dictation aloud", "Hear this dictation aloud");
    const star = document.createElement("button");
    star.type = "button";
    star.className = "swap";
    star.textContent = entry.favourite
      ? t("text.Favourite ✓", "Favourite")
      : t("text.Favourite", "Favourite");
    star.setAttribute("aria-pressed", String(Boolean(entry.favourite)));
    star.title = t(
      "text.Mark this dictation so the favourite filter keeps it",
      "Favourite",
    );
    star.addEventListener("click", () => setFavourite(entry));
    const correct = document.createElement("button");
    correct.type = "button";
    correct.className = "swap";
    correct.textContent = t("text.Correct", "Correct");
    correct.title = t("text.Replace the sentence Parlo wrote down", "Correct");
    correct.addEventListener("click", () => editHistory(entry));
    const remove = document.createElement("button");
    remove.type = "button";
    remove.className = "swap";
    remove.textContent = t("text.Delete this dictation?", "Delete");
    remove.title = t("text.Remove this dictation from this computer", "Delete");
    const ask = document.createElement("span");
    ask.className = "history-confirm";
    ask.hidden = true;
    const question = document.createElement("span");
    question.textContent = t("text.Delete this dictation?", "Delete this dictation?");
    const sure = document.createElement("button");
    sure.type = "button";
    sure.className = "install";
    sure.textContent = t("text.Delete", "Delete");
    const keep = document.createElement("button");
    keep.type = "button";
    keep.className = "swap";
    keep.textContent = t("text.Keep", "Keep");
    ask.append(question, sure, keep);
    remove.addEventListener("click", () => {
      ask.hidden = false;
      sure.focus();
    });
    keep.addEventListener("click", () => {
      ask.hidden = true;
      remove.focus();
    });
    sure.addEventListener("click", () => deleteHistory(entry, sure, keep));
    body.push(ask);
    return mediaCard({
      title: when[0],
      meta: when.slice(1).join(" · "),
      body,
      actions: [listen, star, correct, remove],
    });
  });
  const hasEntries = renderMedia(historyList, items, {
    kicker: t("library.tabs.history", "History"),
    title: t(
      "text.Nothing dictated yet. Press Ctrl+Alt+D, speak, and the sentence appears here with the program it landed in.",
      "Nothing dictated yet.",
    ),
    text: t(
      "text.Words, minutes, the programs your text landed in and a four week chart all come from the dictations kept on this computer. Nothing else is counted.",
      "",
    ),
  });
  historyFilters.hidden = !hasEntries;
  if (historyTools) {
    historyTools.hidden = !hasEntries;
  }
  historyClearWrap.hidden = !hasEntries;
  if (!hasEntries) {
    historyClearConfirm.hidden = true;
    setNote(historyNote, "");
    return;
  }
  setNote(
    historyNote,
    shown.length === historyEntries.length
      ? t(
          historyEntries.length === 1
            ? "text.{count} dictation kept on this computer."
            : "text.{count} dictations kept on this computer.",
          "",
          { count: historyEntries.length },
        )
      : t("text.{count} dictation", "", { count: shown.length }) + " / " + historyEntries.length,
  );
}


async function copyHistory(text) {
  try {
    await navigator.clipboard.writeText(text);
    historyNote.textContent = "Copied to the clipboard. Paste it where you need it.";
  } catch (failure) {
    setNote(historyNote, "The clipboard refused the text. Select it here and copy by hand.", "error");
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
    setNote(historyNote, failure.message, "error");
  }
}

async function editHistory(entry) {
  const chosen = window.prompt("Correct what Parlo wrote down.", entry.text || "");
  if (chosen === null) {
    return;
  }
  try {
    const updated = await historyPost("/history/edit", { id: entry.id, text: chosen });
    await loadHistory();
    renderLearned(updated.suggestions || []);
    historyNote.textContent = "Corrected on this computer. Nothing was sent anywhere.";
  } catch (failure) {
    setNote(historyNote, failure.message, "error");
  }
}

const historyLearn = document.getElementById("history-learn");

function renderLearned(pairs) {
  if (!historyLearn) {
    return;
  }
  historyLearn.replaceChildren();
  historyLearn.hidden = pairs.length === 0;
  for (const pair of pairs) {
    const line = document.createElement("p");
    line.textContent =
      "Parlo heard “" + pair.heard + "” and you wrote “" + pair.written + "”. ";
    const learn = document.createElement("button");
    learn.type = "button";
    learn.className = "swap";
    learn.textContent = "Learn it";
    learn.title = "Keep this pair in the vocabulary so the recogniser writes it next time";
    learn.addEventListener("click", () => learnTerm(pair, line));
    line.append(learn);
    historyLearn.append(line);
  }
}

async function learnTerm(pair, line) {
  try {
    const response = await fetch("/vocabulary/accept", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({ heard: pair.heard, written: pair.written }),
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "That pair could not be kept.");
    }
    line.textContent = "Kept: “" + pair.heard + "” now becomes “" + pair.written + "”.";
  } catch (failure) {
    line.textContent = failure.message;
  }
}

async function deleteHistory(entry, ...buttons) {
  for (const button of buttons) {
    button.disabled = true;
  }
  historyNote.textContent = "Deleting this dictation…";
  try {
    await historyPost("/history/delete", { id: entry.id });
    await loadHistory();
  } catch (failure) {
    historyNote.textContent = failure.message;
    for (const button of buttons) {
      button.disabled = false;
    }
  }
}

const historyClearWrap = document.getElementById("history-clear-wrap");
const historyClearConfirm = document.getElementById("history-clear-confirm");
const historyClearYes = document.getElementById("history-clear-yes");
const historyClearKeep = document.getElementById("history-clear-keep");

async function clearHistory() {
  historyClearYes.disabled = true;
  historyClearKeep.disabled = true;
  historyNote.textContent = "Deleting every dictation…";
  try {
    const response = await fetch("/history/clear", {
      method: "POST",
      headers: { Authorization: "Bearer " + token },
    });
    if (!response.ok) {
      throw new Error("status " + response.status);
    }
    historyClearConfirm.hidden = true;
    await loadHistory();
  } catch (failure) {
    setNote(historyNote, "The dictations could not be deleted. Try again.", "error");
  } finally {
    historyClearYes.disabled = false;
    historyClearKeep.disabled = false;
  }
}



historyClear.addEventListener("click", () => {
  historyClearConfirm.hidden = false;
  historyClearYes.focus();
});
historyClearKeep.addEventListener("click", () => {
  historyClearConfirm.hidden = true;
  historyClear.focus();
});
historyClearYes.addEventListener("click", () => clearHistory());
historySearch.addEventListener("input", () => renderHistory());
historyApp.addEventListener("change", () => renderHistory());
historyFavourite.addEventListener("change", () => renderHistory());

window.addEventListener("load", () => {
  loadMeetings();
  loadCatalog();
  loadHistory();
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

micEl.textContent = "not opened yet";
refreshHealth();
setInterval(() => {
  if (document.visibilityState === "visible") {
    refreshHealth();
  }
}, 30000);
document.addEventListener("visibilitychange", () => {
  if (document.visibilityState === "visible") {
    refreshHealth();
  }
});
window.addEventListener("focus", refreshHealth);
setInterval(() => {
  if (document.visibilityState !== "visible") {
    return;
  }
  if (currentRoute === "library") {
    loadLibrary();
  } else if (currentRoute === "home") {
    loadHistory();
  }
}, 4000);

const ROUTES = {
  home: [
    "update",
    "welcome",
    "status-list",
  ],
  library: ["library", "library-tabs", "meetings", "dub", "batch", "stories", "history"],
  voice: ["voice"],
  insights: ["insights"],
  settings: [
    "settings-tabs",
    "settings-layout",
    "settings-foot",
    "you",
    "languages",
    "settings",
    "programs",
    "vocabulary",
    "models",
    "catalog",
    "shortcuts",
    "intelligence",
    "provider",
    "privacy",
    "about",
    "licences",
  ],
};

let currentRoute = "home";

function showRoute(name) {
  const route = ROUTES[name] ? name : "home";
  currentRoute = route;
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
  const parsed = ParloRoutes.parseHash(location.hash);
  libraryTab = parsed.libraryTab;
  settingsTab = parsed.settingsTab;
  showRoute(parsed.route);
  localStorage.setItem("parlo.route", parsed.canonical);
}

function navigate(route, tab) {
  const path = "#" + route + (tab ? "/" + tab : "");
  if (location.hash === path) {
    routeNow();
  } else {
    location.hash = path;
  }
}

const ROUTE_KEYS = { 1: "home", 2: "library", 3: "voice", 4: "insights", 5: "settings" };

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
  meetings: ["meetings"],
  files: ["dub"],
  batch: ["batch"],
  stories: ["stories"],
  history: ["history"],
};

const LIBRARY_LOADERS = {
  meetings: [loadMeetings],
  files: [loadDubProjects],
  batch: [loadBatches, loadWatch],
  stories: [loadStories],
  history: [loadHistory],
};

let libraryTab = "meetings";

function loadLibrary() {
  for (const load of LIBRARY_LOADERS[libraryTab] || []) {
    load();
  }
}

function showLibraryTab(name) {
  const chosen = LIBRARY_TABS[name] ? name : "meetings";
  libraryTab = chosen;
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
  if (currentRoute === "library") {
    loadLibrary();
  }
}

for (const button of document.querySelectorAll("#library-tabs .tab")) {
  button.addEventListener("click", () => {
    navigate("library", button.dataset.tab);
  });
}

const SETTINGS_TABS = {
  you: ["you"],
  languages: ["languages"],
  dictation: ["settings", "programs", "vocabulary"],
  models: ["models", "catalog"],
  shortcuts: ["shortcuts"],
  advanced: ["intelligence", "provider", "privacy", "about"],
};

let settingsTab = "dictation";

function showSettingsTab(name) {
  const chosen = SETTINGS_TABS[name] ? name : "dictation";
  settingsTab = chosen;
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
    navigate("settings", button.dataset.tab);
  });
}

document.addEventListener("click", (event) => {
  const trigger = event.target.closest("[data-focus]");
  if (!trigger) {
    return;
  }
  const field = document.getElementById(trigger.dataset.focus);
  if (field) {
    field.focus();
  }
});

window.addEventListener("hashchange", routeNow);
if (!location.hash) {
  const remembered = localStorage.getItem("parlo.route") || "home";
  history.replaceState(null, "", "#" + remembered);
}
routeNow();
checkLocalUpdate();
loadModels();
loadSettings().then(showNameGateIfNeeded);
applyLocale();
loadEverything();

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
      ? t("studio.thirdShort", "someone else's voice, with permission")
      : t("studio.ownShort", "your own voice");
  const state = profile.consent.revoked_at
    ? t("studio.revokedShort", "authorisation revoked")
    : t("studio.authorised", "authorised");
  const seal = profile.encrypted ? " · " + t("studio.sealed", "sealed") : "";
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
    voiceStatus.textContent = t(
      "studio.profilesEmpty",
      "No voice profiles on this computer yet.",
    );
    return;
  }
  voiceStatus.textContent =
    profiles.length === 1
      ? t("studio.profileOne", "{count} voice profile on this computer.", {
          count: profiles.length,
        })
      : t("studio.profileMany", "{count} voice profiles on this computer.", {
          count: profiles.length,
        });
  for (const profile of profiles) {
    const item = document.createElement("li");
    item.className = "voice-item";
    const title = document.createElement("p");
    title.className = "voice-item-title";
    title.textContent = profile.label;
    const meta = document.createElement("p");
    meta.className = "voice-item-meta";
    meta.textContent =
      describeVoice(profile) +
      " · " +
      t("studio.since", "since") +
      " " +
      new Date(profile.created_at * 1000).toLocaleDateString(document.documentElement.lang || "en");
    const actions = document.createElement("div");
    actions.className = "voice-item-actions";
    const revoked = Boolean(profile.consent.revoked_at);
    const hasSample = (profile.artefacts || []).includes("sample.wav");
    if (revoked) {
      actions.append(
        voiceButton(t("studio.playDefault", "Play default voice"), () =>
          playVoiceSample(profile),
        ),
      );
      const note = document.createElement("p");
      note.className = "voice-item-note";
      note.textContent = t(
        "studio.revoked",
        "Revoked. The sample is gone and playback falls back to the default voice.",
      );
      item.append(note);
    } else if (hasSample) {
      actions.append(
        voiceButton(t("studio.playSample", "Play sample"), () => playVoiceSample(profile)),
        voiceButton(t("studio.uploadSample", "Upload a recording"), () => pickVoiceSample(profile)),
      );
    } else {
      actions.append(
        voiceButton(t("studio.record", "Record 10 s"), (event) =>
          recordVoiceSample(profile, event.currentTarget),
        ),
        voiceButton(t("studio.uploadSample", "Upload a recording"), () => pickVoiceSample(profile)),
      );
    }
    actions.append(voiceButton(t("studio.export", "Export"), () => exportVoice(profile)));
    if (!profile.consent.revoked_at) {
      actions.append(
        voiceButton(t("studio.revoke", "Revoke"), () =>
          voiceCall(
            "/voice/revoke",
            profile,
            t(
              "studio.revokedDone",
              "Revoked. This voice will not be used again until you create and authorise a new profile.",
            ),
          ),
        ),
      );
    }
    actions.append(
      voiceButton(t("studio.delete", "Delete"), () => {
        if (
          !window.confirm(
            t(
              "studio.deleteConfirm",
              "Delete {label} and everything made from it? This cannot be undone.",
              { label: profile.label },
            ),
          )
        ) {
          return;
        }
        voiceCall(
          "/voice/delete",
          profile,
          t("studio.deleted", "Deleted, with the sample and anything made from it."),
        );
      }),
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
      throw new Error(payload.error || t("studio.revokeError", "The request failed."));
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
    voiceStatus.textContent = t(
      "studio.listError",
      "The voice list is not available right now.",
    );
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
    showError(
      t(
        "studio.permissionError",
        "Tick the permission box before creating a profile for someone else's voice.",
      ),
    );
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
        origin: voiceOrigin.value,
        declared_authorisation: voiceAuthorisation.checked,
        encrypt: document.getElementById("voice-encrypt").checked,
      }),
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(payload.error || t("studio.profileError", "The profile could not be created."));
    }
    voiceStatus.textContent = t(
      "studio.profileCreated",
      "Created {label}. Record a sample when you are ready; it stays on this computer.",
      { label: payload.label },
    );
    voiceForm.reset();
    voiceAuthorisationRow.hidden = true;
    await loadVoices();
  } catch (failure) {
    showError(failure.message);
  } finally {
    button.disabled = false;
  }
});

const compareRun = document.getElementById("compare-run");
const compareText = document.getElementById("compare-text");
const compareFirst = document.getElementById("compare-first");
const compareSecond = document.getElementById("compare-second");
const compareNote = document.getElementById("compare-note");
const compareReadings = document.getElementById("compare-readings");

if (compareRun) {
  compareRun.addEventListener("click", async () => {
    const sentence = compareText.value.trim();
    compareReadings.replaceChildren();
    if (!sentence) {
      compareNote.textContent = t(
        "studio.compareEmpty",
        "Write the sentence both voices should read.",
      );
      return;
    }
    compareRun.disabled = true;
    compareNote.textContent = t(
      "studio.compareWorking",
      "Reading it twice. This takes a few seconds.",
    );
    try {
      const response = await fetch("/voice/compare", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
        body: JSON.stringify({ text: sentence, voices: [compareFirst.value, compareSecond.value] }),
      });
      const answer = await response.json().catch(() => ({}));
      if (!response.ok) {
        throw new Error(
          answer.error || t("studio.compareFailed", "The two voices could not be read."),
        );
      }
      for (const reading of answer.readings) {
        const line = document.createElement("li");
        line.textContent =
          reading.label + ", " + reading.language + ", from " +
          (reading.start_ms / 1000).toFixed(2) + " s for " +
          (reading.duration_ms / 1000).toFixed(2) + " s";
        compareReadings.append(line);
      }
      const decoded = atob(answer.audio);
      const bytes = new Uint8Array(decoded.length);
      for (let index = 0; index < decoded.length; index += 1) {
        bytes[index] = decoded.charCodeAt(index);
      }
      play(new Blob([bytes], { type: "audio/wav" }));
      compareNote.textContent = t(
        "studio.compareDone",
        "First voice, then the second, in the order chosen.",
      );
    } catch (failure) {
      compareNote.textContent = failure.message;
      showError(failure.message);
    } finally {
      compareRun.disabled = false;
    }
  });
}

const packName = document.getElementById("pack-name");
const packSave = document.getElementById("pack-save");
const packImport = document.getElementById("pack-import");
const packImportFile = document.getElementById("pack-import-file");
const packNote = document.getElementById("pack-note");
const packList = document.getElementById("pack-list");

async function sendPack(body) {
  const response = await fetch("/voice/packs", {
    method: "POST",
    headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
    body,
  });
  const answer = await response.json().catch(() => ({}));    if (!response.ok) {
      throw new Error(answer.error || t("studio.packSaveError", "The pack could not be saved."));
    }
  return answer;
}

async function exportPack(name) {
  try {
    const response = await fetch("/voice/packs/" + encodeURIComponent(name), {
      cache: "no-store",
      headers: { Authorization: "Bearer " + token },
    });
    if (!response.ok) {
      throw new Error(t("studio.packExportError", "The pack could not be exported."));
    }
    const link = document.createElement("a");
    link.href = URL.createObjectURL(await response.blob());
    link.download = name + ".json";
    link.click();
    setTimeout(() => URL.revokeObjectURL(link.href), 5000);
    packNote.textContent = t("studio.packSaved", "Saved {name} on this computer.", { name });
  } catch (failure) {
    packNote.textContent = failure.message;
  }
}

async function loadPacks() {
  if (!packList) {
    return;
  }
  try {
    const response = await fetch("/voice/packs", { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    const values = await response.json();
    packList.replaceChildren();
    for (const pack of values.packs || []) {
      const item = document.createElement("li");
      const use = document.createElement("button");
      use.type = "button";
      use.className = "swap";
      use.textContent = t("studio.packUse", "Use {name}", { name: pack.name });
      use.addEventListener("click", () => {
        compareFirst.value = pack.voices[0];
        compareSecond.value = pack.voices[1];
        compareNote.textContent = t(
          "studio.packChosen",
          "Pack {name} is chosen. Write the sentence and read it in both.",
          { name: pack.name },
        );
        compareText.focus();
      });
      const save = document.createElement("button");
      save.type = "button";
      save.className = "swap";
      save.textContent = t("studio.export", "Export");
      save.addEventListener("click", () => exportPack(pack.name));
      item.textContent = pack.name + ": " + pack.voices.join(", ") + " ";
      item.append(use, save);
      packList.append(item);
    }
  } catch (failure) {
    showError(failure.message);
  }
}

if (packSave) {
  packSave.addEventListener("click", async () => {
    const name = packName.value.trim();
    if (!name) {
      packNote.textContent = t("studio.packNameError", "Give the pack a name first.");
      return;
    }
    try {
      const saved = await sendPack(
        JSON.stringify({
          format: "parlo-voice-pack",
          version: 1,
          name,
          voices: [compareFirst.value, compareSecond.value],
          notice: "Nothing in this pack leaves this computer.",
        }),
      );
      packNote.textContent = t("studio.packSaved", "Saved {name} on this computer.", {
        name: saved.name,
      });
      await loadPacks();
    } catch (failure) {
      packNote.textContent = failure.message;
    }
  });
}

if (packImport) {
  packImport.addEventListener("click", () => packImportFile.click());
  packImportFile.addEventListener("change", async () => {
    const file = packImportFile.files && packImportFile.files[0];
    if (!file) {
      return;
    }
    try {
      const saved = await sendPack(await file.text());
      packNote.textContent = t(
        "studio.packImported",
        "Imported {name} with {count} voices, and they are installed.",
        { name: saved.name, count: saved.voices.length },
      );
      await loadPacks();
    } catch (failure) {
      packNote.textContent = failure.message;
      showError(failure.message);
    } finally {
      packImportFile.value = "";
    }
  });
}

loadPacks();

const voiceImport = document.getElementById("voice-import");
const voiceImportFile = document.getElementById("voice-import-file");
const voiceImportNote = document.getElementById("voice-import-note");

if (voiceImport && voiceImportFile) {
  voiceImport.addEventListener("click", () => voiceImportFile.click());
  voiceImportFile.addEventListener("change", async () => {
    const file = voiceImportFile.files && voiceImportFile.files[0];
    if (!file) {
      return;
    }
    try {
      const response = await fetch("/voice/import", {
        method: "POST",
        headers: { "Content-Type": "application/octet-stream", Authorization: "Bearer " + token },
        body: await file.arrayBuffer(),
      });
      const answer = await response.json().catch(() => ({}));
      if (!response.ok) {
        throw new Error(answer.error || t("studio.importError", "The voice file could not be read."));
      }
      voiceImportNote.textContent = t(
        "studio.importNote",
        "Imported {label}, and its consent record came with it.",
        { label: answer.label },
      );
      await loadVoices();
    } catch (failure) {
      showError(failure.message);
    } finally {
      voiceImportFile.value = "";
    }
  });
}

loadVoices();

const voiceSampleFile = document.getElementById("voice-sample-file");
let voiceSampleTarget = "";

function pickVoiceSample(profile) {
  if (!voiceSampleFile) {
    return;
  }
  voiceSampleTarget = profile.id;
  voiceSampleFile.value = "";
  voiceSampleFile.click();
}

if (voiceSampleFile) {
  voiceSampleFile.addEventListener("change", async () => {
    const file = voiceSampleFile.files && voiceSampleFile.files[0];
    if (!file || !voiceSampleTarget) {
      return;
    }
    clearError();
    voiceStatus.textContent = t("studio.saving", "Saving the sample");
    try {
      const response = await fetch("/voice/sample", {
        method: "POST",
        headers: {
          "Content-Type": "audio/wav",
          Authorization: "Bearer " + token,
          "X-Parlo-Voice": voiceSampleTarget,
        },
        body: await file.arrayBuffer(),
      });
      const answer = await response.json().catch(() => ({}));
      if (!response.ok) {
        throw new Error(answer.error || t("studio.uploadError", "The recording could not be read."));
      }
      voiceStatus.textContent = t("studio.uploadNote", "Imported the recording for {label}.", {
        label: answer.label || t("studio.namePlaceholder", "My voice"),
      });
      await loadVoices();
    } catch (failure) {
      voiceStatus.textContent = failure.message;
      showError(failure.message);
    } finally {
      voiceSampleFile.value = "";
      voiceSampleTarget = "";
    }
  });
}

const studioText = document.getElementById("studio-text");
const studioVoice = document.getElementById("studio-voice");
const studioGenerate = document.getElementById("studio-generate");
const studioNote = document.getElementById("studio-note");
const studioResult = document.getElementById("studio-result");
const studioAudio = document.getElementById("studio-audio");
const studioDownload = document.getElementById("studio-download");
const studioTakes = document.getElementById("studio-takes");
const studioCount = document.getElementById("studio-count");
const studioClear = document.getElementById("studio-clear");
const studioTakesClear = document.getElementById("studio-takes-clear");
const studioTabs = Array.from(document.querySelectorAll("[data-studio-tab]"));

function paintStudioCount() {
  if (studioText && studioCount) {
    studioCount.textContent =
      studioText.value.length + " / " + (studioText.maxLength > 0 ? studioText.maxLength : 4096);
  }
}

function showStudioTab(name, moveFocus) {
  const chosen = studioTabs.find((tab) => tab.dataset.studioTab === name) || studioTabs[0];
  if (!chosen) {
    return;
  }
  for (const tab of studioTabs) {
    const selected = tab === chosen;
    tab.setAttribute("aria-selected", String(selected));
    tab.tabIndex = selected ? 0 : -1;
    const panel = document.getElementById(tab.getAttribute("aria-controls"));
    if (panel) {
      panel.hidden = !selected;
    }
  }
  if (moveFocus) {
    chosen.focus();
  }
}

for (const tab of studioTabs) {
  tab.addEventListener("click", () => showStudioTab(tab.dataset.studioTab));
  tab.addEventListener("keydown", (event) => {
    const step = document.documentElement.dir === "rtl" ? -1 : 1;
    const index = studioTabs.indexOf(tab);
    let next = -1;
    if (event.key === "ArrowRight") {
      next = index + step;
    } else if (event.key === "ArrowLeft") {
      next = index - step;
    } else if (event.key === "Home") {
      next = 0;
    } else if (event.key === "End") {
      next = studioTabs.length - 1;
    }
    if (next < 0 || next >= studioTabs.length) {
      return;
    }
    event.preventDefault();
    showStudioTab(studioTabs[next].dataset.studioTab, true);
  });
}

if (studioTabs.length) {
  showStudioTab("voices");
}

if (studioText) {
  studioText.addEventListener("input", paintStudioCount);
  studioText.addEventListener("keydown", (event) => {
    if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
      event.preventDefault();
      studioGenerate.click();
    }
  });
  paintStudioCount();
}

if (studioClear) {
  studioClear.addEventListener("click", () => {
    studioText.value = "";
    paintStudioCount();
    studioText.focus();
  });
}

if (studioTakesClear) {
  studioTakesClear.addEventListener("click", () => {
    const empty = document.createElement("li");
    empty.className = "studio-empty";
    empty.textContent = t(
      "studio.takesEmpty",
      "Takes you generate this session appear here.",
    );
    studioTakes.replaceChildren(empty);
    studioNote.textContent = t("studio.takesCleared", "Takes cleared.");
  });
}

function studioTake(text, url) {
  const item = document.createElement("li");
  item.className = "studio-take";
  const title = document.createElement("span");
  title.className = "studio-take-text";
  title.textContent = text.length > 72 ? text.slice(0, 72).trimEnd() + "…" : text;
  const play = document.createElement("button");
  play.type = "button";
  play.className = "voice-action";
  play.textContent = t("studio.playSample", "Play sample");
  play.addEventListener("click", () => {
    const audio = new Audio(url);
    audio.play().catch(() => {
      studioNote.textContent = t(
        "studio.pronounceAudio",
        "Pronunciation shown. Audio is unavailable: {reason}",
        { reason: t("studio.failed", "The voice could not be generated.") },
      );
    });
  });
  const download = document.createElement("a");
  download.className = "studio-take-download";
  download.href = url;
  download.download = "parlo-studio.wav";
  download.textContent = t("studio.download", "Download WAV");
  item.append(title, play, download);
  const empty = studioTakes.querySelector(".studio-empty");
  if (empty) {
    empty.remove();
  }
  studioTakes.prepend(item);
  while (studioTakes.children.length > 5) {
    studioTakes.lastElementChild.remove();
  }
}

function fillCompareVoices(voices) {
  const first = document.getElementById("compare-first");
  const second = document.getElementById("compare-second");
  const run = document.getElementById("compare-run");
  const note = document.getElementById("compare-note");
  if (!first || !second || !run) {
    return;
  }
  const previous = [first.value, second.value];
  const options = () =>
    voices.map((voice) => {
      const option = document.createElement("option");
      option.value = voice.id;
      option.textContent = voice.name || voice.id;
      return option;
    });
  const ids = voices.map((voice) => voice.id);
  first.replaceChildren(...options());
  second.replaceChildren(...options());
  first.value = ids.includes(previous[0]) ? previous[0] : ids[0] || "";
  second.value =
    ids.includes(previous[1]) && previous[1] !== first.value
      ? previous[1]
      : ids.find((id) => id !== first.value) || "";
  const ready = ids.length >= 2;
  run.disabled = !ready;
  if (!ready) {
    note.textContent = t(
      "studio.compareNeedTwo",
      "Install two voices in Library, Models, to compare them.",
    );
  }
}

async function loadStudioVoices() {
  if (!studioVoice || !studioGenerate) {
    return;
  }
  try {
    const response = await fetch("/models", { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    const state = await response.json();
    const voices = (state.models || []).filter(
      (model) => model.kind === "voice" && model.installed,
    );
    const previous = studioVoice.value;
    const automatic = document.createElement("option");
    automatic.value = "";
    automatic.textContent = t("studio.voiceAuto", "Automatic voice");
    studioVoice.replaceChildren(
      automatic,
      ...voices.map((voice) => {
        const option = document.createElement("option");
        option.value = voice.id;
        option.textContent = voice.name || voice.id;
        return option;
      }),
    );
    if (voices.some((voice) => voice.id === previous)) {
      studioVoice.value = previous;
    }
    studioGenerate.disabled = !voices.length;
    fillCompareVoices(voices);
    if (!voices.length) {
      studioNote.textContent = t(
        "studio.noVoice",
        "Install a voice in Library, Models, and it will appear here.",
      );
    } else if (studioNote.textContent === t("studio.noVoice", "Install a voice in Library, Models, and it will appear here.")) {
      studioNote.textContent = "";
    }
  } catch (failure) {
    return;
  }
}

if (studioTakes && !studioTakes.children.length) {
  const empty = document.createElement("li");
  empty.className = "studio-empty";
  empty.textContent = t("studio.takesEmpty", "Takes you generate this session appear here.");
  studioTakes.append(empty);
}

if (studioGenerate) {
  studioGenerate.addEventListener("click", async () => {
    const text = studioText.value.trim();
    if (!text) {
      studioNote.textContent = t("studio.emptyText", "Write something to read aloud first.");
      studioText.focus();
      return;
    }
    studioGenerate.disabled = true;
    studioGenerate.textContent = t("studio.generating", "Generating…");
    studioNote.textContent = t("studio.generating", "Generating…");
    try {
      const body = { input: text, response_format: "wav" };
      if (studioVoice.value) {
        body.voice = studioVoice.value;
      }
      const response = await fetch("/v1/audio/speech", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
        body: JSON.stringify(body),
      });
      if (!response.ok) {
        const detail = await response.json().catch(() => ({}));
        const message =
          detail && detail.error && detail.error.message
            ? detail.error.message
            : detail && detail.error
              ? detail.error
              : t("studio.failed", "The voice could not be generated.");
        throw new Error(message);
      }
      const url = URL.createObjectURL(await response.blob());
      studioAudio.src = url;
      studioDownload.href = url;
      studioDownload.download = "parlo-studio.wav";
      studioResult.hidden = false;
      studioNote.textContent = t("studio.ready", "Ready to play.");
      studioTake(text, url);
      studioAudio.play().catch(() => {});
    } catch (failure) {
      studioNote.textContent = failure.message || t("studio.failed", "The voice could not be generated.");
    } finally {
      studioGenerate.disabled = studioVoice.options.length < 2;
      studioGenerate.textContent = t("studio.generate", "Generate speech");
    }
  });
}

loadStudioVoices();


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

const homeCaptions = document.getElementById("home-captions");
if (homeCaptions) {
  homeCaptions.addEventListener("click", () => {
    window.open("/captions?t=" + encodeURIComponent(token), "_blank", "noopener");
  });
}

const homeLive = document.getElementById("home-live");
if (homeLive) {
  homeLive.addEventListener("click", () => {
    window.open("/live?t=" + encodeURIComponent(token), "_blank", "noopener");
  });
}

const nameGate = document.getElementById("name-gate");
const nameInput = document.getElementById("name-input");
const nameSave = document.getElementById("name-save");
const nameNote = document.getElementById("name-note");
const gateProgress = document.getElementById("gate-progress");
const gateFrom = document.getElementById("gate-from");
const gateTo = document.getElementById("gate-to");
const gateSwap = document.getElementById("gate-swap");
const gatePairSave = document.getElementById("gate-pair-save");
const gatePairNote = document.getElementById("gate-pair-note");
const gateMicCheck = document.getElementById("gate-mic-check");
const gateMicNext = document.getElementById("gate-mic-next");
const gateMicSkip = document.getElementById("gate-mic-skip");
const gateMicNote = document.getElementById("gate-mic-note");
const gateShortcutKey = document.getElementById("gate-shortcut-key");
const gateFinish = document.getElementById("gate-finish");
const GATE_STEPS = ["gate-name", "gate-languages", "gate-microphone", "gate-style", "gate-shortcut"];
const styleControls = [
  ...document.querySelectorAll(".style-card"),
  ...document.querySelectorAll(".style-chip"),
];
const settingsStyle = document.getElementById("settings-style");
const styleSheet = document.getElementById("style-sheet");
const toneOpen = document.getElementById("tone-open");
const toneNow = document.getElementById("tone-now");
const STYLES = ["formal", "casual", "excited"];
const STYLE_NAMES = { formal: "Formal", casual: "Casual", excited: "Excited" };

function gateStep(name) {
  const index = GATE_STEPS.indexOf(name);
  for (const [position, id] of GATE_STEPS.entries()) {
    const step = document.getElementById(id);
    if (step) {
      step.hidden = position !== index;
    }
  }
  if (gateProgress) {
    paintGateProgress(index, GATE_STEPS.length);
  }
}

function paintGate() {
  gateFrom.value = savedSettings.from;
  gateTo.value = savedSettings.to;
  gateShortcutKey.textContent = currentShortcut || "Ctrl+Alt+D";
}

function storedStyle() {
  const value = localStorage.getItem("parlo.style");
  return STYLES.includes(value) ? value : "";
}

function paintStyleControls() {
  const chosen = storedStyle();
  for (const control of styleControls) {
    control.setAttribute("aria-pressed", String(control.dataset.style === chosen));
  }
  if (settingsStyle) {
    settingsStyle.value = chosen || "formal";
  }
  if (toneNow) {
    const style = chosen || "formal";
    toneNow.textContent = t("home." + style, STYLE_NAMES[style]);
  }
}

if (toneOpen && styleSheet) {
  toneOpen.addEventListener("click", () => styleSheet.showModal());
}

if (styleSheet) {
  styleSheet.addEventListener("click", (event) => {
    if (event.target.closest("[data-style]") || event.target === styleSheet) {
      styleSheet.close();
    }
  });
}

function chooseStyle(style) {
  localStorage.setItem("parlo.style", style);
  paintStyleControls();
  fetch("/style", {
    method: "POST",
    headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
    body: JSON.stringify({ style }),
  }).catch(() => {});
}

function closeGate() {
  nameGate.hidden = true;
  document.body.classList.remove("gating");
  navigate("home");
  paintHome();
}

function showNameGateIfNeeded() {
  if (storedName()) {
    return;
  }
  document.body.classList.add("gating");
  nameGate.hidden = false;
  paintGate();
  gateStep("gate-name");
  nameInput.focus();
}

function storeName(raw) {
  const value = raw.trim().replace(/\s+/g, " ").slice(0, 40);
  if (value.length < 2) {
    return "";
  }
  localStorage.setItem("parlo.name", value);
  fetch("/settings", {
    method: "POST",
    headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
    body: JSON.stringify({ name: value }),
  }).catch(() => {});
  paintHome();
  paintSettingsName();
  return value;
}

function paintSettingsName() {
  if (settingsName) {
    settingsName.value = storedName();
  }
}

nameSave.addEventListener("click", () => {
  if (!storeName(nameInput.value)) {
    nameNote.textContent = t(
      "gate.name.short",
      "Two letters at least, so it reads like a name.",
    );
    nameInput.focus();
    return;
  }
  nameNote.textContent = "";
  paintGate();
  gateStep("gate-languages");
});

nameInput.addEventListener("keydown", (event) => {
  if (event.key === "Enter") {
    event.preventDefault();
    nameSave.click();
  }
});

for (const control of styleControls) {
  control.addEventListener("click", () => {
    chooseStyle(control.dataset.style);
  });
}

document.getElementById("style-save").addEventListener("click", () => {
  paintGate();
  gateStep("gate-shortcut");
});
document.getElementById("style-skip").addEventListener("click", () => {
  if (!storedStyle()) {
    chooseStyle("formal");
  }
  paintGate();
  gateStep("gate-shortcut");
});

function keepGatePair(changed) {
  if (gateFrom.value !== gateTo.value) {
    return;
  }
  const other = gateFrom.value === "pt" ? "en" : "pt";
  if (changed === "from") {
    gateTo.value = other;
  } else {
    gateFrom.value = other;
  }
}

gateFrom.addEventListener("change", () => keepGatePair("from"));
gateTo.addEventListener("change", () => keepGatePair("to"));
gateSwap.addEventListener("click", () => {
  const from = gateFrom.value;
  gateFrom.value = gateTo.value;
  gateTo.value = from;
});

gatePairSave.addEventListener("click", async () => {
  gatePairSave.disabled = true;
  gatePairNote.textContent = t("gate.pair.saving", "Saving the pair…");
  try {
    const response = await fetch("/settings", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({ from: gateFrom.value, to: gateTo.value, write: "" }),
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      gatePairNote.textContent = detail.error || "The pair could not be saved. Try again.";
      return;
    }
    reflectSettings(detail);
    gatePairNote.textContent = "";
    gateStep("gate-microphone");
  } catch (failure) {
    gatePairNote.textContent = "The pair could not be saved. Check that Parlo is still running.";
  } finally {
    gatePairSave.disabled = false;
  }
});

gateMicCheck.addEventListener("click", async () => {
  gateMicCheck.disabled = true;
  gateMicNote.textContent = "Waiting for Windows…";
  try {
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
    for (const track of stream.getTracks()) {
      track.stop();
    }
    localStorage.setItem("parlo.mic", "ready");
    if (homeMic) {
      homeMic.textContent = statusWord("ready");
    }
    gateMicNote.textContent = "The microphone answers. Continue when you are ready.";
    gateMicNext.disabled = false;
  } catch (failure) {
    gateMicNote.textContent =
      "Windows did not allow it. Open Settings > Privacy & security > Microphone, allow desktop apps, then try again.";
    gateMicCheck.disabled = false;
  }
});

gateMicNext.addEventListener("click", () => gateStep("gate-style"));
gateMicSkip.addEventListener("click", () => gateStep("gate-style"));
gateFinish.addEventListener("click", closeGate);

if (settingsStyle) {
  settingsStyle.addEventListener("change", () => {
    chooseStyle(settingsStyle.value);
  });
}

const settingsName = document.getElementById("settings-name");
const settingsNameSave = document.getElementById("settings-name-save");
const settingsNameNote = document.getElementById("settings-name-note");

if (settingsNameSave) {
  settingsNameSave.addEventListener("click", () => {
    if (storeName(settingsName.value)) {
      settingsNameNote.textContent = "Saved. The greeting and the sign off use it from now on.";
    } else {
      settingsNameNote.textContent = "Two letters at least, so it reads like a name.";
      settingsName.focus();
    }
  });
}

const programsName = document.getElementById("programs-name");
const programsStyle = document.getElementById("programs-style");
const programsWrite = document.getElementById("programs-write");
const programsSave = document.getElementById("programs-save");
const programsNote = document.getElementById("programs-note");
const programsList = document.getElementById("programs-list");
const programsEmpty = document.getElementById("programs-empty");
const programsKnown = document.getElementById("programs-known");
let appProfiles = {};

function paintPrograms() {
  if (!programsList) {
    return;
  }
  programsList.replaceChildren();
  const entries = Object.entries(appProfiles);
  programsList.hidden = !entries.length;
  if (programsEmpty) {
    programsEmpty.hidden = entries.length > 0;
  }
  for (const [program, profile] of entries) {
    const row = document.createElement("div");
    const term = document.createElement("dt");
    term.textContent = program.charAt(0).toUpperCase() + program.slice(1);
    const detail = document.createElement("dd");
    const parts = [profile.style || "the usual style"];
    if (profile.write) {
      parts.push("writes " + (LANGUAGE_NAMES[profile.write] || profile.write));
    }
    detail.textContent = parts.join(" · ") + " ";
    const forget = document.createElement("button");
    forget.type = "button";
    forget.className = "swap";
    forget.textContent = "Forget";
    forget.title = "Let this program use the usual style again";
    forget.addEventListener("click", () => saveProgram(program, "", ""));
    detail.append(forget);
    row.append(term, detail);
    programsList.append(row);
  }
  if (programsKnown) {
    programsKnown.replaceChildren();
    const names = new Map();
    for (const name of [
      ...Object.keys(appProfiles),
      ...historyEntries.map((entry) => entry.app),
    ]) {
      if (!name) {
        continue;
      }
      const key = name.toLowerCase();
      const known = names.get(key);
      if (!known || (known === key && name !== key)) {
        names.set(key, name);
      }
    }
    for (const name of names.values()) {
      const option = document.createElement("option");
      option.value = name;
      programsKnown.append(option);
    }
  }
}

async function saveProgram(program, style, write) {
  try {
    const response = await fetch("/profiles", {
      method: "POST",
      headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
      body: JSON.stringify({ program, style, write }),
    });
    const detail = await response.json().catch(() => ({}));
    if (!response.ok) {
      throw new Error(detail.error || "The program could not be saved.");
    }
    appProfiles = detail;
    paintPrograms();
    programsNote.textContent =
      style || write
        ? "Saved. The next dictation into " + program + " follows this."
        : "Forgotten. " + program + " uses the usual style again.";
  } catch (failure) {
    programsNote.textContent = failure.message;
  }
}

if (programsSave) {
  programsSave.addEventListener("click", () => {
    const program = programsName.value.trim();
    if (!program) {
      programsNote.textContent = "Name the program first, as Windows calls it.";
      programsName.focus();
      return;
    }
    saveProgram(program, programsStyle.value, programsWrite.value);
  });
}

async function loadProfiles() {
  if (!programsList) {
    return;
  }
  try {
    const response = await fetch("/profiles", {
      headers: { Authorization: "Bearer " + token },
    });
    if (!response.ok) {
      throw new Error("status " + response.status);
    }
    appProfiles = await response.json();
    paintPrograms();
  } catch (failure) {
    programsNote.textContent = "The programs could not be read from Parlo.";
  }
}

loadProfiles();
paintSettingsName();
paintStyleControls();

document.addEventListener("click", (event) => {
  const preset = event.target.closest(".preset");
  if (!preset) {
    return;
  }
  vocabularyKind.value = "snippet";
  vocabularyKind.dispatchEvent(new Event("change"));
  document.getElementById("vocabulary-wrong").value = preset.dataset.wrong;
  document.getElementById("vocabulary-right").value = preset.dataset.right.replace(
    "{{name}}",
    storedName() || "Alex",
  );
  document.getElementById("vocabulary-right").focus();
});

const vocabularyHint = document.getElementById("vocabulary-hint");
if (vocabularyHint) {
  const examples = {
    term: 'A name Parlo keeps getting wrong: hear "parlo" and write "Parlo".',
    abbreviation: 'A short form you say out loud: "qr" becomes "quarta-feira".',
    snippet: 'A phrase that becomes longer text: say "my channel" and Parlo writes https://www.youtube.com/@example.',
  };
  const paintHint = () => {
    const kind = vocabularyKind.value;
    vocabularyHint.textContent = examples[kind] || examples.term;
    vocabularyRight.placeholder =
      kind === "snippet" ? "https://www.youtube.com/@example" : "Parlo";
  };
  vocabularyKind.addEventListener("change", paintHint);
  paintHint();
}

paintHome();
const historyExport = document.getElementById("history-export");
const historyDays = document.getElementById("history-days");
const vocabularyExport = document.getElementById("vocabulary-export");
const vocabularyImport = document.getElementById("vocabulary-import");
const vocabularyFile = document.getElementById("vocabulary-file");

function saveAsText(text, name, type) {
  const address = URL.createObjectURL(new Blob([text], { type }));
  const link = document.createElement("a");
  link.href = address;
  link.download = name;
  document.body.append(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(address);
}

if (historyExport) {
  historyExport.addEventListener("click", async () => {
    const note = document.getElementById("history-note");
    try {
      const response = await fetch("/history/export", {
        cache: "no-store",
        headers: { Authorization: "Bearer " + token },
      });
      if (!response.ok) {
        throw new Error("The dictations could not be saved.");
      }
      saveAsText(await response.text(), "parlo-dictations.md", "text/markdown");
      if (note) {
        note.textContent = "Saved parlo-dictations.md.";
      }
    } catch (failure) {
      if (note) {
        note.textContent = failure.message || "The dictations could not be saved.";
      }
    }
  });
}

if (historyDays) {
  const note = document.getElementById("history-note");
  fetch("/settings", {
    cache: "no-store",
    headers: { Authorization: "Bearer " + token },
  })
    .then((response) => (response.ok ? response.json() : null))
    .then((settings) => {
      if (settings) {
        historyDays.value = String(settings.history_days || 0);
      }
    })
    .catch(() => {});
  historyDays.addEventListener("change", async () => {
    const days = Number(historyDays.value) || 0;
    try {
      const response = await fetch("/settings", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
        body: JSON.stringify({ history_days: days }),
      });
      const detail = await response.json().catch(() => ({}));
      if (!response.ok) {
        throw new Error(detail.error || "That could not be saved.");
      }
      if (note) {
        note.textContent = days
          ? "Dictations older than " + days + " days are gone, and new ones fall off the same way."
          : "Dictations are kept until the list is full.";
      }
    } catch (failure) {
      if (note) {
        note.textContent = failure.message || "That could not be saved.";
      }
    }
  });
}

if (vocabularyExport) {
  vocabularyExport.addEventListener("click", async () => {
    try {
      const response = await fetch("/vocabulary/export", {
        cache: "no-store",
        headers: { Authorization: "Bearer " + token },
      });
      if (!response.ok) {
        throw new Error("The list could not be saved.");
      }
      saveAsText(await response.text(), "parlo-words.tsv", "text/tab-separated-values");
      vocabularyNote.textContent = "Saved parlo-words.tsv.";
    } catch (failure) {
      vocabularyNote.textContent = failure.message || "The list could not be saved.";
    }
  });
}

if (vocabularyImport && vocabularyFile) {
  vocabularyImport.addEventListener("click", () => vocabularyFile.click());
  vocabularyFile.addEventListener("change", async () => {
    const file = vocabularyFile.files && vocabularyFile.files[0];
    if (!file) {
      return;
    }
    try {
      const text = await file.text();
      const response = await fetch("/vocabulary/import", {
        method: "POST",
        headers: vocabularyHeaders(),
        body: JSON.stringify({ text }),
      });
      const detail = await response.json().catch(() => ({}));
      if (!response.ok) {
        throw new Error(detail.error || "That list could not be read.");
      }
      renderVocabulary(detail);
      vocabularyNote.textContent =
        detail.added === 1 ? "One entry added." : detail.added + " entries added.";
    } catch (failure) {
      vocabularyNote.textContent = failure.message || "That list could not be read.";
    } finally {
      vocabularyFile.value = "";
    }
  });
}
const localeRecommendation = document.getElementById("locale-recommendation");
let recommendationTable = null;

async function showRecommendation(language) {
  if (!localeRecommendation) {
    return;
  }
  if (!recommendationTable) {
    try {
      const response = await fetch("/models", { cache: "no-store" });
      const payload = await response.json();
      recommendationTable = payload.recommendations || [];
    } catch {
      recommendationTable = [];
    }
  }
  const row = recommendationTable.find((item) => item.language === language);
  const picks = row ? row.picks || [] : [];
  if (!picks.length) {
    localeRecommendation.textContent = t(
      "locale.noModel",
      "Parlo has no local model for this language yet.",
    );
    return;
  }
  const words = picks.map((pick) => {
    const name = t("text." + pick.name, pick.name);
    if (pick.feature === "voice") {
      return t("locale.readAloud", "{name} for read aloud", { name });
    }
    if (pick.feature === "dictation") {
      return t("locale.dictation", "{name} for dictation", { name });
    }
    return t("locale.translation", "{name} for translation", { name });
  });
  localeRecommendation.textContent = t("locale.recommends", "Parlo recommends {items}.", {
    items: words.join(", "),
  });
}

if (settingsLocale) {
  settingsLocale.addEventListener("change", () => {
    showRecommendation(settingsLocale.value);
  });
  if (settingsLocale.value) {
    showRecommendation(settingsLocale.value);
  }
}

const pronounceText = document.getElementById("pronounce-text");
const pronounceRun = document.getElementById("pronounce-run");
const pronounceNote = document.getElementById("pronounce-note");
const pronounceResult = document.getElementById("pronounce-result");
const pronounceWritten = document.getElementById("pronounce-written");
const pronounceIpa = document.getElementById("pronounce-ipa");
const pronounceRespell = document.getElementById("pronounce-respell");

if (pronounceRun) {
  pronounceRun.addEventListener("click", async () => {
    const text = pronounceText.value.trim();
    if (!text) {
      pronounceNote.textContent = t(
        "studio.pronounceEmpty",
        "Write a word or a sentence first.",
      );
      pronounceText.focus();
      return;
    }
    pronounceRun.disabled = true;
    pronounceNote.textContent = t("studio.pronounceLookup", "Looking it up…");
    try {
      const response = await fetch("/pronounce?text=" + encodeURIComponent(text), {
        cache: "no-store",
        headers: { Authorization: "Bearer " + token },
      });
      const detail = await response.json().catch(() => ({}));
      if (!response.ok) {
        throw new Error(
          detail.error || t("studio.pronounceBad", "That word could not be looked up."),
        );
      }
      pronounceWritten.textContent = detail.word || detail.text;
      pronounceIpa.textContent = detail.ipa || t("studio.noDictionary", "not in the dictionary");
      pronounceRespell.textContent = detail.respell || detail.text;
      pronounceResult.hidden = false;
      const speech = await fetch("/v1/audio/speech", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
        body: JSON.stringify({ input: detail.respell || detail.text, response_format: "wav" }),
      });
      if (!speech.ok) {
        const error = await speech.json().catch(() => ({}));
        pronounceNote.textContent = t(
          "studio.pronounceAudio",
          "Pronunciation shown. Audio is unavailable: {reason}",
          { reason: error.error || "install a voice first." },
        );
        return;
      }
      const audio = new Audio(URL.createObjectURL(await speech.blob()));
      audio.addEventListener("ended", () => URL.revokeObjectURL(audio.src), { once: true });
      await audio.play();
      pronounceNote.textContent = t("studio.pronouncePlaying", "Playing {word}.", {
        word: detail.word || text,
      });
    } catch (failure) {
      pronounceNote.textContent =
        failure.message ||
        t(
          "studio.pronounceFailed",
          "The pronunciation could not be played. Check that Parlo is running.",
        );
    } finally {
      pronounceRun.disabled = false;
    }
  });
}

const retentionDays = document.getElementById("retention-days");
const retentionCheck = document.getElementById("retention-check");
const retentionNote = document.getElementById("retention-note");
const retentionResult = document.getElementById("retention-result");
const retentionCount = document.getElementById("retention-count");
const retentionDelete = document.getElementById("retention-delete");

if (retentionCheck && retentionDays && retentionDelete) {
  const check = async () => {
    retentionCheck.disabled = true;
    retentionNote.textContent = "Checking…";
    try {
      const response = await fetch("/retention?days=" + encodeURIComponent(retentionDays.value), {
        cache: "no-store",
        headers: { Authorization: "Bearer " + token },
      });
      const detail = await response.json().catch(() => ({}));
      if (!response.ok) {
        throw new Error(detail.error || "The meetings could not be checked.");
      }
      const count = detail.count || 0;
      const titles = (detail.meetings || [])
        .slice(0, 3)
        .map((meeting) => meeting.title)
        .join(", ");
      retentionResult.hidden = false;
      retentionCount.textContent = count
        ? count + " meeting(s) older than " + detail.days + " days" + (titles ? ": " + titles : ".")
        : "No meetings are older than " + detail.days + " days.";
      retentionDelete.hidden = count === 0;
      retentionDelete.textContent = "Delete " + count + " meeting(s) permanently";
      retentionNote.textContent = count
        ? "Nothing has been deleted yet. Confirm below if the list is right."
        : "Nothing to delete.";
    } catch (failure) {
      retentionNote.textContent = failure.message || "The meetings could not be checked.";
    } finally {
      retentionCheck.disabled = false;
    }
  };

  retentionCheck.addEventListener("click", check);
  retentionDelete.addEventListener("click", async () => {
    retentionDelete.disabled = true;
    retentionNote.textContent = "Deleting…";
    try {
      const response = await fetch("/retention", {
        method: "POST",
        headers: { "Content-Type": "application/json", Authorization: "Bearer " + token },
        body: JSON.stringify({ days: Number(retentionDays.value), apply: true }),
      });
      const detail = await response.json().catch(() => ({}));
      if (!response.ok) {
        throw new Error(detail.error || "The meetings could not be deleted.");
      }
      retentionNote.textContent = detail.removed + " meeting(s) deleted.";
      retentionResult.hidden = true;
      await loadMeetings();
    } catch (failure) {
      retentionNote.textContent = failure.message || "The meetings could not be deleted.";
    } finally {
      retentionDelete.disabled = false;
    }
  });
}
