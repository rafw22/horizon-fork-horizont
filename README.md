# Horizont ⚡

> **High-Performance Multi-Core Rust Minecraft Engine**  
> Maintained by **rafw22**.

---

## 🌟 What is Horizont?

**Horizont** is a community-driven, performance-first fork of the Horizon native Rust Minecraft client. While upstream focuses on strict vanilla parity and a closed compilation pipeline, **Horizont** is built for extreme performance, fast boot times, power-user utilities, native VR/6DoF support, and a modern modding ecosystem.

---

## 🚀 Key Features

### 1. ⚡ Multi-Core Parallel Entity AI (Rayon Engine / RAFW)
- Upstream Horizon ticks all entity brains, navigation, sensors, and collision checks sequentially on a single thread, causing major tick drops when hundreds of mobs are spawned.
- **Horizont** parallelizes entity ticking across all available CPU cores using **Rayon** (Rayon-Accelerated Fanout Worker).
- **Benchmark:** 1,764 skeletons and 200 Withers run smoothly at **45–100+ FPS** with tick times stabilized at **15–19.9 ms** (well within the 50 ms budget), where upstream dropped to 1–5 FPS.

### 2. 🥽 Native 6DoF VR Subsystem & Quest Crossplay (`horizont-vr`)
- **OpenXR 1.0 Runtime Integration:** Full 6DoF head and dual-hand motion controller tracking with low-latency stereoscopic rendering and asymmetric projection frustums.
- **Gesture Combat & Interactions:** Physical weapon swings, punches, and interactive mining with haptic feedback.
- **Multiplayer VR Crossplay:** Synchronizes VR head and hand poses over custom channel `horizont:vr_pose` so desktop players see VR players moving their hands and looking around naturally in real time.
- **Meta Quest Standalone Support:** Optimized Snapdragon XR2 presets, Fixed Foveated Rendering (FFR), and multi-view stereo pipeline.

### 3. 📥 In-Game Modrinth Downloader (`horizont-downloader`)
- Built-in asynchronous content browser and downloader connecting directly to the **Modrinth API v2**.
- Download **Resource Packs**, **Shader Packs**, and **World Maps** directly from the in-game UI with live progress indicators and automatic extraction into `resourcepacks/`, `shaderpacks/`, and `saves/`.

### 4. 🗺️ Toggleable World Downloader (WDL) with 3D Wireframes (`horizont-wdl`)
- Seamlessly captures multiplayer chunk packets and saves them locally into Anvil `.mca` region files (`saves/Downloaded_<Server>/`).
- **Real-Time 3D Wireframe Gizmos:**
  - Visual overlay toggled in game options (`OnlyMissing` or `AllChunks`).
  - Neon-highlighted 3D wireframe bounding columns around un-downloaded chunks ensure 100% complete map dumps with zero missing voids.

### 5. ⚡ 1-Second Instant Boot & World Bypass
- Upstream loads and decodes 144 MB of menu panorama textures and waits for title frames before handoff.
- **Horizont** introduces an optimized instant-boot pipeline:
  - `--last-world`: Automatically finds and jumps straight into your most recently played singleplayer world.
  - `--quickPlaySingleplayer <name>` / `--server <host:port>`: Zero-click direct game entry.
  - Bypasses redundant menu textures, slicing boot time from 6–8 seconds down to **~1 second**.

### 6. 🔌 Native Dynamic Mod Loader (Zero-Cost DLLs)
- **No WASM latency:** WebAssembly introduces heavy FFI marshalling overhead and lacks direct GPU access.
- **No 30 GB recompile cache:** Upstream native mixins require recompiling the entire Rust game engine per modded instance.
- **Horizont Mod Loader:** Drop `.dll` (Windows) or `.so` (Linux) files into `mods/` for instant dynamic loading via `libloading` with zero-cost native FFI.

### 7. 🎨 Ergonomic UI & Misclick Prevention
- Completely redesigned Options menu layout separating gameplay settings from utility actions.
- Horizont-specific settings and Mod menu moved into dedicated, clearly separated button rows to eliminate accidental misclicks.

---

## 💻 Supported Platforms & Architectures

Horizont is cross-platform and natively engineered for modern multi-core hardware:

| Operating System | Architecture | Target Triple | Distribution & Packaging |
| :--- | :--- | :--- | :--- |
| **Windows 10 / 11** | `x86_64` (AMD64 / Intel 64) | `x86_64-pc-windows-msvc` | **Inno Setup Installer** (`.exe`) & **Portable ZIP** |
| **Windows 11 on ARM** | `aarch64` (Qualcomm Snapdragon X Elite / Surface) | `aarch64-pc-windows-msvc` | Native ARM64 Portable ZIP |
| **Arch Linux / Manjaro / EndeavourOS** | `x86_64` & `aarch64` | `x86_64-unknown-linux-gnu` | **PKGBUILD / AUR** (`horizont-bin`, `horizont-git`) |
| **Ubuntu / Debian / Fedora** | `x86_64` & `aarch64` | `x86_64-unknown-linux-gnu` | Standalone Tarball (`.tar.gz`) / AppImage |
| **macOS (Sonoma / Sequoia)** | Apple Silicon (`aarch64` M1–M4) & Intel (`x86_64`) | `aarch64-apple-darwin` / `x86_64-apple-darwin` | `.dmg` / Portable bundle |
| **Meta Quest 2 / 3 / Pro / 3S** | `aarch64` (Snapdragon XR2) | `aarch64-linux-android` | Standalone OpenXR APK |

---

## 📦 Installation & Quick Start

### Windows (Setup Installer or Portable)
1. Download **`Horizont-Setup-v0.1.0-beta.exe`** from the [Releases](https://github.com/rafw22/horizon-fork-horizont/releases) page and run the installer.
   - *Or download `Horizont-v0.1.0-beta-Windows-x64-Portable.zip`, extract anywhere, and run `horizont_client.exe` directly.*
2. Horizont bundles both `horizont_client.exe` and `horizont-launcher.exe` in the same directory for zero-friction launch without external dependency errors.

### Arch Linux (AUR)
```bash
# Using yay or paru
yay -S horizont-bin
# Or manual build from PKGBUILD
cd packaging/arch && makepkg -si
```

### Manual Compilation from Source
```bash
# Clone the repository
git clone https://github.com/rafw22/horizon-fork-horizont.git
cd horizon-fork-horizont

# Build release binaries (Client & Prepare)
cargo build --release -p horizon-client -p horizon-prepare

# Build launcher
cargo build --release --manifest-path launcher/Cargo.toml
```

---

## 📜 License & Attribution
Horizont is licensed under the **MIT License**, consistent with upstream Horizon.  
Minecraft is a trademark of Mojang Synergies AB. Horizont is an independent open-source project.
