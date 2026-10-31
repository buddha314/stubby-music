//! Eurorack signal conventions shared by every module.
//!
//! All ports carry `f32` at audio rate. Keep conventions here; modules must not
//! invent their own.

/// Nominal peak of an audio signal.
pub const AUDIO_PEAK: f32 = 1.0;

/// A gate or trigger is high when its signal is above this value.
pub const GATE_THRESHOLD: f32 = 0.5;

/// The value a module writes for a high gate.
pub const GATE_HIGH: f32 = 1.0;

/// Pitch CV is 1.0 per octave (V/oct) relative to a module's base frequency.
#[inline]
pub fn voct_to_hz(base_hz: f32, cv: f32) -> f32 {
    base_hz * cv.exp2()
}

#[inline]
pub fn is_gate_high(signal: f32) -> bool {
    signal > GATE_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_volt_is_one_octave() {
        assert!((voct_to_hz(261.63, 1.0) - 523.26).abs() < 0.01);
        assert!((voct_to_hz(440.0, -1.0) - 220.0).abs() < 0.001);
    }
}
