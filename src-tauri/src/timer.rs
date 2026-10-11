use serde::Serialize;
use std::time::{Duration, SystemTime};

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub duration_seconds: u64,
    pub remaining_seconds: u64,
    pub status: &'static str,
    pub mode: &'static str,
    pub phase: &'static str,
    pub completed_sessions: u64,
    pub finished_count: u64,
    pub next_phase: &'static str,
}

pub struct Timer {
    duration: Duration,
    remaining: Duration,
    deadline: Option<SystemTime>,
    finished: bool,
    pomodoro: bool,
    phase: &'static str,
    completed_sessions: u64,
    finished_count: u64,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            duration: Duration::from_secs(1500),
            remaining: Duration::from_secs(1500),
            deadline: None,
            finished: false,
            pomodoro: false,
            phase: "focus",
            completed_sessions: 0,
            finished_count: 0,
        }
    }
}

impl Timer {
    fn remaining_at(&self, now: SystemTime) -> Duration {
        self.deadline
            .map(|end| end.duration_since(now).unwrap_or_default())
            .unwrap_or(self.remaining)
    }

    pub fn snapshot(&self, now: SystemTime) -> Snapshot {
        let remaining = self.remaining_at(now);
        Snapshot {
            mode: if self.pomodoro { "pomodoro" } else { "timer" },
            phase: self.phase,
            completed_sessions: self.completed_sessions,
            finished_count: self.finished_count,
            next_phase: self.next_phase(),
            duration_seconds: self.duration.as_secs(),
            remaining_seconds: remaining.as_secs() + u64::from(remaining.subsec_nanos() > 0),
            status: if self.finished {
                "finished"
            } else if self.deadline.is_some() {
                "running"
            } else if self.remaining < self.duration {
                "paused"
            } else {
                "idle"
            },
        }
    }

    pub fn tick(&mut self, now: SystemTime) -> bool {
        if self.deadline.is_some() && self.remaining_at(now).is_zero() {
            self.deadline = None;
            self.remaining = Duration::ZERO;
            self.finished = true;
            self.finished_count += 1;
            if self.pomodoro && self.phase == "focus" {
                self.completed_sessions += 1;
            }
            return true;
        }
        false
    }

    fn next_phase(&self) -> &'static str {
        if !self.pomodoro {
            return "focus";
        }
        if self.phase == "focus" {
            if self.completed_sessions >= 4 {
                "long_break"
            } else {
                "short_break"
            }
        } else {
            "focus"
        }
    }

    fn select_phase(&mut self, phase: &'static str) {
        self.phase = phase;
        self.duration = Duration::from_secs(match phase {
            "short_break" => 300,
            "long_break" => 900,
            _ => 1500,
        });
        self.remaining = self.duration;
        self.deadline = None;
        self.finished = false;
    }

    pub fn action(
        &mut self,
        action: &str,
        seconds: Option<u64>,
        now: SystemTime,
    ) -> Result<(), String> {
        match action {
            "pomodoro" | "timer" | "toggle_pomodoro" => {
                self.pomodoro = match action {
                    "pomodoro" => true,
                    "timer" => false,
                    _ => !self.pomodoro,
                };
                self.completed_sessions = 0;
                self.select_phase("focus");
            }
            "toggle" => {
                let action = if self.deadline.is_some() {
                    "pause"
                } else {
                    "start"
                };
                self.action(action, None, now)?;
            }
            "start" => {
                if self.deadline.is_none() {
                    if self.finished {
                        if self.pomodoro {
                            let next = self.next_phase();
                            if self.phase == "long_break" {
                                self.completed_sessions = 0;
                            }
                            self.select_phase(next);
                        } else {
                            self.remaining = self.duration;
                            self.finished = false;
                        }
                    }
                    self.deadline = Some(now + self.remaining);
                }
            }
            "pause" => {
                if self.deadline.is_some() {
                    let remaining = self.remaining_at(now);
                    if remaining.is_zero() {
                        return Ok(());
                    }
                    self.remaining = remaining;
                    self.deadline = None;
                }
            }
            "reset" => {
                if self.pomodoro {
                    self.completed_sessions = 0;
                    self.select_phase("focus");
                }
                self.remaining = self.duration;
                self.deadline = None;
                self.finished = false;
            }
            "set_duration" => {
                let seconds = seconds
                    .filter(|s| (1..=86400).contains(s))
                    .ok_or("Choose a duration between 1 second and 24 hours.")?;
                self.pomodoro = false;
                self.phase = "focus";
                self.completed_sessions = 0;
                self.duration = Duration::from_secs(seconds);
                self.remaining = self.duration;
                self.deadline = None;
                self.finished = false;
            }
            _ => return Err("Unknown timer action".into()),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pause_preserves_fractional_time_and_resume_finishes_once() {
        let now = SystemTime::UNIX_EPOCH;
        let mut timer = Timer::default();
        timer.action("set_duration", Some(5), now).unwrap();
        timer.action("start", None, now).unwrap();
        timer
            .action("pause", None, now + Duration::from_millis(1250))
            .unwrap();
        assert_eq!(
            timer
                .snapshot(now + Duration::from_secs(100))
                .remaining_seconds,
            4
        );
        timer
            .action("start", None, now + Duration::from_secs(100))
            .unwrap();
        assert!(!timer.tick(now + Duration::from_millis(103749)));
        assert!(timer.tick(now + Duration::from_millis(103750)));
        assert!(!timer.tick(now + Duration::from_secs(200)));
        assert_eq!(timer.snapshot(now).status, "finished");
        assert_eq!(timer.snapshot(now).finished_count, 1);
        timer.action("start", None, now).unwrap();
        assert_eq!(timer.snapshot(now).remaining_seconds, 5);
    }
    #[test]
    fn toggle_starts_pauses_resumes_and_restarts_finished_timer() {
        let now = SystemTime::UNIX_EPOCH;
        let mut timer = Timer::default();
        timer.action("toggle", None, now).unwrap();
        assert_eq!(timer.snapshot(now).status, "running");
        timer
            .action("toggle", None, now + Duration::from_secs(10))
            .unwrap();
        assert_eq!(timer.snapshot(now).status, "paused");
        assert_eq!(timer.snapshot(now).remaining_seconds, 1490);
        timer
            .action("toggle", None, now + Duration::from_secs(20))
            .unwrap();
        assert_eq!(timer.snapshot(now).status, "running");
        assert!(timer.tick(now + Duration::from_secs(1510)));
        timer
            .action("toggle", None, now + Duration::from_secs(1511))
            .unwrap();
        assert_eq!(
            timer
                .snapshot(now + Duration::from_secs(1511))
                .remaining_seconds,
            1500
        );
        assert_eq!(timer.snapshot(now).status, "running");
    }

    #[test]
    fn catches_up_after_sleep_and_resets() {
        let now = SystemTime::UNIX_EPOCH;
        let mut timer = Timer::default();
        timer.action("start", None, now).unwrap();
        assert!(timer.tick(now + Duration::from_secs(3600)));
        timer.action("reset", None, now).unwrap();
        assert_eq!(timer.snapshot(now).remaining_seconds, 1500);
        assert_eq!(timer.snapshot(now).status, "idle");
    }
    #[test]
    fn pomodoro_cycles_four_focus_sessions_then_long_break() {
        let mut now = SystemTime::UNIX_EPOCH;
        let mut timer = Timer::default();
        timer.action("pomodoro", None, now).unwrap();
        for session in 1..=4 {
            timer.action("toggle", None, now).unwrap();
            assert_eq!(timer.snapshot(now).phase, "focus");
            now += Duration::from_secs(1500);
            assert!(timer.tick(now));
            assert!(!timer.tick(now));
            assert_eq!(timer.snapshot(now).completed_sessions, session);
            assert_eq!(timer.snapshot(now).status, "finished");
            timer.action("toggle", None, now).unwrap();
            let duration = if session == 4 { 900 } else { 300 };
            assert_eq!(timer.snapshot(now).duration_seconds, duration);
            assert_eq!(
                timer.snapshot(now).phase,
                if session == 4 {
                    "long_break"
                } else {
                    "short_break"
                }
            );
            now += Duration::from_secs(duration);
            assert!(timer.tick(now));
        }
        timer.action("toggle", None, now).unwrap();
        assert_eq!(timer.snapshot(now).phase, "focus");
        assert_eq!(timer.snapshot(now).completed_sessions, 0);
    }

    #[test]
    fn expired_pause_still_completes_focus_once_and_waits_after_sleep() {
        let now = SystemTime::UNIX_EPOCH;
        let mut timer = Timer::default();
        timer.action("pomodoro", None, now).unwrap();
        timer.action("start", None, now).unwrap();
        let wake = now + Duration::from_secs(10000);
        timer.action("pause", None, wake).unwrap();
        assert!(timer.tick(wake));
        assert!(!timer.tick(wake + Duration::from_secs(10000)));
        assert_eq!(timer.snapshot(wake).completed_sessions, 1);
        assert_eq!(timer.snapshot(wake).status, "finished");
        assert_eq!(timer.snapshot(wake).next_phase, "short_break");
    }

    #[test]
    fn pomodoro_pause_reset_and_switch_to_custom_timer() {
        let now = SystemTime::UNIX_EPOCH;
        let mut timer = Timer::default();
        timer.action("pomodoro", None, now).unwrap();
        timer.action("start", None, now).unwrap();
        timer
            .action("pause", None, now + Duration::from_secs(10))
            .unwrap();
        assert_eq!(timer.snapshot(now).remaining_seconds, 1490);
        assert_eq!(timer.snapshot(now).completed_sessions, 0);
        timer
            .action("start", None, now + Duration::from_secs(20))
            .unwrap();
        assert!(timer.tick(now + Duration::from_secs(1510)));
        timer
            .action("start", None, now + Duration::from_secs(1510))
            .unwrap();
        timer.action("reset", None, now).unwrap();
        assert_eq!(timer.snapshot(now).phase, "focus");
        assert_eq!(timer.snapshot(now).completed_sessions, 0);
        assert_eq!(timer.snapshot(now).status, "idle");
        timer.action("set_duration", Some(30), now).unwrap();
        assert_eq!(timer.snapshot(now).mode, "timer");
        assert_eq!(timer.snapshot(now).duration_seconds, 30);
    }

    #[test]
    fn rejects_invalid_durations_without_changing_timer() {
        let now = SystemTime::UNIX_EPOCH;
        let mut timer = Timer::default();
        for seconds in [None, Some(0), Some(86401)] {
            assert!(timer.action("set_duration", seconds, now).is_err());
        }
        assert_eq!(timer.snapshot(now).duration_seconds, 1500);
    }
}
