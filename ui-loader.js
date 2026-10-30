import init, { class_build, mount_game_companion } from './ui/dist/aion2_ui.js';

await init();
mount_game_companion();
// renderer.js is a classic script that runs before this resolves; it listens for this event to refresh its combat report.
window.aion2Companion = { classBuild: class_build };
window.dispatchEvent(new Event('aion2-companion-ready'));
