# Horizont Documentation

**Horizont** is a high-performance, multi-core Rust Minecraft engine and fork of Horizon. These pages provide technical documentation for developers, modders, and server owners.

---

## ⚡ Key Horizont Innovations

- **Multi-Core Rayon Ticking:** Parallel mob AI and entity simulation eliminating single-core bottlenecks.
- **1-Second Fast-Boot:** Menu panorama bypass and direct world jumping (`--last-world`, `--quickPlaySingleplayer`, `--server`).
- **World Downloader (WDL):** In-game chunk downloader with real-time 3D highlighted wireframes of missing chunks.
- **Native Dynamic Mod Loader:** Drop compiled `.dll` / `.so` plugins into `mods/` with zero rebuild delay.

---

## Build and Run

- [Architecture](architecture.md): where each part of the code lives, how `Game` is organised.
- [Build from source](../README.md#build-from-source): your own copy of the game, resource preparation, `cargo run`.
- [Native client](native-client.md): options, controls, the dedicated server, `horizon-ctl` scripting, packaging, architecture.
- [Host a server](hosting.md): install `horizon-server`, prepare game files, `server.toml`, web panel, Docker.
- [The anticheat](anticheat.md): dedicated server checks (violation levels, setbacks, alerts).
- [The chat filter](chat-filter.md): dedicated server chat filter and moderation.
- [Hosting a server: design](server.md): server design, backups, services, Docker.
- [For server owners](owners.md): `server.properties` key mapping to `server.toml`.
- [Graphics](graphics.md): graphics settings screen, shader support, performance benchmarks.
- [Benchmark](benchmark.md): engine performance against vanilla 26.3 and Java performance mods.
- [Mods](mods.md): content mods (`.hmod`), native dynamic plugins (`.dll`/`.so`), and hooks.
- [Resource packs](resource-packs.md): resource pack support and shader packs.

---

## Architecture: World Generation & Simulation

- [World generation](world-generation.md): noise pipeline to features.
- [World height](world-height.md): dimension types and tall levels.
- [Pregeneration](pregeneration.md): `/pregen` command and lighting pipeline.
- [World data](world-data.md): chunk storage, palettes, region files.
- [Block-state codecs](block-state-codecs.md), [scheduled ticks](scheduled-ticks.md).
- [Collision geometry](collision-geometry.md): shapes, collision, movement, and fluids.
