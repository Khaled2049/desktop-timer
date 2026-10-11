export type BreakPrompt = { title: string; message: string };

const quickPrompts: BreakPrompt[] = [
  { title: "Rest your eyes.", message: "Pick something at least 20 feet away and hold your gaze on it for 20 seconds. Blink slowly a few times." },
  { title: "Roll your shoulders.", message: "Lift them to your ears, roll them back and down. Five slow circles, then five the other way." },
  { title: "Stand and reach.", message: "Get on your feet, lace your fingers, and push your palms to the ceiling. Hold for three slow breaths." },
  { title: "Drink some water.", message: "Refill your glass and finish half of it before you sit back down." },
  { title: "Loosen your neck.", message: "Drop your ear toward one shoulder and hold for ten seconds. Switch sides, then look slowly left and right." },
  { title: "Breathe out slowly.", message: "In through your nose for four, hold for four, out for six. Repeat five times with your shoulders relaxed." },
  { title: "Shake out your hands.", message: "Let your wrists go loose and shake for ten seconds. Then spread your fingers wide and make a fist, five times." },
  { title: "Find a window.", message: "Walk to the nearest one and look as far into the distance as you can. Name three things you can see." },
];

const longPrompts: BreakPrompt[] = [
  { title: "Take a real walk.", message: "Leave the room for a few minutes. Around the block, up the stairs, or just to another floor and back." },
  { title: "Step outside.", message: "Get some daylight and fresh air. Leave your phone on the desk if you can." },
  { title: "Stretch your whole body.", message: "Fold forward and hang for a few breaths, open your chest, then stretch each leg. Take your time." },
  { title: "Refuel.", message: "Refill your water and have something to eat away from the screen." },
  { title: "Close your eyes.", message: "Sit back or lie down, shut your eyes for two full minutes, and let your jaw and shoulders go soft." },
];

// Every screen shows the same prompt because the pick only depends on shared values.
export function pickBreakPrompt(long: boolean, finishedCount: number, date = new Date()): BreakPrompt {
  const prompts = long ? longPrompts : quickPrompts;
  const day = Math.floor((date.getTime() - date.getTimezoneOffset() * 60000) / 86400000);
  return prompts[(finishedCount + day) % prompts.length];
}
