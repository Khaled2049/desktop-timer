export type TimerState = {
  durationSeconds: number;
  remainingSeconds: number;
  status: "idle" | "running" | "paused" | "finished";
  mode: "timer" | "pomodoro";
  phase: "focus" | "short_break" | "long_break";
  completedSessions: number;
  finishedCount: number;
  nextPhase: "focus" | "short_break" | "long_break";
};
export const phaseNames = { focus: "Focus", short_break: "Short break", long_break: "Long break" };
export function startLabel(state: TimerState): string {
  if (state.mode === "pomodoro") {
    const phase = state.status === "finished" ? state.nextPhase : state.phase;
    return `Start ${phaseNames[phase].toLowerCase()}`;
  }
  return state.status === "finished" ? "Start again" : "Start timer";
}
