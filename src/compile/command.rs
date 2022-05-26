use crate::util::fs;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::CompileFlags;

/// Single Compilation Database Command Representation
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompilationCommand {
    /// Module name. NOTE: not sure if this required
    #[serde(
        rename(serialize = "module_name"),
        skip_serializing_if = "Option::is_none"
    )]
    pub name: Option<String>,
    /// The path of the main file for the compilation, which may be relative to `directory`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,
    /// The working directory for the compilation
    pub directory: String,
    /// The compile command, this is alias with commandLine or split form of command
    pub command: String,
    /// Source code files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<PathBuf>>,
    /// For SwiftFileList
    pub file_lists: Vec<PathBuf>,
    /// The name of the build output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    /// Index store path. Kept for the caller to further process.
    #[serde(skip)]
    pub index_store_path: Option<PathBuf>,
}

impl CompilationCommand {
    /// Generate a map of filespaths in workspaces and their compilation flags
    ///
    /// Examples
    ///
    /// ```no_run
    /// use xbase::compile::CompilationCommand;
    /// let build_logs_lines = vec![];
    /// let cursor_where_the_line_matches = 1;
    /// let command = CompilationCommand::swift_module(&build_logs_lines, cursor_where_the_line_matches);
    ///
    /// command.compile_flags();
    /// ```
    pub fn compile_flags<'a>(&'a self) -> Result<HashMap<PathBuf, CompileFlags>> {
        let (mut info, flags) = (
            HashMap::default(),
            CompileFlags::from_command(&self.command)?,
        );

        // Swift File Lists
        self.file_lists.iter().for_each(|path| {
            match fs::get_files_list(&path) {
                Ok(file_list) => {
                    file_list.into_iter().for_each(|file_path: PathBuf| {
                        info.insert(file_path, flags.clone());
                    });
                }
                Err(e) => tracing::error!("Fail to get file lists {e}"),
            };
        });

        // Swift Module Files
        self.files.as_ref().map(|files| {
            files.iter().for_each(|file| {
                info.insert(file.clone(), flags.clone());
            })
        });

        // Single File Command
        self.file
            .as_ref()
            .map(|file| info.insert(file.clone(), flags.clone()));

        Ok(info)
    }
}
