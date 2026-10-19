use std::path::Path;

fn main() {
    #[cfg(target_os = "linux")]
    {
        // If libasound.so is missing but libasound.so.2 exists in /usr/lib/x86_64-linux-gnu,
        // create a local symlink in .lib if needed and instruct cargo to link against it.
        let default_lib = Path::new("/usr/lib/x86_64-linux-gnu/libasound.so.2");
        let local_lib_dir = Path::new(".lib");
        let local_symlink = local_lib_dir.join("libasound.so");

        if default_lib.exists() && !local_symlink.exists() {
            let _ = std::fs::create_dir_all(local_lib_dir);
            #[cfg(unix)]
            let _ = std::os::unix::fs::symlink(default_lib, &local_symlink);
        }

        if local_symlink.exists() {
            println!("cargo:rustc-link-search=native=.lib");
        }
    }
}
