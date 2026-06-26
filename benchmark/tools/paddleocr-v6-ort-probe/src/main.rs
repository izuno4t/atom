//! PP-OCRv6 を Rust から in-process で動かす検証プローブ（rec 優先）。
//!
//! rec.onnx を ort(ONNX Runtime) で実行し、前処理と CTC デコードを Rust で実装する。
//! 仕様は inference.yml と Python 版 golden(`benchmark/ocr-eval/rec_golden.py`) に一致:
//!   - 入力 x: [1,3,48,W]、CHW、BGR、(px/255-0.5)/0.5
//!   - RecResizeImg: 高さ48・縦横比維持・最大幅320・0パディング
//!   - 出力: [1,T,18710]、CTCLabelDecode
//!     index0=blank, 1..=18708=辞書, 18709=space
//!
//! 使い方:
//!   paddleocr-v6-ort-probe <rec.onnx> <charset.txt> <image> <out.txt>

use image::imageops::FilterType;
use ndarray::Array4;
use ort::session::Session;
use ort::value::TensorRef;

const IMG_H: usize = 48;
const IMG_W: usize = 320;

fn main() -> ort::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        eprintln!("usage: paddleocr-v6-ort-probe <rec.onnx> <charset.txt> <image> <out.txt>");
        std::process::exit(2);
    }
    let (model, charset_path, image_path, out_path) = (&args[1], &args[2], &args[3], &args[4]);

    let charset_raw = std::fs::read_to_string(charset_path).expect("read charset");
    let mut charset: Vec<String> = charset_raw.split('\n').map(str::to_string).collect();
    if charset.last().map(String::is_empty).unwrap_or(false) {
        charset.pop(); // ファイル末尾の改行による空要素を落とす
    }

    let input = preprocess(image_path);

    let mut session = Session::builder()?.commit_from_file(model)?;
    let outputs = session.run(ort::inputs![TensorRef::from_array_view(&input)?])?;
    let (_shape, data) = outputs[0].try_extract_tensor::<f32>()?;

    // 出力は [1, T, C]。C = blank(1) + 辞書 + space(1)。
    let c = charset.len() + 2;
    let t = data.len() / c;
    let text = ctc_decode(data, t, c, &charset);

    std::fs::write(out_path, text).expect("write output");
    Ok(())
}

/// 画像を rec 入力テンソル [1,3,48,320] へ。BGR・縦横比維持・0パディング・正規化。
fn preprocess(image_path: &str) -> Array4<f32> {
    let img = image::open(image_path).expect("open image").to_rgb8();
    let (w, h) = img.dimensions();
    let ratio = w as f32 / h as f32;
    let resized_w = ((IMG_H as f32 * ratio).ceil() as usize).min(IMG_W).max(1);
    let resized = image::imageops::resize(&img, resized_w as u32, IMG_H as u32, FilterType::Triangle);

    let mut tensor = Array4::<f32>::zeros((1, 3, IMG_H, IMG_W));
    for y in 0..IMG_H {
        for x in 0..resized_w {
            let px = resized.get_pixel(x as u32, y as u32);
            // BGR 順で正規化 (px/255 - 0.5)/0.5
            let b = (px[2] as f32 / 255.0 - 0.5) / 0.5;
            let g = (px[1] as f32 / 255.0 - 0.5) / 0.5;
            let r = (px[0] as f32 / 255.0 - 0.5) / 0.5;
            tensor[[0, 0, y, x]] = b;
            tensor[[0, 1, y, x]] = g;
            tensor[[0, 2, y, x]] = r;
        }
    }
    tensor
}

/// CTC greedy decode。連続重複を畳み込み、blank(0) を捨てる。
/// index 1..=charset.len() を辞書へ、最後の index を space へ写像する。
fn ctc_decode(data: &[f32], t: usize, c: usize, charset: &[String]) -> String {
    let space_index = charset.len() + 1; // 0=blank, 1..=len=辞書, len+1=space
    let mut out = String::new();
    let mut prev = usize::MAX;
    for ti in 0..t {
        let row = &data[ti * c..ti * c + c];
        let mut best = 0usize;
        let mut best_v = f32::NEG_INFINITY;
        for (i, &v) in row.iter().enumerate() {
            if v > best_v {
                best_v = v;
                best = i;
            }
        }
        if best != prev && best != 0 {
            if best == space_index {
                out.push(' ');
            } else if best <= charset.len() {
                out.push_str(&charset[best - 1]);
            }
        }
        prev = best;
    }
    out
}
