//! # Horizont World Downloader (WDL) & Chunk Visualizer
//!
//! Provides automated world dumping for multiplayer servers, coupled with an
//! in-game 3D chunk grid overlay highlighting downloaded vs missing chunks.
//!
//! ## Core Features:
//! 1. **Background Chunk Interception:**
//!    - Intercepts incoming chunk packets (`LevelChunkWithLight`) as the player travels.
//!    - Tracks all downloaded chunk coordinates `(cx, cz)`.
//! 2. **3D Real-Time Missing Chunk Highlighting:**
//!    - Renders a 3D bounding box / column around all chunks in render distance
//!      that the server has NOT yet sent (or that have not yet been stored).
//!    - Customizable highlight colors (default: bright neon crimson `0xFFFF_0033`).
//!    - Eliminates the frustration of missing chunks or holes in downloaded worlds!

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// A 3D line segment to be rendered by the engine's gizmo line pipeline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WdlLine {
    pub start: [f64; 3],
    pub end: [f64; 3],
    pub color: u32,
    pub width: f32,
}

/// Global World Downloader singleton.
pub static WDL: OnceLock<WorldDownloader> = OnceLock::new();

/// State and configuration of the World Downloader.
pub struct WorldDownloader {
    /// Whether world downloading is currently recording chunks.
    pub active: Mutex<bool>,
    /// Whether the 3D in-game chunk highlight overlay is enabled.
    pub highlight_enabled: Mutex<bool>,
    /// ARGB color for chunks that have NOT been downloaded yet (default: neon red).
    pub color_missing: Mutex<u32>,
    /// ARGB color for chunks that HAVE been downloaded (default: faint green).
    pub color_downloaded: Mutex<u32>,
    /// Set of chunk coordinates (chunk_x, chunk_z) saved in the current session.
    pub downloaded_chunks: Mutex<HashSet<(i32, i32)>>,
    /// Destination folder for saved region files (e.g. `saves/Downloaded_Server/`).
    pub target_folder: Mutex<Option<PathBuf>>,
}

impl Default for WorldDownloader {
    fn default() -> Self {
        Self {
            active: Mutex::new(true),
            highlight_enabled: Mutex::new(true),
            // Default bright neon red (0xFFFF0033) for missing chunks so they stand out clearly in the world
            color_missing: Mutex::new(0xFFFF_0033),
            // Default faint emerald green (0x4000FF66) for confirmed downloaded chunks
            color_downloaded: Mutex::new(0x4000_FF66),
            downloaded_chunks: Mutex::new(HashSet::new()),
            target_folder: Mutex::new(None),
        }
    }
}

impl WorldDownloader {
    /// Initialize WDL session targeting a specific world save directory.
    pub fn start_session(&self, save_name: &str) {
        let folder = Path::new("saves").join(format!("WDL_{}", save_name));
        let _ = std::fs::create_dir_all(&folder);
        if let Ok(mut target) = self.target_folder.lock() {
            *target = Some(folder.clone());
        }
        if let Ok(mut active) = self.active.lock() {
            *active = true;
        }
        log::info!("Horizont WDL: started recording session into {:?}", folder);
    }

    /// Mark a chunk (chunk_x, chunk_z) as received and saved.
    pub fn on_chunk_received(&self, cx: i32, cz: i32) {
        if let Ok(mut set) = self.downloaded_chunks.lock() {
            set.insert((cx, cz));
        }
    }

    /// Check if a chunk (chunk_x, chunk_z) is recorded as downloaded.
    pub fn is_downloaded(&self, cx: i32, cz: i32) -> bool {
        self.downloaded_chunks
            .lock()
            .map(|set| set.contains(&(cx, cz)))
            .unwrap_or(false)
    }

    /// Total count of unique chunks captured in this session.
    pub fn downloaded_count(&self) -> usize {
        self.downloaded_chunks
            .lock()
            .map(|set| set.len())
            .unwrap_or(0)
    }

    /// Toggle the in-game 3D highlight overlay on/off.
    pub fn toggle_highlight(&self) -> bool {
        if let Ok(mut enabled) = self.highlight_enabled.lock() {
            *enabled = !*enabled;
            *enabled
        } else {
            false
        }
    }

    /// Set a custom highlight color for missing (un-downloaded) chunks.
    pub fn set_missing_color(&self, argb: u32) {
        if let Ok(mut color) = self.color_missing.lock() {
            *color = argb;
        }
    }

    /// Generates 3D line gizmos for all chunks in view distance.
    /// Missing chunks are outlined in `color_missing`; downloaded ones in `color_downloaded`.
    pub fn generate_gizmo_lines(
        &self,
        player_x: f64,
        player_z: f64,
        view_distance: i32,
        min_y: i32,
        max_y: i32,
    ) -> Vec<WdlLine> {
        let highlight = self
            .highlight_enabled
            .lock()
            .map(|h| *h)
            .unwrap_or(false);
        if !highlight {
            return Vec::new();
        }

        let missing_color = self
            .color_missing
            .lock()
            .map(|c| *c)
            .unwrap_or(0xFFFF_0033);
        let downloaded_color = self
            .color_downloaded
            .lock()
            .map(|c| *c)
            .unwrap_or(0x4000_FF66);

        let center_cx = (player_x / 16.0).floor() as i32;
        let center_cz = (player_z / 16.0).floor() as i32;

        let radius = view_distance.clamp(2, 16);
        let y0 = min_y as f64;
        let y1 = (max_y + 1) as f64;

        let mut lines = Vec::new();
        let downloaded = self.downloaded_chunks.lock().ok();

        for cx in (center_cx - radius)..=(center_cx + radius) {
            for cz in (center_cz - radius)..=(center_cz + radius) {
                let is_saved = downloaded
                    .as_ref()
                    .is_some_and(|set| set.contains(&(cx, cz)));

                // If downloaded, we can optionally draw subtle borders; if missing, draw bold alert box!
                let (color, width, draw_box) = if is_saved {
                    (downloaded_color, 1.0, false) // Subtle or skipped
                } else {
                    (missing_color, 3.0, true) // Prominent 3D box for missing chunks
                };

                let x0 = (cx * 16) as f64;
                let z0 = (cz * 16) as f64;
                let x1 = x0 + 16.0;
                let z1 = z0 + 16.0;

                if draw_box {
                    // 4 vertical corner columns
                    lines.push(WdlLine { start: [x0, y0, z0], end: [x0, y1, z0], color, width });
                    lines.push(WdlLine { start: [x1, y0, z0], end: [x1, y1, z0], color, width });
                    lines.push(WdlLine { start: [x0, y0, z1], end: [x0, y1, z1], color, width });
                    lines.push(WdlLine { start: [x1, y0, z1], end: [x1, y1, z1], color, width });

                    // Top square ring
                    lines.push(WdlLine { start: [x0, y1, z0], end: [x1, y1, z0], color, width });
                    lines.push(WdlLine { start: [x1, y1, z0], end: [x1, y1, z1], color, width });
                    lines.push(WdlLine { start: [x1, y1, z1], end: [x0, y1, z1], color, width });
                    lines.push(WdlLine { start: [x0, y1, z1], end: [x0, y1, z0], color, width });

                    // Bottom square ring
                    lines.push(WdlLine { start: [x0, y0, z0], end: [x1, y0, z0], color, width });
                    lines.push(WdlLine { start: [x1, y0, z0], end: [x1, y0, z1], color, width });
                    lines.push(WdlLine { start: [x1, y0, z1], end: [x0, y0, z1], color, width });
                    lines.push(WdlLine { start: [x0, y0, z1], end: [x0, y0, z0], color, width });
                }
            }
        }

        lines
    }
}

/// Global WDL instance.
pub fn get() -> &'static WorldDownloader {
    WDL.get_or_init(WorldDownloader::default)
}
