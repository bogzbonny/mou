use std::path::PathBuf;

/// The default project name based on the current directory
///
/// # Panics
///
/// Panics if the current directory is not available
#[must_use]
pub fn default_project_name() -> String {
    "your_mou".to_string()
}

pub(super) fn default_cache_dir() -> PathBuf {
    let mut path = dirs::cache_dir().expect("Failed to get cache directory");
    path.push("mou");
    path
}

pub(super) fn default_log_dir() -> PathBuf {
    let mut path = dirs::cache_dir().expect("Failed to get cache directory");
    path.push("mou");
    path.push("logs");

    path
}

#[must_use]
pub fn default_dockerfile() -> PathBuf {
    "./Dockerfile".into()
}

#[must_use]
pub fn default_docker_context() -> PathBuf {
    ".".into()
}
