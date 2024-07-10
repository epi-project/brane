//  BUILD.rs
//    by Lut99
//
//  Created:
//    01 May 2024, 13:44:20
//  Last edited:
//    01 May 2024, 15:25:50
//  Auto updated?
//    Yes
//
//  Description:
//!   Build script for the `brane-ctl` crate.
//!
//!   This builds `cfssl` and `cfssljson` from source to deal with ARM machines.
//

use std::fs;
use std::path::PathBuf;


/***** ENTRYPOINT *****/
fn main() {
    // Get the target directory to build to
    let target_dir: PathBuf = match std::env::var("OUT_DIR") {
        Ok(dir) => dir.into(),
        Err(err) => panic!("Failed to get environment variable 'OUT_DIR': {err}"),
    };
    let build_dir: PathBuf = target_dir.join("cfssl");

    // Decide if we can download the binaries or have to clone them
    #[cfg(target_arch = "x86_64")]
    #[cfg(any(windows, target_os = "macos", target_os = "linux"))]
    {
        // We can fetch them from the internet
        use download::{download_file, DownloadSecurity};
        use hex_literal::hex;

        // Get the URL & checksum of the binary to download
        let files: [(&str, [u8; 32], bool); 2] = [
            #[cfg(windows)]
            (
                "https://github.com/cloudflare/cfssl/releases/download/v1.6.3/cfssl_1.6.3_windows_amd64.exe",
                hex!("32496dadebd738cccd72ebbc5b17fa31e822b2379d2741fe844e5c37a0d91f90"),
                false,
            ),
            #[cfg(windows)]
            (
                "https://github.com/cloudflare/cfssl/releases/download/v1.6.3/cfssljson_1.6.3_windows_amd64.exe",
                hex!("52c324780980102f973df2175ce2e25b8577f11dd4f7f97970f2cf6e96ce3455"),
                true,
            ),
            #[cfg(target_os = "macos")]
            (
                "https://github.com/cloudflare/cfssl/releases/download/v1.6.3/cfssl_1.6.3_darwin_amd64",
                hex!("ee4d6494f2866204611e417e3b51e68013daf1ea742a803d49ff06319948f1b2"),
                false,
            ),
            #[cfg(target_os = "macos")]
            (
                "https://github.com/cloudflare/cfssl/releases/download/v1.6.3/cfssljson_1.6.3_darwin_amd64",
                hex!("53462962d45f08cdaf689a8c2980624158dad975af119d74be84adab962986c1"),
                true,
            ),
            #[cfg(target_os = "linux")]
            (
                "https://github.com/cloudflare/cfssl/releases/download/v1.6.3/cfssl_1.6.3_linux_amd64",
                hex!("16b42bfc592dc4d0ba1e51304f466cae7257edec13743384caf4106195ab6047"),
                false,
            ),
            #[cfg(target_os = "linux")]
            (
                "https://github.com/cloudflare/cfssl/releases/download/v1.6.3/cfssljson_1.6.3_linux_amd64",
                hex!("3b26c85877e2233684216ec658594be474954bc62b6f08780b369e234ccc67c9"),
                true,
            ),
        ];

        // Prepare the build directory we output to
        if !build_dir.exists() {
            if let Err(err) = fs::create_dir_all(&build_dir) {
                panic!("Failed to create cfssl cache directory '{}': {}", build_dir.display(), err);
            }
        }

        // Now download both files
        for (url, checksum, is_cfssljson) in files {
            // Nothing to do if the file already exists
            let path: PathBuf = build_dir.join(if !is_cfssljson { "cfssl" } else { "cfssljson" });
            if !path.exists() {
                // Run it using the download crate
                if let Err(err) = download_file(url, &path, DownloadSecurity::all(&checksum), None) {
                    panic!("Failed to download file '{}' to '{}': {}", url, path.display(), err);
                }
            }

            // Report where it lives
            if !is_cfssljson {
                println!("cargo:rustc-env=CFSSL_PATH={}", path.display());
            } else {
                println!("cargo:rustc-env=CFSSLJSON_PATH={}", path.display());
            }
        }
    }

    #[cfg(any(not(target_arch = "x86_64"), not(any(windows, target_os = "macos", target_os = "linux"))))]
    {
        // Build from source
        use std::process::{Command, ExitStatus};

        // Assert that the build directory is a git repo
        let git_dir: PathBuf = build_dir.join(".git");
        if build_dir.exists() && !git_dir.exists() {
            // Remove it to force the clone
            if let Err(err) = fs::remove_dir_all(&build_dir) {
                panic!("Failed to remove old build cache directory '{}': {}", build_dir.display(), err);
            }
        }

        // Clone the build directory if it does not exist
        if !build_dir.exists() {
            let mut cmd: Command = Command::new("git");
            cmd.arg("clone");
            cmd.arg("--recursive");
            cmd.arg("https://github.com/cloudflare/cfssl");
            cmd.arg(build_dir.as_os_str());
            let status: ExitStatus = match cmd.status() {
                Ok(status) => status,
                Err(err) => panic!("Failed to spawn {cmd:?}: {err}"),
            };
            if !status.success() {
                panic!("Failed to execute {cmd:?} (see output above)");
            }
        }

        // Build if the binary does not yet exist
        let cfssl_path: PathBuf = build_dir.join("cmd").join("cfssl");
        let cfssljson_path: PathBuf = build_dir.join("cmd").join("cfssljson");

        let cfssl_bin_path: PathBuf = cfssl_path.join("cfssl");
        let cfssljson_bin_path: PathBuf = cfssljson_path.join("cfssljson");

        if !cfssl_path.exists() || !cfssljson_path.exists() {
            // Fetch the tag we need
            let mut cmd: Command = Command::new("git");
            cmd.arg("fetch");
            cmd.arg("--tags");
            cmd.arg("--all");
            cmd.current_dir(&build_dir);
            let status: ExitStatus = match cmd.status() {
                Ok(status) => status,
                Err(err) => panic!("Failed to spawn {cmd:?}: {err}"),
            };
            if !status.success() {
                panic!("Failed to execute {cmd:?} (see output above)");
            }

            // Checkout to the correct tag
            let mut cmd: Command = Command::new("git");
            cmd.arg("checkout");
            cmd.arg("v1.6.3");
            cmd.current_dir(&build_dir);
            let status: ExitStatus = match cmd.status() {
                Ok(status) => status,
                Err(err) => panic!("Failed to spawn {cmd:?}: {err}"),
            };
            if !status.success() {
                panic!("Failed to execute {cmd:?} (see output above)");
            }
        }

        // Now run the build command for go to build the `cfssl` binary
        if !cfssl_bin_path.exists() {
            let mut cmd: Command = Command::new("go");
            cmd.arg("build");
            cmd.arg(".");
            cmd.current_dir(&cfssl_path);
            let status: ExitStatus = match cmd.status() {
                Ok(status) => status,
                Err(err) => panic!("Failed to spawn {cmd:?}: {err}"),
            };
            if !status.success() {
                panic!("Failed to execute {cmd:?} (see output above)");
            }
        }

        // Same for the `cfssljson` binary
        if !cfssljson_bin_path.exists() {
            let mut cmd: Command = Command::new("go");
            cmd.arg("build");
            cmd.arg(".");
            cmd.current_dir(&cfssljson_path);
            let status: ExitStatus = match cmd.status() {
                Ok(status) => status,
                Err(err) => panic!("Failed to spawn {cmd:?}: {err}"),
            };
            if !status.success() {
                panic!("Failed to execute {cmd:?} (see output above)");
            }
        }

        // OK, publish the paths
        println!("cargo:rustc-env=CFSSL_PATH={}", cfssl_bin_path.display());
        println!("cargo:rustc-env=CFSSLJSON_PATH={}", cfssljson_bin_path.display());
    }
}
