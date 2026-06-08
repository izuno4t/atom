use std::fs;
use std::io::{self, Read};
use std::path::Path;

use zip::ZipArchive;

use crate::{AstNode, ConversionOptions};

pub(super) fn unzip_part(path: &Path, part: &str) -> io::Result<String> {
    let file = fs::File::open(path)?;
    let mut archive = ZipArchive::new(file).map_err(zip_error)?;
    unzip_part_from_archive(&mut archive, part)
}

pub(super) fn open_zip_archive(path: &Path) -> io::Result<ZipArchive<fs::File>> {
    let file = fs::File::open(path)?;
    ZipArchive::new(file).map_err(zip_error)
}

pub(super) fn unzip_part_from_archive(
    archive: &mut ZipArchive<fs::File>,
    part: &str,
) -> io::Result<String> {
    let mut file = archive.by_name(part).map_err(zip_error)?;
    let mut output = String::new();
    file.read_to_string(&mut output)?;
    Ok(output)
}

fn unzip_part_bytes(path: &Path, part: &str) -> io::Result<Vec<u8>> {
    let file = fs::File::open(path)?;
    let mut archive = ZipArchive::new(file).map_err(zip_error)?;
    let mut file = archive.by_name(part).map_err(zip_error)?;
    let mut output = Vec::new();
    file.read_to_end(&mut output)?;
    Ok(output)
}

pub(super) fn materialize_ooxml_media(
    package_path: &Path,
    options: &ConversionOptions,
    ast: &mut [AstNode],
    warnings: &mut Vec<String>,
) -> io::Result<()> {
    if options.inline_base64_media {
        inline_ooxml_media(package_path, ast, warnings);
    } else if let Some(media_dir) = &options.extract_media {
        extract_and_rewrite_ooxml_media(package_path, media_dir, ast, warnings)?;
    }
    Ok(())
}

fn extract_and_rewrite_ooxml_media(
    package_path: &Path,
    output_dir: &Path,
    ast: &mut [AstNode],
    warnings: &mut Vec<String>,
) -> io::Result<()> {
    fs::create_dir_all(output_dir)?;
    rewrite_media_paths(ast, &mut |media_path| {
        let Some(file_name) = media_file_name(media_path) else {
            return media_path.to_string();
        };
        let package_part = ooxml_media_package_part(media_path);
        match unzip_part_bytes(package_path, &package_part) {
            Ok(bytes) => {
                if let Err(error) = fs::write(output_dir.join(file_name), bytes) {
                    warnings.push(format!(
                        "failed to write media {media_path} to {}: {error}",
                        output_dir.display()
                    ));
                }
            }
            Err(error) => warnings.push(format!(
                "failed to extract media {media_path} from {package_part}: {error}"
            )),
        }
        extracted_media_reference(output_dir, file_name)
    });
    Ok(())
}

fn inline_ooxml_media(package_path: &Path, ast: &mut [AstNode], warnings: &mut Vec<String>) {
    rewrite_media_paths(ast, &mut |media_path| {
        let package_part = ooxml_media_package_part(media_path);
        match unzip_part_bytes(package_path, &package_part) {
            Ok(bytes) => {
                let mime = media_mime_type(media_path);
                format!("data:{mime};base64,{}", encode_base64(&bytes))
            }
            Err(error) => {
                warnings.push(format!(
                    "failed to inline media {media_path} from {package_part}: {error}"
                ));
                media_path.to_string()
            }
        }
    });
}

fn rewrite_media_paths(ast: &mut [AstNode], rewrite: &mut dyn FnMut(&str) -> String) {
    for node in ast {
        match node {
            AstNode::Image { path, .. } => *path = rewrite(path),
            AstNode::Table { rows } => {
                for row in rows {
                    for cell in &mut row.cells {
                        if let Some(image) = &mut cell.image {
                            *image = rewrite(image);
                        }
                    }
                }
            }
            AstNode::List { items, .. } => {
                for item in items {
                    rewrite_media_paths(item, rewrite);
                }
            }
            _ => {}
        }
    }
}

fn media_file_name(media_path: &str) -> Option<&str> {
    media_path
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
}

fn extracted_media_reference(output_dir: &Path, file_name: &str) -> String {
    output_dir
        .file_name()
        .and_then(|name| name.to_str())
        .map(|directory| format!("{directory}/{file_name}"))
        .unwrap_or_else(|| file_name.to_string())
}

fn media_mime_type(media_path: &str) -> &'static str {
    match Path::new(media_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("tif" | "tiff") => "image/tiff",
        _ => "image/png",
    }
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = *chunk.get(1).unwrap_or(&0);
        let third = *chunk.get(2).unwrap_or(&0);
        encoded.push(TABLE[(first >> 2) as usize] as char);
        encoded.push(TABLE[(((first & 0b0000_0011) << 4) | (second >> 4)) as usize] as char);
        if chunk.len() > 1 {
            encoded.push(TABLE[(((second & 0b0000_1111) << 2) | (third >> 6)) as usize] as char);
        } else {
            encoded.push('=');
        }
        if chunk.len() > 2 {
            encoded.push(TABLE[(third & 0b0011_1111) as usize] as char);
        } else {
            encoded.push('=');
        }
    }
    encoded
}

fn ooxml_media_package_part(media_path: &str) -> String {
    let normalized = media_path.trim_start_matches('/');
    if normalized.starts_with("word/media/") {
        normalized.to_string()
    } else if let Some(relative) = normalized.strip_prefix("media/") {
        format!("word/media/{relative}")
    } else {
        format!("word/media/{normalized}")
    }
}

fn zip_error(error: zip::result::ZipError) -> io::Error {
    io::Error::other(error.to_string())
}

pub(super) fn read_numbered_parts_from_archive(
    archive: &mut ZipArchive<fs::File>,
    prefix: &str,
    suffix: &str,
    max: usize,
) -> Vec<String> {
    let mut parts = Vec::new();
    for index in 1..=max {
        let part = format!("{prefix}{index}{suffix}");
        let Ok(content) = unzip_part_from_archive(archive, &part) else {
            continue;
        };
        if !content.trim().is_empty() {
            parts.push(content);
        }
    }
    parts
}
