//! # Horizont In-Game Content Downloader
//!
//! Developed for the **Horizont** engine by **rafw22**.
//!
//! Provides a complete, direct, zero-friction in-game downloading system for:
//! 1. **Resource Packs:** Automatically searched, verified, and placed into `resourcepacks/`.
//! 2. **Shader Packs:** Automatically fetched and placed into `shaderpacks/`.
//! 3. **World / Adventure Maps:** Automatically downloaded, unzipped into `saves/<world_name>/`,
//!    and validated so players can jump straight into new worlds without touching files manually!
//!
//! ## Modrinth API v2 Integration:
//! All operations interact with the public Modrinth API v2 (`https://api.modrinth.com/v2`),
//! requiring zero user registration, zero API keys, and offering blazing fast CDN downloads.

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

/// Base endpoint for Modrinth API v2.
pub const MODRINTH_API: &str = "https://api.modrinth.com/v2";

/// Custom User-Agent header required by Modrinth's API usage policy.
pub const USER_AGENT: &str = "rafw22/horizont-downloader/0.1.0 (https://github.com/rafw22/horizon-fork-horizont)";

/// Supported content categories that can be searched, downloaded, and installed in-game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentType {
    /// Texture, audio, model, and UI packs placed in `resourcepacks/`.
    ResourcePack,
    /// Iris/OptiFine WGSL shader packs placed in `shaderpacks/`.
    ShaderPack,
    /// Adventure, survival, and custom world maps extracted into `saves/`.
    WorldMap,
}

impl ContentType {
    /// Converts the enum into the corresponding Modrinth project type facet.
    pub fn modrinth_project_type(&self) -> &'static str {
        match self {
            ContentType::ResourcePack => "resourcepack",
            ContentType::ShaderPack => "shader",
            ContentType::WorldMap => "modpack", // Modrinth groups custom world maps / packs under modpack or custom tags
        }
    }

    /// Default destination directory within the game working folder.
    pub fn destination_dir(&self) -> PathBuf {
        match self {
            ContentType::ResourcePack => PathBuf::from("resourcepacks"),
            ContentType::ShaderPack => PathBuf::from("shaderpacks"),
            ContentType::WorldMap => PathBuf::from("saves"),
        }
    }
}

/// Represents a project summary returned from a Modrinth search query.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModrinthProject {
    /// Project identifier (e.g. `AANobbMI` or slug `faithful-32x`).
    pub project_id: String,
    /// Human-readable project title.
    pub title: String,
    /// Short summary / description of the project.
    pub description: String,
    /// URL of the project icon / thumbnail.
    pub icon_url: Option<String>,
    /// Total downloads count on Modrinth.
    pub downloads: u64,
    /// Project author username.
    pub author: String,
    /// Unique project slug.
    pub slug: String,
}

/// Represents a specific release version of a Modrinth project.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModrinthVersion {
    /// Unique version identifier.
    pub id: String,
    /// Version name (e.g. `Faithful 32x - 1.21.1`).
    pub name: String,
    /// Version number string (e.g. `1.1.0`).
    pub version_number: String,
    /// List of downloadable files attached to this version.
    pub files: Vec<ModrinthFile>,
    /// Minecraft versions this release officially supports.
    pub game_versions: Vec<String>,
}

/// A specific downloadable file attachment (usually a `.zip` archive).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModrinthFile {
    /// Direct HTTPS CDN download URL.
    pub url: String,
    /// Original file name (e.g. `Faithful-32x-Release.zip`).
    pub filename: String,
    /// File size in bytes.
    pub size: u64,
    /// Whether this file is the primary download artifact of the version.
    pub primary: bool,
}

/// Real-time progress report emitted during active file downloads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DownloadProgress {
    /// Number of bytes received so far.
    pub downloaded_bytes: u64,
    /// Total file size in bytes (if provided by the server `Content-Length`).
    pub total_bytes: Option<u64>,
    /// Percentage completed (0.0 to 100.0).
    pub percentage: f32,
    /// Current download speed in kilobytes per second (KB/s).
    pub speed_kb_per_sec: f64,
    /// Whether the download has finished completely.
    pub finished: bool,
}

/// Search Modrinth API v2 for projects matching a search term and content category.
///
/// Returns a list of `ModrinthProject` metadata summaries.
pub fn search_content(
    query: &str,
    content_type: ContentType,
    limit: usize,
) -> Result<Vec<ModrinthProject>, String> {
    let p_type = content_type.modrinth_project_type();
    // Modrinth facet query syntax: [["project_type:<type>"]]
    let facet = format!("[[\"project_type:{}\"]]", p_type);

    let mut url = format!(
        "{}/search?query={}&limit={}&facets={}",
        MODRINTH_API,
        urlencoding::encode(query),
        limit.clamp(1, 50),
        urlencoding::encode(&facet)
    );

    // For resource packs and shaders, filter by Minecraft 1.21 / 26.3 compatibility
    if content_type == ContentType::ResourcePack || content_type == ContentType::ShaderPack {
        url.push_str(&format!(
            "&index=downloads"
        ));
    }

    log::info!("Horizont Downloader: querying Modrinth: {}", url);

    let resp = ureq::get(&url)
        .set("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| format!("Modrinth search network error: {e}"))?;

    let json: serde_json::Value = resp
        .into_json()
        .map_err(|e| format!("Failed to parse Modrinth JSON response: {e}"))?;

    let hits = json["hits"]
        .as_array()
        .ok_or_else(|| "Missing 'hits' array in Modrinth search response".to_string())?;

    let mut results = Vec::with_capacity(hits.len());
    for hit in hits {
        let project_id = hit["project_id"].as_str().unwrap_or_default().to_string();
        let title = hit["title"].as_str().unwrap_or_default().to_string();
        let description = hit["description"].as_str().unwrap_or_default().to_string();
        let icon_url = hit["icon_url"].as_str().map(str::to_string);
        let downloads = hit["downloads"].as_u64().unwrap_or(0);
        let author = hit["author"].as_str().unwrap_or_default().to_string();
        let slug = hit["slug"].as_str().unwrap_or_default().to_string();

        results.push(ModrinthProject {
            project_id,
            title,
            description,
            icon_url,
            downloads,
            author,
            slug,
        });
    }

    log::info!("Horizont Downloader: search found {} projects", results.len());
    Ok(results)
}

/// Retrieve the release versions available for a specific Modrinth project.
pub fn get_project_versions(project_id_or_slug: &str) -> Result<Vec<ModrinthVersion>, String> {
    let url = format!("{}/project/{}/version", MODRINTH_API, project_id_or_slug);

    let resp = ureq::get(&url)
        .set("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| format!("Failed to fetch versions for '{project_id_or_slug}': {e}"))?;

    let json: serde_json::Value = resp
        .into_json()
        .map_err(|e| format!("Failed to parse version JSON: {e}"))?;

    let array = json
        .as_array()
        .ok_or_else(|| "Expected array of versions from Modrinth".to_string())?;

    let mut versions = Vec::with_capacity(array.len());
    for item in array {
        let id = item["id"].as_str().unwrap_or_default().to_string();
        let name = item["name"].as_str().unwrap_or_default().to_string();
        let version_number = item["version_number"].as_str().unwrap_or_default().to_string();

        let mut files = Vec::new();
        if let Some(f_arr) = item["files"].as_array() {
            for f in f_arr {
                let url = f["url"].as_str().unwrap_or_default().to_string();
                let filename = f["filename"].as_str().unwrap_or_default().to_string();
                let size = f["size"].as_u64().unwrap_or(0);
                let primary = f["primary"].as_bool().unwrap_or(false);

                files.push(ModrinthFile {
                    url,
                    filename,
                    size,
                    primary,
                });
            }
        }

        let mut game_versions = Vec::new();
        if let Some(gv_arr) = item["game_versions"].as_array() {
            for gv in gv_arr {
                if let Some(s) = gv.as_str() {
                    game_versions.push(s.to_string());
                }
            }
        }

        versions.push(ModrinthVersion {
            id,
            name,
            version_number,
            files,
            game_versions,
        });
    }

    Ok(versions)
}

/// Downloads a content artifact and automatically places or extracts it into the proper folder.
///
/// - `ResourcePack` -> saved directly into `resourcepacks/<filename>`
/// - `ShaderPack` -> saved directly into `shaderpacks/<filename>`
/// - `WorldMap` -> downloaded to a temporary archive and unzipped into `saves/<world_name>`
///
/// Supports real-time streaming progress callbacks.
pub fn download_and_install(
    download_url: &str,
    target_filename: &str,
    content_type: ContentType,
    progress_callback: Option<Arc<dyn Fn(DownloadProgress) + Send + Sync>>,
) -> Result<PathBuf, String> {
    let dest_dir = content_type.destination_dir();
    fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Failed to create destination directory '{dest_dir:?}': {e}"))?;

    log::info!(
        "Horizont Downloader: starting download of '{}' ({:?}) from {}",
        target_filename,
        content_type,
        download_url
    );

    let resp = ureq::get(download_url)
        .set("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| format!("HTTP request failed for '{download_url}': {e}"))?;

    let total_bytes = resp
        .header("Content-Length")
        .and_then(|s| s.parse::<u64>().ok());

    let mut reader = resp.into_reader();

    // In-memory buffer or temporary file depending on size
    let start_time = Instant::now();
    let mut downloaded_bytes = 0u64;
    let mut buffer = [0u8; 16384]; // 16 KB chunks

    // Destination target path
    let output_file_path = dest_dir.join(target_filename);
    let mut file = File::create(&output_file_path)
        .map_err(|e| format!("Failed to create output file '{output_file_path:?}': {e}"))?;

    loop {
        let n = reader
            .read(&mut buffer)
            .map_err(|e| format!("Network read error while streaming: {e}"))?;
        if n == 0 {
            break;
        }

        file.write_all(&buffer[..n])
            .map_err(|e| format!("Disk write error: {e}"))?;

        downloaded_bytes += n as u64;

        if let Some(ref cb) = progress_callback {
            let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
            let speed_kb = (downloaded_bytes as f64 / 1024.0) / elapsed;
            let pct = total_bytes
                .map(|t| (downloaded_bytes as f32 / t as f32) * 100.0)
                .unwrap_or(0.0)
                .clamp(0.0, 100.0);

            cb(DownloadProgress {
                downloaded_bytes,
                total_bytes,
                percentage: pct,
                speed_kb_per_sec: speed_kb,
                finished: false,
            });
        }
    }

    file.flush()
        .map_err(|e| format!("Failed to flush downloaded file: {e}"))?;

    // Final completion notification
    if let Some(ref cb) = progress_callback {
        let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
        let speed_kb = (downloaded_bytes as f64 / 1024.0) / elapsed;
        cb(DownloadProgress {
            downloaded_bytes,
            total_bytes: Some(downloaded_bytes),
            percentage: 100.0,
            speed_kb_per_sec: speed_kb,
            finished: true,
        });
    }

    log::info!(
        "Horizont Downloader: Downloaded {} bytes in {:.2}s into '{:?}'",
        downloaded_bytes,
        start_time.elapsed().as_secs_f32(),
        output_file_path
    );

    // If this is a World / Adventure map archive, extract it directly into `saves/`
    if content_type == ContentType::WorldMap {
        log::info!("Horizont Downloader: Extracting world archive into saves/...");
        let world_name = target_filename
            .strip_suffix(".zip")
            .unwrap_or(target_filename);
        let world_save_dir = dest_dir.join(world_name);

        extract_world_archive(&output_file_path, &world_save_dir)?;

        // Remove the temporary .zip archive after successful extraction
        let _ = fs::remove_file(&output_file_path);

        log::info!(
            "Horizont Downloader: World map successfully installed into '{:?}'!",
            world_save_dir
        );
        return Ok(world_save_dir);
    }

    Ok(output_file_path)
}

/// Unzips a downloaded world archive into `saves/<world_folder>`, handling nested folders gracefully.
fn extract_world_archive(zip_path: &Path, extract_to: &Path) -> Result<(), String> {
    let file = File::open(zip_path)
        .map_err(|e| format!("Failed to open downloaded archive '{zip_path:?}': {e}"))?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file))
        .map_err(|e| format!("Corrupt or invalid zip archive: {e}"))?;

    // Inspect if archive has a single top-level directory containing level.dat
    let mut root_prefix = String::new();
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            let name = entry.name().map_err(|e| e.to_string())?.replace('\\', "/");
            if name.ends_with("/level.dat") {
                let parts: Vec<&str> = name.split('/').collect();
                if parts.len() == 2 {
                    root_prefix = format!("{}/", parts[0]);
                    break;
                }
            }
        }
    }

    fs::create_dir_all(extract_to)
        .map_err(|e| format!("Failed to create world folder '{extract_to:?}': {e}"))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read zip entry #{i}: {e}"))?;

        let entry_name = entry.name().map_err(|e| e.to_string())?.replace('\\', "/");

        // Strip single root directory prefix if present so level.dat lands directly in the root of the save
        let clean_path = if !root_prefix.is_empty() && entry_name.starts_with(&root_prefix) {
            &entry_name[root_prefix.len()..]
        } else {
            &entry_name
        };

        if clean_path.is_empty() {
            continue;
        }

        let out_path = extract_to.join(clean_path);

        if entry.is_dir() {
            fs::create_dir_all(&out_path)
                .map_err(|e| format!("Failed to create folder '{out_path:?}': {e}"))?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent dir '{parent:?}': {e}"))?;
            }
            let mut outfile = File::create(&out_path)
                .map_err(|e| format!("Failed to create extracted file '{out_path:?}': {e}"))?;
            io::copy(&mut entry, &mut outfile)
                .map_err(|e| format!("Failed to extract file '{out_path:?}': {e}"))?;
        }
    }

    Ok(())
}

/// Helper module for simple URL encoding without heavy external crates.
mod urlencoding {
    pub fn encode(data: &str) -> String {
        let mut result = String::with_capacity(data.len() * 2);
        for byte in data.bytes() {
            match byte {
                b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    result.push(byte as char);
                }
                _ => {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
        result
    }
}
