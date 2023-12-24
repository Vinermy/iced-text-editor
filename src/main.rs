use iced::theme::Text;
use iced::widget::{button, column, container, horizontal_space, row, text, text_editor};
use iced::{executor, Application, Command, Element, Length, Settings, Theme};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io;

fn main() -> iced::Result {
    Editor::run(Settings::default())
}

struct Editor {
    path: Option<PathBuf>,
    content: text_editor::Content,
    error: Option<Error>,
}
#[derive(Debug, Clone)]
enum Message {
    New,
    Edit(text_editor::Action),
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    FileSaved(Result<PathBuf, Error>),
    Open,
    Save,
}

impl Application for Editor {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                path: None,
                content: text_editor::Content::new(),
                error: None,
            },
            Command::perform(load_file(default_file()), Message::FileOpened),
        )
    }

    fn title(&self) -> String {
        String::from("A rust iced editor")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Edit(action) => {
                self.content.edit(action);
                self.error = None;
                Command::none()
            }

            Message::FileOpened(Ok((path, content))) => {
                self.path = Some(path);
                self.content = text_editor::Content::with(&content);
                Command::none()
            }

            Message::FileOpened(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }

            Message::Open => Command::perform(pick_file(), Message::FileOpened),
            Message::New => {
                self.path = None;
                self.content = text_editor::Content::new();

                Command::none()
            }
            Message::Save => {
                let text = self.content.text();

                Command::perform(save_file(self.path.clone(), text), Message::FileSaved)
            }
            Message::FileSaved(Ok((path))) => {
                self.path = Some(path);
                Command::none()
            }
            Message::FileSaved(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let controls = row![
            button("Open").on_press(Message::Open),
            button("New").on_press(Message::New),
            button("Save").on_press(Message::Save)
        ]
        .spacing(10);

        let input = text_editor(&self.content).on_edit(Message::Edit);

        let status_bar = {
            let status = if let Some(Error::IoFailed(error)) = self.error.as_ref() {
                text(error.to_string())
            } else {
                match self.path.as_deref().and_then(Path::to_str) {
                    None => text(""),
                    Some(path) => text(path).size(14),
                }
            };

            let position = {
                let (line, column) = self.content.cursor_position();
                text(format!("{}:{}", line + 1, column + 1))
            };

            row![status, horizontal_space(Length::Fill), position]
        };

        container(column![controls, input, status_bar].spacing(10))
            .padding(10)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

fn default_file() -> PathBuf {
    PathBuf::from(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")))
}

async fn pick_file() -> Result<(PathBuf, Arc<String>), Error> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Choose a text file...")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?;

    load_file(handle.path().to_owned()).await
}

async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), Error> {
    let contents = tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|error| error.kind())
        .map_err(Error::IoFailed)?;

    Ok((path, contents))
}

async fn save_file(path: Option<PathBuf>, text: String) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .set_title("Choose a file name...")
            .save_file()
            .await
            .ok_or(Error::DialogClosed)
            .map(|handle| handle.path().to_owned())?
    };

    tokio::fs::write(&path, text)
        .await
        .map_err(|error| Error::IoFailed(error.kind()))?;

    Ok(path)
}

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
    IoFailed(io::ErrorKind),
}
