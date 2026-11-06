//! Types and data models for the BabyDra File Chooser / Dialog service.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Dialog operation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileDialogMode {
    /// Pick a single existing file.
    OpenFile,
    /// Pick one or more existing files.
    OpenMultipleFiles,
    /// Pick a directory / folder.
    OpenFolder,
    /// Specify a destination file for saving.
    SaveFile,
}

impl Default for FileDialogMode {
    fn default() -> Self {
        Self::OpenFile
    }
}

/// File filter definition specifying allowed extensions and/or MIME types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileFilter {
    /// Human-readable filter description, e.g. "Images (*.png, *.jpg)"
    pub name: String,
    /// Glob/extension patterns, e.g. `["*.png", "*.jpg", "*.jpeg"]`
    pub patterns: Vec<String>,
    /// MIME types, e.g. `["image/png", "image/jpeg"]`
    pub mime_types: Vec<String>,
}

impl FileFilter {
    /// Creates a new filter matching all files (`*.*`).
    pub fn all_files() -> Self {
        Self {
            name: "All Files (*.*)".to_string(),
            patterns: vec!["*.*".to_string(), "*".to_string()],
            mime_types: Vec::new(),
        }
    }

    /// Creates a filter with given display name and file extension patterns.
    pub fn new(name: impl Into<String>, patterns: &[&str]) -> Self {
        Self {
            name: name.into(),
            patterns: patterns.iter().map(|p| p.to_string()).collect(),
            mime_types: Vec::new(),
        }
    }

    /// Common filter for standard image files.
    pub fn images() -> Self {
        Self::new(
            "Images (*.png, *.jpg, *.jpeg, *.webp, *.svg)",
            &["*.png", "*.jpg", "*.jpeg", "*.webp", "*.svg", "*.bmp", "*.gif"],
        )
    }

    /// Common filter for text documents.
    pub fn text_documents() -> Self {
        Self::new(
            "Documents (*.txt, *.md, *.pdf, *.doc, *.docx)",
            &["*.txt", "*.md", "*.pdf", "*.doc", "*.docx", "*.rtf", "*.odt"],
        )
    }

    /// Common filter for audio files.
    pub fn audio() -> Self {
        Self::new(
            "Audio (*.mp3, *.wav, *.flac, *.ogg, *.m4a)",
            &["*.mp3", "*.wav", "*.flac", "*.ogg", "*.m4a", "*.aac"],
        )
    }

    /// Common filter for video files.
    pub fn video() -> Self {
        Self::new(
            "Video (*.mp4, *.mkv, *.webm, *.avi, *.mov)",
            &["*.mp4", "*.mkv", "*.webm", "*.avi", "*.mov"],
        )
    }
}

/// Options to configure the File Chooser Dialog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDialogOptions {
    /// Dialog window title.
    pub title: String,
    /// Operational mode.
    pub mode: FileDialogMode,
    /// Initial folder to display.
    pub initial_folder: Option<PathBuf>,
    /// Initial filename pre-populated in Save / Open mode.
    pub initial_file: Option<String>,
    /// Custom label for the confirm button ("Select", "Open", "Save", etc.).
    pub accept_label: Option<String>,
    /// List of available filters.
    pub filters: Vec<FileFilter>,
    /// Index of the default filter.
    pub default_filter_index: Option<usize>,
    /// Whether hidden files (`.hidden`) should be visible initially.
    pub show_hidden: bool,
}

impl Default for FileDialogOptions {
    fn default() -> Self {
        Self {
            title: "Open File".to_string(),
            mode: FileDialogMode::OpenFile,
            initial_folder: None,
            initial_file: None,
            accept_label: None,
            filters: vec![FileFilter::all_files()],
            default_filter_index: Some(0),
            show_hidden: false,
        }
    }
}

impl FileDialogOptions {
    /// Creates options for opening a folder (directory picker).
    pub fn open_folder(title: impl Into<String>, initial_folder: Option<PathBuf>) -> Self {
        Self {
            title: title.into(),
            mode: FileDialogMode::OpenFolder,
            initial_folder,
            initial_file: None,
            accept_label: Some("Select".to_string()),
            filters: Vec::new(),
            default_filter_index: None,
            show_hidden: false,
        }
    }

    /// Creates options for opening a single file.
    pub fn open_file(title: impl Into<String>, initial_folder: Option<PathBuf>) -> Self {
        Self {
            title: title.into(),
            mode: FileDialogMode::OpenFile,
            initial_folder,
            initial_file: None,
            accept_label: Some("Open".to_string()),
            filters: vec![FileFilter::all_files()],
            default_filter_index: Some(0),
            show_hidden: false,
        }
    }

    /// Creates options for opening multiple files.
    pub fn open_multiple(title: impl Into<String>, initial_folder: Option<PathBuf>) -> Self {
        Self {
            title: title.into(),
            mode: FileDialogMode::OpenMultipleFiles,
            initial_folder,
            initial_file: None,
            accept_label: Some("Open".to_string()),
            filters: vec![FileFilter::all_files()],
            default_filter_index: Some(0),
            show_hidden: false,
        }
    }

    /// Creates options for saving a file.
    pub fn save_file(
        title: impl Into<String>,
        initial_folder: Option<PathBuf>,
        initial_name: Option<String>,
    ) -> Self {
        Self {
            title: title.into(),
            mode: FileDialogMode::SaveFile,
            initial_folder,
            initial_file: initial_name,
            accept_label: Some("Save".to_string()),
            filters: vec![FileFilter::all_files()],
            default_filter_index: Some(0),
            show_hidden: false,
        }
    }
}

/// Result returned from the File Chooser Dialog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDialogResult {
    /// Whether the user cancelled the dialog.
    pub cancelled: bool,
    /// List of chosen paths (empty if cancelled).
    pub paths: Vec<PathBuf>,
}

impl FileDialogResult {
    /// Successful selection.
    pub fn success(paths: Vec<PathBuf>) -> Self {
        Self {
            cancelled: false,
            paths,
        }
    }

    /// User cancelled or closed the dialog.
    pub fn cancel() -> Self {
        Self {
            cancelled: true,
            paths: Vec::new(),
        }
    }

    /// Returns the single selected path if available.
    pub fn single_path(&self) -> Option<&PathBuf> {
        if self.cancelled {
            None
        } else {
            self.paths.first()
        }
    }
}

/// A quick access bookmark item for the sidebar dock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuickAccessPlace {
    pub name: String,
    pub path: PathBuf,
    pub icon_name: String,
}

