use std::error::Error;

use iced::{Button, Column, Text};
use kira::manager::AudioManager;

#[derive(Debug, Copy, Clone)]
pub enum Message {
	GoToDemoSelect,
}

pub struct DrumFillDemo {
	audio_manager: AudioManager,
	back_button: iced::button::State,
}

impl DrumFillDemo {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		Ok(Self {
			audio_manager: AudioManager::new(Default::default())?,
			back_button: iced::button::State::new(),
		})
	}

	pub fn view(&mut self) -> iced::Element<'_, Message> {
		Column::new()
			.push(
				Button::new(&mut self.back_button, Text::new("Back"))
					.on_press(Message::GoToDemoSelect),
			)
			.push(Text::new("drum fill demo"))
			.into()
	}
}
