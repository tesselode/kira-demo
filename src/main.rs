mod ui;

use std::{error::Error, time::Duration};

use iced::{executor, Application, Command, Container, Length, Subscription};
use ui::{
	screen::{
		demo_select,
		demo_select::DemoSelect,
		drum_fill_demo,
		drum_fill_demo::DrumFillDemo,
		underwater_demo::{self, UnderwaterDemo},
	},
	style::AppStyles,
};

#[derive(Debug, Copy, Clone)]
enum Message {
	CheckForEvents,
	DemoSelect(demo_select::Message),
	DrumFillDemo(drum_fill_demo::Message),
	UnderwaterDemo(underwater_demo::Message),
}

enum Screen {
	DemoSelect(DemoSelect),
	DrumFillDemo(DrumFillDemo),
	UnderwaterDemo(UnderwaterDemo),
}

struct App {
	screen: Screen,
}

impl Application for App {
	type Executor = executor::Default;
	type Message = Message;
	type Flags = ();

	fn new(_: ()) -> (Self, Command<Self::Message>) {
		(
			Self {
				screen: Screen::DemoSelect(DemoSelect::new()),
			},
			Command::none(),
		)
	}

	fn title(&self) -> String {
		"Kira demo".into()
	}

	fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
		match message {
			Message::CheckForEvents => match &mut self.screen {
				Screen::DrumFillDemo(screen) => {
					screen.check_for_events().unwrap();
				}
				_ => {}
			},
			Message::DemoSelect(message) => match message {
				demo_select::Message::GoToDrumFillDemo => {
					self.screen = Screen::DrumFillDemo(DrumFillDemo::new().unwrap());
				}
				demo_select::Message::GoToUnderwaterDemo => {
					self.screen = Screen::UnderwaterDemo(UnderwaterDemo::new().unwrap());
				}
			},
			Message::DrumFillDemo(message) => match message {
				drum_fill_demo::Message::GoToDemoSelect => {
					self.screen = Screen::DemoSelect(DemoSelect::new());
				}
				message => {
					if let Screen::DrumFillDemo(screen) = &mut self.screen {
						screen.update(message).unwrap();
					}
				}
			},
			Message::UnderwaterDemo(message) => match message {
				underwater_demo::Message::GoToDemoSelect => {
					self.screen = Screen::DemoSelect(DemoSelect::new());
				}
				message => {
					if let Screen::UnderwaterDemo(screen) = &mut self.screen {
						screen.update(message).unwrap();
					}
				}
			},
		}
		Command::none()
	}

	fn subscription(&self) -> Subscription<Self::Message> {
		match &self.screen {
			Screen::DrumFillDemo(_) => {
				iced::time::every(Duration::from_millis(16)).map(|_| Message::CheckForEvents)
			}
			_ => Subscription::none(),
		}
	}

	fn view(&mut self) -> iced::Element<'_, Self::Message> {
		Container::new(match &mut self.screen {
			Screen::DemoSelect(screen) => screen.view().map(|message| Message::DemoSelect(message)),
			Screen::DrumFillDemo(screen) => {
				screen.view().map(|message| Message::DrumFillDemo(message))
			}
			Screen::UnderwaterDemo(screen) => screen
				.view()
				.map(|message| Message::UnderwaterDemo(message)),
		})
		.width(Length::Fill)
		.height(Length::Fill)
		.style(AppStyles)
		.into()
	}
}

fn main() -> Result<(), Box<dyn Error>> {
	App::run(iced::Settings {
		window: iced::window::Settings {
			size: (650, 400),
			..Default::default()
		},
		..Default::default()
	})?;
	Ok(())
}
