use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use clap::Parser;
use dialoguer::MultiSelect;
use reqwest::{get, header::USER_AGENT, IntoUrl, StatusCode};
use serde_json::Value;

const NERD_FONTS_API: &str = "https://api.github.com/repos/ryanoasis/nerd-fonts";
const NERD_FONTS_REPO: &str = "https://github.com/ryanoasis/nerd-fonts";

/// Install Nerd Fonts
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Cli {
    /// keep the downloaded font archives
    #[command(subcommand)]
    KeepArchives,
    /// show already installed Nerd Fonts in the menu
    #[command(subcommand)]
    All,
    /// show the list of installed Nerd Fonts
    #[command(subcommand)]
    ListInstalled,
    /// show the list of all Nerd Fonts
    #[command(subcommand)]
    ListAll,
    /// directly install the specified Nerd Fonts
    Install {
        /// font name
        #[arg(short)]
        fonts: String,
        /// install/uninstall/list/update Nerd Fonts for all users
        #[arg(short)]
        global: bool,
    },
    /// uninstall the specified Nerd Fonts
    #[command(subcommand)]
    Uninstall,
    /// update all installed Nerd Fonts
    #[command(subcommand)]
    Update,
}

impl Cli {
    /// home path
    fn home() -> PathBuf {
        env!("HOME").into()
    }

    /// download path
    fn down_path() -> PathBuf {
        let mut path = Self::home();
        path.push("Downloads");
        path.push("getnf");
        path
    }

    /// getnf path
    fn getnf_path() -> PathBuf {
        let mut path = Self::home();
        path.push("/.local/share/getnf");
        path
    }

    /// getnf path
    fn font_releases_path() -> PathBuf {
        let mut path = Self::home();
        path.push("/.local/share/getnf/releases");
        path
    }

    /// getnf path
    fn release_file_path() -> PathBuf {
        let mut path = Self::home();
        path.push("/.local/share/getnf/release");
        path
    }

    /// getnf path
    fn all_fonts_file_path() -> PathBuf {
        let mut path = Self::home();
        path.push("/.local/share/getnf/all_fonts");
        path
    }
}

fn request(url: impl IntoUrl) -> Value {
    let client = reqwest::blocking::Client::new();
    let mut resp = client.get(url).header(USER_AGENT, "getnf").send().unwrap();
    let mut buf = String::new();
    resp.read_to_string(&mut buf).ok();
    serde_json::from_str::<Value>(&buf).unwrap()
}

/// font dir
fn font_dir(global: bool) -> PathBuf {
    match std::env::consts::OS {
        "linux" => {
            if global {
                "/usr/local/share/fonts/".into()
            } else {
                (env!("HOME").to_string() + "/.local/share/fonts/").into()
            }
        }
        "macos" => {
            if global {
                "/Library/Fonts".into()
            } else {
                (env!("HOME").to_string() + "/Library/Fonts").into()
            }
        }
        "windows" => "".into(),
        _ => "".into(),
    }
}

fn create_dirs(path: &Path) {
    fs::create_dir(path).ok();
}

fn latest_release_version() -> String {
    let body = request(NERD_FONTS_API.to_string() + "/releases/latest");
    let tag = &body["tag_name"];
    tag.as_str().unwrap().into()
}

fn list_remote_all_fonts_() -> Vec<String> {
    let body = request(NERD_FONTS_API.to_string() + "/contents/patched-fonts?ref=master");
    let body = body.as_array().unwrap();
    let mut fonts = vec![];
    for font in body {
        fonts.push(font["name"].as_str().unwrap().into());
    }
    fonts
}

fn create_all_fonts_file() {
    let body = request(NERD_FONTS_API.to_string() + "/contents/patched-fonts?ref=master");
    let body = body.as_array().unwrap();
    let mut f = fs::File::create(Cli::all_fonts_file_path()).unwrap();
    for font in body {
        f.write_all(font["name"].to_string().as_bytes()).ok();
    }
    f.flush().ok();
}

fn list_installed_fonts(global: bool) -> Vec<String> {
    let dir = font_dir(global);
    let dirs = fs::read_dir(dir).unwrap();
    let mut fds = vec![];
    for dir in dirs {
        fds.push(dir.unwrap().file_name().to_string_lossy().to_string());
    }
    fds
}

fn install_fonts(fonts: &[&String]) {
    let client = reqwest::blocking::Client::new();
    let latest = latest_release_version();

    for font in fonts {
        let mut file_name = PathBuf::new();
        file_name.push(font.to_string() + ".tar.xz");
        let resp = client
            .get(
                NERD_FONTS_REPO.to_string()
                    + "/releases/download/"
                    + &latest
                    + "/"
                    + file_name.to_string_lossy().to_string().as_ref(),
            )
            .header(USER_AGENT, "getnf")
            .send()
            .unwrap();
        if resp.status() != StatusCode::OK {
            eprintln!("{}", resp.text().unwrap());
            return;
        }

        let mut path = Cli::down_path();
        path.push(&file_name);
        if path.exists() {
            fs::remove_file(&path).ok();
        }
        let mut f = fs::File::create_new(path).unwrap();
        f.write_all(&resp.bytes().unwrap()).ok();
    }

    // local opts=("--location" "--remote-header-name" "--remote-name")
    // if [[ $ON_MAC == "false" ]]; then
    // 	opts+=("--silent")
    // 	echo -n "Downloading $1...  "
    // 	trap stop_loading_animation SIGINT
    // 	start_loading_animation
    // 	curl "${opts[@]}" "$NERDFONTSREPO/releases/download/$RELEASE/$1.tar.xz"
    // 	stop_loading_animation
    // 	confirm "Done"
    // else
    // 	# TODO: figure out how to disable messages about bg process termination
    // 	opts+=("--progress-bar")
    // 	info "Downloading $1... "
    // 	curl "${opts[@]}" "$NERDFONTSREPO/releases/download/$RELEASE/$1.tar.xz"
    // 	erase_line
    // 	erase_line
    // 	echo "Downloading $1... ${GREEN}Done${RESET}"
    // fi
}

fn main() {
    let cli = Cli::parse();
    match cli {
        Cli::KeepArchives => todo!(),
        Cli::All => todo!(),
        Cli::ListInstalled => todo!(),
        Cli::ListAll => todo!(),
        Cli::Install { fonts, global } => {
            // fs::create_dir_all(font_dir(global)).ok();
            // fs::create_dir_all(DOWN_DIR).ok();
            // fs::create_dir_all(FONT_RELEASES_DIR).ok();
            // let latest = latest_release_version();
            // create_all_fonts_file();
            // list_installed_fonts(global);

            let fonts = list_remote_all_fonts_();

            let selection = MultiSelect::new()
                .with_prompt("choose fonts")
                .items(&fonts)
                .interact()
                .unwrap();

            let choosed_fonts = fonts
                .iter()
                .enumerate()
                .filter(|(i, _)| selection.contains(i))
                .map(|(_, f)| f)
                .collect::<Vec<_>>();
            install_fonts(&choosed_fonts);
        }
        Cli::Uninstall => todo!(),
        Cli::Update => todo!(),
    }
}
