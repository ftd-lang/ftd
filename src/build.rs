pub fn build() {
    let fpm_config = fpm::fpm_check();
    println!("Building... {}", fpm_config.base_dir.as_str());
    std::fs::create_dir_all(format!("{}/.build", &fpm_config.base_dir).as_str())
        .expect("failed to create build folder");

    process_dir(fpm_config.base_dir.clone(), 0, fpm_config.base_dir);
}

pub fn process_dir(directory: String, depth: usize, base_path: String) -> u32 {
    let mut count: u32 = 0;
    for entry in std::fs::read_dir(&directory).expect("Panic! Unable to process the directory") {
        let e = entry.expect("Panic: Doc not found");
        let md = std::fs::metadata(e.path()).expect("Doc Metadata evaluation failed");
        let doc_path = e
            .path()
            .to_str()
            .expect("Directory path is expected")
            .to_string();
        if md.is_dir() {
            // Iterate the children
            count += process_dir(doc_path, depth + 1, base_path.as_str().to_string());
        } else if doc_path.as_str().ends_with(".ftd") {
            // process the document
            let doc = std::fs::read_to_string(doc_path).expect("cant read file");
            let id = e.path().clone();
            let id = id.to_str().expect(">>>").split('/');
            let len = id.clone().count();

            write(
                id.skip(len - (depth + 1))
                    .take_while(|_| true)
                    .collect::<Vec<&str>>()
                    .join("/")
                    .as_str(),
                doc,
                base_path.as_str().to_string(),
            );
            count += 1;
        }
    }
    count
}

fn write(id: &str, doc: String, base_path: String) {
    use std::io::Write;

    let lib = fpm::Library {};
    let b = match ftd::p2::Document::from(id, &*doc, &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse {}: {:?}", id, &e);
            return;
        }
    };

    std::fs::create_dir_all(format!(
        "{}/.build/{}",
        base_path.as_str(),
        id.replace(".ftd", "")
    ))
    .expect("failed to create directory folder for doc");

    let mut f = std::fs::File::create(format!(
        "{}/.build/{}",
        base_path.as_str(),
        id.replace(".ftd", "/index.html")
    ))
    .expect("failed to create .html file");

    let doc = b.to_rt("main", id);

    f.write_all(
        ftd::html()
            .replace(
                "__ftd_data__",
                serde_json::to_string_pretty(&doc.data)
                    .expect("failed to convert document to json")
                    .as_str(),
            )
            .replace(
                "__ftd_external_children__",
                serde_json::to_string_pretty(&doc.external_children)
                    .expect("failed to convert document to json")
                    .as_str(),
            )
            .replace("__ftd__", b.html("main", id).as_str())
            .replace("__ftd_js__", ftd::js())
            .as_bytes(),
    )
    .expect("failed to write to .html file");
}
