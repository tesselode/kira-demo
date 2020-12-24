use std::error::Error;

use iced::{Align, Button, Column, Length, Row, Text};
use kira::{
	manager::{AudioManager, AudioManagerSettings},
	sound::Sound,
	MetronomeSettings, Tempo,
};

use crate::ui::{common::screen_wrapper::ScreenWrapper, style::AppStyles};

const EXPLANATION_TEXT: &str = "This demo uses \
a sequence to play a short drum sample repeatedly and \
keep track of which beat of music is currently playing. \
This beat is used to determine what kind of drum fill \
to play.

When the drum fill is triggered, a second sequence waits \
for the right beat, stops the previous sequence, starts \
the drum fill, and then starts a new loop.";

#[derive(Debug, Copy, Clone)]
pub enum Message {
	GoToDemoSelect,
	StartSequence,
	Stop,
}

#[derive(Debug, Copy, Clone)]
pub enum AudioEvent {
	StartLoop,
	StartFill(usize),
	Beat(usize),
}

#[derive(Debug, Copy, Clone)]
pub enum PlaybackState {
	Stopped,
	Looping(usize),
	QueueingFill(usize),
	PlayingFill(usize),
}

impl PlaybackState {
	fn to_string(&self) -> String {
		match self {
			PlaybackState::Stopped => "Stopped".into(),
			PlaybackState::Looping(beat) => format!("Beat {}", beat),
			PlaybackState::QueueingFill(beats) => format!("Queueing {} beat drum fill", beats),
			PlaybackState::PlayingFill(beats) => format!("Playing {} beat drum fill", beats),
		}
	}
}

pub struct DrumFillDemo {
	playback_state: PlaybackState,
	screen_wrapper: ScreenWrapper<Message>,
	play_button: iced::button::State,
	play_drum_fill_button: iced::button::State,
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
			playback_state: PlaybackState::Stopped,
			screen_wrapper: ScreenWrapper::new("Drum fill demo".into(), Message::GoToDemoSelect),
			play_button: iced::button::State::new(),
			play_drum_fill_button: iced::button::State::new(),
		})
	}

	fn stop(&mut self) -> Result<(), Box<dyn Error>> {
		self.playback_state = PlaybackState::Stopped;
		Ok(())
	}

	pub fn check_for_events(&mut self) -> Result<(), Box<dyn Error>> {
		Ok(())
	}

	pub fn update(&mut self, message: Message) -> Result<(), Box<dyn Error>> {
		match message {
			Message::StartSequence => {}
			Message::Stop => {}
			_ => {}
		}
		Ok(())
	}

	pub fn view(&mut self) -> iced::Element<'_, Message> {
		let play_button = Button::new(
			&mut self.play_button,
			Text::new(match self.playback_state {
				PlaybackState::Stopped => "Start loop",
				_ => "Stop",
			})
			.size(24),
		)
		.on_press(match self.playback_state {
			PlaybackState::Stopped => Message::StartSequence,
			_ => Message::Stop,
		})
		.style(AppStyles);

		let play_drum_fill_button = {
			let mut button = Button::new(
				&mut self.play_drum_fill_button,
				Text::new("Play drum fill").size(24),
			)
			.style(AppStyles);
			if let PlaybackState::Looping(_) = self.playback_state {
				button = button.on_press(Message::StartSequence);
			}
			button
		};

		self.screen_wrapper.view(
			Column::new()
				.align_items(Align::Center)
				.spacing(16)
				.push(
					Row::new()
						.spacing(16)
						.push(play_button)
						.push(play_drum_fill_button),
				)
				.push(Text::new(self.playback_state.to_string()).size(24))
				.push(
					Column::new()
						.width(Length::Fill)
						.max_width(600)
						.push(Text::new(EXPLANATION_TEXT)),
				),
		)
	}
}
