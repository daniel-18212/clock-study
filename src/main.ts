import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./style.css";

type Mode = "pomodoro" | "stopwatch" | "countdown";
type Settings = {
  focus: number;
  shortBreak: number;
  longBreak: number;
  cycles: number;
  countdown: number;
};
type Snapshot = {
  mode: Mode;
  seconds: number;
  running: boolean;
  elapsed: boolean;
  phase: string;
  cycle: number;
  alarm: number;
  notice: string | null;
  laps: number[];
  settings: Settings;
};
type Action = { type: "toggle" | "reset" | "lap" }
  | { type: "select"; value: Mode }
  | { type: "configure"; value: Settings };

function element<T extends HTMLElement>(id: string): T {
  const node = document.getElementById(id);
  if (!node) throw new Error("Elemento ausente: " + id);
  return node as T;
}

const clock = document.querySelector<HTMLElement>(".clock")!;
const modeButtons = document.querySelectorAll<HTMLButtonElement>("[data-mode]");
const floatButton = element<HTMLButtonElement>("float");
const display = element<HTMLButtonElement>("display");
const phase = element("phase");
const presets = element("presets");
const start = element<HTMLButtonElement>("start");
const notice = element("notice");
const editor = element<HTMLDialogElement>("editor");
const form = element<HTMLFormElement>("settings");
const fields = element("fields");
const editNote = element("edit-note");
const editTitle = element("editor-title");
const editError = element("editor-error");
const lapDialog = element<HTMLDialogElement>("laps");
const lapHistory = element<HTMLOListElement>("lap-history");

let state: Snapshot | undefined;
let floating = false;
let supportsFloating = false;
let pending = false;
let shown = "";
let shownMode: Mode | undefined;
let editMode: Mode | undefined;
let alarm: number | undefined;
let sound: AudioContext | undefined;
let lapButton: HTMLButtonElement | undefined;
let historyButton: HTMLButtonElement | undefined;

const twoDigits = (value: number) => String(value).padStart(2, "0");

function format(seconds: number): string {
  const minutes = Math.floor(seconds / 60);
  const mmss = twoDigits(minutes % 60) + ":" + twoDigits(seconds % 60);
  return minutes < 60 ? mmss : twoDigits(Math.floor(minutes / 60)) + ":" + mmss;
}

function setNotice(value?: string) {
  notice.textContent = value ?? "";
  notice.hidden = !value;
}

const svgNamespace = "http://www.w3.org/2000/svg";
const digitWidth = 62;
const digitHeight = 95;
const glyphSize = 100;
const glyphContext = document.createElement("canvas").getContext("2d");
const glyphPositions = new Map<string, { x: number; y: number }>();

function digitFace(face: Element, digit: string) {
  let position = glyphPositions.get(digit);
  if (!position) {
    const style = getComputedStyle(display);
    if (glyphContext) {
      glyphContext.font = `${style.fontWeight} ${glyphSize}px ${style.fontFamily}`;
      glyphContext.textAlign = "center";
      glyphContext.textBaseline = "alphabetic";
    }
    const bounds = glyphContext?.measureText(digit);
    // Center the visible ink on the hinge, regardless of the font's line box.
    position = {
      x: (digitWidth + (bounds ? bounds.actualBoundingBoxLeft - bounds.actualBoundingBoxRight : 0)) / 2,
      y: (digitHeight + (bounds ? bounds.actualBoundingBoxAscent - bounds.actualBoundingBoxDescent : 70)) / 2,
    };
    glyphPositions.set(digit, position);
  }

  let glyph = face.querySelector("text");
  if (!glyph) {
    const svg = document.createElementNS(svgNamespace, "svg");
    svg.setAttribute("viewBox", `0 0 ${digitWidth} ${digitHeight}`);
    glyph = document.createElementNS(svgNamespace, "text");
    glyph.setAttribute("font-size", String(glyphSize));
    glyph.setAttribute("text-anchor", "middle");
    glyph.setAttribute("dominant-baseline", "alphabetic");
    svg.append(glyph);
    face.append(svg);
  }
  glyph.setAttribute("x", String(position.x));
  glyph.setAttribute("y", String(position.y));
  glyph.textContent = digit;
}

function drawTime(next: string) {
  if (next === shown) return;
  if (next.length !== shown.length) {
    display.replaceChildren();
    for (const character of next) {
      const part = document.createElement("span");
      part.className = character === ":" ? "colon" : "digit";
      part.setAttribute("aria-hidden", "true");
      if (character !== ":") for (const side of ["upper", "lower"]) {
        const face = document.createElement("span");
        face.className = side;
        digitFace(face, character);
        part.append(face);
      }
      display.append(part);
    }
  } else {
    [...display.children].forEach((part, index) => {
      const character = next[index];
      if (character === shown[index]) return;
      part.querySelectorAll(".flap").forEach((node) => node.remove());
      digitFace(part.querySelector(".upper")!, character);
      digitFace(part.querySelector(".lower")!, character);
      for (const side of ["falling", "landing"]) {
        const flap = document.createElement("span");
        flap.className = "flap " + side;
        digitFace(flap, side === "falling" ? shown[index] : character);
        flap.addEventListener("animationend", () => flap.remove(), { once: true });
        part.append(flap);
      }
    });
  }
  shown = next;
  display.classList.toggle("hours", next.length > 5);
}

function tone() {
  if (!sound || sound.state !== "running") return;
  const at = sound.currentTime;
  for (const [i, hz] of [660, 880, 1046].entries()) {
    const oscillator = sound.createOscillator();
    const volume = sound.createGain();
    const onset = at + i * 0.15;
    oscillator.frequency.value = hz;
    volume.gain.setValueAtTime(0.001, onset);
    volume.gain.exponentialRampToValueAtTime(0.075, onset + 0.025);
    volume.gain.exponentialRampToValueAtTime(0.001, onset + 0.23);
    oscillator.connect(volume);
    volume.connect(sound.destination);
    oscillator.start(onset);
    oscillator.stop(onset + 0.24);
  }
}

function render(next: Snapshot) {
  state = next;
  if (alarm !== undefined && alarm !== next.alarm && !document.hidden) tone();
  alarm = next.alarm;
  const time = format(next.seconds);
  drawTime(time);
  const editable = next.mode !== "stopwatch" && !floating;
  display.classList.toggle("readonly", !editable);
  display.setAttribute("aria-label", time + (editable ? ". Editar tempo." : ""));
  phase.textContent = (next.notice ? next.notice + " · " : "") + next.phase
    + (next.mode === "pomodoro" && !floating ? " · " + next.cycle + "º ciclo" : "");
  start.textContent = next.running ? "Ⅱ" : "▶";
  const operation = next.running ? "Pausar" : next.elapsed && !next.notice ? "Retomar" : "Iniciar";
  start.setAttribute("aria-label", operation);
  start.title = operation + " (Espaço)";

  if (shownMode !== next.mode) {
    shownMode = next.mode;
    for (const button of modeButtons) button.setAttribute("aria-pressed", String(button.dataset.mode === next.mode));
    drawPresets(next.mode);
  }
  if (lapButton) lapButton.disabled = !next.running;
  if (historyButton) historyButton.disabled = !next.laps.length;
}

async function send(action?: Action): Promise<boolean> {
  try {
    render(await invoke<Snapshot>("timer_action", { action: action ?? null }));
    setNotice();
    return true;
  } catch (error) {
    if (editor.open) editError.textContent = String(error);
    else setNotice(String(error));
    return false;
  }
}

function preset(label: string, callback: () => void): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.textContent = label;
  button.addEventListener("click", callback);
  presets.append(button);
  return button;
}

function configure(changes: Partial<Settings>) {
  if (!state) return;
  if (state.elapsed && !state.notice && !window.confirm("Alterar o tempo reinicia a sessão atual. Continuar?")) return;
  void send({ type: "configure", value: { ...state.settings, ...changes } });
}

function drawPresets(mode: Mode) {
  presets.replaceChildren();
  lapButton = undefined;
  historyButton = undefined;
  if (mode === "pomodoro") {
    preset("25 / 5", () => configure({ focus: 25 * 60, shortBreak: 5 * 60 }));
    preset("50 / 10", () => configure({ focus: 50 * 60, shortBreak: 10 * 60 }));
    preset("Editar tempos", openEditor);
  } else if (mode === "countdown") {
    for (const minutes of [5, 15, 25, 45]) preset(minutes + " min", () => configure({ countdown: minutes * 60 }));
    preset("Editar duração", openEditor);
  } else {
    lapButton = preset("Marcar volta", () => void send({ type: "lap" }));
    historyButton = preset("Ver voltas", showLaps);
  }
}

function showLaps() {
  if (!state || !state.laps.length) return;
  lapHistory.replaceChildren();
  for (const milliseconds of state.laps) {
    const item = document.createElement("li");
    item.textContent = format(Math.floor(milliseconds / 1000)) + "," + twoDigits(Math.floor(milliseconds / 10) % 100);
    lapHistory.append(item);
  }
  lapDialog.showModal();
}

function numberInput(name: string, label: string, value: number, min: number, max: number) {
  const input = document.createElement("input");
  input.type = "number";
  input.inputMode = "numeric";
  input.name = name;
  input.value = twoDigits(value);
  input.min = String(min);
  input.max = String(max);
  input.step = "1";
  input.required = true;
  input.setAttribute("aria-label", label);
  input.addEventListener("focus", () => input.select());
  return input;
}

type DurationKey = Exclude<keyof Settings, "cycles">;
const durationLabels: Record<DurationKey, string> = {
  focus: "Foco", shortBreak: "Pausa curta", longBreak: "Pausa longa", countdown: "Duração",
};

function durationRow(key: DurationKey, seconds: number) {
  const row = document.createElement("div");
  row.className = "duration";
  const caption = document.createElement("span");
  const label = durationLabels[key];
  caption.textContent = label;
  row.append(
    caption,
    numberInput(key + "-h", label + " em horas", Math.floor(seconds / 3600), 0, 99),
    numberInput(key + "-m", label + " em minutos", Math.floor(seconds / 60) % 60, 0, 59),
    numberInput(key + "-s", label + " em segundos", seconds % 60, 0, 59),
  );
  fields.append(row);
}

function openEditor() {
  if (!state || state.mode === "stopwatch" || floating || editor.open) return;
  const pomodoro = state.mode === "pomodoro";
  editMode = state.mode;
  editTitle.textContent = pomodoro ? "Tempos do Pomodoro" : "Duração do temporizador";
  editError.textContent = "";
  editNote.hidden = !state.elapsed;
  fields.replaceChildren();
  const headings = document.createElement("div");
  headings.className = "duration duration-head";
  for (const title of ["", "Horas", "Min", "Seg"]) {
    const cell = document.createElement("span");
    cell.textContent = title;
    headings.append(cell);
  }
  fields.append(headings);
  const keys: DurationKey[] = pomodoro ? ["focus", "shortBreak", "longBreak"] : ["countdown"];
  for (const key of keys) durationRow(key, state.settings[key]);
  if (pomodoro) {
    const row = document.createElement("label");
    row.className = "duration";
    const caption = document.createElement("span");
    caption.textContent = "Pausa longa a cada";
    const unit = document.createElement("span");
    unit.textContent = "blocos";
    row.append(caption, numberInput("cycles", "Blocos antes da pausa longa", state.settings.cycles, 1, 20), unit);
    fields.append(row);
  }
  editor.showModal();
  fields.querySelector<HTMLInputElement>('input[name$="-m"]')?.focus();
}

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  if (!state || !editMode) return;
  const settings = { ...state.settings };
  const values = new FormData(form);
  const keys: DurationKey[] = editMode === "pomodoro" ? ["focus", "shortBreak", "longBreak"] : ["countdown"];
  for (const key of keys) {
    const seconds = Number(values.get(key + "-h")) * 3600
      + Number(values.get(key + "-m")) * 60 + Number(values.get(key + "-s"));
    if (seconds === 0) {
      editError.textContent = durationLabels[key] + ": informe pelo menos 1 segundo.";
      fields.querySelector<HTMLInputElement>('input[name="' + key + '-s"]')?.focus();
      return;
    }
    settings[key] = seconds;
  }
  if (editMode === "pomodoro") settings.cycles = Number(values.get("cycles"));
  if (await send({ type: "configure", value: settings })) editor.close();
});
form.addEventListener("input", () => { editError.textContent = ""; });

async function toggleFloating() {
  if (pending || !supportsFloating) return;
  pending = true;
  try {
    await invoke("set_floating", { enabled: !floating });
    floating = !floating;
    clock.classList.toggle("floating", floating);
    floatButton.textContent = floating ? "Expandir" : "Flutuar";
    floatButton.setAttribute("aria-pressed", String(floating));
    if (state) render(state);
  } catch (error) {
    setNotice("Não foi possível mudar a janela. " + String(error));
  } finally {
    pending = false;
  }
}

function toggleTimer() {
  if (!sound) {
    try { sound = new AudioContext(); } catch { /* A notificação nativa continua disponível. */ }
  }
  void sound?.resume().catch(() => {});
  void send({ type: "toggle" });
}

for (const button of modeButtons) {
  button.addEventListener("click", () => void send({ type: "select", value: button.dataset.mode as Mode }));
}
floatButton.addEventListener("click", () => void toggleFloating());
display.addEventListener("click", openEditor);
start.addEventListener("click", toggleTimer);
element("reset").addEventListener("click", () => void send({ type: "reset" }));
element("cancel").addEventListener("click", () => editor.close());
document.addEventListener("keydown", (event) => {
  if (event.repeat || event.altKey || event.ctrlKey || event.metaKey || editor.open || lapDialog.open) return;
  if (event.code === "Space" && !(event.target instanceof HTMLButtonElement)) toggleTimer();
  else if (event.key.toLowerCase() === "r") void send({ type: "reset" });
  else if (event.key.toLowerCase() === "e") openEditor();
  else if (event.key.toLowerCase() === "f" || (floating && event.key === "Escape")) void toggleFloating();
  else return;
  event.preventDefault();
});

async function initialize() {
  supportsFloating = await invoke<boolean>("supports_floating");
  document.body.classList.toggle("mobile", !supportsFloating);
  floatButton.hidden = !supportsFloating;
  await listen<Snapshot>("clock-tick", ({ payload }) => render(payload));
  await send();

  if (!supportsFloating && !localStorage.getItem("notification-permission-requested")) {
    void invoke("request_android_notifications")
      .then(() => localStorage.setItem("notification-permission-requested", "1"))
      .catch((error: unknown) => setNotice("Não foi possível pedir permissão de notificações. " + String(error)));
  }
}

document.addEventListener("visibilitychange", () => {
  if (!document.hidden && document.body.classList.contains("mobile")) void send();
});

void initialize()
  .catch((error: unknown) => setNotice("Não foi possível iniciar o relógio. " + String(error)));
