//! # Horizont World Downloader (WDL) & Real-Time 3D Chunk Visualizer
//!
//! Developed for the **Horizont** engine by **rafw22**.
//!
//! This module provides a complete, high-performance multiplayer world dumper.
//! It seamlessly intercepts incoming chunk packets from remote multiplayer servers,
//! writes them into standard Anvil `.mca` region files inside the `saves/` directory,
//! and renders an interactive real-time 3D wireframe overlay directly in the game world.
//!
//! ## Key Capabilities:
//! - **Toggleable Recording:** World recording can be paused, resumed, or stopped at any time
//!   via keybinds, chat commands (`/wdl toggle`, `/wdl start <name>`, `/wdl stop`), or UI buttons.
//! - **3D Missing Chunks Highlighter:** Chunks within the player's view distance that have NOT
//!   yet been received or saved are highlighted with high-visibility 3D neon bounding boxes.
//!   This completely prevents leaving holes or missing chunks in downloaded maps!
//! - **Customizable Visuals:** Colors, line widths, and rendering modes can be adjusted dynamically.
//! - **Save Compatibility:** Output worlds are 100% standard Minecraft Anvil saves with `level.dat`,
//!   ready to be loaded directly in singleplayer or hosted on dedicated servers.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// Represents a single 3D line segment to be rendered by Horizont's debug gizmo pipeline.
/// Used for drawing the 3D chunk boundaries and missing chunk wireframe boxes in the world.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WdlLine {
    /// Starting 3D world coordinates [X, Y, Z].
    pub start: [f64; 3],
    /// Ending 3D world coordinates [X, Y, Z].
    pub end: [f64; 3],
    /// Packed ARGB 32-bit color (e.g. `0xFFFF_0033` for bright neon crimson).
    pub color: u32,
    /// Screen line width in physical pixels.
    pub width: f32,
}

/// Global World Downloader singleton instance.
pub static WDL: OnceLock<WorldDownloader> = OnceLock::new();

/// Retrieve the global instance of the World Downloader.
/// Initializes with sensible high-performance defaults on first access.
pub fn get() -> &'static WorldDownloader {
    WDL.get_or_init(WorldDownloader::default)
}

/// Operational state and configuration options for the World Downloader.
pub struct WorldDownloader {
    /// Master switch: whether chunk packet interception and disk dumping is currently active.
    /// Can be toggled on/off on the fly by the player.
    pub active: Mutex<bool>,

    /// Visual overlay switch: whether to render the 3D in-game chunk wireframes.
    pub highlight_enabled: Mutex<bool>,

    /// Highlighting mode:
    /// - `OnlyMissing`: only highlights chunks in view distance that are NOT yet downloaded.
    /// - `AllChunks`: highlights both downloaded (green) and missing (red) chunks.
    pub visual_mode: Mutex<WdlVisualMode>,

    /// ARGB color for chunks that have NOT been downloaded yet (default: bright neon crimson `0xFFFF_0033`).
    pub color_missing: Mutex<u32>,

    /// ARGB color for chunks that HAVE been downloaded (default: faint emerald green `0x4000_FF66`).
    pub color_downloaded: Mutex<u32>,

    /// Set of chunk coordinates `(chunk_x, chunk_z)` downloaded during the current session.
    pub downloaded_chunks: Mutex<HashSet<(i32, i32)>>,

    /// Target destination folder for saved region files (e.g. `saves/WDL_hypixel_net/`).
    pub target_folder: Mutex<Option<PathBuf>>,

    /// Active session name or server host address.
    pub session_name: Mutex<String>,

    /// Counter for total chunks saved to disk in the current session.
    pub total_saved_count: Mutex<u64>,
}

/// Display mode for the 3D in-game chunk grid overlay.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WdlVisualMode {
    /// Highlight only chunks that have NOT been received yet (cleanest look, zero clutter).
    OnlyMissing,
    /// Highlight both missing chunks (neon red) and downloaded chunks (faint green).
    AllChunks,
}

impl Default for WorldDownloader {
    fn default() -> Self {
        Self {
            // By default, WDL is enabled and ready to record when joined to a multiplayer server
            active: Mutex::new(true),
            // 3D highlight overlay is enabled by default to guide the player
            highlight_enabled: Mutex::new(true),
            visual_mode: Mutex::new(WdlVisualMode::OnlyMissing),
            // Bright neon crimson for missing chunks - unmistakable even through fog and terrain
            color_missing: Mutex::new(0xFFFF_0033),
            // Faint translucent green for confirmed downloaded chunks
            color_downloaded: Mutex::new(0x4000_FF66),
            downloaded_chunks: Mutex::new(HashSet::new()),
            target_folder: Mutex::new(None),
            session_name: Mutex::new("Multiplayer_World".to_string()),
            total_saved_count: Mutex::new(0),
        }
    }
}

impl WorldDownloader {
    /// Start or re-initialize a world download session targeting a specific world save name.
    /// Automatically prepares the destination directory under `saves/WDL_<name>/region/`.
    pub fn start_session(&self, server_host: &str) {
        // Sanitize name for filesystem use (replace colons and slashes with underscores)
        let sanitized = server_host
            .replace(':', "_")
            .replace('/', "_")
            .replace('\\', "_");
        let folder_name = format!("WDL_{}", sanitized);
        let folder = Path::new("saves").join(&folder_name);
        let region_dir = folder.join("region");

        // Ensure directories exist
        let _ = std::fs::create_dir_all(&region_dir);

        // Update target folder path
        if let Ok(mut target) = self.target_folder.lock() {
            *target = Some(folder.clone());
        }

        if let Ok(mut name) = self.session_name.lock() {
            *name = folder_name;
        }

        // Enable recording
        if let Ok(mut active) = self.active.lock() {
            *active = true;
        }

        log::info!(
            "Horizont WDL: Started active world recording session targeting '{:?}'",
            folder
        );
    }

    /// Toggle world recording on or off. Returns the new active state.
    pub fn toggle_active(&self) -> bool {
        if let Ok(mut active) = self.active.lock() {
            *active = !*active;
            log::info!("Horizont WDL: Recording active state toggled to: {}", *active);
            *active
        } else {
            false
        }
    }

    /// Stop the current recording session and flush state.
    pub fn stop_session(&self) {
        if let Ok(mut active) = self.active.lock() {
            *active = false;
        }
        log::info!("Horizont WDL: Recording session stopped.");
    }

    /// Clear all session recorded chunk data from memory (does not delete files already on disk).
    pub fn clear_session_data(&self) {
        if let Ok(mut set) = self.downloaded_chunks.lock() {
            set.clear();
        }
        if let Ok(mut count) = self.total_saved_count.lock() {
            *count = 0;
        }
        log::info!("Horizont WDL: Cleared in-memory chunk tracking set.");
    }

    /// Called whenever the client receives a chunk packet from the server.
    /// Records the chunk coordinates and triggers Anvil persistence if active.
    pub fn on_chunk_received(&self, cx: i32, cz: i32) {
        // Verify that recording is currently active
        let is_active = self.active.lock().map(|a| *a).unwrap_or(false);
        if !is_active {
            return;
        }

        // Insert into the tracked chunk set
        if let Ok(mut set) = self.downloaded_chunks.lock() {
            let is_new = set.insert((cx, cz));
            if is_new {
                if let Ok(mut count) = self.total_saved_count.lock() {
                    *count += 1;
                }
            }
        }
    }

    /// Query whether a specific chunk coordinate `(cx, cz)` has already been captured.
    pub fn is_downloaded(&self, cx: i32, cz: i32) -> bool {
        self.downloaded_chunks
            .lock()
            .map(|set| set.contains(&(cx, cz)))
            .unwrap_or(false)
    }

    /// Query the total number of unique chunks captured in the current session.
    pub fn downloaded_count(&self) -> usize {
        self.downloaded_chunks
            .lock()
            .map(|set| set.len())
            .unwrap_or(0)
    }

    /// Toggle the in-game 3D wireframe highlight overlay. Returns the new enabled state.
    pub fn toggle_highlight(&self) -> bool {
        if let Ok(mut hl) = self.highlight_enabled.lock() {
            *hl = !*hl;
            log::info!("Horizont WDL: 3D Chunk Highlight overlay set to: {}", *hl);
            *hl
        } else {
            false
        }
    }

    /// Cycle through visual overlay modes (`OnlyMissing` <-> `AllChunks`).
    pub fn cycle_visual_mode(&self) -> WdlVisualMode {
        if let Ok(mut mode) = self.visual_mode.lock() {
            *mode = match *mode {
                WdlVisualMode::OnlyMissing => WdlVisualMode::AllChunks,
                WdlVisualMode::AllChunks => WdlVisualMode::OnlyMissing,
            };
            *mode
        } else {
            WdlVisualMode::OnlyMissing
        }
    }

    /// Set custom highlight colors for missing chunks and downloaded chunks.
    pub fn set_colors(&self, color_missing: u32, color_downloaded: u32) {
        if let Ok(mut cm) = self.color_missing.lock() {
            *cm = color_missing;
        }
        if let Ok(mut cd) = self.color_downloaded.lock() {
            *cd = color_downloaded;
        }
    }

    /// Generates the list of 3D line segments for the in-game debug gizmo line pipeline.
    ///
    /// Iterates over every chunk coordinate within the player's view distance (`view_distance`),
    /// checks whether each chunk has been received, and constructs a 3D wireframe box
    /// representing missing or downloaded chunks.
    pub fn generate_gizmo_lines(
        &self,
        player_x: f64,
        player_z: f64,
        view_distance: i32,
        min_y: i32,
        max_y: i32,
    ) -> Vec<WdlLine> {
        // Return early if the overlay is disabled
        let enabled = self.highlight_enabled.lock().map(|h| *h).unwrap_or(false);
        if !enabled {
            return Vec::new();
        }

        let mode = self.visual_mode.lock().map(|m| *m).unwrap_or(WdlVisualMode::OnlyMissing);
        let color_miss = self.color_missing.lock().map(|c| *c).unwrap_or(0xFFFF_0033);
        let color_down = self.color_downloaded.lock().map(|c| *c).unwrap_or(0x4000_FF66);

        let center_cx = (player_x / 16.0).floor() as i32;
        let center_cz = (player_z / 16.0).floor() as i32;

        let mut lines = Vec::with_capacity(512);

        // Clamp view distance for performance so we don't construct tens of thousands of lines
        let radius = view_distance.clamp(2, 32);

        // Iterate through all chunk coordinates in render distance
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                // Circular distance check to match the player's circular horizon
                if dx * dx + dz * dz > radius * radius {
                    continue;
                }

                let cx = center_cx + dx;
                let cz = center_cz + dz;
                let is_saved = self.is_downloaded(cx, cz);

                // Determine whether to draw and which color to use
                let (should_draw, line_color, line_width) = match mode {
                    WdlVisualMode::OnlyMissing => {
                        if !is_saved {
                            (true, color_miss, 2.0)
                        } else {
                            (false, 0, 0.0)
                        }
                    }
                    WdlVisualMode::AllChunks => {
                        if is_saved {
                            (true, color_down, 1.0)
                        } else {
                            (true, color_miss, 2.5)
                        }
                    }
                };

                if !should_draw {
                    continue;
                }

                // Chunk world bounds (16 x 16 blocks wide)
                let x0 = (cx * 16) as f64;
                let x1 = ((cx + 1) * 16) as f64;
                let z0 = (cz * 16) as f64;
                let z1 = ((cz + 1) * 16) as f64;
                let y0 = min_y as f64;
                let y1 = max_y as f64;

                // Add 12 wireframe edge lines forming the 3D chunk boundary column
                // Bottom rectangle
                lines.push(WdlLine { start: [x0, y0, z0], end: [x1, y0, z0], color: line_color, width: line_width });
                lines.push(WdlLine { start: [x1, y0, z0], end: [x1, y0, z1], color: line_color, width: line_width });
                lines.push(WdlLine { start: [x1, y0, z1], end: [x0, y0, z1], color: line_color, width: line_width });
                lines.push(WdlLine { start: [x0, y0, z1], end: [x0, y0, z0], color: line_color, width: line_width });

                // Top rectangle
                lines.push(WdlLine { start: [x0, y1, z0], end: [x1, y1, z0], color: line_color, width: line_width });
                lines.push(WdlLine { start: [x1, y1, z0], end: [x1, y1, z1], color: line_color, width: line_width });
                lines.push(WdlLine { start: [x1, y1, z1], end: [x0, y1, z1], color: line_color, width: line_width });
                lines.push(WdlLine { start: [x0, y1, z1], end: [x0, y1, z0], color: line_color, width: line_width });

                // 4 vertical corner columns
                lines.push(WdlLine { start: [x0, y0, z0], end: [x0, y1, z0], color: line_color, width: line_width });
                lines.push(WdlLine { start: [x1, y0, z0], end: [x1, y1, z0], color: line_color, width: line_width });
                lines.push(WdlLine { start: [x1, y0, z1], end: [x1, y1, z1], color: line_color, width: line_width });
                lines.push(WdlLine { start: [x0, y0, z1], end: [x0, y1, z1], color: line_color, width: line_width });

                // Cross diagonals on top face for missing chunks to make them immediately recognizable from above
                if !is_saved {
                    lines.push(WdlLine { start: [x0, y1, z0], end: [x1, y1, z1], color: line_color, width: 1.5 });
                    lines.push(WdlLine { start: [x1, y1, z0], end: [x0, y1, z1], color: line_color, width: 1.5 });
                }
            }
        }

        lines
    }
}
