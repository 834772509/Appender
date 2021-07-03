// 禁用变量命名警告
#![allow(non_snake_case)]

#[macro_use]
extern crate lazy_static;

mod validator;
mod util;
mod core;

#[cfg(test)]
mod test;

use std::path::{PathBuf, Path};
use clap::{Arg, SubCommand, AppSettings, App};
use crate::validator::is_valid_path;
use crate::core::{addResource, exportResource};


fn main() {
    let matches = App::new(clap::crate_name!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(clap::crate_version!())
        .subcommands(vec![
            // 增加
            SubCommand::with_name("add")
                .about("Add Resources")
                .arg(Arg::with_name("TargetFile")
                    .help("Target File Path")
                    .required(true)
                    .validator(is_valid_path)
                    .index(1))
                .arg(Arg::with_name("Resources")
                    .help("Resources")
                    .required(true)
                    .validator(is_valid_path)
                    .index(2))
                .arg(Arg::with_name("id")
                    .help("Resources ID")
                    .required(true)
                    .index(3))
                .arg(Arg::with_name("newFilePath")
                    .help("new file path")
                    .index(4)
                // .arg(Arg::with_name("compression")
                //     .short("c")
                //     .long("compression")
                //     .value_name("compression")
                //     .default_value("5")
                //     .help("compression")
                ),
            // 释放资源
            SubCommand::with_name("export")
                .about("export Resources")
                .arg(Arg::with_name("TargetFile")
                    .help("Target File Path")
                    .required(true)
                    .validator(is_valid_path)
                    .index(1))
                .arg(Arg::with_name("id")
                    .help("Resources ID")
                    .required(true)
                    .index(2))
                .arg(Arg::with_name("outputPath")
                    .help("outputPath")
                    .required(true)
                    .index(3)),
        ])
        .get_matches();

    // 增加资源
    if let Some(matches) = matches.subcommand_matches("add") {
        let targerFile = PathBuf::from(matches.value_of("TargetFile").unwrap());
        let resources = PathBuf::from(matches.value_of("Resources").unwrap());
        let id = matches.value_of("id").unwrap();
        let outputFile = matches.value_of("newFilePath").map(|path| Path::new(path));
        // let compression = matches.value_of("compression").unwrap();

        println!("Adding {} resources id {} to {}......", resources.to_str().unwrap(), id, targerFile.to_str().unwrap());
        if let Err(e) = addResource(&*targerFile, &*resources, id,None, outputFile) {
            println!("Resource increase failed: {}",e);
            return;
        }
        println!("Resources increase successfully");
    }

    // 释放资源
    if let Some(matches) = matches.subcommand_matches("export") {
        let targetFile = PathBuf::from(matches.value_of("TargetFile").unwrap());
        let id = matches.value_of("id").unwrap();
        let outputPath = PathBuf::from(matches.value_of("outputPath").unwrap());
        println!("export resources id \"{}\" from {} to {}", id, targetFile.to_str().unwrap(), outputPath.to_str().unwrap());
        if let Err(e) = exportResource(&*targetFile, id, &*outputPath) {
            println!("Resource export failed: {}",e);
            return;
        }
        println!("Resource export successfully");
    }
}
