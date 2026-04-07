use super::{PluginCLI, PluginCommand};
use anyhow::Result;

pub mod build;
pub mod deploy;
pub mod pack;

pub async fn parse(args: &PluginCLI) -> Result<()> {
    match &args.command {
        PluginCommand::Build {
            plugin_path,
            output_path,
            tmp_output_path,
            build_as_root,
            build_with_dev,
            follow_symlinks,
            output_filename_source,
            container_engine,
            compression_method,
            compression_level,
        } => {
            build::Builder::new(
                plugin_path.into(),
                output_path.into(),
                tmp_output_path.into(),
                build_as_root.clone(),
                build_with_dev.clone(),
                follow_symlinks.clone(),
                output_filename_source.clone(),
                container_engine.clone(),
                compression_method.clone(),
                compression_level.clone(),
            )?
            .run()
            .await
        }
        PluginCommand::Pack {
            plugin_path,
            output_path,
            follow_symlinks,
            output_filename_source,
            compression_method,
            compression_level,
            build_with_dev,
            zip_basename,
            zip_version,
        } => {
            pack::Packer::new(
                plugin_path.into(),
                output_path.into(),
                follow_symlinks.clone(),
                output_filename_source.clone(),
                compression_method.clone(),
                compression_level.clone(),
                build_with_dev.clone(),
                zip_basename.clone(),
                zip_version.clone(),
            )?
            .run()
        }
        PluginCommand::New => todo!(),
        PluginCommand::Deploy {
            plugin_path,
            output_path,
            tmp_output_path,
            build_as_root,
            build_with_dev,
            follow_symlinks,
            output_filename_source,
            container_engine,
            deck_ip,
            deck_port,
            deck_pass,
            deck_key,
            deck_dir,
            compression_method,
            compression_level,
        } => {
            deploy::Deployer::new(
                plugin_path.into(),
                output_path.into(),
                tmp_output_path.into(),
                build_as_root.clone(),
                build_with_dev.clone(),
                follow_symlinks.clone(),
                output_filename_source.clone(),
                container_engine.clone(),
                compression_method.clone(),
                compression_level.clone(),
                deck_ip.clone(),
                deck_port.clone(),
                deck_pass.clone(),
                deck_key.clone(),
                deck_dir.clone(),
            )?
            .run()
            .await
        }
    }
}
