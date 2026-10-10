import { invoke } from "@tauri-apps/api/core";
import "./break-reminder.css";
import { type TimerState, startLabel } from "./timer-state";

document.documentElement.classList.add("break-screen");
document.body.innerHTML = `
  <main class="break-content" aria-labelledby="break-title">
    <div class="break-symbol" aria-hidden="true">
      <svg viewBox="0 0 64 64" fill="none"><path d="M32 49V31m0 8C17 39 12 30 13 17c13 0 21 7 19 22Zm0-9c0-12 7-18 19-18 1 12-5 20-19 20" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
    </div>
    <p class="break-eyebrow">YOUR TIMER IS COMPLETE</p>
    <h1 id="break-title">A moment to move.</h1>
    <p class="break-message">Stand up, stretch a little, and look around.<br />Give your eyes a view beyond the screen.</p>
    <button id="dismiss-break" autofocus>I’m ready</button>
    <p class="break-hint">Press Escape to dismiss · Control + S to start again</p>
    <p id="break-error" role="alert" hidden></p>
  </main>
`;

const button = document.querySelector<HTMLButtonElement>("#dismiss-break")!;
let dismissing = false;
let pomodoro = false;
async function dismiss() {
  if (dismissing) return;
  dismissing = true;
  button.disabled = true;
  try {
    if (pomodoro) await invoke("timer_action", { action: "start", seconds: null });
    await invoke("dismiss_break");
  } catch (cause) {
    const error = document.querySelector<HTMLParagraphElement>("#break-error")!;
    error.textContent = `Couldn't dismiss the reminder: ${String(cause)}`;
    error.hidden = false;
  } finally {
    dismissing = false;
    button.disabled = false;
  }
}
button.addEventListener("click", () => void dismiss());
window.addEventListener("keydown", event => {
  if (event.key === "Escape") {
    event.preventDefault();
    void invoke("dismiss_break").catch(cause => {
      const error = document.querySelector<HTMLParagraphElement>("#break-error")!;
      error.textContent = String(cause);
      error.hidden = false;
    });
  }
});
button.focus();

async function renderPhase() {
  try {
    const state = await invoke<TimerState>("get_timer");
    pomodoro = state.mode === "pomodoro";
    if (!pomodoro) return;
    button.textContent = startLabel(state);
    const hint = document.querySelector<HTMLElement>(".break-hint")!;
    hint.textContent = "Press Escape to dismiss · Control + S to start the next phase";
    if (state.phase !== "focus") {
      document.querySelector<HTMLElement>("#break-title")!.textContent = "Ready for a fresh focus?";
      document.querySelector<HTMLElement>(".break-message")!.textContent = "Your break is complete. Start your next focus session when you’re ready.";
    } else {
      document.querySelector<HTMLElement>(".break-eyebrow")!.textContent = state.nextPhase === "long_break" ? "FOUR FOCUS SESSIONS COMPLETE · 15 MINUTE BREAK" : "FOCUS COMPLETE · 5 MINUTE BREAK";
    }
  } catch (cause) {
    console.error("Could not load the break phase", cause);
  }
}
void renderPhase();
