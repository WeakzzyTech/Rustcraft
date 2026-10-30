//! iOS entry point: the app's executable, which reads the launch arguments
//! from the app's Documents folder and runs the client, with its data in the
//! app's Application Support.

#[cfg(target_os = "ios")]
fn main() {
    use rustcraft_client::{LAUNCH_FILE, LaunchArgs, Platform};

    // An app's HOME is its container.
    let documents = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_default()
        .join("Documents");
    rustcraft_client::run(
        LaunchArgs::from_file(&documents.join(LAUNCH_FILE)),
        Platform { data_root: None },
    );
}

/// Built along with the rest of the workspace, where it has nothing to run.
#[cfg(not(target_os = "ios"))]
fn main() {
    eprintln!("rustcraft-ios only runs on iOS; see rustcraft-ios/README.md");
}
