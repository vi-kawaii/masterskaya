use iced::widget::text;
use iced::{Element, Task};

fn main() -> iced::Result {
    iced::application(|| (), update, view)
        .title("pen")
        .run()
}

#[derive(Debug, Clone)]
enum Message {}

fn update(_state: &mut (), _message: Message) -> Task<Message> {
    Task::none()
}

fn view(_state: &()) -> Element<'_, Message> {
    text("pen — iced is alive").into()
}
