//! Local model store scanning.

use crate::gguf::parse_gguf_header;
use crate::{GgufMetadata, DEFAULT_MODEL_STORE};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// A model file found in the local store.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LocalModel {
    pub path: PathBuf,
    pub file_size_bytes: u64,
    pub format: ModelFormat,
    pub metadata: Option<GgufMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum ModelFormat {
    Gguf,
    Safetensors,
    Onnx,
    Unknown,
}

impl ModelFormat {
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|e| e.to_str()) {
            Some("gguf") => ModelFormat::Gguf,
            Some("safetensors") => ModelFormat::Safetensors,
            Some("onnx") => ModelFormat::Onnx,
            _ => ModelFormat::Unknown,
        }
    }
}

/// Scans a directory tree for model files.
pub struct ModelStore {
    root: PathBuf,
}

impl ModelStore {
    pub fn open_default() -> Self {
        Self {
            root: PathBuf::from(DEFAULT_MODEL_STORE),
        }
    }

    pub fn open(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// List all model files under the store, one level of nesting deep
    /// (vendor/model layout). Missing store returns an empty list.
    pub fn list(&self) -> Result<Vec<LocalModel>> {
        let mut models = Vec::new();
        let Some(entries) = self.walk(&self.root, 0) else {
            return Ok(models);
        };
        for model in entries.into_iter().flatten() {
            models.push(model);
        }
        models.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(models)
    }

    /// Depth-limited walk. Returns None when the directory is absent.
    fn walk(&self, dir: &Path, depth: u32) -> Option<Vec<Option<LocalModel>>> {
        if depth > 2 {
            return Some(Vec::new());
        }
        let entries = std::fs::read_dir(dir).ok()?;
        let mut out = Vec::new();
        for entry in entries.flatten() {
            let Ok(ft) = entry.file_type() else { continue };
            let path = entry.path();
            if ft.is_dir() {
                if let Some(mut sub) = self.walk(&path, depth + 1) {
                    out.append(&mut sub);
                }
            } else if ft.is_file() {
                out.push(self.inspect_file(&path));
            }
        }
        Some(out)
    }

    fn inspect_file(&self, path: &Path) -> Option<LocalModel> {
        let format = ModelFormat::from_path(path);
        if format == ModelFormat::Unknown {
            return None;
        }
        let file_size_bytes = std::fs::metadata(path).ok()?.len();
        let metadata = if format == ModelFormat::Gguf {
            std::fs::File::open(path)
                .ok()
                .and_then(|f| parse_gguf_header(std::io::BufReader::new(f)).ok())
                .map(|h| h.metadata)
        } else {
            None
        };
        Some(LocalModel {
            path: path.to_path_buf(),
            file_size_bytes,
            format,
            metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gguf::sample_gguf_bytes;
    use std::fs;

    #[test]
    fn scans_store_and_parses_gguf() {
        let dir = std::env::temp_dir().join(format!("aion-test-store-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        fs::write(dir.join("tiny.gguf"), sample_gguf_bytes()).unwrap();
        fs::write(dir.join("weights.safetensors"), b"xxxx").unwrap();
        fs::write(dir.join("readme.txt"), b"not a model").unwrap();

        let store = ModelStore::open(&dir);
        let models = store.list().unwrap();
        assert_eq!(models.len(), 2);

        let gguf = models
            .iter()
            .find(|m| m.format == ModelFormat::Gguf)
            .expect("gguf found");
        assert_eq!(
            gguf.metadata.as_ref().unwrap().name.as_deref(),
            Some("test-model")
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_store_is_empty() {
        let store = ModelStore::open("/nonexistent/aion/store");
        assert!(store.list().unwrap().is_empty());
    }
}
