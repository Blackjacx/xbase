#[cfg(feature = "gen")]
use std::{fs, path::PathBuf, process::Command};

#[cfg(feature = "gen")]
fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let ts_root = root.join("vscode");
    let lua_root = root.join("lua").join("xbase");
    let ts_types_path = ts_root.join("xbase").join("types.ts");
    let ts_constants_path = ts_root.join("xbase").join("constants.ts");
    let lua_constants_path = lua_root.join("constants.lua");

    gen_ts_types_file(ts_types_path);
    gen_ts_constant(ts_constants_path);
    gen_lua_constant(lua_constants_path);
    eslint_format(ts_root)
}

#[cfg(not(feature = "gen"))]
fn main() {}

#[cfg(feature = "gen")]
fn gen_ts_types_file(path: PathBuf) {
    use typescript_type_def::{write_definition_file, DefinitionFileOptions};
    use xbase::{broadcast::*, server::*, types::*, *};
    let mut buf = Vec::new();
    let content = read_file_content(&path);
    let mut options = DefinitionFileOptions::default();

    options.root_namespace = None;
    options.header = None;

    type Requests = (
        Request,
        RunRequest,
        RegisterRequest,
        DropRequest,
        GetProjectInfoRequest,
    );
    type Responses = (Response, ServerError);
    type Transports = (
        ProjectInfo,
        TargetInfo,
        Runners,
        Operation,
        BuildSettings,
        DeviceLookup,
    );
    type Messages = (Message, ContentLevel, TaskKind, TaskStatus);
    type API = (Messages, Transports, Responses, Requests);

    write_definition_file::<_, API>(&mut buf, options).unwrap();

    let generated = String::from_utf8(buf).unwrap();

    std::fs::write(&path, content + "\n" + &generated).expect("failed to write typescript types");
}

#[cfg(feature = "gen")]
fn gen_ts_constant(path: PathBuf) {
    use xbase::*;
    let mut output = read_file_content(&path);

    macro_rules! export {
        ($key:ident) => {
            output += &format!("export const XBASE_{} = '{}'\n", stringify!($key), &*$key)
        };
    }

    export!(SOCK_ADDR);

    output += &format!(
        "export const XBASE_BIN_ROOT = '{BIN_ROOT}'.replace('$HOME', process.env.HOME!)\n"
    );

    std::fs::write(&path, output).expect("failed to write typescript types");
}

#[cfg(feature = "gen")]
fn gen_lua_constant(path: PathBuf) {
    use xbase::*;
    let mut output = read_file_content(&path);

    macro_rules! export {
        ($key:ident) => {
            output += &format!("M.{} = '{}'\n", stringify!($key), &*$key)
        };
    }

    export!(SOCK_ADDR);
    output += &format!("M.BIN_ROOT = string.gsub('{BIN_ROOT}', '$HOME', vim.env.HOME)\n",);
    output += "\n\nreturn M";

    std::fs::write(&path, output).expect("failed to write typescript types");
}

#[cfg(feature = "gen")]
fn read_file_content(file: &PathBuf) -> String {
    let content = if file.exists() {
        fs::read_to_string(&file).unwrap()
    } else {
        Default::default()
    };
    let lines = content.split('\n').collect::<Vec<&str>>();
    let marker = lines
        .iter()
        .position(|line| line.contains("AUTOGENERATED"))
        .unwrap_or_default();

    String::from(&lines[0..marker + 1].join("\n")) + "\n"
}

#[cfg(feature = "gen")]
fn eslint_format(ts_root: PathBuf) {
    let format_status = Command::new(ts_root.join("node_modules").join(".bin").join("prettier"))
        .arg(ts_root.join("xbase").join("types.ts"))
        .arg("--write")
        .output()
        .expect("failed to run pnpm lint on ts_out")
        .status;

    assert!(format_status.success(), "pnpm lint failed");
}
