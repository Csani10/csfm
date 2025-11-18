use std::{fs, io, iter, path::{Path, PathBuf}, vec};

use iced::{self, Alignment, Border, Element, Length, Task, Theme, border::Radius, widget::{button::{self, Style}, column, container, row, scrollable, text, text_input}, window::Id};
use iced_aw::{ContextMenu, DropDown, Menu, MenuBar, context_menu, drop_down, menu::Item};

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
    path: PathBuf,
    current_files: Vec<(PathBuf, bool)>,
    sidebar_open: bool
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

fn view(state: &'_ CsFM) -> Element<'_, Message> {
    let files: Vec<Element<Message>> = state.current_files.iter().map(|f| {
        if f.1 {
            iced::widget::button(text(f.0.file_name().unwrap().to_string_lossy().to_string())).style(|t, s| dir_button(state)).on_press(Message::CD(f.0.clone())).into()
        }
        else {
            iced::widget::button(text(f.0.file_name().unwrap().to_string_lossy().to_string())).style(|t, s| file_button(state)).into()
        }
    }
    )
    .collect();

    let mut main_view = row![].padding(5).spacing(5);;

    if state.sidebar_open {
        main_view = main_view.push(
            container(column![
                iced::widget::button(text("Home")).style(|t, s| dir_button(state)).on_press(Message::CD(PathBuf::from("/home/csani"))),
                iced::widget::button(text("Documents")).style(|t, s| dir_button(state)).on_press(Message::CD(PathBuf::from("/home/csani/Dokumentumok"))),
                ].padding(5)).padding(5).style(container_style).width(150).height(Length::Fill)
        );
    }

    main_view = main_view.push(container(scrollable(column(files).spacing(5).padding(5)).width(Length::Fill).height(Length::Fill)).style(container_style).padding(5));

    column![
        container(
            row![
            iced::widget::button(if state.sidebar_open {"<"} else {">"}).on_press(Message::ToggleSidebar),
            iced::widget::button("Up").on_press(Message::Up),
            text_input("Path", &state.path.to_string_lossy().to_string()).on_input(Message::PathChanged).on_submit(Message::CDToPath).padding(5),
        ].padding(5).spacing(5)
        ).style(container_style).padding(5),
        main_view
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

impl Default for CsFM {
    fn default() -> Self {
        let path = std::env::current_dir().unwrap_or(PathBuf::from("/"));
        let current_files = get_files(path.clone());
        CsFM {
            path,
            current_files,
            sidebar_open: false
        }
    }
}

pub fn main() -> iced::Result {
    iced::application("CsFM", update, view).theme(theme).run()
}