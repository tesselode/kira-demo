use std::error::Error;

use iced::{Button, Column, Text};
use kira::{
	manager::{AudioManager, AudioManagerSettings},
	sequence::{Sequence, SequenceId},
	sound::Sound,
	sound::SoundId,
	Duration, MetronomeSettings, Tempo,
};

#[derive(Debug, Copy, Clone)]
pub enum Message {
	GoToDemoSelect,
	Play,
}

#[derive(Debug, Copy, Clone)]
pub enum AudioEvent {
	Beat(usize),
}

#[derive(Debug, Copy, Clone)]
pub enum PlaybackState {
	Stopped,
	Looping(usize),
}

impl PlaybackState {
	fn to_string(&self) -> String {
		match self {
			PlaybackState::Stopped => "Stopped".into(),
			PlaybackState::Looping(beat) => format!("Beat {}", beat),
		}
	}
}

pub struct DrumFillDemo {
	audio_manager: AudioManager<AudioEvent>,
	loop_sound_id: SoundId,
	fill_2b_sound_id: SoundId,
	fill_3b_sound_id: SoundId,
	fill_4b_sound_id: SoundId,
	playback_state: PlaybackState,
	sequence_id: Option<SequenceId>,
	back_button: iced::button::State,
	play_button: iced::button::State,
}

impl DrumFillDemo {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let mut audio_manager = AudioManager::new(AudioManagerSettings {
			metronome_settings: MetronomeSettings {
				tempo: Tempo(128.0).into(),
				..Default::default()
			},
			..Default::default()
		})?;
		let base_assets_dir = std::env::current_dir()?.join("assets/drum fill demo");
		let loop_sound_id = audio_manager.add_sound(Sound::from_file(
			base_assets_dir.join("loop.ogg"),
			Default::default(),
		)?)?;
		let fill_2b_sound_id = audio_manager.add_sound(Sound::from_file(
			base_assets_dir.join("2 beat fill.ogg"),
			Default::default(),
		)?)?;
		let fill_3b_sound_id = audio_manager.add_sound(Sound::from_file(
			base_assets_dir.join("3 beat fill.ogg"),
			Default::default(),
		)?)?;
		let fill_4b_sound_id = audio_manager.add_sound(Sound::from_file(
			base_assets_dir.join("4 beat fill.ogg"),
			Default::default(),
		)?)?;
		Ok(Self {
			audio_manager,
			loop_sound_id,
			fill_2b_sound_id,
			fill_3b_sound_id,
			fill_4b_sound_id,
			playback_state: PlaybackState::Stopped,
			sequence_id: None,
			back_button: iced::button::State::new(),
			play_button: iced::button::State::new(),
		})
	}

	fn start_loop(&mut self) -> Result<(), Box<dyn Error>> {
		self.playback_state = PlaybackState::Looping(1);
		self.audio_manager.start_sequence({
			let mut sequence = Sequence::new();
			sequence.wait_for_interval(1.0);
			sequence.start_loop();
			sequence.play(self.loop_sound_id, Default::default());
			sequence.emit_custom_event(AudioEvent::Beat(1));
			sequence.wait(Duration::Beats(1.0));
			sequence.emit_custom_event(AudioEvent::Beat(2));
			sequence.wait(Duration::Beats(1.0));
			sequence.emit_custom_event(AudioEvent::Beat(3));
			sequence.wait(Duration::Beats(1.0));
			sequence.emit_custom_event(AudioEvent::Beat(4));
			sequence.wait(Duration::Beats(1.0));
			sequence
		})?;
		self.audio_manager.start_metronome()?;
		Ok(())
	}

	pub fn check_for_events(&mut self) -> Result<(), Box<dyn Error>> {
		for event in self.audio_manager.events() {
			match event {
				kira::Event::Custom(event) => match event {
					AudioEvent::Beat(beat) => {
						self.playback_state = PlaybackState::Looping(beat);
					}
				},
				_ => {}
			}
		}
		Ok(())
	}

	pub fn update(&mut self, message: Message) -> Result<(), Box<dyn Error>> {
		match message {
			Message::Play => {
				self.start_loop()?;
			}
			_ => {}
		}
		Ok(())
	}

	pub fn view(&mut self) -> iced::Element<'_, Message> {
		Column::new()
			.push(
				Button::new(&mut self.back_button, Text::new("Back"))
					.on_press(Message::GoToDemoSelect),
			)
			.push(Button::new(&mut self.play_button, Text::new("Play")).on_press(Message::Play))
			.push(Text::new(self.playback_state.to_string()))
			.into()
	}
}
