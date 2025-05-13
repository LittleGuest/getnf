use std::{env, fs, io::Read, path::PathBuf};

use clap::{Parser, Subcommand};
use dialoguer::MultiSelect;
use reqwest::{header::USER_AGENT, IntoUrl};
use serde_json::Value;

const NERD_FONTS_API: &str = "https://api.github.com/repos/ryanoasis/nerd-fonts";
const NERD_FONTS_REPO: &str = "https://github.com/ryanoasis/nerd-fonts";

/// install nerd fonts
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// install/uninstall/list/update Nerd Fonts for all users
    #[arg(short, long)]
    global: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// show the list of installed Nerd Fonts
    #[command(short_flag = 'l')]
    ListInstalled,
    /// show the list of all Nerd Fonts
    #[command(short_flag = 'L')]
    ListAll,
    /// directly install the specified Nerd Fonts
    #[command(short_flag = 'i')]
    Install {
        /// font name
        #[arg(short)]
        fonts: Option<String>,
    },
    /// uninstall the specified Nerd Fonts
    #[command(short_flag = 'u')]
    Uninstall {
        /// font name
        #[arg(short)]
        fonts: Option<String>,
    },
    /// update all installed Nerd Fonts
    #[command(short_flag = 'U')]
    Update {
        /// font name
        #[arg(short)]
        fonts: Option<String>,
    },
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
                let xdg_data_home = env::var("XDG_DATA_HOME")
                    .unwrap_or_else(|_| format!("{}/.local/share", env::var("HOME").unwrap()));
                PathBuf::from(xdg_data_home).join("fonts")
            }
        }
        "macos" => {
            if global {
                "/Library/Fonts".into()
            } else {
                PathBuf::from(env::var("HOME").unwrap()).join("Library/Fonts")
            }
        }
        "windows" => {
            if global {
                let windir = env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
                PathBuf::from(windir).join("Fonts")
            } else {
                let local_appdata =
                    std::env::var("LOCALAPPDATA").expect("未找到 LOCALAPPDATA 环境变量");
                PathBuf::from(local_appdata)
                    .join("Microsoft")
                    .join("Windows")
                    .join("Fonts")
            }
        }
        _ => "".into(),
    }
}

fn latest_release_version() -> String {
    let body = request(NERD_FONTS_API.to_string() + "/releases/latest");
    let tag = &body["tag_name"];
    tag.as_str().unwrap().into()
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

fn list_remote_fonts() -> Vec<String> {
    let body = request(NERD_FONTS_API.to_string() + "/contents/patched-fonts?ref=master");
    let body = body.as_array().unwrap();
    let mut fonts = vec![];
    for font in body {
        fonts.push(font["name"].as_str().unwrap().into());
    }
    fonts
}

fn install_fonts(fonts: &[String], global: bool) {
    if fonts.is_empty() {
        return;
    }

    let latest = latest_release_version();

    for font in fonts {
        let mut file_name = PathBuf::new();
        file_name.push(font.to_string() + ".tar.xz");
        let url = NERD_FONTS_REPO.to_string()
            + "/releases/download/"
            + &latest
            + "/"
            + file_name.to_string_lossy().to_string().as_ref();

        let mut archive = arkiv::Archive::download(url).unwrap();
        archive.unpack(font_dir(global).join(font)).unwrap();
    }
}

fn uninstall_fonts(fonts: &[String], global: bool) {
    for font in fonts {
        fs::remove_dir_all(font_dir(global).join(font)).ok();
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::ListInstalled => {
            list_installed_fonts(cli.global)
                .into_iter()
                .for_each(|f| println!("{f}"));
        }
        Commands::ListAll => {
            list_remote_fonts()
                .into_iter()
                .for_each(|f| println!("{f}"));
        }
        Commands::Install { fonts } => {
            let choosed_fonts = if let Some(fonts) = fonts {
                fonts.split(',').map(|f| f.to_string()).collect::<Vec<_>>()
            } else {
                let fonts = list_remote_fonts();

                let selection = MultiSelect::new()
                    .with_prompt("choose fonts")
                    .items(&fonts)
                    .interact()
                    .unwrap();
                fonts
                    .into_iter()
                    .enumerate()
                    .filter(|(i, _)| selection.contains(i))
                    .map(|(_, f)| f)
                    .collect::<Vec<_>>()
            };

            install_fonts(&choosed_fonts, cli.global);
        }
        Commands::Uninstall { fonts } => {
            let choosed_fonts = if let Some(fonts) = fonts {
                fonts.split(',').map(|f| f.to_string()).collect::<Vec<_>>()
            } else {
                let fonts = list_installed_fonts(cli.global);

                let selection = MultiSelect::new()
                    .with_prompt("choose fonts")
                    .items(&fonts)
                    .interact()
                    .unwrap();
                fonts
                    .into_iter()
                    .enumerate()
                    .filter(|(i, _)| selection.contains(i))
                    .map(|(_, f)| f)
                    .collect::<Vec<_>>()
            };

            uninstall_fonts(&choosed_fonts, cli.global);
        }
        Commands::Update { fonts } => {
            let choosed_fonts = if let Some(fonts) = fonts {
                fonts.split(',').map(|f| f.to_string()).collect::<Vec<_>>()
            } else {
                let fonts = list_remote_fonts();

                let selection = MultiSelect::new()
                    .with_prompt("choose fonts")
                    .items(&fonts)
                    .interact()
                    .unwrap();
                fonts
                    .into_iter()
                    .enumerate()
                    .filter(|(i, _)| selection.contains(i))
                    .map(|(_, f)| f)
                    .collect::<Vec<_>>()
            };

            install_fonts(&choosed_fonts, cli.global);
        }
    }
}
