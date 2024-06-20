fn main() {
    let skip_js_build = std::env::var("SKIP_JS_BUILD").unwrap_or_default();
    if skip_js_build.is_empty() {
        eprintln!("SKIP_JS_BUILD=1 not set, proceeding to build the UI.");

        // Check to make sure `npm` is installed
        std::process::Command::new("npm")
            .arg("-v")
            .status()
            .expect("npm to be installed");
        // Check to make sure `tsc` is installed
        std::process::Command::new("tsc")
            .arg("-v")
            .status()
            .expect("typescript (tsc) to be installed");

        let ui_path = std::fs::canonicalize("./ui/").expect("resolve cloud-hello/ui/");

        // Install npm dependencies
        std::process::Command::new("npm")
            .arg("install")
            .current_dir(ui_path.clone())
            .status()
            .expect("npm install");

        // Build the UI
        std::process::Command::new("npm")
            .args(["run", "build"])
            .current_dir(ui_path)
            .status()
            .expect("npm run build");
    }
}
