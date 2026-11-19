"""The goal chime: a short rising two-note bell, played when a goal step is done.

Our own sound (CC0, made here; no third-party samples): two soft bell tones a fifth
apart (A5 then E6), each a sine with a quieter octave partial and an exponential decay,
the second starting 0.12 s after the first. 48 kHz mono, 16-bit.

Reproduce: python3 tools/audio/make_goal_chime.py
Writes assets/audio/sfx/goal_chime/goal_chime.wav.
"""
from pathlib import Path
import math
import struct
import wave

RATE = 48_000
LENGTH_S = 1.4
OUT = Path(__file__).resolve().parents[2] / "assets/audio/sfx/goal_chime/goal_chime.wav"


def bell(t, freq, start):
    if t < start:
        return 0.0
    u = t - start
    attack = min(1.0, u / 0.004)  # a 4 ms onset, no click
    env = attack * math.exp(-u * 3.2)
    return env * (math.sin(math.tau * freq * u) + 0.25 * math.sin(math.tau * 2 * freq * u))


def main():
    frames = bytearray()
    for n in range(int(RATE * LENGTH_S)):
        t = n / RATE
        s = 0.45 * (bell(t, 880.0, 0.0) + bell(t, 1318.5, 0.12))
        frames += struct.pack("<h", int(max(-1.0, min(1.0, s)) * 32767))
    OUT.parent.mkdir(parents=True, exist_ok=True)
    with wave.open(str(OUT), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(RATE)
        w.writeframes(bytes(frames))
    print("wrote", OUT)


if __name__ == "__main__":
    main()
