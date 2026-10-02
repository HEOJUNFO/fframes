from pathlib import Path
import wave

import numpy as np

RATE = 48000
DURATION = 14.5
rng = np.random.default_rng(71362)
mix = np.zeros((int(RATE * DURATION), 2), dtype=np.float64)


def add(at, signal, gain=1.0, pan=0.0):
    start = round(at * RATE)
    length = min(len(signal), len(mix) - start)
    if length <= 0:
        return
    signal = signal[:length] * gain
    mix[start:start + length, 0] += signal * np.sqrt((1 - pan) / 2)
    mix[start:start + length, 1] += signal * np.sqrt((1 + pan) / 2)


def tone(seconds, start, end, decay=4, noise=0):
    t = np.arange(round(seconds * RATE)) / RATE
    f = start * (end / start) ** (t / seconds)
    phase = np.cumsum(f) * 2 * np.pi / RATE
    envelope = np.minimum(t / 0.008, 1) * np.exp(-t * decay)
    envelope *= np.clip((seconds - t) / 0.025, 0, 1)
    return (np.sin(phase) + noise * rng.normal(size=len(t))) * envelope


def swish(at, seconds, gain, pan=0):
    t = np.arange(round(seconds * RATE)) / RATE
    white = rng.normal(size=len(t))
    smooth = np.convolve(white, np.ones(17) / 17, mode="same")
    envelope = np.sin(np.pi * t / seconds) ** 2
    ring = np.sin(2 * np.pi * (100 * t + 450 * t * t))
    add(at, (smooth * 1.8 + ring * 0.11) * envelope, gain, pan)


# Uneven clockwork, a tiny squeak and a nervous pizzicato pulse.
swish(0.08, 0.65, 0.17, 0.3)
for i, at in enumerate(np.arange(0.18, 3.45, 0.19)):
    add(at, tone(0.07, 720 + i % 3 * 140, 280, 48, 0.3), 0.10, 0.4)
for i, at in enumerate([0.22, 0.68, 1.14, 1.60, 2.06, 2.52, 2.98]):
    f = [146.83, 164.81, 146.83, 116.54][i % 4]
    add(at, tone(0.38, f, f * 0.99, 11), 0.24, -0.25)
add(2.80, tone(0.31, 1350, 2600, 5), 0.10, 0.5)
swish(3.32, 0.35, 0.65, -0.4)

# Rust arrives on heavy, slightly lopsided steps.
for i, at in enumerate([3.55, 4.01, 4.51, 5.03, 5.55, 6.03]):
    add(at, tone(0.40, 110, 43, 10, 0.04), 0.58)
    add(at + 0.03, tone(0.16, 640, 140, 18, 0.8), 0.11, -0.4)
    add(at + 0.18, tone(0.28, [73.42, 87.31, 98.00][i % 3], 65, 9), 0.23, 0.2)
swish(6.10, 0.55, 0.8, -0.2)

# A rising motor resolves into a fast, bright rush.
for i, at in enumerate(np.arange(6.40, 8.62, 0.14)):
    f = [146.83, 220, 293.66, 349.23][i % 4]
    add(at, tone(0.22, f, f, 16), 0.18, (i % 3 - 1) * 0.6)
    if i % 2 == 0:
        add(at, tone(0.16, 100, 47, 25), 0.40)
swish(6.40, 2.18, 0.45, 0.4)
add(6.43, tone(1.70, 100, 550, 0.5), 0.08, -0.3)

# Stop the rush for the cotton-paper burst, then let the ghost whistle away.
add(8.66, tone(0.85, 150, 34, 6, 0.14), 0.80)
add(8.67, tone(0.14, 1400, 210, 25, 1.0), 0.33)
for i in range(22):
    add(8.72 + i * 0.035, tone(0.055, 1800 + i * 60, 430, 65, 2),
        0.020, (i % 7 - 3) / 4)
add(9.12, tone(0.50, 420, 170, 6), 0.15, 0.45)
add(9.76, tone(0.30, 380, 730, 7), 0.08, 0.60)

# A warm suspended chord and a small final wink.
for f, pan in [(146.83, -0.5), (220, 0.5), (293.66, -0.2), (349.23, 0.3)]:
    add(10.16, tone(4.30, f, f * 0.998, 0.65), 0.13, pan)
add(10.16, tone(0.50, 110, 45, 7), 0.45)
swish(10.20, 0.62, 0.28, 0.2)
add(10.97, tone(0.30, 130, 46, 13, 0.04), 0.48, 0.4)
add(11.22, tone(0.42, 880, 1174.66, 9), 0.075, 0.6)
add(11.43, tone(0.43, 1174.66, 880, 9), 0.055, 0.7)

# Short stereo room, leaving the attacks sharp.
dry = mix.copy()
for seconds, level in [(0.083, 0.15), (0.137, 0.09), (0.219, 0.06)]:
    delay = round(seconds * RATE)
    mix[delay:] += dry[:-delay, ::-1] * level
mix *= np.minimum(np.arange(len(mix)) / (RATE * 0.03), 1)[:, None]
mix *= np.minimum(np.arange(len(mix))[::-1] / (RATE * 0.45), 1)[:, None]
mix = np.tanh(mix * 1.1)
mix *= 0.78 / max(np.max(np.abs(mix)), 1e-10)
out = Path(__file__).with_name("fable.wav")
with wave.open(str(out), "wb") as wav:
    wav.setnchannels(2)
    wav.setsampwidth(2)
    wav.setframerate(RATE)
    wav.writeframes((mix * 32767).astype("<i2").tobytes())
print(out)
