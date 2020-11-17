use iced::{Element, Sandbox, Text};

struct App;

impl Sandbox for App {
	type Message = ();

	fn new() -> Self {
		Self
	}

	fn title(&self) -> String {
		"Kira demo".into()
	}

	fn update(&mut self, _message: Self::Message) {}

	fn view(&mut self) -> Element<'_, Self::Message> {
		Text::new("Hello, world!").into()
	}
}

fn main() {
	App::run(Default::default())
}
