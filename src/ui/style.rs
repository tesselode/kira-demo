use iced::{Background, Color};

pub struct AppStyles;

impl iced::container::StyleSheet for AppStyles {
	fn style(&self) -> iced::container::Style {
		iced::container::Style {
			background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
			text_color: Some(Color::from_rgb(0.9, 0.9, 0.9)),
			..Default::default()
		}
	}
}

impl iced::button::StyleSheet for AppStyles {
	fn active(&self) -> iced::button::Style {
		iced::button::Style {
			background: Some(Background::Color(Color::from_rgb(0.25, 0.25, 0.25))),
			text_color: Color::from_rgb(0.9, 0.9, 0.9),
			..Default::default()
		}
	}

	fn hovered(&self) -> iced::button::Style {
		let active = self.active();

		iced::button::Style {
			background: Some(Background::Color(Color::from_rgb(0.33, 0.33, 0.3))),
			shadow_offset: active.shadow_offset + iced::Vector::new(0.0, 1.0),
			..active
		}
	}

	fn pressed(&self) -> iced::button::Style {
		iced::button::Style {
			shadow_offset: iced::Vector::default(),
			..self.active()
		}
	}

	fn disabled(&self) -> iced::button::Style {
		let active = self.active();

		iced::button::Style {
			shadow_offset: iced::Vector::default(),
			background: active.background.map(|background| match background {
				Background::Color(color) => Background::Color(Color {
					a: color.a * 0.5,
					..color
				}),
			}),
			text_color: Color {
				a: active.text_color.a * 0.5,
				..active.text_color
			},
			..active
		}
	}
}
