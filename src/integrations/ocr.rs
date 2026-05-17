use crate::OcrEngine;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub trait OcrBackend {
    fn recognize(&self, input: &Path) -> io::Result<String>;
}

pub fn recognize_with(backend: &dyn OcrBackend, input: &Path) -> io::Result<String> {
    backend.recognize(input)
}

pub struct SubprocessOcrBackend {
    pub engine: OcrEngine,
}

pub struct OcrRsBackend {
    pub det_model_path: PathBuf,
    pub rec_model_path: PathBuf,
    pub charset_path: PathBuf,
}

impl OcrBackend for SubprocessOcrBackend {
    fn recognize(&self, input: &Path) -> io::Result<String> {
        run_subprocess(&self.engine, input)
    }
}

impl OcrBackend for OcrRsBackend {
    fn recognize(&self, input: &Path) -> io::Result<String> {
        let image = image::open(input).map_err(io::Error::other)?;
        let engine = ocr_rs::OcrEngine::new(
            &self.det_model_path,
            &self.rec_model_path,
            &self.charset_path,
            None,
        )
        .map_err(io::Error::other)?;
        let results = engine.recognize(&image).map_err(io::Error::other)?;
        Ok(results
            .into_iter()
            .map(|result| result.text)
            .collect::<Vec<_>>()
            .join("\n"))
    }
}

impl OcrRsBackend {
    pub fn from_env() -> io::Result<Self> {
        Ok(Self {
            det_model_path: required_env_path("BONJIL_OCR_RS_DET_MODEL")?,
            rec_model_path: required_env_path("BONJIL_OCR_RS_REC_MODEL")?,
            charset_path: required_env_path("BONJIL_OCR_RS_CHARSET")?,
        })
    }
}

pub fn backend_for_engine(engine: &OcrEngine) -> io::Result<Option<Box<dyn OcrBackend>>> {
    match engine {
        OcrEngine::None => Ok(None),
        OcrEngine::Auto | OcrEngine::OcrRs => Ok(Some(Box::new(OcrRsBackend::from_env()?))),
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
        "bonjil-ocr-{}-{page_index}-{nonce}.png",
        std::process::id()
    ))
}
