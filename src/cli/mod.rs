pub mod plugin;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct CLI {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::ValueEnum, Clone)]
pub enum ContainerEngine {
    Docker,
    Podman,
}

impl ContainerEngine {
    pub fn bin_name(&self) -> &str {
        match self {
            Self::Docker => "docker",
            Self::Podman => "podman",
        }
    }
}


#[derive(clap::ValueEnum, Clone)]
pub enum CompressMethod {
    Deflate,
    Store,
}

#[derive(Subcommand)]
pub enum Command {
    Plugin(PluginCLI),
}

#[derive(Parser)]
pub struct PluginCLI {
    #[command(subcommand)]
    command: PluginCommand,
}

#[derive(clap::ValueEnum, Clone)]
pub enum FilenameSource {
    PluginName,
    Directory,
}

/// How to name the output `.zip` file for `plugin pack`.
#[derive(clap::ValueEnum, Clone, Copy, Default)]
pub enum PackZipBasename {
    /// e.g. `Fantastic.zip`
    Name,
    /// e.g. `Fantastic-0.6.0-alpha1.zip` (version from `package.json`, or `--zip-version`)
    #[default]
    NameVersion,
}

#[derive(Subcommand)]
pub enum PluginCommand {
    Build {
        #[arg(default_value = "./")]
        plugin_path: PathBuf,

        #[arg(short, long, default_value = "./out")]
        output_path: PathBuf,

        #[arg(short, long, default_value = "/tmp/decky")]
        tmp_output_path: PathBuf,

        #[arg(short, long, default_value = "false")]
        build_as_root: bool,

        #[arg(short = 'd', long, default_value = "false")]
        build_with_dev: bool,

        #[arg(short = 'S', long, default_value = "true")]
        follow_symlinks: bool,

        #[arg(short = 's', long, value_enum, default_value = "plugin-name")]
        output_filename_source: FilenameSource,

        #[arg(short = 'e', long = "engine", default_value = "docker")]
        container_engine: ContainerEngine,

        #[arg(short = 'm', long, default_value = "deflate")]
        compression_method: CompressMethod,

        #[arg(short = 'l', long)]
        compression_level: Option<i32>,
    },
    Pack {
        #[arg(default_value = "./")]
        plugin_path: PathBuf,

        #[arg(short, long, default_value = "./out")]
        output_path: PathBuf,

        #[arg(short = 'S', long, default_value = "true")]
        follow_symlinks: bool,

        #[arg(short = 's', long, value_enum, default_value = "plugin-name")]
        output_filename_source: FilenameSource,

        #[arg(short = 'm', long, default_value = "deflate")]
        compression_method: CompressMethod,

        #[arg(short = 'l', long)]
        compression_level: Option<i32>,

        #[arg(short = 'd', long, default_value = "false")]
        build_with_dev: bool,

        /// Zip file name: `name` → `Fantastic.zip`; `name-version` → `Fantastic-0.6.0-alpha1.zip`
        #[arg(long, value_enum, default_value_t = PackZipBasename::NameVersion)]
        zip_basename: PackZipBasename,

        /// Override the version segment in the zip filename (only used with `name-version`)
        #[arg(long = "zip-version")]
        zip_version: Option<String>,
    },
    New,
    Deploy {
        #[arg(default_value = "./")]
        plugin_path: PathBuf,

        #[arg(short, long, default_value = "./out")]
        output_path: PathBuf,

        #[arg(short, long, default_value = "/tmp/decky")]
        tmp_output_path: PathBuf,

        #[arg(short, long, default_value = "false")]
        build_as_root: bool,

        #[arg(short = 'd', long, default_value = "false")]
        build_with_dev: bool,

        #[arg(short = 's', long, value_enum, default_value = "plugin-name")]
        output_filename_source: FilenameSource,

        #[arg(short = 'e', long = "engine", default_value = "docker")]
        container_engine: ContainerEngine,

        #[arg(short = 'm', long, default_value = "deflate")]
        compression_method: CompressMethod,

        #[arg(short = 'l', long)]
        compression_level: Option<i32>,

        #[arg(short = 'S', long, default_value = "true")]
        follow_symlinks: bool,

        #[arg(short = 'i', long)]
        deck_ip: Option<String>,

        #[arg(short = 'p', long)]
        deck_port: Option<String>,

        #[arg(short = 'x', long)]
        deck_pass: Option<String>,

        #[arg(short = 'k', long)]
        deck_key: Option<String>,

        #[arg(short = 'c', long)]
        deck_dir: Option<String>,
    },
}
