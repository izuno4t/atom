use std::io;
use std::path::Path;
use std::time::Instant;

use crate::pipeline::ooxml_package::{
    materialize_ooxml_media, open_zip_archive, read_numbered_parts_from_archive, unzip_part,
    unzip_part_from_archive,
};
use crate::*;

pub(crate) fn convert_ooxml_file(
    path: &Path,
    input_format: String,
    options: &ConversionOptions,
) -> io::Result<ConversionResult> {
    let started = Instant::now();
    let mut warnings = Vec::new();
    let mut metadata = vec![("parser".to_string(), "zip+ooxml-package".to_string())];
    let mut ast = match input_format.as_str() {
        "docx" => parse_docx_file(path, &mut metadata, &mut warnings),
        "pptx" => parse_pptx_file(path, &mut metadata, &mut warnings)?,
        "xlsx" | "xlsm" => parse_xlsx_file(path, &mut metadata, &mut warnings)?,
        _ => vec![unsupported_node(&input_format)],
    };

    materialize_ooxml_media(path, options, &mut ast, &mut warnings)?;
    let rendered = render(&ast, options);
    let mut media = collect_media_paths(&ast);
    media.sort();
    media.dedup();
    let media_candidates = collect_media_candidates(&ast);
    let features = report_features(options, &media);
    Ok(ConversionResult {
        ast,
        markdown: rendered,
        report: ConversionReport {
            input_path: path.to_string_lossy().to_string(),
            input_format,
            output_format: format_name(options.format).to_string(),
            flavor: flavor_name(options.flavor).to_string(),
            warnings,
            metadata,
            elapsed_ms: started.elapsed().as_millis(),
            used_ocr: false,
            ocr_engine: None,
            used_llm: options.llm != LlmBackend::None,
            llm_destination: llm_destination(&options.llm),
            media,
            media_candidates,
            features,
        },
    })
}

fn parse_docx_file(
    path: &Path,
    metadata: &mut Vec<(String, String)>,
    warnings: &mut Vec<String>,
) -> Vec<AstNode> {
    match unzip_part(path, "word/document.xml") {
        Ok(xml) => {
            metadata.push(("part".to_string(), "word/document.xml".to_string()));
            let rels = unzip_part(path, "word/_rels/document.xml.rels").unwrap_or_default();
            record_optional_part(
                metadata,
                &rels,
                "relationships",
                "word/_rels/document.xml.rels",
            );
            let footnotes = unzip_part(path, "word/footnotes.xml").unwrap_or_default();
            let comments = unzip_part(path, "word/comments.xml").unwrap_or_default();
            record_optional_part(metadata, &footnotes, "part", "word/footnotes.xml");
            record_optional_part(metadata, &comments, "part", "word/comments.xml");
            ooxml::docx::parse_document_xml_with_rels_and_notes(
                &xml, &rels, &footnotes, &comments, warnings,
            )
        }
        Err(error) => {
            warnings.push(format!("failed to extract DOCX document.xml: {error}"));
            vec![unsupported_node("docx")]
        }
    }
}

fn parse_pptx_file(
    path: &Path,
    metadata: &mut Vec<(String, String)>,
    warnings: &mut Vec<String>,
) -> io::Result<Vec<AstNode>> {
    let mut archive = open_zip_archive(path)?;
    let slides = read_numbered_parts_from_archive(&mut archive, "ppt/slides/slide", ".xml", 200);
    if slides.is_empty() {
        warnings.push("failed to extract PPTX slide parts from package.".to_string());
        return Ok(vec![unsupported_node("pptx")]);
    }

    metadata.push(("slides".to_string(), slides.len().to_string()));
    let rels =
        read_numbered_parts_from_archive(&mut archive, "ppt/slides/_rels/slide", ".xml.rels", 200);
    if !rels.is_empty() {
        metadata.push(("slide_relationships".to_string(), rels.len().to_string()));
    }

    Ok(slides
        .iter()
        .enumerate()
        .flat_map(|(index, slide)| {
            let slide_rels = rels.get(index).map(String::as_str).unwrap_or_default();
            ooxml::parse_pptx_slide_xml_with_rels(slide, slide_rels, warnings)
        })
        .collect())
}

fn parse_xlsx_file(
    path: &Path,
    metadata: &mut Vec<(String, String)>,
    warnings: &mut Vec<String>,
) -> io::Result<Vec<AstNode>> {
    let mut archive = open_zip_archive(path)?;
    let shared_strings =
        unzip_part_from_archive(&mut archive, "xl/sharedStrings.xml").unwrap_or_default();
    record_optional_part(metadata, &shared_strings, "part", "xl/sharedStrings.xml");
    let sheets = read_numbered_parts_from_archive(&mut archive, "xl/worksheets/sheet", ".xml", 200);
    if sheets.is_empty() {
        warnings.push("failed to extract XLSX worksheet parts from package.".to_string());
        return Ok(vec![unsupported_node("xlsx")]);
    }

    metadata.push(("worksheets".to_string(), sheets.len().to_string()));
    let mut ast = Vec::new();
    let multiple_sheets = sheets.len() > 1;
    for (index, sheet) in sheets.iter().enumerate() {
        if multiple_sheets {
            ast.push(AstNode::Heading {
                level: 1,
                text: format!("Sheet {}", index + 1),
            });
        }
        ast.extend(ooxml::parse_xlsx_sheet_xml_with_warnings(
            sheet,
            &shared_strings,
            warnings,
        ));
    }
    Ok(ast)
}

fn record_optional_part(
    metadata: &mut Vec<(String, String)>,
    content: &str,
    key: &str,
    value: &str,
) {
    if !content.is_empty() {
        metadata.push((key.to_string(), value.to_string()));
    }
}
