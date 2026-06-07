pub mod docx;
pub(crate) mod docx_body_order;
pub(crate) mod docx_media;
pub(crate) mod docx_relationships;
pub mod pptx;
pub mod xlsx;
mod xml;

pub use docx::{
    parse_document_xml, parse_document_xml_with_rels, parse_document_xml_with_rels_and_notes,
};
pub use pptx::{parse_pptx_slide_xml, parse_pptx_slide_xml_with_rels};
pub use xlsx::{parse_xlsx_sheet_xml, parse_xlsx_sheet_xml_with_warnings};
