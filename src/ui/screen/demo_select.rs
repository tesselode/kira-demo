use iced::{Align, Button, Column, Container, Length, Text};

use crate::ui::style::AppStyles;

#[derive(Debug, Copy, Clone)]
pub enum Message {
	GoToDrumFillDemo,
	GoToUnderwaterDemo,
}

pub struct DemoSelect {
	drum_fill_demo_button: iced::button::State,
	underwater_demo_button: iced::button::State,
}

impl DemoSelect {
	pub fn new() -> Self {
		Self {
			drum_fill_demo_button: iced::button::State::new(),
			underwater_demo_button: iced::button::State::new(),
		}
	}

	pub fn view(&mut self) -> iced::Element<'_, Message> {
		Container::new(
			Column::new()
				.spacing(16)
				.align_items(Align::Center)
				.push(Text::new("Select a demo").size(48))
				.push(
					Button::new(
						&mut self.drum_fill_demo_button,
						Text::new("Drum fill demo").size(24),
					)
					.on_press(Message::GoToDrumFillDemo)
					.style(AppStyles),
				)
				.push(
					Button::new(
						&mut self.underwater_demo_button,
						Text::new("Underwater demo").size(24),
					)
					.on_press(Message::GoToUnderwaterDemo)
					.style(AppStyles),
				),
		)
		.width(Length::Fill)
		.height(Length::Fill)
		.align_x(Align::Center)
		.align_y(Align::Center)
		.into()
	}
}
