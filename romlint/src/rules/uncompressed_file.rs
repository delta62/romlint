use crate::filemeta::FileMeta;
use crate::linter::{Diagnostic, Rule};

#[derive(Default)]
pub struct UncompressedFile {
    compressed_file_count: u64,
    uncompressed_file_count: u64,
    compressed_size: u64,
    uncompressed_size: u64,
}

impl Rule for UncompressedFile {
    fn check(&mut self, file: &FileMeta) -> Result<(), Diagnostic> {
        if let Some(archive) = file.archive() {
            self.compressed_file_count += 1;
            self.compressed_size += archive.compressed_size;
            self.uncompressed_size += archive.uncompressed_size;
            Ok(())
        } else {
            self.uncompressed_file_count += 1;
            self.uncompressed_size += file.metadata().len();
            Err(Diagnostic::from_file(file, "File is not compressed"))
        }
    }

    fn help_text(&self) -> Vec<String> {
        let mut texts = vec![];

        // No way to generate an estimate of how much space could be saved
        if self.compressed_file_count == 0 {
            return texts;
        }

        let avg_compression_ratio =
            (1.0 - (self.compressed_size as f64) / (self.uncompressed_size as f64)) * 100.0;
        let saved_space = self.uncompressed_size - self.compressed_size;

        if self.uncompressed_file_count > 0 {
            let estimated_savings = (self.uncompressed_file_count as f64) * avg_compression_ratio;
            texts.push(format!(
                "Estimated disk space saved by adding compression: {}",
                estimated_savings
            ));
        }

        if saved_space > 0 {
            texts.push(format!(
                "Space saved from compression: {} ({:.2}% compression ratio)",
                fmt_size(saved_space),
                avg_compression_ratio
            ));
        }

        texts
    }
}

fn fmt_size(bytes: u64) -> String {
    let bytes = bytes as f64;
    if bytes < 1000.0 {
        format!("{}b", bytes)
    } else if bytes < 1_000_000.0 {
        format!("{:.2}kB", bytes / 1000.0)
    } else if bytes < 1_000_000_000.0 {
        format!("{:.2}MB", bytes / 1_000_000.0)
    } else {
        format!("{:.2}GB", bytes / 1_000_000_000.0)
    }
}
