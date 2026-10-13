use serde::Deserialize;
use serde_json::{Map, Value};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};
use crate::cli::Spinner;

//Script to be added to code
pub(crate) const INITRAMFS_SCRIPT: &str =
    include_str!("../scripts/shared/initramfs.sh");

pub(crate) const BOOTC_ROOTFS_SCRIPT: &str =
    include_str!("../scripts/shared/bootc-rootfs.sh");

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

struct AssembleFolder {
    pre_scripts: Option<Vec<PathBuf>>,
    post_scripts: Option<Vec<PathBuf>>,
    rootfs: Option<PathBuf>,
}

// Assembly build structure should look like this:
// folder
// |-- (main config name).json/yml
// |-- include/(config name).json/yml
// |-- (another config name but its includes to the first).json/yml
// |-- pre-scripts/*
// |-- post-scripts/*
// |-- rootfs/*

pub fn assembly(
    config_path: impl AsRef<Path>,
) -> Result<()> {
    let config = load_config(config_path.as_ref())?;

    let assemble_config: AssembleConfig =
        serde_json::from_value(config)?;

    //Check if config exists
    if !config_path.as_ref().exists() {
        return Err(format!(
            "Configuration file does not exist: {}",
            config_path.as_ref().display()
        )
        .into());
    }

    if let Some(assemble_folder) = config_path.as_ref().parent() {
        search_assembly_folder()?;
    }

    //Okay prepare Containerfile to build, first we need to download template with base and its look like https://raw.githubusercontent.com/ImmutableLinux/ilsm-containerfiles/refs/heads/main/Containerfile-(base).ilsm
    //The supported base distros are: arch, debian, ubuntu, fedora and opensuse


    Ok(())
}

fn search_assembly_folder() -> Result<AssembleFolder> {
    Spinner::new("Analysing assembly folder...");

    let mut assembleFolder = AssembleFolder {
        pre_scripts: None,
        post_scripts: None,
        rootfs: None,
    };

    //Check pre-scripts folder if, add to assembleFolder.pre_scripts with value of Vec<PathBuf> with all files in pre-scripts folder
    if Path::new("pre-scripts").exists() {
        assembleFolder.pre_scripts = Some(
            fs::read_dir("pre-scripts")?
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .collect(),
        );
    }

    //Check post-scripts folder if, add to assembleFolder.post_scripts with value of Vec<PathBuf> with all files in post-scripts folder
    if Path::new("post-scripts").exists() {
        assembleFolder.post_scripts = Some(
            fs::read_dir("post-scripts")?
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .collect(),
        );
    }

    //Check rootfs folder if, add to assembleFolder.rootfs with value of PathBuf with path to rootfs folder
    if Path::new("rootfs").exists() {
        assembleFolder.rootfs = Some(PathBuf::from("rootfs"));
    }
    Ok(assembleFolder)
}

fn prepare_temporary_build_folder(assemblyFolder: AssembleFolder) -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir();
    let build_dir = temp_dir.join(format!(
        "ilsm-build-{}",
        chrono::Local::now().format("%Y%m%d%H%M%S")
    ));

    fs::create_dir_all(&build_dir)?;

    //Create shared folder in build_dir
    let shared_dir = build_dir.join("shared");
    fs::create_dir_all(&shared_dir)?;

    //Copy initramfs.sh and bootc-rootfs.sh to shared folder
    fs::write(shared_dir.join("initramfs.sh"), INITRAMFS_SCRIPT)?;
    fs::write(shared_dir.join("bootc-rootfs.sh"), BOOTC_ROOTFS_SCRIPT)?;

    //Copy pre-scripts and post-scripts
    if let Some(pre_scripts) = assemblyFolder.pre_scripts {
        let pre_scripts_dir = shared_dir.join("pre-scripts");
        fs::create_dir_all(&pre_scripts_dir)?;

        for script in pre_scripts {
            let script_name = script
                .file_name()
                .ok_or("Invalid pre-script file name")?;
            fs::copy(&script, pre_scripts_dir.join(script_name))?;
        }
    }

    if let Some(post_scripts) = assemblyFolder.post_scripts {
        let post_scripts_dir = shared_dir.join("post-scripts");
        fs::create_dir_all(&post_scripts_dir)?;

        for script in post_scripts {
            let script_name = script
                .file_name()
                .ok_or("Invalid post-script file name")?;
            fs::copy(&script, post_scripts_dir.join(script_name))?;
        }
    }

    //Copy rootfs folder if exists
    if let Some(rootfs) = assemblyFolder.rootfs {
        let rootfs_dir = build_dir.join("rootfs");
        fs::create_dir_all(&rootfs_dir)?;
        for entry in fs::read_dir(rootfs)? {
            let entry = entry?;
            let entry_path = entry.path();
            let entry_name = entry_path
                .file_name()
                .ok_or("Invalid rootfs file name")?;
            fs::copy(&entry_path, rootfs_dir.join(entry_name))?;
        }
    }
    Ok(build_dir)
}

fn download_and_modify_containerfile(base: &str, packages: &[String]) -> Result<String> {
    let url = format!(
        "https://raw.githubusercontent.com/ImmutableLinux/ilsm-containerfiles/refs/heads/main/Containerfile-{}.ilsm",
        base
    );

    let response = reqwest::blocking::get(&url)?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download Containerfile for base '{}': {}",
            base,
            response.status()
        )
        .into());
    }

    //Okay the post-scripts, pre-scripts will be copied to /tmp/ilsm-build-DATETIME/shared/. but its later
    //Replace [PACKAGES] to list of packages and it looks like this: [PACKAGES] -> package1 package2 package3
    let packages_str = packages.join(" ");

    let mut containerfile_content = response.text()?;

    containerfile_content = containerfile_content.replace("[PACKAGES]", &packages_str);

    //Add pre-scripts up to the line where it says #PRE-SCRIPTS-HERE and add the pre-scripts to the containerfile_content, and templates use FROM scratch AS ctx COPY ./shared/ /shared
    containerfile_content = containerfile_content.replace("#PRE-SCRIPTS-HERE", "RUN for script in /shared/pre-scripts/*; do chmod +x \"$script\" && \"$script\"; done");

    //The same for post-scripts, but it should be added to the line where it says #POST-SCRIPTS-HERE
    containerfile_content = containerfile_content.replace("#POST-SCRIPTS-HERE", "RUN for script in /shared/post-scripts/*; do chmod +x \"$script\" && \"$script\"; done");

    //The same for rootfs, but it should be added to the line where it says #ROOTFS-HERE
    containerfile_content = containerfile_content.replace("#ROOTFS-HERE", "COPY ./shared/rootfs/ /");
    Ok(containerfile_content)
}