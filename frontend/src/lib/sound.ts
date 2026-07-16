// Simple sound effects using Web Audio API — no external files needed.

let audioCtx: AudioContext | null = null;

function ctx(): AudioContext {
  if (!audioCtx) audioCtx = new AudioContext();
  return audioCtx;
}

function play(freq: number, duration: number, type: OscillatorType = "sine", volume = 0.15) {
  try {
    const c = ctx();
    const osc = c.createOscillator();
    const gain = c.createGain();
    osc.type = type;
    osc.frequency.value = freq;
    gain.gain.setValueAtTime(volume, c.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.001, c.currentTime + duration);
    osc.connect(gain);
    gain.connect(c.destination);
    osc.start();
    osc.stop(c.currentTime + duration);
  } catch { /* audio not available */ }
}

export const sounds = {
  tilePlace: () => play(600, 0.08, "sine", 0.1),

  tilePickup: () => play(400, 0.06, "square", 0.06),

  tileReturn: () => play(300, 0.08, "sine", 0.08),

  moveSubmit: () => {
    play(523, 0.1, "sine", 0.12);
    setTimeout(() => play(659, 0.1, "sine", 0.12), 100);
    setTimeout(() => play(784, 0.15, "sine", 0.12), 200);
  },

  invalidMove: () => {
    play(200, 0.2, "sawtooth", 0.08);
    setTimeout(() => play(150, 0.3, "sawtooth", 0.06), 150);
  },

  yourTurn: () => {
    play(440, 0.12, "sine", 0.1);
    setTimeout(() => play(660, 0.12, "sine", 0.1), 120);
    setTimeout(() => play(880, 0.2, "sine", 0.1), 240);
  },

  gameOver: () => {
    play(523, 0.15, "sine", 0.12);
    setTimeout(() => play(659, 0.15, "sine", 0.12), 150);
    setTimeout(() => play(784, 0.15, "sine", 0.12), 300);
    setTimeout(() => play(1047, 0.3, "sine", 0.15), 450);
  },

  chatMessage: () => play(800, 0.06, "sine", 0.05),
};
