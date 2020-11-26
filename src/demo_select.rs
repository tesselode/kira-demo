use iced::{Button, Column, Text};

#[derive(Debug, Copy, Clone)]
pub enum Message {
	GoToDrumFillDemo,
}

pub struct DemoSelect {
	drum_fill_demo_button: iced::button::State,
}

impl DemoSelect {
	pub fn new() -> Self {
		Self {
			drum_fill_demo_button: iced::button::State::new(),
		}
	}

	pub fn view(&mut self) -> iced::Element<'_, Message> {
		Column::new()
			.push(Text::new("Select a demo"))
			.push(
				Button::new(&mut self.drum_fill_demo_button, Text::new("Drum fill demo"))
					.on_press(Message::GoToDrumFillDemo),
			)
			.into()
	}
}
