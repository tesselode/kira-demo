use std::marker::PhantomData;

use iced::{Align, Button, Row, Text};

use crate::ui::style::AppStyles;

pub struct Header<Message: Clone> {
	back_button: iced::button::State,
	back_button_message: Message,
	text: String,
	message: PhantomData<Message>,
}

impl<Message: Clone> Header<Message> {
	pub fn new(text: String, back_button_message: Message) -> Self {
		Self {
			back_button: iced::button::State::new(),
			back_button_message,
			text,
			message: PhantomData,
		}
	}

	pub fn view(&mut self) -> iced::Element<'_, Message> {
		Row::new()
			.padding(16)
			.spacing(16)
			.align_items(Align::Center)
			.push(
				Button::new(&mut self.back_button, Text::new("Back"))
					.on_press(self.back_button_message.clone())
					.style(AppStyles),
			)
			.push(Text::new(&self.text))
			.into()
	}
}
