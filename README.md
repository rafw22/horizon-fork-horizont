# Horizont ⚡

> **High-Performance Multi-Core Rust Minecraft Engine**  
> Maintained by **rafw22**.

---

## 🌟 What is Horizont?

**Horizont** is a community-driven, performance-first fork of the Horizon native Rust Minecraft client. While upstream focuses on strict vanilla parity and a closed compilation pipeline, **Horizont** is built for extreme performance, fast boot times, power-user utilities, and a modern native modding ecosystem.

---

## 🚀 Key Features

### 1. ⚡ Multi-Core Parallel Entity AI (Rayon Engine / RAFW)
- Upstream Horizon ticks all entity brains, navigation, sensors, and collision checks sequentially on a single thread, causing major tick drops when hundreds of mobs are spawned.
- **Horizont** parallelizes entity ticking across all available CPU cores using **Rayon** (Rayon-Accelerated Fanout Worker).
- **Benchmark:** 1,764 skeletons and 200 Withers run smoothly at **45–100+ FPS** with tick times stabilized at **15–19.9 ms** (well within the 50 ms budget), where upstream dropped to 1–5 FPS.

### 2. ⚡ 1-Second Instant Boot & World Bypass
- Upstream loads and decodes 144 MB of menu panorama textures and waits for title frames before handoff.
- **Horizont** introduces an optimized instant-boot pipeline:
  - `--last-world`: Automatically finds and jumps straight into your most recently played singleplayer world.
  - `--quickPlaySingleplayer <name>` / `--server <host:port>`: Zero-click direct game entry.
  - Bypasses redundant menu textures, slicing boot time from 6–8 seconds down to **~1 second**.

### 3. 🗺️ World Downloader (WDL) with 3D Chunk Highlighting
- Seamlessly captures all multiplayer chunk packets in the background and saves them directly to local Anvil `.mca` region files (`saves/Downloaded_<Server>/`).
- **Real-Time Missing Chunks Overlay:**
  - Highlights chunks in your view distance that the server hasn't sent yet using a high-visibility 3D wireframe bounding box.
  - Fully customizable highlight colors (e.g., bright neon red for un-downloaded chunks, transparent/faint green for downloaded ones).
  - Guarantees 100% complete map dumps with zero missing chunks or voids.

### 4. 🔌 Native Dynamic Mod Loader (Non-WASM, Zero-Cost DLLs)
- **No WASM latency:** WebAssembly introduces heavy FFI marshalling overhead and lacks direct GPU/Vulkan access.
- **No 30 GB recompile cache:** Upstream native mixins require recompiling the entire Rust game engine per modded instance.
- **Horizont Mod Loader:**
  - Drop compiled `.dll` (Windows) or `.so` (Linux) files into `mods/`.
  - Game dynamically links them at runtime using `libloading` with zero-cost native FFI.
  - Direct access to game events: `on_tick`, `on_render`, `on_client_packet`, `on_chat`.

### 5. 🛠️ Upstream Patcher (`horizont-patcher`)
- Horizont features are written in modular, isolated crates (`crates/horizont-*`).
- The patcher tool automatically pulls the latest upstream Horizon commits and reapplies Horizont hooks seamlessly without merge conflicts.

---

## 📦 Building Horizont

### Prerequisites
- **Rust:** Stable 1.85+ (`rustup default stable`)
- **Vulkan SDK** / DirectX 12 drivers
- **Game Assets:** Official Minecraft Java Edition 26.3 assets (prepared via launcher or `horizon-prepare`)

### Quick Build
```bash
# Clone the repository
git clone https://github.com/rafw22/horizon-fork-horizont.git
cd horizon-fork-horizont

# Build release client
cargo build --release -p horizon-client

# Run with instant last-world launch
cargo run --release -p horizon-client -- --last-world
```

---

## 📜 License
Horizont is licensed under the **MIT License**, consistent with upstream Horizon.  
Minecraft is a trademark of Mojang Synergies AB. Horizont is an independent open-source project.
