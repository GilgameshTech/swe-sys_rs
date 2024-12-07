use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::collections::HashSet;

use bindgen::callbacks::{ MacroParsingBehavior, ParseCallbacks };

fn main() {
    let mut _cfg = cc::Build::new();
    let mut cfg = _cfg.opt_level(2);
    cfg.warnings(false);
    add_c_files(&mut cfg, "swisseph-2.10.03");
    cfg.compile("swisseph");

    let macros = Arc::new(RwLock::new(HashSet::new()));
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .blocklist_function("__.*")
        .clang_arg("-O2")
        .layout_tests(false)
        .parse_callbacks(Box::new(MacroCallback {macros: macros.clone()}))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rustc-link-lib=swisseph");
}

fn add_c_files(build: &mut cc::Build, path: impl AsRef<Path>) {
    let path = path.as_ref();
    if !path.exists() {
        panic!("Path {} does not exist", path.display());
    }
    let dir = path.read_dir().unwrap();
    let mut paths = dir.collect::<io::Result<Vec<_>>>().unwrap();
    paths.sort_by_key(|e| e.path());

    for e in paths {
        let path = e.path();
        if e.file_type().unwrap().is_dir() {
        } else if path.extension().and_then(|s| s.to_str()) == Some("c") {
            if path.file_stem().is_some() {
                build.file(&path);
            }
        }
    }
}

#[derive(Debug)]
struct MacroCallback {
    macros: Arc<RwLock<HashSet<String>>>,
}

impl ParseCallbacks for MacroCallback {
    fn will_parse_macro(&self, name: &str) -> MacroParsingBehavior {
        self.macros.write().unwrap().insert(name.into());

        if name == "FP_NORMAL" 
        || name == "FP_SUBNORMAL" 
        || name == "FP_ZERO" 
        || name == "FP_NAN" 
        || name == "FP_INFINITE" 
        {
            return MacroParsingBehavior::Ignore;
        }

        MacroParsingBehavior::Default
    }
}
