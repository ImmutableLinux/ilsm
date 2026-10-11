use serde::Deserialize;
use serde_json::{Map, Value};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

//Assembly configuration parser. It supports JSON and YAML formats, and allows for recursive includes of other configuration files.

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn parse_file(path: &Path) -> Result<Value> {
    let content = fs::read_to_string(path)?;

    match path.extension().and_then(|s| s.to_str()) {
        Some("json") => Ok(serde_json::from_str(&content)?),
        Some("yaml") | Some("yml") => Ok(serde_yaml::from_str(&content)?),
        _ => Err(format!("Unsupported file extension: {:?}", path).into()),
    }
}


fn merge(base: &mut Value, overlay: Value) {
    match (base, overlay) {
        (Value::Object(base), Value::Object(overlay)) => {
            for (key, value) in overlay {
                match base.get_mut(&key) {
                    Some(existing) => {
                        if key == "packages" {
                            match (existing, value) {
                                (Value::Array(base), Value::Array(mut overlay)) => {
                                    base.append(&mut overlay);
                                }
                                (base, overlay) => *base = overlay,
                            }
                        } else {
                            merge(existing, value);
                        }
                    }
                    None => {
                        base.insert(key, value);
                    }
                }
            }
        }
        (base, overlay) => *base = overlay,
    }
}

fn load_recursive(
    path: &Path,
    stack: &mut Vec<PathBuf>,
) -> Result<Value> {
    let path = fs::canonicalize(path)?;

    if stack.contains(&path) {
        return Err(format!(
            "Include path detected: {}",
            path.display()
        )
        .into());
    }

    stack.push(path.clone());

    let result = (|| -> Result<Value> {
        let mut current = parse_file(&path)?;

        let object = current
            .as_object_mut()
            .ok_or("Main configuration element must be an object")?;

        let includes = object.remove("include");

        let mut combined = Value::Object(Map::new());

        if let Some(includes) = includes {
            let includes = match includes {
                Value::String(path) => vec![Value::String(path)],
                Value::Array(paths) => paths,
                _ => return Err(
                    "include must be a string or an array of strings".into()
                ),
            };

            for include in includes {
                let include = include
                    .as_str()
                    .ok_or("Every include element must be a string")?;

                let include_path = path
                    .parent()
                    .unwrap_or(Path::new("."))
                    .join(include);

                let included = load_recursive(&include_path, stack)?;

                merge(&mut combined, included);
            }
        }

        merge(&mut combined, current);

        Ok(combined)
    })();

    stack.pop();
    result
}

fn load_config(path: impl AsRef<Path>) -> Result<Value> {
    let mut stack = Vec::new();
    load_recursive(path.as_ref(), &mut stack)
}

// Config structure

#[derive(Debug, Deserialize)]
struct AssembleConfig {
    /// What base distro will be use to build container bootc image
    base: String,

    /// List of packages to install in the container bootc image
    packages: Vec<String>,
}

// Assembly build structure should look like this:
// folder
// |-- (main config name).json/yml
// |-- include/(config name).json/yml
// |-- (another config name but its includes to the first).json/yml
// |-- pre-scripts/*
// |-- post-scripts/*
// |-- rootfs/*
struct AssembleBuild {
    // Config file
    assemble_config: AssembleConfig,

    // Pre-scripts to run before installing packages to the container bootc image
    pre_scripts: Vec<PathBuf>,

    // Post-scripts to run after installing packages to the container bootc image
    post_scripts: Vec<PathBuf>,

    // Files to copy to the container bootc image
    rootfs: PathBuf,
}