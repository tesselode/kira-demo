use std::error::Error;

use iced::{Button, Column, Text};
use kira::{
	group::GroupId,
	manager::{AudioManager, AudioManagerSettings},
	playable::PlayableSettings,
	sequence::{EventReceiver, Sequence, SequenceInstanceId, SequenceSettings},
	sound::{Sound, SoundId},
	AudioResult, Duration, MetronomeSettings, Tempo,
};

use crate::ui::common::screen_wrapper::ScreenWrapper;

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

enum PlaybackState {
	Stopped,
	PlayingLoop {
		beat_tracker_sequence: (SequenceInstanceId, EventReceiver<usize>),
		loop_sequence_id: SequenceInstanceId,
		current_beat: usize,
	},
	QueueingFill {
		beat_tracker_sequence: (SequenceInstanceId, EventReceiver<usize>),
		loop_sequence_id: SequenceInstanceId,
		current_beat: usize,
		drum_fill_length: usize,
	},
	PlayingFill {
		beat_tracker_sequence: (SequenceInstanceId, EventReceiver<usize>),
		loop_sequence_id: SequenceInstanceId,
		current_beat: usize,
		drum_fill_length: usize,
	},
}

pub struct DrumFillDemo {
	audio_manager: AudioManager,
	group_id: GroupId,
	loop_sound_id: SoundId,
	fill_2b_sound_id: SoundId,
	fill_3b_sound_id: SoundId,
	fill_4b_sound_id: SoundId,
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
		let group_id = audio_manager.add_group([])?;
		let base_assets_dir = std::env::current_dir()?.join("assets/drum fill demo");
		let loop_sound_id = audio_manager.add_sound(Sound::from_file(
			base_assets_dir.join("loop.ogg"),
			PlayableSettings::default().groups([group_id]),
		)?)?;
		let fill_2b_sound_id = audio_manager.add_sound(Sound::from_file(
			base_assets_dir.join("2 beat fill.ogg"),
			PlayableSettings::default().groups([group_id]),
		)?)?;
		let fill_3b_sound_id = audio_manager.add_sound(Sound::from_file(
			base_assets_dir.join("3 beat fill.ogg"),
			PlayableSettings::default().groups([group_id]),
		)?)?;
		let fill_4b_sound_id = audio_manager.add_sound(Sound::from_file(
			base_assets_dir.join("4 beat fill.ogg"),
			PlayableSettings::default().groups([group_id]),
		)?)?;
		Ok(Self {
			audio_manager,
			group_id,
			loop_sound_id,
			fill_2b_sound_id,
			fill_3b_sound_id,
			fill_4b_sound_id,
			playback_state: PlaybackState::Stopped,
			screen_wrapper: ScreenWrapper::new("Drum fill demo".into(), Message::GoToDemoSelect),
			play_button: iced::button::State::new(),
			play_drum_fill_button: iced::button::State::new(),
		})
	}

	pub fn check_for_events(&mut self) -> Result<(), Box<dyn Error>> {
		match &mut self.playback_state {
			PlaybackState::Stopped => {}
			PlaybackState::PlayingLoop {
				beat_tracker_sequence,
				loop_sequence_id: _,
				current_beat,
			} => {
				while let Some(beat) = beat_tracker_sequence.1.pop() {
					*current_beat = *beat;
				}
			}
			PlaybackState::QueueingFill {
				beat_tracker_sequence,
				loop_sequence_id: _,
				current_beat,
				drum_fill_length: _,
			}
			| PlaybackState::PlayingFill {
				beat_tracker_sequence,
				loop_sequence_id: _,
				current_beat,
				drum_fill_length: _,
			} => {
				while let Some(beat) = beat_tracker_sequence.1.pop() {
					*current_beat = *beat;
				}
			}
		}
		Ok(())
	}

	pub fn start_beat_tracker(
		&mut self,
	) -> AudioResult<(SequenceInstanceId, EventReceiver<usize>)> {
		self.audio_manager.start_sequence(
			{
				let mut sequence = Sequence::new(SequenceSettings::new().groups([self.group_id]));
				sequence.wait_for_interval(1.0);
				sequence.start_loop();
				for i in 1..=4 {
					sequence.emit(i);
					sequence.wait(Duration::Beats(1.0));
				}
				sequence
			},
			Default::default(),
		)
	}

	pub fn start_loop(&mut self) -> AudioResult<SequenceInstanceId> {
		self.audio_manager
			.start_sequence(
				{
					let mut sequence =
						Sequence::<()>::new(SequenceSettings::new().groups([self.group_id]));
					match &mut self.playback_state {
						PlaybackState::Stopped => {
							sequence.wait_for_interval(1.0);
						}
						PlaybackState::PlayingLoop {
							beat_tracker_sequence: _,
							loop_sequence_id,
							current_beat,
						} => {
							match current_beat {
								1 => {
									sequence.wait_for_interval(1.0);
									sequence.play(self.fill_3b_sound_id, Default::default());
								}
								2 => {
									sequence.wait_for_interval(1.0);
									sequence.play(self.fill_2b_sound_id, Default::default());
								}
								3 | 4 => {
									sequence.wait_for_interval(4.0);
									sequence.play(self.fill_4b_sound_id, Default::default());
								}
								_ => unreachable!(),
							}
							sequence.stop_sequence(*loop_sequence_id);
							sequence.wait_for_interval(4.0);
						}
						_ => unreachable!(),
					}
					sequence.start_loop();
					sequence.play(self.loop_sound_id, Default::default());
					sequence.wait(Duration::Beats(4.0));
					sequence
				},
				Default::default(),
			)
			.map(|sequence| sequence.0)
	}

	pub fn update(&mut self, message: Message) -> Result<(), Box<dyn Error>> {
		match message {
			Message::StartSequence => {
				let playback_state =
					std::mem::replace(&mut self.playback_state, PlaybackState::Stopped);
				match playback_state {
					PlaybackState::Stopped => {
						self.playback_state = PlaybackState::PlayingLoop {
							beat_tracker_sequence: self.start_beat_tracker()?,
							loop_sequence_id: self.start_loop()?,
							current_beat: 1,
						};
						self.audio_manager.start_metronome()?;
					}
					PlaybackState::PlayingLoop {
						beat_tracker_sequence,
						loop_sequence_id: _,
						current_beat,
					} => {
						self.playback_state = PlaybackState::QueueingFill {
							beat_tracker_sequence,
							loop_sequence_id: self.start_loop()?,
							current_beat,
							drum_fill_length: match current_beat {
								1 => 3,
								2 => 2,
								3 | 4 => 4,
								_ => unreachable!(),
							},
						}
					}
					_ => {}
				};
			}
			Message::Stop => {
				self.audio_manager
					.stop_group(self.group_id, Default::default())?;
				self.audio_manager.stop_metronome()?;
				self.playback_state = PlaybackState::Stopped;
			}
			_ => {}
		}
		Ok(())
	}

	pub fn view(&mut self) -> iced::Element<'_, Message> {
		let mut column = Column::new().push(
			Button::new(
				&mut self.play_button,
				Text::new(match self.playback_state {
					PlaybackState::Stopped => "Play",
					_ => "Stop",
				}),
			)
			.on_press(match self.playback_state {
				PlaybackState::Stopped => Message::StartSequence,
				_ => Message::Stop,
			}),
		);
		match &self.playback_state {
			PlaybackState::Stopped => {}
			PlaybackState::PlayingLoop {
				beat_tracker_sequence: _,
				loop_sequence_id: _,
				current_beat,
			} => {
				column = column.push(Text::new(format!("Beat {}", current_beat)));
			}
			PlaybackState::QueueingFill {
				beat_tracker_sequence: _,
				loop_sequence_id: _,
				current_beat,
				drum_fill_length: _,
			}
			| PlaybackState::PlayingFill {
				beat_tracker_sequence: _,
				loop_sequence_id: _,
				current_beat,
				drum_fill_length: _,
			} => {
				column = column.push(Text::new(format!("Beat {}", current_beat)));
			}
		}
		column.into()
	}
}
