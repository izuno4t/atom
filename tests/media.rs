use std::io;
use std::path::{Path, PathBuf};

use anything_to_markdown::media::{self, VectorRasterizer};

struct StubRasterizer;

impl VectorRasterizer for StubRasterizer {
    fn rasterize(&self, input: &Path, output: &Path) -> io::Result<PathBuf> {
        assert_eq!(input, Path::new("image.wmf"));
        assert_eq!(output, Path::new("target/media/image.png"));
        Ok(output.to_path_buf())
    }
}

#[test]
fn wmf_emf_rasterization_boundary_outputs_png_path() {
    let output = media::rasterize_vector_image(
        &StubRasterizer,
        Path::new("image.wmf"),
        Path::new("target/media"),
    )
    .unwrap();

    assert_eq!(output, PathBuf::from("target/media/image.png"));
    assert!(media::is_vector_image(Path::new("diagram.emf")));
}
