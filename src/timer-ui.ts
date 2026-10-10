import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import { type TimerState, phaseNames, startLabel } from "./timer-state";

const countdown = document.querySelector<HTMLDivElement>("#countdown")!;
const status = document.querySelector<HTMLParagraphElement>("#status")!;
const toggle = document.querySelector<HTMLButtonElement>("#toggle")!;
const reset = document.querySelector<HTMLButtonElement>("#reset")!;
const minutes = document.querySelector<HTMLInputElement>("#minutes")!;
const seconds = document.querySelector<HTMLInputElement>("#seconds")!;
const form = document.querySelector<HTMLFormElement>("#duration-form")!;
const error = document.querySelector<HTMLParagraphElement>("#error")!;
const presets = document.querySelectorAll<HTMLButtonElement>("[data-minutes]");
const modeButtons = document.querySelectorAll<HTMLButtonElement>("[data-mode]");
let current: TimerState;
let busy = false;

function showError(message: string) {
  error.textContent = message;
  error.hidden = false;
}

function render(state: TimerState) {
  const durationChanged = !current || current.durationSeconds !== state.durationSeconds;
  current = state;
  countdown.textContent = `${Math.floor(state.remainingSeconds / 60).toString().padStart(2, "0")}:${(state.remainingSeconds % 60).toString().padStart(2, "0")}`;
  status.textContent = { idle: "Ready", running: "Running", paused: "Paused", finished: "Complete" }[state.status];
  toggle.textContent = state.status === "running" ? "Pause timer" : state.status === "paused" ? "Resume timer" : startLabel(state);
  const pomodoro = state.mode === "pomodoro";
  modeButtons.forEach(button => button.setAttribute("aria-pressed", String(button.dataset.mode === state.mode)));
  document.querySelector<HTMLElement>(".duration")!.hidden = pomodoro;
  document.querySelector<HTMLElement>("#pomodoro-info")!.hidden = !pomodoro;
  reset.textContent = pomodoro ? "Reset cycle" : "Reset";
  if (pomodoro) {
    const session = state.phase === "focus" && state.status !== "finished" ? state.completedSessions + 1 : state.completedSessions;
    document.querySelector<HTMLElement>("#phase-label")!.textContent = `${phaseNames[state.phase]} · Session ${session} of 4`;
    document.querySelectorAll<HTMLElement>("#sessions span").forEach((dot, index) => { dot.classList.toggle("complete", index < state.completedSessions); });
    if (state.status === "finished") status.textContent = `Next: ${phaseNames[state.nextPhase].toLowerCase()}`;
  }
  if (durationChanged) {
    minutes.value = String(Math.floor(state.durationSeconds / 60));
    seconds.value = String(state.durationSeconds % 60);
  }
  presets.forEach(button => button.setAttribute("aria-pressed", String(Number(button.dataset.minutes) * 60 === state.durationSeconds)));
}

async function act(action: string, duration?: number) {
  if (busy) return;
  busy = true;
  error.hidden = true;
  toggle.disabled = reset.disabled = true;
  try {
    await invoke("timer_action", { action, seconds: duration ?? null });
    render(await invoke<TimerState>("get_timer"));
  } catch (cause) {
    showError(String(cause));
  } finally {
    busy = false;
    toggle.disabled = reset.disabled = false;
  }
}

toggle.addEventListener("click", () => void act("toggle"));
modeButtons.forEach(button => button.addEventListener("click", () => {
  if (button.dataset.mode !== current?.mode) void act(button.dataset.mode!);
}));
reset.addEventListener("click", () => void act("reset"));
presets.forEach(button => button.addEventListener("click", () => void act("set_duration", Number(button.dataset.minutes) * 60)));
form.addEventListener("submit", event => {
  event.preventDefault();
  const duration = Number(minutes.value) * 60 + Number(seconds.value);
  if (!form.reportValidity() || !Number.isInteger(duration) || duration < 1 || duration > 86400) {
    showError("Choose a duration between 1 second and 24 hours.");
    return;
  }
  void act("set_duration", duration);
});

async function init() {
  toggle.disabled = reset.disabled = true;
  if (!isTauri()) {
    document.querySelectorAll<HTMLButtonElement>("button").forEach(button => { button.disabled = true; });
    showError("Open the desktop app with npm run tauri dev to use the timer.");
    return;
  }
  try {
    await listen<string>("break-reminder-error", event => showError(event.payload));
    await listen<TimerState>("timer-updated", event => render(event.payload));
    render(await invoke<TimerState>("get_timer"));
    toggle.disabled = reset.disabled = false;
  } catch (cause) {
    showError(`Couldn't connect to the timer: ${String(cause)}`);
  }
}
void init();
