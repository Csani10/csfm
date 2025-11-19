use std::{fmt::Error, fs, io, iter, path::{Path, PathBuf}, vec};

use iced::{
    self, Alignment, Border, Element, Length, Task, Theme,
    advanced::graphics::{core::Element as CoreElement, text::cosmic_text::ttf_parser::loca},
    border::Radius,
    widget::{button::{self, Style}, column, container, row, scrollable, text, text_input},
    window::Id,
};
use iced_aw::{ContextMenu, DropDown, Menu, MenuBar, context_menu, drop_down, menu::Item};
use serde::Deserialize;

#[derive(Debug, Clone)]
enum Message {
    PathChanged(String),
    CDToPath,
    CD(PathBuf),
    QuitApp(Option<Id>),
    ToggleSidebar,
    Up,
    None,
}

struct CsFM {
    config: Config,
    path: PathBuf,
    current_files: Vec<(PathBuf, bool)>,
    sidebar_open: bool
}

#[derive(Clone, Deserialize)]
struct Config {
    pub theme: String,
    pub sidebar_loc: Vec<Location>
}

#[derive(Clone, Deserialize)]
struct Location {
    pub title: String,
    pub path: String
}

fn theme(_state: &CsFM) -> Theme {
    Theme::GruvboxDark
}

fn update(state: &mut CsFM, message: Message) -> Task<Message> {
    match message {
        Message::None => {
            Task::none()
        }
        Message::PathChanged(s) => {
            let path = PathBuf::from(s);
            let path_str = path.clone().to_string_lossy().to_string();
            state.path = path;
            println!("{path_str}");
            Task::none()
        }
        Message::CDToPath => {
            let files = get_files(PathBuf::from(&state.path));
            
            if !files.is_empty() {
                state.current_files = files.clone();
            }

            for file in files {
                let path = file.0.to_string_lossy().to_string();
                let is_dir = file.1;
                println!("{path}, {is_dir}");
            }

            Task::none()
        }
        Message::Up => {
            state.path = state.path.parent().unwrap_or(PathBuf::from("/").as_path()).to_path_buf();

            Task::done(Message::CDToPath)
        }
        Message::CD(path) => {
            state.path = path;

            Task::done(Message::CDToPath)
        }
        Message::QuitApp(id) => {
            iced::window::close(id.unwrap())
        }
        Message::ToggleSidebar => {
            state.sidebar_open = !state.sidebar_open;

            Task::none()
        }
    }
}

fn dir_button(state: &'_ CsFM) -> iced::widget::button::Style {
    let theme = theme(state);
    
    Style {
        border: Border {
            color: theme.palette().primary,
            width: 2.0,
            radius: Radius::new(10.0)
        },
        text_color: theme.extended_palette().primary.strong.color,
        ..Default::default()
    }
}

fn file_button(state: &'_ CsFM) -> iced::widget::button::Style {
    let theme = theme(state);
    
    Style {
        border: Border {
            color: theme.extended_palette().secondary.strong.color,
            width: 2.0,
            radius: Radius::new(10.0)
        },
        text_color: theme.extended_palette().secondary.strong.color,
        ..Default::default()
    }
}

fn container_style(theme: &Theme) -> iced::widget::container::Style {
   iced::widget::container::Style { border: Border { color: theme.palette().primary, width: 5.0, radius: Radius::new(10) }, ..Default::default() } 
}

fn locations(state: &CsFM) -> Vec<Element<Message>> {
    let mut locs = vec![
        iced::widget::text("Places").into(),
    ];

    for location in state.config.sidebar_loc.iter() {
        locs.push(iced::widget::button(text(location.title.clone())).style(|t, s| dir_button(state)).on_press(Message::CD(PathBuf::from(PathBuf::from(location.path.clone())))).into());
    }

    locs
}


fn view(state: &CsFM) -> Element<'_, Message> {
    // ----- FILE LIST -----
    let files: Vec<Element<Message>> = state
        .current_files
        .iter()
        .map(|f| {
            let name = f
                .0
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();

            if f.1 {
                // Directory
                iced::widget::button(text(name))
                    .style(|_, _| dir_button(state))
                    .on_press(Message::CD(f.0.clone()))
                    .into()
            } else {
                // File
                iced::widget::button(text(name))
                    .style(|_, _| file_button(state))
                    .into()
            }
        })
        .collect();

    let file_list = container(
        scrollable(
            column(files)
                .spacing(5)
                .padding(5)
        )
        .width(Length::Fill)
        .height(Length::Fill)
    )
    .style(container_style)
    .padding(5);

    // ----- SIDEBAR -----
    let mut main_view = row![].padding(5).spacing(5);

    if state.sidebar_open {
        let sidebar =
            column![]
                .extend(locations(state))  // <── FIXED HERE
                .padding(5)
                .spacing(5);

        main_view = main_view.push(
            container(sidebar)
                .padding(5)
                .style(container_style)
                .width(150)
                .height(Length::Fill),
        );
    }

    // Push FILE LIST into main_view
    main_view = main_view.push(file_list);


    // ----- TOP BAR -----
    let top_bar = container(
        row![
            iced::widget::button(if state.sidebar_open { "<" } else { ">" })
                .on_press(Message::ToggleSidebar),

            iced::widget::button("Up")
                .on_press(Message::Up),

            text_input(
                "Path",
                &state.path.to_string_lossy().to_string()
            )
            .on_input(Message::PathChanged)
            .on_submit(Message::CDToPath)
            .padding(5),
        ]
        .padding(5)
        .spacing(5)
    )
    .style(container_style)
    .padding(5);


    // ----- FINAL LAYOUT -----
    column![
        top_bar,
        main_view,
    ]
    .padding(5)
    .into()
}


fn get_files(path: PathBuf) -> Vec<(PathBuf, bool)> {
    let mut files_and_dirs = vec![];

    let mut files = vec![];
    let mut dirs = vec![];

    let paths = fs::read_dir(path);

    if paths.is_err() {
        return files_and_dirs;
    }

    for path in paths.unwrap() {
        let path = path.unwrap();

        if path.path().is_dir() {
            dirs.push((path.path(), true));
        }
        else {
            files.push((path.path(), false));
        }
    }

    dirs.sort();
    files.sort();

    files_and_dirs.append(&mut dirs);
    files_and_dirs.append(&mut files);

    
    files_and_dirs
}

fn load_config() -> Config {
    let path = std::env::home_dir()
        .unwrap()
        .join(".config/csdesktop/csfm.toml");

    println!("{}", path.clone().to_string_lossy().to_string());

    let data = std::fs::read_to_string(path).unwrap();
    let config: Config = toml::from_str(&data).unwrap();
    
    config
}


impl Default for CsFM {
    fn default() -> Self {
        let path = std::env::current_dir().unwrap_or(PathBuf::from("/"));
        let current_files = get_files(path.clone());
        let cfg = load_config();
        CsFM {
            config: cfg,
            path,
            current_files,
            sidebar_open: true
        }
    }
}

pub fn main() -> iced::Result {
    iced::application("CsFM", update, view).theme(theme).run()
}