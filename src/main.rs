use std::path::Path;
use std::sync::Arc;
use iced::{Theme, Element, Application, Settings, Length, executor, Command};
use iced::widget::{container, text, text_editor, column, row, horizontal_space, button};
use tokio::io;

fn main() -> iced::Result {
    Editor::run(Settings::default())
}

struct Editor {
    content: text_editor::Content,
    error: Option<Error>
}
#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    FileOpened(Result<Arc<String>, Error>),
    Open,
}

impl Application for Editor {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
       (
           Self {
               content: text_editor::Content::new(),
               error: None
           },
           Command::perform(load_file(format!("{}/src/main.rs", env!
           ("CARGO_MANIFEST_DIR"))), Message::FileOpened)
       )
    }

    fn title(&self) -> String {
        String::from("A rust iced editor")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Edit(action) => {
                self.content.edit(action);
                Command::none()
            }

            Message::FileOpened(Ok(content)) => {
                self.content = text_editor::Content::with(&content);
                Command::none()
            }

            Message::FileOpened(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }

            Message::Open => {
                Command::perform(pick_file(), Message::FileOpened)
            }
        }

    }

    fn view(&self) -> Element<'_, Message> {
        let controls = row![button("Open").on_press(Message::Open)];

        let input = text_editor(&self.content).on_edit(Message::Edit);

        let position = {
            let (line, column) = self.content.cursor_position();
            text(format!("{}:{}", line + 1, column + 1))
        };

        let status_bar = row![
            horizontal_space(Length::Fill),
            position
        ];

        container(column![controls, input, status_bar].spacing(10)).padding(10)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

async fn pick_file() -> Result<Arc<String>, Error> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Choose a text file...")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?;

    load_file(handle.path()).await
}

async fn load_file(path: impl AsRef<Path>) -> Result<Arc<String>, Error> {
    tokio::fs::read_to_string(path)
        .await
        .map(Arc::new)
        .map_err(|error| error.kind())
        .map_err(Error::IO)
}

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
    IO(io::ErrorKind)
}