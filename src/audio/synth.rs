use rodio::Source;
use std::f32::consts::PI;
use std::num::NonZero;
use std::time::Duration;

pub const SAMPLE_RATE: u32 = 44100;

/// Fast pseudo-random number generator for audio synthesis (XorShift32)
#[derive(Debug, Clone)]
pub struct FastRng {
    state: u32,
}

impl FastRng {
    pub fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 0x12345678 } else { seed },
        }
    }

    #[inline(always)]
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    #[inline(always)]
    pub fn next_f32(&mut self) -> f32 {
        // [0.0, 1.0)
        (self.next_u32() >> 8) as f32 / 16777216.0
    }

    #[inline(always)]
    pub fn next_signed(&mut self) -> f32 {
        // [-1.0, 1.0)
        self.next_f32() * 2.0 - 1.0
    }

    #[inline(always)]
    pub fn gen_range_u16(&mut self, min: u16, max: u16) -> u16 {
        if min >= max { min } else { min + (self.next_u32() as u16 % (max - min)) }
    }

    #[inline(always)]
    pub fn gen_range_u8(&mut self, min: u8, max: u8) -> u8 {
        if min >= max { min } else { min + (self.next_u32() as u8 % (max - min)) }
    }

    #[inline(always)]
    pub fn gen_range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }

    #[inline(always)]
    pub fn gen_bool(&mut self, p: f32) -> bool {
        self.next_f32() < p
    }
}

/// Stereo Binaural Beats Source:
/// Left ear: sine at base_freq
/// Right ear: sine at base_freq + beat_freq
pub struct BinauralSource {
    phase_left: f32,
    phase_right: f32,
    base_freq: f32,
    beat_freq: f32,
    channel_toggle: bool, // alternating L, R
}

impl BinauralSource {
    pub fn new(base_freq: f32, beat_freq: f32) -> Self {
        Self {
            phase_left: 0.0,
            phase_right: 0.0,
            base_freq,
            beat_freq,
            channel_toggle: false,
        }
    }

    #[allow(dead_code)]
    pub fn set_beat_freq(&mut self, beat_freq: f32) {
        self.beat_freq = beat_freq;
    }
}

impl Iterator for BinauralSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let sample_rate = SAMPLE_RATE as f32;
        self.channel_toggle = !self.channel_toggle;

        if self.channel_toggle {
            // Left channel
            let sample = (self.phase_left * 2.0 * PI).sin() * 0.4;
            self.phase_left = (self.phase_left + self.base_freq / sample_rate).fract();
            Some(sample)
        } else {
            // Right channel
            let right_freq = self.base_freq + self.beat_freq;
            let sample = (self.phase_right * 2.0 * PI).sin() * 0.4;
            self.phase_right = (self.phase_right + right_freq / sample_rate).fract();
            Some(sample)
        }
    }
}

impl Source for BinauralSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> NonZero<u16> {
        NonZero::new(2).unwrap()
    }
    fn sample_rate(&self) -> NonZero<u32> {
        NonZero::new(SAMPLE_RATE).unwrap()
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// Procedural Noise Generator supporting White, Pink, and Brown noise.
pub struct NoiseSource {
    kind: NoiseKind,
    rng: FastRng,
    // Filter states for Pink noise (Paul Kellet's filter)
    b0: f32,
    b1: f32,
    b2: f32,
    b3: f32,
    b4: f32,
    b5: f32,
    b6: f32,
    // Filter state for Brown noise (leaky integration)
    last_brown: f32,
    channel_toggle: bool,
    current_sample: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseKind {
    White,
    Pink,
    Brown,
}

impl NoiseSource {
    pub fn new(kind: NoiseKind, seed: u32) -> Self {
        Self {
            kind,
            rng: FastRng::new(seed),
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            b3: 0.0,
            b4: 0.0,
            b5: 0.0,
            b6: 0.0,
            last_brown: 0.0,
            channel_toggle: false,
            current_sample: 0.0,
        }
    }
}

impl Iterator for NoiseSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.channel_toggle = !self.channel_toggle;
        if self.channel_toggle {
            let white = self.rng.next_signed();
            let sample = match self.kind {
                NoiseKind::White => white * 0.15,
                NoiseKind::Pink => {
                    self.b0 = 0.99886 * self.b0 + white * 0.0555179;
                    self.b1 = 0.99332 * self.b1 + white * 0.0750759;
                    self.b2 = 0.96900 * self.b2 + white * 0.1538520;
                    self.b3 = 0.86650 * self.b3 + white * 0.3104856;
                    self.b4 = 0.55000 * self.b4 + white * 0.5329522;
                    self.b5 = -0.7616 * self.b5 - white * 0.0168980;
                    let pink = self.b0 + self.b1 + self.b2 + self.b3 + self.b4 + self.b5 + self.b6 + white * 0.5362;
                    self.b6 = white * 0.115926;
                    pink * 0.04
                }
                NoiseKind::Brown => {
                    self.last_brown = (self.last_brown + (0.02 * white)) / 1.02;
                    self.last_brown * 1.5
                }
            };
            self.current_sample = sample.clamp(-1.0, 1.0);
        }
        Some(self.current_sample)
    }
}

impl Source for NoiseSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> NonZero<u16> {
        NonZero::new(2).unwrap()
    }
    fn sample_rate(&self) -> NonZero<u32> {
        NonZero::new(SAMPLE_RATE).unwrap()
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// Procedural Rain Sound Generator
/// High-density pink noise + sporadic droplet impacts + anti-fatigue thunder rolls
pub struct RainSource {
    #[allow(dead_code)]
    rng: FastRng,
    pink: NoiseSource,
    thunder_counter: u32,
    thunder_active: bool,
    thunder_sample_idx: u32,
    thunder_duration: u32,
    thunder_rumble: f32,
    channel_toggle: bool,
    current_sample: f32,
    lp_filter: f32,
}

impl RainSource {
    pub fn new(seed: u32) -> Self {
        Self {
            rng: FastRng::new(seed),
            pink: NoiseSource::new(NoiseKind::Pink, seed + 10),
            thunder_counter: 0,
            thunder_active: false,
            thunder_sample_idx: 0,
            thunder_duration: 0,
            thunder_rumble: 0.0,
            channel_toggle: false,
            current_sample: 0.0,
            lp_filter: 0.0,
        }
    }
}

impl Iterator for RainSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.channel_toggle = !self.channel_toggle;
        if self.channel_toggle {
            let base_rain = self.pink.next().unwrap_or(0.0) * 1.2;

            // Sporadic raindrops (sharp burst of high frequency)
            let is_drop = self.rng.next_f32() < 0.0015;
            let drop_click = if is_drop {
                self.rng.next_signed() * 0.35
            } else {
                0.0
            };

            // Anti-fatigue distant thunder event (every ~2-3 minutes)
            self.thunder_counter = self.thunder_counter.wrapping_add(1);
            if !self.thunder_active && self.thunder_counter > SAMPLE_RATE * 120 {
                if self.rng.next_f32() < 0.00005 {
                    self.thunder_active = true;
                    self.thunder_sample_idx = 0;
                    self.thunder_duration = SAMPLE_RATE * 6; // 6-second roll
                    self.thunder_counter = 0;
                }
            }

            let mut thunder = 0.0;
            if self.thunder_active {
                self.thunder_sample_idx += 1;
                let progress = self.thunder_sample_idx as f32 / self.thunder_duration as f32;
                if progress >= 1.0 {
                    self.thunder_active = false;
                } else {
                    let env = (progress * PI).sin().powi(2);
                    let noise = self.rng.next_signed();
                    self.thunder_rumble = (self.thunder_rumble + noise * 0.02) / 1.015;
                    thunder = self.thunder_rumble * env * 1.8;
                }
            }

            // Gentle low-pass on the aggregate
            let raw = base_rain + drop_click + thunder;
            self.lp_filter = self.lp_filter * 0.7 + raw * 0.3;
            self.current_sample = self.lp_filter.clamp(-1.0, 1.0);
        }

        Some(self.current_sample)
    }
}

impl Source for RainSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> NonZero<u16> {
        NonZero::new(2).unwrap()
    }
    fn sample_rate(&self) -> NonZero<u32> {
        NonZero::new(SAMPLE_RATE).unwrap()
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// Resonant wood snap / fracture voice with acoustic cavity resonator and mechanical click transient
#[derive(Debug, Clone, Copy, Default)]
struct WoodSnap {
    y1: f32,
    y2: f32,
    a1: f32,
    a2: f32,
    click_env: f32,
    click_decay: f32,
    noise_weight: f32,
}

impl WoodSnap {
    fn trigger(&mut self, freq: f32, decay_ms: f32, amplitude: f32, noise_amt: f32) {
        let w0 = 2.0 * PI * freq / (SAMPLE_RATE as f32);
        let decay_samples = (decay_ms / 1000.0) * (SAMPLE_RATE as f32);
        let r = (-2.5 / decay_samples.max(1.0)).exp();
        self.a1 = 2.0 * r * w0.cos();
        self.a2 = -(r * r);

        // Exact normalized bounded resonance: amplitude is strictly bounded by `amplitude`
        self.y1 = amplitude * r * w0.sin();
        self.y2 = 0.0;

        // Fast attack transient click (decays in ~1-2 ms)
        self.click_env = amplitude * 0.7;
        self.click_decay = (-1.0 / (0.0012 * SAMPLE_RATE as f32)).exp();
        self.noise_weight = noise_amt;
    }

    #[inline(always)]
    fn tick(&mut self, rng: &mut FastRng) -> f32 {
        let y = self.a1 * self.y1 + self.a2 * self.y2;
        self.y2 = self.y1;
        self.y1 = y;

        // Prevent denormal / subnormal floating point numbers
        if self.y1.abs() < 1e-7 && self.y2.abs() < 1e-7 {
            self.y1 = 0.0;
            self.y2 = 0.0;
        }

        let click = if self.click_env > 1e-4 {
            let c = (rng.next_signed() * self.noise_weight + (1.0 - self.noise_weight)) * self.click_env;
            self.click_env *= self.click_decay;
            c
        } else {
            self.click_env = 0.0;
            0.0
        };

        y + click
    }
}

/// Procedural Fireplace / Campfire Sound Generator
/// Multi-layered organic physical model:
/// 1. Warm turbulent flame draft & roar (Kellet pink noise with chaotic flicker modulation)
/// 2. Continuous ember sizzle & moisture evaporation (~2.5 - 5.5 kHz bandpassed flutter)
/// 3. Resonant wood crackles & clustering grain fractures (Poisson-distributed micro-bursts)
/// 4. Deep hollow log pops & resin bursts with trailing ember showers
/// 5. Spatial stereo imaging with subtle panning of individual crackles and draft
pub struct FireplaceSource {
    rng: FastRng,
    // Pink noise filter states for flame turbulence (Paul Kellet filter)
    b0: f32,
    b1: f32,
    b2: f32,
    b3: f32,
    b4: f32,
    b5: f32,
    b6: f32,
    // Warm flame draft low-pass filter states
    flame_lp_l: f32,
    flame_lp_r: f32,
    draft_val: f32,
    flicker_lfo: f32,
    sample_counter: u32,

    // Continuous ember sizzle bandpass
    sizzle_hp: f32,
    sizzle_lp: f32,
    last_sizzle_in: f32,

    // Polyphonic wood crackle voices
    voices: [WoodSnap; 4],
    pans: [f32; 4],
    voice_idx: usize,

    // Cluster burst fracture state
    cluster_left: u8,
    cluster_gap: u16,
    cluster_pan: f32,

    // Log pop / resin burst state (Poisson process)
    thump: WoodSnap,
    thump_pan: f32,
    samples_to_pop: u32,
    ember_tail: u32,

    // Stereo channel toggling and sample buffer
    channel_toggle: bool,
    current_left: f32,
    current_right: f32,
}

impl FireplaceSource {
    pub fn new(seed: u32) -> Self {
        let mut rng = FastRng::new(seed);
        let u = rng.next_f32().max(1e-6);
        let initial_pop_delay = (-u.ln() * 2.5 * SAMPLE_RATE as f32) as u32;

        Self {
            rng,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            b3: 0.0,
            b4: 0.0,
            b5: 0.0,
            b6: 0.0,
            flame_lp_l: 0.0,
            flame_lp_r: 0.0,
            draft_val: 0.5,
            flicker_lfo: 0.0,
            sample_counter: 0,
            sizzle_hp: 0.0,
            sizzle_lp: 0.0,
            last_sizzle_in: 0.0,
            voices: [WoodSnap::default(); 4],
            pans: [0.0; 4],
            voice_idx: 0,
            cluster_left: 0,
            cluster_gap: 0,
            cluster_pan: 0.0,
            thump: WoodSnap::default(),
            thump_pan: 0.0,
            samples_to_pop: initial_pop_delay,
            ember_tail: 0,
            channel_toggle: false,
            current_left: 0.0,
            current_right: 0.0,
        }
    }
}

impl Iterator for FireplaceSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.channel_toggle = !self.channel_toggle;
        if self.channel_toggle {
            // Advancing one stereo frame at SAMPLE_RATE (44.1 kHz)
            self.sample_counter = self.sample_counter.wrapping_add(1);

            // 1. Warm Flame Draft & Roar
            self.flicker_lfo = (self.flicker_lfo + 0.7 / SAMPLE_RATE as f32).fract();
            if self.sample_counter % 128 == 0 {
                let target = 0.40 + self.rng.next_f32() * 0.35 + 0.12 * (self.flicker_lfo * 2.0 * PI).sin();
                self.draft_val = self.draft_val * 0.92 + target * 0.08;
            }

            // Pink noise generation (Paul Kellet's filter)
            let white = self.rng.next_signed();
            self.b0 = 0.99886 * self.b0 + white * 0.0555179;
            self.b1 = 0.99332 * self.b1 + white * 0.0750759;
            self.b2 = 0.96900 * self.b2 + white * 0.1538520;
            self.b3 = 0.86650 * self.b3 + white * 0.3104856;
            self.b4 = 0.55000 * self.b4 + white * 0.5329522;
            self.b5 = -0.7616 * self.b5 - white * 0.0168980;
            let pink = (self.b0 + self.b1 + self.b2 + self.b3 + self.b4 + self.b5 + self.b6 + white * 0.5362) * 0.04;
            self.b6 = white * 0.115926;

            // Lowpass flame warmth around 280-380 Hz
            self.flame_lp_l += 0.045 * (pink - self.flame_lp_l);
            self.flame_lp_r += 0.042 * (pink - self.flame_lp_r);
            let flame_l = self.flame_lp_l * self.draft_val * 1.6;
            let flame_r = self.flame_lp_r * self.draft_val * 1.6;

            // 2. Ember Sizzle & Moisture Hiss (bandpassed ~2.5 - 5.5 kHz)
            let sw = self.rng.next_signed();
            self.sizzle_hp = 0.75 * (self.sizzle_hp + sw - self.last_sizzle_in);
            self.last_sizzle_in = sw;
            self.sizzle_lp += 0.32 * (self.sizzle_hp - self.sizzle_lp);
            let sizzle = self.sizzle_lp * 0.04;

            // 3. Resonant Wood Crackles & Micro-Bursts
            // Trigger new cluster burst
            if self.cluster_left == 0 && self.rng.next_f32() < 0.00012 {
                self.cluster_left = self.rng.gen_range_u8(2, 6);
                self.cluster_gap = 0;
                self.cluster_pan = self.rng.gen_range_f32(-0.6, 0.6);
            }

            // Trigger isolated micro-crackle
            let isolated = self.rng.next_f32() < 0.0004;

            if self.cluster_left > 0 {
                if self.cluster_gap == 0 {
                    let amp = self.rng.gen_range_f32(0.20, 0.55);
                    let freq = self.rng.gen_range_f32(1100.0, 3000.0);
                    let decay = self.rng.gen_range_f32(2.0, 6.0);
                    self.voices[self.voice_idx].trigger(freq, decay, amp, 0.45);
                    self.pans[self.voice_idx] = (self.cluster_pan + self.rng.gen_range_f32(-0.1, 0.1)).clamp(-0.85, 0.85);
                    self.voice_idx = (self.voice_idx + 1) % 4;

                    self.cluster_left -= 1;
                    if self.cluster_left > 0 {
                        self.cluster_gap = self.rng.gen_range_u16(250, 1100);
                    }
                } else {
                    self.cluster_gap -= 1;
                }
            }

            if isolated {
                let amp = self.rng.gen_range_f32(0.06, 0.28);
                let freq = self.rng.gen_range_f32(1400.0, 3800.0);
                let decay = self.rng.gen_range_f32(1.2, 4.0);
                self.voices[self.voice_idx].trigger(freq, decay, amp, 0.55);
                self.pans[self.voice_idx] = self.rng.gen_range_f32(-0.75, 0.75);
                self.voice_idx = (self.voice_idx + 1) % 4;
            }

            // 4. Wood Pops (Trapped gas/resin explosion)
            if self.samples_to_pop == 0 {
                let u = self.rng.next_f32().max(1e-6);
                self.samples_to_pop = (-u.ln() * 3.5 * SAMPLE_RATE as f32) as u32;

                // Deep hollow log thud (130-210 Hz, 35-55 ms)
                let thump_freq = self.rng.gen_range_f32(130.0, 210.0);
                let thump_decay = self.rng.gen_range_f32(35.0, 55.0);
                let thump_amp = self.rng.gen_range_f32(0.35, 0.60);
                self.thump.trigger(thump_freq, thump_decay, thump_amp, 0.1);
                self.thump_pan = self.rng.gen_range_f32(-0.45, 0.45);

                // Simultaneous sharp acoustic fracture crack
                let crack_freq = self.rng.gen_range_f32(900.0, 1800.0);
                self.voices[self.voice_idx].trigger(crack_freq, 8.0, thump_amp * 0.7, 0.5);
                self.pans[self.voice_idx] = self.thump_pan;
                self.voice_idx = (self.voice_idx + 1) % 4;

                self.ember_tail = (SAMPLE_RATE as f32 * self.rng.gen_range_f32(0.06, 0.16)) as u32;
            } else {
                self.samples_to_pop -= 1;
            }

            // Trailing spark flurry after pop
            if self.ember_tail > 0 {
                self.ember_tail -= 1;
                if self.rng.next_f32() < 0.04 {
                    let amp = self.rng.gen_range_f32(0.04, 0.15);
                    let freq = self.rng.gen_range_f32(2200.0, 4500.0);
                    let decay = self.rng.gen_range_f32(1.0, 2.5);
                    self.voices[self.voice_idx].trigger(freq, decay, amp, 0.7);
                    self.pans[self.voice_idx] = (self.thump_pan + self.rng.gen_range_f32(-0.25, 0.25)).clamp(-0.85, 0.85);
                    self.voice_idx = (self.voice_idx + 1) % 4;
                }
            }

            // Sum crackle voices
            let mut crackle_l = 0.0;
            let mut crackle_r = 0.0;
            for v in 0..4 {
                let val = self.voices[v].tick(&mut self.rng);
                let pan = self.pans[v];
                crackle_l += val * (0.5 - 0.5 * pan);
                crackle_r += val * (0.5 + 0.5 * pan);
            }

            let thump_val = self.thump.tick(&mut self.rng);
            let thump_l = thump_val * (0.5 - 0.5 * self.thump_pan);
            let thump_r = thump_val * (0.5 + 0.5 * self.thump_pan);

            let out_l = flame_l + sizzle + crackle_l * 0.70 + thump_l * 0.80;
            let out_r = flame_r + sizzle + crackle_r * 0.70 + thump_r * 0.80;

            // Soft analog saturation
            self.current_left = (out_l * 0.95).tanh();
            self.current_right = (out_r * 0.95).tanh();

            Some(self.current_left)
        } else {
            Some(self.current_right)
        }
    }
}

impl Source for FireplaceSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> NonZero<u16> {
        NonZero::new(2).unwrap()
    }
    fn sample_rate(&self) -> NonZero<u32> {
        NonZero::new(SAMPLE_RATE).unwrap()
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// Procedural Forest / Wind / Birds Sound Generator
/// Gentle wind rustle with occasional melodic bird chirps
pub struct ForestSource {
    rng: FastRng,
    pink: NoiseSource,
    wind_lfo_phase: f32,
    chirp_active: bool,
    chirp_phase: f32,
    chirp_freq: f32,
    chirp_env: f32,
    chirp_duration: u32,
    chirp_sample: u32,
    channel_toggle: bool,
    current_sample: f32,
}

impl ForestSource {
    pub fn new(seed: u32) -> Self {
        Self {
            rng: FastRng::new(seed),
            pink: NoiseSource::new(NoiseKind::Pink, seed + 30),
            wind_lfo_phase: 0.0,
            chirp_active: false,
            chirp_phase: 0.0,
            chirp_freq: 2400.0,
            chirp_env: 0.0,
            chirp_duration: 0,
            chirp_sample: 0,
            channel_toggle: false,
            current_sample: 0.0,
        }
    }
}

impl Iterator for ForestSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.channel_toggle = !self.channel_toggle;
        if self.channel_toggle {
            // Wind swell modulated by slow LFO (~0.1 Hz)
            self.wind_lfo_phase = (self.wind_lfo_phase + 0.12 / SAMPLE_RATE as f32).fract();
            let wind_lfo = ((self.wind_lfo_phase * 2.0 * PI).sin() * 0.5 + 0.5) * 0.6 + 0.4;
            let wind = self.pink.next().unwrap_or(0.0) * wind_lfo * 0.6;

            // Sparse bird chirp trigger (~ every 5-15 seconds)
            if !self.chirp_active && self.rng.next_f32() < 0.00003 {
                self.chirp_active = true;
                self.chirp_sample = 0;
                self.chirp_duration = (SAMPLE_RATE as f32 * (0.15 + self.rng.next_f32() * 0.2)) as u32;
                self.chirp_freq = 2200.0 + self.rng.next_f32() * 800.0;
            }

            let mut chirp = 0.0;
            if self.chirp_active {
                self.chirp_sample += 1;
                let t = self.chirp_sample as f32 / self.chirp_duration as f32;
                if t >= 1.0 {
                    self.chirp_active = false;
                } else {
                    // Swept frequency chirp
                    let instant_freq = self.chirp_freq + (t * 2.0 * PI).sin() * 500.0;
                    self.chirp_phase = (self.chirp_phase + instant_freq / SAMPLE_RATE as f32).fract();
                    self.chirp_env = (t * PI).sin();
                    chirp = (self.chirp_phase * 2.0 * PI).sin() * self.chirp_env * 0.22;
                }
            }

            let out = wind + chirp;
            self.current_sample = out.clamp(-1.0, 1.0);
        }

        Some(self.current_sample)
    }
}

impl Source for ForestSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> NonZero<u16> {
        NonZero::new(2).unwrap()
    }
    fn sample_rate(&self) -> NonZero<u32> {
        NonZero::new(SAMPLE_RATE).unwrap()
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// Procedural Ocean Waves Sound Generator
/// Slow 10-14s periodic swell with cresting foam and surf
pub struct OceanSource {
    pink: NoiseSource,
    brown: NoiseSource,
    swell_phase: f32,
    channel_toggle: bool,
    current_sample: f32,
}

impl OceanSource {
    pub fn new(seed: u32) -> Self {
        Self {
            pink: NoiseSource::new(NoiseKind::Pink, seed + 40),
            brown: NoiseSource::new(NoiseKind::Brown, seed + 50),
            swell_phase: 0.0,
            channel_toggle: false,
            current_sample: 0.0,
        }
    }
}

impl Iterator for OceanSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.channel_toggle = !self.channel_toggle;
        if self.channel_toggle {
            // ~11 second wave period
            self.swell_phase = (self.swell_phase + 0.09 / SAMPLE_RATE as f32).fract();
            let swell = (self.swell_phase * 2.0 * PI).sin().max(0.0).powi(2);

            let deep_water = self.brown.next().unwrap_or(0.0) * (swell * 0.7 + 0.3);
            let foam = self.pink.next().unwrap_or(0.0) * swell.powi(2) * 1.2;

            let out = (deep_water + foam) * 0.7;
            self.current_sample = out.clamp(-1.0, 1.0);
        }

        Some(self.current_sample)
    }
}

impl Source for OceanSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> NonZero<u16> {
        NonZero::new(2).unwrap()
    }
    fn sample_rate(&self) -> NonZero<u32> {
        NonZero::new(SAMPLE_RATE).unwrap()
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// Procedural Cafe Chatter / Ambient Murmur Generator
pub struct CafeSource {
    rng: FastRng,
    pink: NoiseSource,
    filter_mod_phase: f32,
    clink_env: f32,
    clink_freq: f32,
    clink_phase: f32,
    channel_toggle: bool,
    current_sample: f32,
}

impl CafeSource {
    pub fn new(seed: u32) -> Self {
        Self {
            rng: FastRng::new(seed),
            pink: NoiseSource::new(NoiseKind::Pink, seed + 60),
            filter_mod_phase: 0.0,
            clink_env: 0.0,
            clink_freq: 3200.0,
            clink_phase: 0.0,
            channel_toggle: false,
            current_sample: 0.0,
        }
    }
}

impl Iterator for CafeSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.channel_toggle = !self.channel_toggle;
        if self.channel_toggle {
            self.filter_mod_phase = (self.filter_mod_phase + 0.25 / SAMPLE_RATE as f32).fract();
            let mod_amt = (self.filter_mod_phase * 2.0 * PI).sin() * 0.2 + 0.8;
            let murmur = self.pink.next().unwrap_or(0.0) * mod_amt * 0.6;

            // Occasional ceramic cup clink
            if self.clink_env <= 0.001 && self.rng.next_f32() < 0.0001 {
                self.clink_env = 0.3;
                self.clink_freq = 2800.0 + self.rng.next_f32() * 800.0;
            }

            let mut clink = 0.0;
            if self.clink_env > 0.001 {
                self.clink_phase = (self.clink_phase + self.clink_freq / SAMPLE_RATE as f32).fract();
                clink = (self.clink_phase * 2.0 * PI).sin() * self.clink_env;
                self.clink_env *= 0.992;
            }

            let out = murmur + clink;
            self.current_sample = out.clamp(-1.0, 1.0);
        }

        Some(self.current_sample)
    }
}

impl Source for CafeSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> NonZero<u16> {
        NonZero::new(2).unwrap()
    }
    fn sample_rate(&self) -> NonZero<u32> {
        NonZero::new(SAMPLE_RATE).unwrap()
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// Singing Bowl / Zen Bell Chime for timer completion
pub struct BellSource {
    sample_idx: usize,
    total_samples: usize,
    channel_toggle: bool,
    freq: f32,
}

impl BellSource {
    pub fn new(freq: f32, duration_secs: f32) -> Self {
        Self {
            sample_idx: 0,
            total_samples: (SAMPLE_RATE as f32 * duration_secs) as usize,
            channel_toggle: false,
            freq,
        }
    }
}

impl Iterator for BellSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.sample_idx >= self.total_samples {
            return None;
        }

        self.channel_toggle = !self.channel_toggle;
        let t = self.sample_idx as f32 / SAMPLE_RATE as f32;
        if self.channel_toggle {
            self.sample_idx += 1;
        }

        let decay1 = (-t * 1.2).exp();
        let decay2 = (-t * 2.0).exp();
        let decay3 = (-t * 3.5).exp();

        let s1 = (2.0 * PI * self.freq * t).sin() * decay1 * 0.5;
        let s2 = (2.0 * PI * (self.freq * 2.04) * t).sin() * decay2 * 0.25;
        let s3 = (2.0 * PI * (self.freq * 3.12) * t).sin() * decay3 * 0.15;

        Some(s1 + s2 + s3)
    }
}

impl Source for BellSource {
    fn current_span_len(&self) -> Option<usize> {
        Some(self.total_samples.saturating_sub(self.sample_idx))
    }
    fn channels(&self) -> NonZero<u16> {
        NonZero::new(2).unwrap()
    }
    fn sample_rate(&self) -> NonZero<u32> {
        NonZero::new(SAMPLE_RATE).unwrap()
    }
    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(self.total_samples as f32 / SAMPLE_RATE as f32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_rng() {
        let mut rng = FastRng::new(42);
        for _ in 0..100 {
            let val = rng.next_f32();
            assert!((0.0..1.0).contains(&val));
            let s = rng.next_signed();
            assert!((-1.0..1.0).contains(&s));
            let r_u16 = rng.gen_range_u16(5, 10);
            assert!((5..10).contains(&r_u16));
        }
    }

    #[test]
    fn test_binaural_source() {
        let mut src = BinauralSource::new(200.0, 10.0);
        for _ in 0..100 {
            let s = src.next().unwrap();
            assert!(s.abs() <= 1.0);
        }
    }

    #[test]
    fn test_procedural_sources() {
        let mut rain = RainSource::new(10);
        let mut fire = FireplaceSource::new(20);
        let mut ocean = OceanSource::new(30);
        let mut bell = BellSource::new(528.0, 0.5);

        for _ in 0..100 {
            assert!(rain.next().unwrap().abs() <= 1.0);
            assert!(fire.next().unwrap().abs() <= 1.0);
            assert!(ocean.next().unwrap().abs() <= 1.0);
            assert!(bell.next().unwrap().abs() <= 1.5);
        }
    }

    #[test]
    fn test_fireplace_source() {
        let mut fire = FireplaceSource::new(42);
        let mut non_zero_samples = 0;
        let mut min_val = 1.0f32;
        let mut max_val = -1.0f32;

        for _ in 0..20_000 {
            let s = fire.next().unwrap();
            assert!(s.abs() <= 1.0, "Sample out of bounds: {}", s);
            assert!(!s.is_nan(), "Sample is NaN");
            if s < min_val { min_val = s; }
            if s > max_val { max_val = s; }
            if s.abs() > 0.005 {
                non_zero_samples += 1;
            }
        }

        assert!(non_zero_samples > 2_000, "Fireplace produced too few non-zero samples");
        assert!(min_val < -0.01 && max_val > 0.01, "Fireplace has insufficient dynamic range");
    }
}
