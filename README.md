# Zeno 🌿

> A calm, minimalist terminal ambient noise generator and focus companion built with Rust.

Zeno pairs multi-track ambient sound mixing with flexible focus timing (Pomodoro, count-up stopwatch, and countdowns) in a single zero-dependency binary with living ASCII visualizers.

---

## Features

- **Ambient Multi-Track Audio Mixing**:
  - Pre-packaged procedural ambient tracks:
    - Rain / Thunderstorm
    - Fireplace / Campfire crackles
    - Forest / Gentle wind & birds
    - Coffee Shop / Murmur
    - White, Pink, and Brown noise
    - Ocean waves
  - Pure sine wave binaural beats generator (Alpha 10Hz, Beta 14Hz, Gamma 40Hz).
  - Anti-fatigue audio events: randomized distant thunder rolls, fireplace ember sparks, and forest bird chirps.
  - Smooth volume fading and crossfades (no pops or audio clipping).
  - Master volume and one-touch global mute (`m`).
  - Auto-discovery of custom audio files (`.ogg`, `.mp3`, `.flac`, `.wav`) in `~/.config/zeno/sounds/`.

- **Presets System**:
  - Built-in factory scenes:
    - *Cozy Cabin*: Fireplace (60%) + Rain (35%) + Forest (15%) with Campfire visualizer.
    - *Midnight Express*: Brown Noise (50%) + Rain (30%) + White Noise (10%) with Windowpane visualizer.
    - *Rainy Cafe*: Cafe (45%) + Rain (40%) with Rainfall visualizer.
    - *Deep Space*: Brown Noise (70%) + White Noise (10%) + 40Hz Gamma binaural tone with Starfield visualizer.
    - *Zen Garden*: Forest (50%) + Rain (20%) + 10Hz Alpha binaural tone with Zen Pond Ripples visualizer.
    - *Ocean Slumber*: Ocean Waves (65%) + Pink Noise (25%) with Ocean Shoreline visualizer.
  - In-TUI quick preset creation (`S` / `Ctrl+S`): dial in any mix, name it, and save it.
  - Preset browser modal (`p`): navigate, preview, apply, or delete custom presets (`d`).
  - Persistent serialization to `~/.config/zeno/config.toml`.

- **Focus Timers**:
  - **Pomodoro Mode**: 25m Focus / 5m Short Break / 15m Long Break (after 4 cycles).
  - **Stopwatch Mode**: Open-ended count-up timer with lap recording.
  - **Countdown Mode**: Quick-set custom duration timers.
  - Big ASCII digit clock display with interval completion progress bar.
  - Gentle singing bowl / Zen bell chime and system desktop notifications on completion.

- **Living ASCII Visualizers & Atmospheres (`v`)**:
  - *Rainfall*: Falling rain streaks with puddle splash ripples.
  - *Windowpane*: Gentle condensation and trickling rivulets meandering down glass.
  - *Zen Pond Ripples*: Concentric water ripples expanding from droplet impacts on a tranquil pond with floating lotus blossoms.
  - *Ocean Shoreline*: Rhythmic rolling surf and receding tidal wash leaving glistening wet sand.
  - *Campfire*: Cellular automata heat simulation with fiery color gradients and rising sparks.
  - *Fireflies*: Bioluminescent motes drifting peacefully in the dark with smooth organic wandering and glowing auras.
  - *Drifting Leaves*: Autumn breeze with swaying, falling leaves that settle into a gentle ground bed.
  - *Aurora Borealis*: Translucent shimmering polar curtains dancing across starry skies.
  - *Deep Space Starfield*: 3D cosmic starfield drifting and warping through deep space.
  - *Spectrum Wave*: Audio spectrum equalizer reacting in real-time to active track frequencies.
  - *Breathing Mandala*: Concentric geometric focus ring paced to relaxing box-breathing rhythm (Inhale, Hold, Exhale, Rest).
  - *Game of Life*: Conway's cellular automata evolving in slow harmony with decaying ghost trails and auto-seeding.

---

## Keyboard Controls

| Key | Action |
| :--- | :--- |
| `Space` | Start / Pause active timer |
| `Tab` | Switch focus between Timer, Mixer, and Visualizer |
| `r` | Reset active timer / skip to next Pomodoro interval |
| `1` / `2` / `3` | Switch timer mode (`1`: Pomodoro, `2`: Stopwatch, `3`: Countdown) |
| `↑` / `↓` / `j` / `k` | Navigate sound tracks in mixer (or cycle atmosphere when visualizer focused) |
| `←` / `→` / `h` / `l` | Decrease / Increase volume of selected track |
| `Enter` | Toggle selected track ON / OFF (or open atmosphere menu when visualizer focused) |
| `v` | Open Ambient Atmosphere Menu (browse and select from 12 living animations) |
| `p` | Open Preset Browser modal |
| `S` / `Ctrl+S` | Save current mix as a new User Preset |
| `b` | Cycle binaural beats frequency (Alpha 10Hz / Beta 14Hz / Gamma 40Hz) |
| `m` | Mute / Unmute all audio output |
| `+` / `-` | Master volume up / down |
| `?` | Toggle keybindings & help modal |
| `q` / `Esc` | Quit application (or close active modal) |

---

## Command Line Usage

```bash
# Launch default experience
zeno

# Launch with a specific factory or user preset
zeno --preset "Cozy Cabin"
zeno -p "Deep Space"

# Start directly in specific timer modes
zeno --pomodoro
zeno --stopwatch
zeno --countdown 45

# List all available presets
zeno --list-presets
```

---

## Configuration

Settings and custom presets are stored at `~/.config/zeno/config.toml`:

```toml
theme = "CatppuccinMocha"
master_volume = 0.8
muted = false
work_duration_mins = 25
short_break_mins = 5
long_break_mins = 15
auto_advance_pomodoro = false
visualizer = "starfield"

[[presets]]
name = "Late Night Hack"
description = "Rain, gentle fire, and 40Hz focus tone"
visualizer = "window"
binaural_hz = 40.0

[presets.tracks]
rain = 0.60
fireplace = 0.25
brown_noise = 0.15
```
