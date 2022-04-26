use crate::util::fs;
use crate::util::regex::matches_compile_swift_sources;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::CompileFlags;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompileCommand {
    /// Module name
    /// NOTE: not sure if this required
    #[serde(
        rename(serialize = "module_name"),
        skip_serializing_if = "Option::is_none"
    )]
    pub name: Option<String>,

    /// The path of the main file for the compilation, which may be relative to `directory`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,

    /// The wroking directory for the compilation
    pub directory: String,

    /// The compile command, this is alias with commandLine or split form of command
    pub command: String,

    /// Source code files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<String>>,

    /// For SwiftFileList
    pub file_lists: Vec<String>,

    /// The name of the build output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,

    /// Index store path. Kept for the caller to further process.
    #[serde(skip)]
    pub index_store_path: Option<String>,
}

impl CompileCommand {
    pub fn can_parse(line: &str) -> bool {
        matches_compile_swift_sources(line)
    }

    /// Parse starting from current line as swift module
    /// Matching r"^CompileSwiftSources\s*"
    pub fn swift_module(lines: &Vec<String>, cursor: usize) -> Option<CompileCommand> {
        let directory = match lines.get(cursor) {
            Some(s) => s.trim().replace("cd ", ""),
            None => {
                tracing::error!("Found COMPILE_SWIFT_MODULE_PATERN but no more lines");
                return None;
            }
        };

        let command = match lines.get(cursor) {
            Some(s) => s.trim().to_string(),
            None => {
                tracing::error!("Found COMPILE_SWIFT_MODULE_PATERN but couldn't extract command");
                return None;
            }
        };

        let arguments = match shell_words::split(&command) {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Fail to create swift module command {e}");
                return None;
            }
        };

        // NOTE: This is never changed
        let file = Default::default();
        let output = Default::default();
        let mut name = Default::default();
        let mut files = Vec::default();
        let mut file_lists = Vec::default();
        let mut index_store_path = None;

        for i in 0..arguments.len() {
            let val = &arguments[i];
            if val == "-module-name" {
                name = Some(arguments[i + 1].to_owned());
            } else if val == "-index-store-path" {
                index_store_path = Some(arguments[i + 1].to_owned());
            } else if val.ends_with(".swift") {
                files.push(val.to_owned());
            } else if val.ends_with(".SwiftFileList") {
                file_lists.push(val.replace("@", "").to_owned());
            }
        }

        let command = Self {
            directory,
            command: arguments.join(" "),
            name,
            file,
            output,
            files: if files.is_empty() { None } else { Some(files) },
            file_lists,
            index_store_path,
        };

        tracing::debug!("Got Swift commands for {}", command.directory);
        tracing::trace!("{:#?}", command);
        Some(command)
    }

    /// Get a HashMap of workspace files and compile flags
    pub fn compile_flags<'a>(&'a self) -> Result<HashMap<PathBuf, CompileFlags>> {
        let mut info: HashMap<PathBuf, CompileFlags, _> = HashMap::default();
        let flags = CompileFlags::from_command(&self.command)?;

        // Swift File Lists
        self.file_lists.iter().for_each(|path| {
            let path = &PathBuf::from(path.as_str());
            match fs::get_files_list(path) {
                Ok(flist) => {
                    flist.into_iter().for_each(|file_path: PathBuf| {
                        info.insert(file_path, flags.clone());
                    });
                }
                Err(e) => tracing::error!("Fail to get file lists {e}"),
            };
        });

        // Swift Module Files
        if let Some(ref files) = self.files {
            for file in files {
                let file_path = PathBuf::from(file);
                info.insert(file_path, flags.clone());
            }
        };

        // Single File Command
        if let Some(ref file) = self.file {
            let file_path = PathBuf::from(file);

            info.insert(file_path, flags.clone());
        }

        Ok(info)
    }
}
