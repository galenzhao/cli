use anyhow::{anyhow, Context, Result};
use glob::glob;
use itertools::Itertools;
use log::info;
use serde_json::Value;
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;
use zip::{write::FileOptions, CompressionMethod, ZipWriter};

use crate::cli::{CompressMethod, FilenameSource, PackZipBasename};

#[derive(Clone)]
pub struct Packer {
    pub plugin_root: PathBuf,
    pub output_root: PathBuf,
    pub follow_symlinks: bool,
    pub output_filename_source: FilenameSource,
    pub compression_method: CompressMethod,
    pub compression_level: Option<i32>,
    pub build_with_dev: bool,
    pub zip_basename: PackZipBasename,
    pub zip_version: Option<String>,
}

impl Packer {
    fn read_plugin_name(plugin_root: &Path) -> Result<String> {
        let plugin_json = plugin_root.join("plugin.json");
        plugin_json
            .exists()
            .then_some(())
            .ok_or_else(|| anyhow!("Could not find plugin.json"))?;

        let contents = std::fs::read_to_string(&plugin_json)?;
        let json: Value = serde_json::from_str(&contents)?;

        json["name"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("plugin.json is missing required field `name`"))
    }

    fn read_package_version(plugin_root: &Path) -> Option<String> {
        let path = plugin_root.join("package.json");
        let contents = std::fs::read_to_string(&path).ok()?;
        let json: Value = serde_json::from_str(&contents).ok()?;
        json["version"]
            .as_str()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn validate_plugin_root(plugin_root: &Path) -> Result<()> {
        plugin_root
            .join("package.json")
            .exists()
            .then_some(())
            .ok_or_else(|| anyhow!("Could not find package.json"))?;

        plugin_root
            .join("dist")
            .exists()
            .then_some(())
            .ok_or_else(|| anyhow!("Could not find dist/ (build frontend first)"))?;

        // plugin.json validation happens in read_plugin_name()
        Ok(())
    }

    fn zip_path(
        &self,
        filename: &str,
        path: PathBuf,
        zip: &mut ZipWriter<File>,
        opts: FileOptions,
    ) -> Result<()> {
        let name = path
            .strip_prefix(&self.plugin_root)
            .map(|name| name.to_path_buf())
            .and_then(|name| {
                name.strip_prefix("defaults")
                    .map(|path| path.to_path_buf())
                    .or(Ok(name))
            })
            .map(|name| Path::new(filename).join(name))?;

        info!("Zipping {:?}", name);

        if path.is_file() {
            let bytes = std::fs::read(&path).unwrap();

            let method = match self.compression_method {
                CompressMethod::Deflate => CompressionMethod::Deflated,
                CompressMethod::Store => CompressionMethod::Stored,
            };

            let mut opts = opts.compression_method(method);

            if method == CompressionMethod::Deflated {
                opts = match self.compression_level {
                    Some(level) => opts.compression_level(Some(level)),
                    None => opts.compression_level(Some(9)),
                }
            }

            zip.start_file(name.to_str().unwrap(), opts)?;
            zip.write_all(&bytes)?;
        } else if !name.as_os_str().is_empty() {
            zip.add_directory(name.to_str().unwrap(), opts)?;
        }

        Ok(())
    }

    pub fn run(&self) -> Result<()> {
        Self::validate_plugin_root(&self.plugin_root)?;

        if !self.output_root.exists() {
            std::fs::create_dir(&self.output_root)?;
        }

        let plugin_name = Self::read_plugin_name(&self.plugin_root)?;

        // Root folder name inside the zip (same as `plugin build`).
        let filename: String = match &self.output_filename_source {
            FilenameSource::PluginName => plugin_name,
            FilenameSource::Directory => self
                .plugin_root
                .file_name()
                .ok_or_else(|| anyhow!("Could not determine plugin directory name"))?
                .to_string_lossy()
                .to_string(),
        };

        let pkg_version = Self::read_package_version(&self.plugin_root);
        let zip_stem = match self.zip_basename {
            PackZipBasename::Name => filename.clone(),
            PackZipBasename::NameVersion => {
                let v = self
                    .zip_version
                    .clone()
                    .or(pkg_version)
                    .filter(|s| !s.is_empty());
                match v {
                    Some(ver) => format!("{}-{}", filename, ver),
                    None => {
                        log::warn!(
                            "pack: `name-version` zip name needs a version; set package.json \"version\" or pass --zip-version; using name-only file name"
                        );
                        filename.clone()
                    }
                }
            }
        };

        let zip_filename = format!(
            "{}{}.zip",
            zip_stem,
            if self.build_with_dev { "-dev" } else { "" }
        );

        let file = std::fs::File::create(&self.output_root.join(zip_filename))
            .context("Could not create zip file")?;
        let mut zip = zip::ZipWriter::new(file);

        struct DirDirective<'a> {
            path: &'a str,
            mandatory: bool,
            permissions: FileOptions,
        }

        let directories = vec![
            DirDirective {
                path: "dist",
                mandatory: true,
                permissions: FileOptions::default(),
            },
            DirDirective {
                path: "bin",
                mandatory: false,
                permissions: FileOptions::default().unix_permissions(0o755),
            },
            DirDirective {
                path: "defaults",
                mandatory: false,
                permissions: FileOptions::default(),
            },
            DirDirective {
                path: "py_modules",
                mandatory: false,
                permissions: FileOptions::default().unix_permissions(0o755),
            },
        ];

        let expected_files = vec![
            "LICENSE",
            "main.py",
            "package.json",
            "plugin.json",
            "README.md",
        ]
        .into_iter()
        .map(|f| f.to_string());

        let python_files = glob(&format!("{}/*.py", self.plugin_root.to_string_lossy()))
            .unwrap()
            .map(|f| {
                f.unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .into_iter();

        let files = expected_files.chain(python_files).unique();

        for file in files {
            let full_path = self.plugin_root.join(&file);
            self.zip_path(&filename, full_path, &mut zip, Default::default())?;
        }

        for directory in directories {
            let full_path = self.plugin_root.join(&directory.path);

            if directory.mandatory == false && !full_path.exists() {
                info!("Optional directory {} not found. Continuing", &directory.path);
                continue;
            }

            let dir_entries = WalkDir::new(full_path).follow_links(self.follow_symlinks);
            for entry in dir_entries {
                let file = entry?;
                self.zip_path(
                    &filename,
                    file.path().to_path_buf(),
                    &mut zip,
                    directory.permissions,
                )?;
            }
        }

        zip.finish()?;
        Ok(())
    }

    pub fn new(
        plugin_root: PathBuf,
        output_root: PathBuf,
        follow_symlinks: bool,
        output_filename_source: FilenameSource,
        compression_method: CompressMethod,
        compression_level: Option<i32>,
        build_with_dev: bool,
        zip_basename: PackZipBasename,
        zip_version: Option<String>,
    ) -> Result<Self> {
        if !plugin_root.exists() {
            return Err(anyhow!("Could not find plugin root"));
        }

        Ok(Self {
            plugin_root: plugin_root
                .canonicalize()
                .context("Could not canonicalize plugin root")?,
            output_root,
            follow_symlinks,
            output_filename_source,
            compression_method,
            compression_level,
            build_with_dev,
            zip_basename,
            zip_version,
        })
    }
}

