# How Horizont is built

Who builds Horizont, how changes are checked, and why it is built for extreme performance.

## Who builds it

**Horizont** is an independent, performance-first fork of the Rust Minecraft client, maintained by **rafw22**. While upstream Horizon restricts itself to strict single-threaded vanilla parity, Horizont prioritizes real-world multi-core performance (Rayon parallel entity ticking), 1-second instant boot times, native dynamic DLL modding, and power-user utilities like the built-in World Downloader (WDL).

## Philosophy: Performance & Modding First

Horizont does not compromise gameplay smoothness for dogmatic byte-for-byte single-threaded parity. Where upstream forces all entity brains, navigation, sensors, and collisions through a single CPU core, Horizont parallelizes workloads across all available CPU threads with Rayon, ensuring 60-144+ FPS even with thousands of active mobs.

## How changes are checked

Every change is verified against high performance standards and stability:

- **Harnesses & Benchmarks:** Verifies world generation, mob behaviors, redstone, and protocol packet compatibility without artificial single-thread bottlenecks.
- **Continuous Integration:** Formatting, lints, crate builds, and dynamic modloader validations.
- **Native Modding Ecosystem:** Supports dynamic `.dll` / `.so` plugins dropped straight into `mods/` without 30 GB rebuild caches.

## Why it's safe to run

- **Open source.** All of the code is public on [GitHub](https://github.com/rafw22/horizon-fork-horizont), under the MIT licence.
- **Built from public code.** Clean standalone repository with no bloat or external tracker scripts.
- **Direct launch.** Zero-telemetry by default, zero bloat, instant 1-second startup.

## What it doesn't contain

- **No Minecraft assets or closed files.** Horizont contains no copyrighted Minecraft code, assets, or jars. The engine builds game assets cleanly on your computer from your own copy of Minecraft: Java Edition 26.3.
