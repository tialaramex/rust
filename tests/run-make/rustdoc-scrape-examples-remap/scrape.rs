use run_make_support::{htmldocck, rustc, rustdoc, source_path, tmp_dir};
use std::fs::read_dir;
use std::path::Path;

pub fn scrape() {
    let lib_dir = tmp_dir();
    let out_dir = tmp_dir().join("rustdoc");
    let crate_name = "foobar";
    let deps = read_dir("examples")
        .unwrap()
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "rs"))
        .collect::<Vec<_>>();

    rustc().input("src/lib.rs").crate_name(crate_name).crate_type("lib").emit("metadata").run();

    let mut out_deps = Vec::with_capacity(deps.len());
    for dep in deps {
        let dep_stem = dep.file_stem().unwrap();
        let out_example = out_dir.join(format!("{}.calls", dep_stem.to_str().unwrap()));
        rustdoc()
            .input(&dep)
            .crate_name(&dep_stem)
            .crate_type("bin")
            .output(&out_dir)
            .extern_(crate_name, lib_dir.join(format!("lib{crate_name}.rmeta")))
            .arg("-Zunstable-options")
            .arg("--scrape-examples-output-path")
            .arg(&out_example)
            .arg("--scrape-examples-target-crate")
            .arg(crate_name)
            .run();
        out_deps.push(out_example);
    }

    let mut rustdoc = rustdoc();
    rustdoc
        .input("src/lib.rs")
        .output(&out_dir)
        .crate_name(crate_name)
        .crate_type("lib")
        .arg("-Zunstable-options");
    for dep in out_deps {
        rustdoc.arg("--with-examples").arg(dep);
    }
    rustdoc.run();

    assert!(htmldocck().arg(out_dir).arg("src/lib.rs").status().unwrap().success());
}
