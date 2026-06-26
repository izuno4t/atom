use crate::OcrEngine;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::http;

pub trait OcrBackend {
    fn recognize(&self, input: &Path) -> io::Result<String>;
}

pub fn recognize_with(backend: &dyn OcrBackend, input: &Path) -> io::Result<String> {
    backend.recognize(input)
}

// ocr-rs(PP-OCRv5 mobile, MNN) の既定モデルの取得元。未設定のまま OCR を有効に
// できるよう、モデルが無ければ初回にここからダウンロードしてキャッシュする。
// 出典: zibo-chen/rust-paddle-ocr (Apache-2.0)。
const MODEL_BASE_URL: &str =
    "https://raw.githubusercontent.com/zibo-chen/rust-paddle-ocr/main/models";
const DET_FILE: &str = "PP-OCRv5_mobile_det_fp16.mnn";
const REC_FILE: &str = "PP-OCRv5_mobile_rec_fp16.mnn";
const CHARSET_FILE: &str = "ppocr_keys_v5.txt";

pub struct SubprocessOcrBackend {
    pub engine: OcrEngine,
}

pub struct OcrRsBackend {
    det_model_path: PathBuf,
    rec_model_path: PathBuf,
    charset_path: PathBuf,
}

impl OcrBackend for SubprocessOcrBackend {
    fn recognize(&self, input: &Path) -> io::Result<String> {
        run_subprocess(&self.engine, input)
    }
}

impl OcrBackend for OcrRsBackend {
    fn recognize(&self, input: &Path) -> io::Result<String> {
        let image = image::open(input).map_err(io::Error::other)?;
        // ocr-rs が内部で使う MNN は初期化時にネイティブ側から stdout へ能力バナーを
        // 出す。atom の Markdown 出力(stdout)に混ざらないよう、OCR 実行中だけ stdout を
        // 退避する。失敗(エラー return)時も RAII で確実に元へ戻す。
        let results = {
            let _silencer = StdoutSilencer::new();
            let engine = ocr_rs::OcrEngine::new(
                &self.det_model_path,
                &self.rec_model_path,
                &self.charset_path,
                None,
            )
            .map_err(io::Error::other)?;
            engine.recognize(&image).map_err(io::Error::other)?
        };
        Ok(results
            .into_iter()
            .map(|result| result.text)
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

/// OCR 実行中だけ stdout(fd 1)を `/dev/null` へ退避する RAII ガード。MNN が
/// ネイティブ側から stdout へ出すバナーが atom の出力に混ざるのを防ぐ。
/// Unix 以外では何もしない。
#[cfg(unix)]
struct StdoutSilencer {
    saved_fd: Option<i32>,
}

#[cfg(unix)]
impl StdoutSilencer {
    fn new() -> Self {
        // SAFETY: 標準的な dup/dup2 による fd 退避。途中で失敗したら退避せず素通しする。
        unsafe {
            let saved = libc::dup(libc::STDOUT_FILENO);
            if saved < 0 {
                return Self { saved_fd: None };
            }
            let dev_null = libc::open(c"/dev/null".as_ptr(), libc::O_WRONLY);
            if dev_null < 0 {
                libc::close(saved);
                return Self { saved_fd: None };
            }
            libc::dup2(dev_null, libc::STDOUT_FILENO);
            libc::close(dev_null);
            Self {
                saved_fd: Some(saved),
            }
        }
    }
}

#[cfg(unix)]
impl Drop for StdoutSilencer {
    fn drop(&mut self) {
        if let Some(saved) = self.saved_fd {
            // SAFETY: new() で確保した退避 fd を stdout へ戻し、後始末する。
            // ネイティブ stdio のバッファを先に流してから差し替える。
            unsafe {
                libc::fflush(std::ptr::null_mut());
                libc::dup2(saved, libc::STDOUT_FILENO);
                libc::close(saved);
            }
        }
    }
}

#[cfg(not(unix))]
struct StdoutSilencer;

#[cfg(not(unix))]
impl StdoutSilencer {
    fn new() -> Self {
        Self
    }
}

impl OcrRsBackend {
    /// 環境変数 `ATOM_OCR_RS_*` でモデルパスが与えられていればそれを使う。
    /// いずれも未設定なら、キャッシュ済みモデルを使う(無ければ初回ダウンロード)。
    /// 一部だけ設定した場合は不整合を避けるため3点すべてを必須にする。
    pub fn from_env_or_download() -> io::Result<Self> {
        let env_keys = [
            "ATOM_OCR_RS_DET_MODEL",
            "ATOM_OCR_RS_REC_MODEL",
            "ATOM_OCR_RS_CHARSET",
        ];
        if env_keys.iter().any(|key| std::env::var_os(key).is_some()) {
            return Ok(Self {
                det_model_path: required_env_path(env_keys[0])?,
                rec_model_path: required_env_path(env_keys[1])?,
                charset_path: required_env_path(env_keys[2])?,
            });
        }

        let dir = models_cache_dir()?;
        std::fs::create_dir_all(&dir)?;
        Ok(Self {
            det_model_path: ensure_cached_model(&dir, DET_FILE)?,
            rec_model_path: ensure_cached_model(&dir, REC_FILE)?,
            charset_path: ensure_cached_model(&dir, CHARSET_FILE)?,
        })
    }
}

/// ocr-rs モデルのキャッシュ先 (`$ATOM_HOME/models/ocr-rs` または
/// `$HOME/.atom/models/ocr-rs`)。
fn models_cache_dir() -> io::Result<PathBuf> {
    let home = std::env::var_os("ATOM_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".atom")));
    home.map(|home| home.join("models").join("ocr-rs"))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "cannot determine ATOM_HOME or HOME for the model cache",
            )
        })
}

/// キャッシュに無ければダウンロードし、モデルファイルのパスを返す。
fn ensure_cached_model(dir: &Path, file: &str) -> io::Result<PathBuf> {
    let dest = dir.join(file);
    if dest.exists() && std::fs::metadata(&dest)?.len() > 0 {
        return Ok(dest);
    }
    download_to(&format!("{MODEL_BASE_URL}/{file}"), &dest)?;
    Ok(dest)
}

/// URL から取得して dest へ原子的に書き込む。プロキシ・独自CAは
/// [`super::http`] 経由で環境変数から読まれる。
fn download_to(url: &str, dest: &Path) -> io::Result<()> {
    let agent = http::agent(Some(Duration::from_secs(300)));
    let mut response = agent
        .get(url)
        .call()
        .map_err(|error| io::Error::other(format!("failed to download {url}: {error}")))?;
    let bytes = response
        .body_mut()
        .with_config()
        .limit(64 * 1024 * 1024)
        .read_to_vec()
        .map_err(|error| io::Error::other(format!("failed to read {url}: {error}")))?;
    let tmp = dest.with_extension("download");
    std::fs::write(&tmp, &bytes)?;
    std::fs::rename(&tmp, dest)?;
    Ok(())
}

pub fn backend_for_engine(engine: &OcrEngine) -> io::Result<Option<Box<dyn OcrBackend>>> {
    match engine {
        OcrEngine::None => Ok(None),
        OcrEngine::Auto | OcrEngine::OcrRs => {
            Ok(Some(Box::new(OcrRsBackend::from_env_or_download()?)))
        }
        _ => Ok(Some(Box::new(SubprocessOcrBackend {
            engine: engine.clone(),
        }))),
    }
}

pub fn command_for_engine(engine: &OcrEngine) -> Option<&str> {
    match engine {
        OcrEngine::NdlOcrLite => Some("ndlocr-lite"),
        OcrEngine::NdlKoten => Some("ndl-koten-ocr"),
        OcrEngine::Tesseract => Some("tesseract"),
        OcrEngine::Surya => Some("surya_ocr"),
        OcrEngine::External(command) => Some(command),
        OcrEngine::Auto | OcrEngine::OcrRs | OcrEngine::None => None,
    }
}

pub fn run_subprocess(engine: &OcrEngine, input: &Path) -> io::Result<String> {
    let Some(command) = command_for_engine(engine) else {
        return Ok(String::new());
    };
    let output = Command::new(command).arg(input).output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn recognize_pdf_pages(
    bytes: &[u8],
    page_indices: &[usize],
    backend: &dyn OcrBackend,
) -> io::Result<Vec<(usize, String)>> {
    let pdf = hayro::hayro_syntax::Pdf::new(bytes.to_vec())
        .map_err(|error| io::Error::other(format!("{error:?}")))?;
    let cache = hayro::RenderCache::new();
    let interpreter_settings = hayro::hayro_interpret::InterpreterSettings::default();
    let render_settings = hayro::RenderSettings {
        x_scale: 2.0,
        y_scale: 2.0,
        bg_color: hayro::vello_cpu::color::palette::css::WHITE,
        ..Default::default()
    };
    let mut recognized = Vec::new();
    for page_index in page_indices {
        let Some(page) = pdf.pages().get(*page_index) else {
            continue;
        };
        let pixmap = hayro::render(page, &cache, &interpreter_settings, &render_settings);
        let png = pixmap.into_png().map_err(io::Error::other)?;
        let path = temporary_png_path(*page_index);
        std::fs::write(&path, png)?;
        let text = backend.recognize(&path);
        let _ = std::fs::remove_file(&path);
        recognized.push((*page_index, text?));
    }
    Ok(recognized)
}

fn required_env_path(name: &str) -> io::Result<PathBuf> {
    std::env::var_os(name).map(PathBuf::from).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("{name} must be set for ocr-rs"),
        )
    })
}

fn temporary_png_path(page_index: usize) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!(
        "atom-ocr-{}-{page_index}-{nonce}.png",
        std::process::id()
    ))
}
