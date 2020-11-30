use iced::{Column, Container, Length};

use super::header::Header;

pub struct ScreenWrapper<Message: Clone> {
	header: Header<Message>,
}

impl<Message: Clone> ScreenWrapper<Message> {
	pub fn new(header_text: String, back_button_message: Message) -> Self {
		Self {
			header: Header::new(header_text, back_button_message),
		}
	}

	pub fn view<'a, C: Into<iced::Element<'a, Message>>>(
		&'a mut self,
		contents: C,
	) -> iced::Element<'a, Message> {
		Column::new()
			.push(self.header.view())
			.push(
				Container::new(contents)
					.width(Length::Fill)
					.height(Length::Fill)
					.center_x()
					.center_y(),
			)
			.into()
	}
}
