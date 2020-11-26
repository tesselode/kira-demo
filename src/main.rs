mod demo_select;
mod drum_fill_demo;

use std::error::Error;

use demo_select::DemoSelect;
use drum_fill_demo::DrumFillDemo;
use iced::{Sandbox, Text};

#[derive(Debug, Copy, Clone)]
enum Message {
	DemoSelect(demo_select::Message),
	DrumFillDemo(drum_fill_demo::Message),
}

enum Screen {
	DemoSelect(DemoSelect),
	DrumFillDemo(DrumFillDemo),
}

struct App {
	screen: Screen,
}

impl Sandbox for App {
	type Message = Message;

	fn new() -> Self {
		Self {
			screen: Screen::DemoSelect(DemoSelect::new()),
		}
	}

	fn title(&self) -> String {
		"Kira demo".into()
	}

	fn update(&mut self, message: Self::Message) {
		match message {
			Message::DemoSelect(message) => match message {
				demo_select::Message::GoToDrumFillDemo => {
					self.screen = Screen::DrumFillDemo(DrumFillDemo::new().unwrap());
				}
			},
			Message::DrumFillDemo(message) => match message {
				drum_fill_demo::Message::GoToDemoSelect => {
					self.screen = Screen::DemoSelect(DemoSelect::new());
				}
			},
		}
	}

	fn view(&mut self) -> iced::Element<'_, Self::Message> {
		match &mut self.screen {
			Screen::DemoSelect(screen) => screen.view().map(|message| Message::DemoSelect(message)),
			Screen::DrumFillDemo(screen) => {
				screen.view().map(|message| Message::DrumFillDemo(message))
			}
		}
	}
}

fn main() -> Result<(), Box<dyn Error>> {
	App::run(Default::default())?;
	Ok(())
}
