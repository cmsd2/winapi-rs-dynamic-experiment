use std::env;
use std::path::Path;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    
    let mut res = winres::WindowsResource::new();
    res.set_resource_file(Path::new(&crate_dir).join("src/DpiAwarenessContext.rc").to_str().unwrap());
    res.compile()
        .expect("compile resources");
}