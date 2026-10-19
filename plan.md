# Zeno: Project Plan & Architecture

> A calm, minimalist terminal ambient noise generator and focus companion built with Rust.

---

## 1. Project Overview

**Zeno** is a cozy, low-distraction Terminal User Interface (TUI) application designed for deep work and relaxation. It pairs multi-track ambient sound mixing with flexible focus timing (Pomodoro, count-up stopwatch, and countdowns) in a single zero-dependency binary.

### Key Goals
- **Instant & Lightweight**: Zero external C-library runtime requirements; instant startup with embedded soundscapes.
- **Mixable Ambience & Procedural Sound**: Multi-track audio engine with curated presets, procedural anti-fatigue audio events, and pure tone binaural synthesis.
- **First-Class User Presets**: Easily craft custom mixes in the TUI, name them, and save them for instant recall.
- **Flexible Timers**: Switch effortlessly between structured Pomodoro intervals and open-ended count-up sessions.
- **Zen Visual Flair**: Warm, muted color palettes (Catppuccin, Nord, Gruvbox), big-text clocks, living ASCII particle effects (rain, fire), and audio spectrum visualizers.

---

## 2. Technology Stack

| Layer | Library / Tool | Rationale |
| :--- | :--- | :--- |
| **Language** | **Rust (2021 Edition)** | Performance, memory safety, zero-cost abstractions, single binary distribution. |
| **TUI Rendering** | [`ratatui`](https://crates.io/crates/ratatui) | Modern, actively maintained terminal UI library with rich widgets, layout primitives, and styling. |
| **Terminal Backend** | [`crossterm`](https://crates.io/crates/crossterm) | Cross-platform terminal manipulation (Linux, macOS, Windows) for raw mode and mouse/keyboard events. |
| **Audio Engine** | [`kira`](https://crates.io/crates/kira) or [`rodio`](https://crates.io/crates/rodio) | Pure Rust audio mixing, streaming, tweening/crossfading, and seamless gapless looping. |
| **Async / Events** | [`tokio`](https://crates.io/crates/tokio) or [`crossbeam-channel`](https://crates.io/crates/crossbeam-channel) | Decoupled event loop handling UI ticks, keyboard events, and audio controller commands. |
| **Configuration** | [`serde`](https://crates.io/crates/serde) + [`toml`](https://crates.io/crates/toml) | User preferences and custom presets saved to `~/.config/zeno/config.toml`. |
| **Desktop Alerts** | [`notify-rust`](https://crates.io/crates/notify-rust) | Native system notifications when Pomodoro intervals complete (optional). |

---

## 3. Core Features

### 3.1. Ambient Soundscape Mixer & Audio Enhancements
- **Pre-packaged Ambient Tracks** (embedded via `include_bytes!`):
  - Rain / Thunderstorm
  - Fireplace / Campfire crackle
  - Forest / Birds
  - Coffee Shop / Murmur
  - White, Pink, and Brown noise
  - Ocean waves
- **Multi-Track Mixing**:
  - Independent volume sliders per sound track (`0%` to `100%`).
  - Master volume control & quick mute toggle (`m`).
  - Smooth volume fading / tweening when toggling or adjusting levels (no harsh pops or clipping).

### 3.2. Preset Scenes & User-Created Presets
- **Built-in Factory Presets**:
  - *Cozy Cabin:* 60% Fireplace + 35% Rain + 15% Soft Wind.
  - *Midnight Express:* 50% Train rhythmic click-clack + 30% Night Rain.
  - *Rainy Cafe:* 45% Cafe chatter + 40% Rain on window.
  - *Deep Space:* 70% Brown noise + 30% Low synth drone.
- **First-Class User Preset Creation & Management**:
  - **In-TUI Quick Save (`S` / `Ctrl+S`)**: When you dial in a sound mix you love, hit `S` to open a quick modal: `Save Mix as Preset: [ Enter name... ]`. Saves active tracks, volume levels, binaural setting, and visualizer preference.
  - **Preset Browser Modal (`p`)**: Interactive popup list separating Built-in and User presets. Browse with preview on highlight, apply with `Enter`, or delete user presets with `d`.
  - **Config File Portability**: Presets are serialized to `~/.config/zeno/config.toml`, allowing users to export, share, or edit them in their favorite text editor:
    ```toml
    [[presets]]
    name = "Late Night Hack"
    description = "Rain, gentle fire, and 40Hz focus tone"
    visualizer = "rain"
    binaural_hz = 40.0
    [presets.tracks]
    rain = 0.60
    fireplace = 0.25
    brown_noise = 0.15
    ```

### 3.3. Procedural Audio & Synthesis
- **Procedural Anti-Fatigue Audio Events**:
  - Sparse, randomized audio triggers overlaid on loops to prevent ear fatigue:
    - Distant thunder rolls every 2–4 minutes during rain sessions.
    - Wood crackle and spark pops in fireplace sessions.
    - Sparse birds chirping in forest sessions.
- **Binaural Beats & Frequency Synthesis**:
  - Pure sine wave synthesis for brainwave entrainment:
    - *Alpha Waves (10 Hz):* Relaxation, clarity, calm reflection.
    - *Beta/Gamma Waves (14–40 Hz):* Intense problem solving and deep focus.
- **Custom Sounds & Streaming**:
  - Auto-discovery of local `.ogg`, `.mp3`, and `.flac` files in `~/.config/zeno/sounds/`.
  - Optional support for streaming curated internet radio (e.g., Lofi / SomaFM).

### 3.4. Focus Timers
- **Pomodoro Mode**:
  - Work Session (default: 25 min).
  - Short Break (default: 5 min).
  - Long Break (default: 15 min, after 4 cycles).
  - Auto-advance toggle or manual progression.
- **Count-Up / Stopwatch Mode**:
  - Open-ended focus session tracking elapsed time without deadline pressure.
  - Pause, resume, and lap/reset at will.
- **Custom Countdown Mode**:
  - Quick set timers (e.g. 15m, 45m, 60m).
- **Completion Cues**:
  - Gentle audio bell / singing bowl chime upon timer completion.
  - Optional desktop notification.

### 3.5. TUI & Visual Flair
- **Layout Panels**:
  - **Timer View**: Prominent ASCII big-text clock displaying time remaining/elapsed, current mode, and progress bar.
  - **Sound Mixer Panel**: List of soundscapes with toggle status (`[ON] / [OFF]`) and interactive level meters.
  - **Living ASCII Ambient Visualizer Panel**:
    - *Rain Particle Effect:* Falling rain streaks with subtle puddle splashes.
    - *Fireplace Flame Effect:* Dynamic cellular-automata / particle flame flickering.
    - *Night Sky / Snow:* Soft drifting snow or gentle twinkling stars.
  - **ASCII Audio Spectrum / Waveform Equalizer**:
    - Animated frequency bars or oscillating sine waveforms reacting dynamically to active audio playback.
  - **Footer**: Keybinding hints, active preset badge, and mute status.
- **Color Themes**:
  - Presets: Catppuccin (Mocha & Macchiato), Nord, Gruvbox Dark, Tokyo Night, Monokai.

---

## 4. Software Architecture

```
zeno/
├── Cargo.toml
├── assets/
│   ├── sounds/           # Embedded ambient audio files (.ogg/.mp3)
│   └── icons/
├── src/
│   ├── main.rs           # Entry point, terminal setup, panic hook cleanup
│   ├── app.rs            # Central App state & lifecycle
│   ├── event.rs          # Cross-thread event handler (Input, Tick, AudioEvent)
│   ├── ui/               # Ratatui rendering logic
│   │   ├── mod.rs        # Main layout splitter
│   │   ├── timer.rs      # Big text digits & progress gauges
│   │   ├── mixer.rs      # Sound list, volume sliders, track status
│   │   ├── presets_dialog.rs # Preset browser & save modal dialogs
│   │   ├── visual/       # Living visual effects & animations
│   │   │   ├── mod.rs    # Visual panel coordinator
│   │   │   ├── rain.rs   # ASCII rain particle simulator
│   │   │   ├── fire.rs   # ASCII flame / ember simulator
│   │   │   └── wave.rs   # Audio spectrum / waveform visualizer
│   │   ├── theme.rs      # Color schemes and styling definitions
│   │   └── widgets/      # Custom reusable Ratatui components
│   ├── timer/            # Timer domain logic
│   │   ├── mod.rs        # Timer state machine & mode dispatcher
│   │   ├── pomodoro.rs   # Pomodoro session state & transitions
│   │   └── stopwatch.rs  # Count-up stopwatch logic
│   ├── audio/            # Audio subsystem
│   │   ├── mod.rs        # Public AudioController interface
│   │   ├── engine.rs     # Kira/Rodio manager & output stream
│   │   ├── track.rs      # Audio track metadata, looping, and volume
│   │   ├── presets.rs    # Factory & user preset data structures
│   │   ├── synth.rs      # Pure-tone binaural & procedural event generator
│   │   └── loader.rs     # Asset loading (embedded + disk discovery)
│   └── config.rs         # TOML config file serialization, schema & paths
```

### Event Flow Architecture
```
  [User Keyboard / Mouse] ──┐
                            ├─► [Event Channel] ─► [App::update()]
  [Animation Tick (60 FPS)] ┤                             │
                            │                             ├─► [AudioController (Commands)]
  [Timer Tick (250ms)]   ───┤                             ├─► [ConfigManager (Save Presets)]
                            │                             └─► [Ratatui Frame (Draw)]
  [Audio Stream State]   ───┘
```

---

## 5. Implementation Roadmap

### Phase 1: Foundation & Project Scaffolding
- [ ] Initialize Cargo workspace and configure dependencies (`ratatui`, `crossterm`, `tokio`/`crossbeam`, `serde`).
- [ ] Set up terminal initialization, raw mode, alternate screen, and clean panic hooks.
- [ ] Implement central event loop (`Input`, `Tick`, `Quit`).
- [ ] Create basic skeleton TUI layout with placeholder panels.

### Phase 2: Timer Domain Engine & Display
- [ ] Implement `TimerMode` state machine (Pomodoro, Count-Up, Countdown).
- [ ] Add big-digit ASCII font rendering for the clock display.
- [ ] Implement timer controls: Start (`Space`), Pause, Reset (`r`), Mode Switch (`Tab`).
- [ ] Add progress bar gauges showing current interval completion percentage.

### Phase 3: Audio Engine & Multi-Track Mixing
- [ ] Integrate audio backend (`kira` or `rodio`) with sound streaming.
- [ ] Build `TrackManager` capable of playing multiple simultaneous looping tracks.
- [ ] Implement volume attenuation and smooth crossfades/tweens.
- [ ] Embed initial set of high-quality ambient sound loops (Rain, Fireplace, White Noise).
- [ ] Build pure sine wave binaural beats generator (`synth.rs`).

### Phase 4: Mixer TUI, Presets & Procedural Audio
- [ ] Render interactive sound list with active toggles and volume sliders.
- [ ] Support keyboard navigation (`Up`/`Down` to navigate tracks, `Left`/`Right` to adjust volume, `Enter` to toggle).
- [ ] Implement preset scene browser modal (`p`) for factory presets.
- [ ] Implement in-TUI "Save Current Mix" dialog (`S`) to create new user presets with custom names.
- [ ] Allow deleting or overwriting user presets in the preset modal (`d`).
- [ ] Add procedural sparse audio events (random crackles, thunder, bird chirps).
- [ ] Master volume and global mute controls (`m`).

### Phase 5: Living Visuals, Themes & Animations
- [ ] Implement color themes (Catppuccin, Nord, Gruvbox) switchable via hotkey or config.
- [ ] Build ASCII particle visualizers (rain simulator and fireplace embers).
- [ ] Build dynamic audio spectrum / equalizer wave widget.
- [ ] Desktop notifications on timer intervals (`notify-rust`).
- [ ] Audio bell cue on timer completion.

### Phase 6: Persistence & Custom Sound Loading
- [ ] Implement `~/.config/zeno/config.toml` serialization for settings and `[[presets]]`.
- [ ] Support custom audio file discovery from `~/.config/zeno/sounds/`.
- [ ] CLI arguments support (e.g., `zeno --preset "Late Night Hack"`, `zeno --pomodoro`).

---

## 6. Default Keybindings

| Key | Action |
| :--- | :--- |
| `Space` | Start / Pause active timer |
| `Tab` | Switch focus between Timer / Mixer / Visuals |
| `r` | Reset active timer / Next Pomodoro phase |
| `1` / `2` / `3` | Switch modes (1: Pomodoro, 2: Stopwatch, 3: Countdown) |
| `↑` / `↓` / `j` / `k` | Navigate sound tracks |
| `←` / `→` / `h` / `l` | Decrease / Increase volume of selected track |
| `Enter` | Toggle selected track ON / OFF |
| `p` | Open Preset browser modal (select, preview, apply, or delete) |
| `S` / `Ctrl+S` | Save current mix as a new User Preset (opens naming prompt) |
| `v` | Cycle ambient visualizer (Rain / Fire / Wave / Off) |
| `b` | Toggle binaural beats layer |
| `m` | Mute / Unmute all audio |
| `+` / `-` | Master volume up / down |
| `t` | Cycle color theme |
| `?` | Toggle help / keybindings overlay |
| `q` / `Esc` | Quit application (or close open modal) |
