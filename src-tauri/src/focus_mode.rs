use std::sync::{
    OnceLock,
    atomic::{AtomicBool, Ordering},
    mpsc::{Sender, channel},
};

// macOS has no API for Focus, so these user-made shortcuts do the switching.
const ON_SHORTCUT: &str = "Desktop Timer Focus On";
const OFF_SHORTCUT: &str = "Desktop Timer Focus Off";

static ACTIVE: AtomicBool = AtomicBool::new(false);
static QUEUE: OnceLock<Sender<bool>> = OnceLock::new();

fn run(active: bool) {
    let name = if active { ON_SHORTCUT } else { OFF_SHORTCUT };
    // Fails quietly when the shortcut does not exist, which keeps this opt-in.
    let _ = std::process::Command::new("/usr/bin/shortcuts")
        .args(["run", name])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

// Switches Focus in the background; one worker keeps on and off in order.
pub fn set(active: bool) {
    if ACTIVE.swap(active, Ordering::Relaxed) == active {
        return;
    }
    let queue = QUEUE.get_or_init(|| {
        let (sender, receiver) = channel();
        std::thread::spawn(move || {
            for active in receiver {
                run(active);
            }
        });
        sender
    });
    let _ = queue.send(active);
}

// Used on quit so Focus is not left on after the app is gone.
pub fn turn_off_now() {
    if ACTIVE.swap(false, Ordering::Relaxed) {
        run(false);
    }
}
