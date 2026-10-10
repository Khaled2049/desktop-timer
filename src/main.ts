if (window.location.hash === "#break") {
  await import("./break-reminder");
} else {
  await import("./timer-ui");
}

export {};
