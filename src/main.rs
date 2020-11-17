use iced::{button, Button, Element, Sandbox, Text};
use kira::{
	instance::InstanceId,
	instance::InstanceSettings,
	manager::AudioManager,
	parameter::Tween,
	sound::SoundMetadata,
	sound::SoundSettings,
	sound::{Sound, SoundId},
	Tempo,
};

#[derive(Debug, Copy, Clone)]
enum Message {
	StartLoop,
	StopLoop(InstanceId),
}

struct App {
	audio_manager: AudioManager,
	loop_sound_id: SoundId,
	loop_instance_id: Option<InstanceId>,
	start_loop_button_state: button::State,
}

impl Sandbox for App {
	type Message = Message;

	fn new() -> Self {
		let mut audio_manager = AudioManager::new(Default::default()).unwrap();
		let loop_sound_id = audio_manager
			.add_sound(
				Sound::from_file(
					std::env::current_dir().unwrap().join("assets/loop.ogg"),
					SoundSettings {
						metadata: SoundMetadata {
							semantic_duration: Some(Tempo(102.0).beats_to_seconds(16.0)),
						},
						..Default::default()
					},
				)
				.unwrap(),
			)
			.unwrap();
		Self {
			audio_manager,
			loop_sound_id,
			loop_instance_id: None,
			start_loop_button_state: button::State::new(),
		}
	}

	fn title(&self) -> String {
		"Kira demo".into()
	}

	fn update(&mut self, message: Self::Message) {
		match message {
			Message::StartLoop => {
				self.loop_instance_id = Some(
					self.audio_manager
						.play_sound(self.loop_sound_id, InstanceSettings::new().loop_region(..))
						.unwrap(),
				);
			}
			Message::StopLoop(instance_id) => {
				self.audio_manager
					.stop_instance(instance_id, Some(Tween(1.0)))
					.unwrap();
				self.loop_instance_id = None;
			}
		}
	}

	fn view(&mut self) -> Element<'_, Self::Message> {
		match self.loop_instance_id {
			Some(instance_id) => {
				Button::new(&mut self.start_loop_button_state, Text::new("Stop loop"))
					.on_press(Message::StopLoop(instance_id))
					.into()
			}
			None => Button::new(&mut self.start_loop_button_state, Text::new("Start loop"))
				.on_press(Message::StartLoop)
				.into(),
		}
	}
}

fn main() {
	App::run(Default::default())
}
