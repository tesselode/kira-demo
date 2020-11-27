use std::error::Error;

use iced::{Align, Button, Column, Container, Length, Row, Text};
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
	audio_manager: AudioManager<AudioEvent>,
	loop_sound_id: SoundId,
	fill_2b_sound_id: SoundId,
	fill_3b_sound_id: SoundId,
	fill_4b_sound_id: SoundId,
	playback_state: PlaybackState,
	sequence_ids: Vec<SequenceId>,
	back_button: iced::button::State,
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
			audio_manager,
			loop_sound_id,
			fill_2b_sound_id,
			fill_3b_sound_id,
			fill_4b_sound_id,
			playback_state: PlaybackState::Stopped,
			sequence_ids: vec![],
			back_button: iced::button::State::new(),
			play_button: iced::button::State::new(),
			play_drum_fill_button: iced::button::State::new(),
		})
	}

	/// If no sequence is playing, start the drum loop. If the drum
	/// loop is already playing, queue up a drum fill that takes
	/// us back to the loop.
	///
	/// The drum fill chosen depends on the current beat at the time
	/// it's queued:
	///   - On beat 1, a 3-beat drum fill will be queued for beat 2.
	///   - On beat 2, a 2-beat drum fill will be queued for beat 3.
	///   - On beats 3 or 4, a 4-beat drum fill will be queued for the
	///     beginning of the next measure.
	fn start_sequence(&mut self) -> Result<(), Box<dyn Error>> {
		match self.playback_state {
			PlaybackState::Stopped | PlaybackState::Looping(_) => {
				let mut sequence = Sequence::new();
				if let PlaybackState::Looping(beat) = self.playback_state {
					let previous_sequence_id = *self.sequence_ids.last().unwrap();
					match beat {
						1 => {
							self.playback_state = PlaybackState::QueueingFill(3);
							sequence.wait_for_interval(1.0);
							sequence.play(self.fill_3b_sound_id, Default::default());
							sequence.emit_custom_event(AudioEvent::StartFill(3));
						}
						2 => {
							self.playback_state = PlaybackState::QueueingFill(2);
							sequence.wait_for_interval(1.0);
							sequence.play(self.fill_2b_sound_id, Default::default());
							sequence.emit_custom_event(AudioEvent::StartFill(2));
						}
						_ => {
							self.playback_state = PlaybackState::QueueingFill(4);
							sequence.wait_for_interval(4.0);
							sequence.play(self.fill_4b_sound_id, Default::default());
							sequence.emit_custom_event(AudioEvent::StartFill(4));
						}
					}
					sequence.stop_sequence_and_instances(previous_sequence_id, Some(0.01.into()));
				}
				sequence.wait_for_interval(4.0);
				sequence.start_loop();
				sequence.emit_custom_event(AudioEvent::StartLoop);
				sequence.play(self.loop_sound_id, Default::default());
				for i in 1..=4 {
					sequence.emit_custom_event(AudioEvent::Beat(i));
					sequence.wait(Duration::Beats(1.0));
				}
				self.sequence_ids
					.push(self.audio_manager.start_sequence(sequence)?);
				if let PlaybackState::Stopped = self.playback_state {
					self.audio_manager.start_metronome()?;
				}
			}
			_ => {}
		}
		Ok(())
	}

	fn stop(&mut self) -> Result<(), Box<dyn Error>> {
		for id in self.sequence_ids.drain(..) {
			self.audio_manager
				.stop_sequence_and_instances(id, Some(0.01.into()))?;
		}
		self.audio_manager.stop_metronome()?;
		self.playback_state = PlaybackState::Stopped;
		Ok(())
	}

	pub fn check_for_events(&mut self) -> Result<(), Box<dyn Error>> {
		while let Some(event) = self.audio_manager.pop_event() {
			match event {
				kira::Event::Custom(event) => match event {
					AudioEvent::StartLoop => {
						self.playback_state = PlaybackState::Looping(1);
					}
					AudioEvent::StartFill(beats) => {
						self.playback_state = PlaybackState::PlayingFill(beats);
					}
					AudioEvent::Beat(beat) => {
						if let PlaybackState::Looping(_) = self.playback_state {
							self.playback_state = PlaybackState::Looping(beat);
						}
					}
				},
				_ => {}
			}
		}
		Ok(())
	}

	pub fn update(&mut self, message: Message) -> Result<(), Box<dyn Error>> {
		match message {
			Message::StartSequence => {
				self.start_sequence()?;
			}
			Message::Stop => {
				self.stop()?;
			}
			_ => {}
		}
		Ok(())
	}

	pub fn view(&mut self) -> iced::Element<'_, Message> {
		Column::new()
			.push(
				Row::new()
					.padding(16)
					.spacing(16)
					.align_items(Align::Center)
					.push(
						Button::new(&mut self.back_button, Text::new("Back"))
							.on_press(Message::GoToDemoSelect),
					)
					.push(Text::new("Drum fill demo")),
			)
			.push(
				Container::new(
					Column::new()
						.align_items(Align::Center)
						.spacing(16)
						.push(
							Row::new()
								.spacing(16)
								.push(
									Button::new(
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
									}),
								)
								.push({
									let mut button = Button::new(
										&mut self.play_drum_fill_button,
										Text::new("Play drum fill").size(24),
									);
									if let PlaybackState::Looping(_) = self.playback_state {
										button = button.on_press(Message::StartSequence);
									}
									button
								}),
						)
						.push(Text::new(self.playback_state.to_string()).size(24)),
				)
				.width(Length::Fill)
				.height(Length::Fill)
				.align_x(Align::Center)
				.align_y(Align::Center),
			)
			.into()
	}
}
