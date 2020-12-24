use std::{error::Error, mem::replace};

use iced::{Button, Column, Element, Length, Space, Text};
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
	Play,
	PlayDrumFill,
	Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Beat {
	One,
	Two,
	Three,
	Four,
}

#[derive(Debug, Clone, Copy)]
enum DrumFill {
	TwoBeat,
	ThreeBeat,
	FourBeat,
}

impl DrumFill {
	fn as_usize(self) -> usize {
		match self {
			DrumFill::TwoBeat => 2,
			DrumFill::ThreeBeat => 3,
			DrumFill::FourBeat => 4,
		}
	}

	fn start_interval(self) -> f64 {
		match self {
			DrumFill::FourBeat => 4.0,
			_ => 1.0,
		}
	}
}

impl Beat {
	fn as_usize(self) -> usize {
		match self {
			Beat::One => 1,
			Beat::Two => 2,
			Beat::Three => 3,
			Beat::Four => 4,
		}
	}

	fn drum_fill(self) -> DrumFill {
		match self {
			Beat::One => DrumFill::ThreeBeat,
			Beat::Two => DrumFill::TwoBeat,
			_ => DrumFill::FourBeat,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DrumFillEvent {
	Start,
	Finish,
}

enum PlaybackState {
	Stopped,
	PlayingLoop {
		beat_tracker_sequence: (SequenceInstanceId, EventReceiver<Beat>),
		loop_sequence: (SequenceInstanceId, EventReceiver<DrumFillEvent>),
		current_beat: Beat,
	},
	QueueingFill {
		beat_tracker_sequence: (SequenceInstanceId, EventReceiver<Beat>),
		loop_sequence: (SequenceInstanceId, EventReceiver<DrumFillEvent>),
		current_beat: Beat,
		drum_fill: DrumFill,
	},
	PlayingFill {
		beat_tracker_sequence: (SequenceInstanceId, EventReceiver<Beat>),
		loop_sequence: (SequenceInstanceId, EventReceiver<DrumFillEvent>),
		current_beat: Beat,
		drum_fill: DrumFill,
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

	fn fill_sound(&mut self, fill: DrumFill) -> SoundId {
		match fill {
			DrumFill::TwoBeat => self.fill_2b_sound_id,
			DrumFill::ThreeBeat => self.fill_3b_sound_id,
			DrumFill::FourBeat => self.fill_4b_sound_id,
		}
	}

	fn start_beat_tracker(
		audio_manager: &mut AudioManager,
		group_id: GroupId,
	) -> AudioResult<(SequenceInstanceId, EventReceiver<Beat>)> {
		audio_manager.start_sequence(
			{
				let mut sequence = Sequence::new(SequenceSettings::new().groups([group_id]));
				sequence.wait_for_interval(1.0);
				sequence.start_loop();
				sequence.emit(Beat::One);
				sequence.wait(Duration::Beats(1.0));
				sequence.emit(Beat::Two);
				sequence.wait(Duration::Beats(1.0));
				sequence.emit(Beat::Three);
				sequence.wait(Duration::Beats(1.0));
				sequence.emit(Beat::Four);
				sequence.wait(Duration::Beats(1.0));
				sequence
			},
			Default::default(),
		)
	}

	fn start_loop_sequence(
		audio_manager: &mut AudioManager,
		group_id: GroupId,
		loop_sound_id: SoundId,
	) -> AudioResult<(SequenceInstanceId, EventReceiver<DrumFillEvent>)> {
		audio_manager.start_sequence(
			{
				let mut sequence = Sequence::new(SequenceSettings::new().groups([group_id]));
				sequence.wait_for_interval(1.0);
				sequence.start_loop();
				sequence.play(loop_sound_id, Default::default());
				sequence.wait(Duration::Beats(4.0));
				sequence
			},
			Default::default(),
		)
	}

	fn start_fill_and_loop_sequence(
		audio_manager: &mut AudioManager,
		group_id: GroupId,
		start_interval: f64,
		previous_loop_sequence_id: SequenceInstanceId,
		fill_sound_id: SoundId,
		loop_sound_id: SoundId,
	) -> AudioResult<(SequenceInstanceId, EventReceiver<DrumFillEvent>)> {
		audio_manager.start_sequence(
			{
				let mut sequence = Sequence::new(SequenceSettings::new().groups([group_id]));
				sequence.wait_for_interval(start_interval);
				sequence.emit(DrumFillEvent::Start);
				sequence.stop_sequence_and_instances(previous_loop_sequence_id, Default::default());
				sequence.play(fill_sound_id, Default::default());
				sequence.wait_for_interval(4.0);
				sequence.emit(DrumFillEvent::Finish);
				sequence.start_loop();
				sequence.play(loop_sound_id, Default::default());
				sequence.wait(Duration::Beats(4.0));
				sequence
			},
			Default::default(),
		)
	}

	pub fn update(&mut self, message: Message) -> Result<(), Box<dyn Error>> {
		match message {
			Message::Play => {
				self.playback_state = PlaybackState::PlayingLoop {
					beat_tracker_sequence: Self::start_beat_tracker(
						&mut self.audio_manager,
						self.group_id,
					)?,
					loop_sequence: Self::start_loop_sequence(
						&mut self.audio_manager,
						self.group_id,
						self.loop_sound_id,
					)?,
					current_beat: Beat::One,
				};
				self.audio_manager.start_metronome()?;
			}
			Message::PlayDrumFill => {
				match replace(&mut self.playback_state, PlaybackState::Stopped) {
					PlaybackState::PlayingLoop {
						beat_tracker_sequence,
						loop_sequence,
						current_beat,
					} => {
						let drum_fill = current_beat.drum_fill();
						let fill_sound_id = self.fill_sound(drum_fill);
						let start_interval = drum_fill.start_interval();
						self.playback_state = PlaybackState::QueueingFill {
							beat_tracker_sequence,
							loop_sequence: Self::start_fill_and_loop_sequence(
								&mut self.audio_manager,
								self.group_id,
								start_interval,
								loop_sequence.0,
								fill_sound_id,
								self.loop_sound_id,
							)?,
							current_beat,
							drum_fill,
						}
					}
					_ => unreachable!(),
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

	pub fn check_for_events(&mut self) -> Result<(), Box<dyn Error>> {
		match &mut self.playback_state {
			PlaybackState::PlayingLoop {
				beat_tracker_sequence,
				loop_sequence: _,
				current_beat,
			} => {
				while let Some(beat) = beat_tracker_sequence.1.pop() {
					*current_beat = *beat;
				}
			}
			PlaybackState::QueueingFill {
				beat_tracker_sequence,
				loop_sequence,
				current_beat,
				drum_fill,
			} => {
				while let Some(beat) = beat_tracker_sequence.1.pop() {
					*current_beat = *beat;
				}
				while let Some(event) = loop_sequence.1.pop() {
					match event {
						DrumFillEvent::Start => {
							let playback_state =
								replace(&mut self.playback_state, PlaybackState::Stopped);
							self.playback_state = PlaybackState::PlayingFill {
								beat_tracker_sequence: *beat_tracker_sequence,
								loop_sequence: *loop_sequence,
								current_beat: *current_beat,
								drum_fill: *drum_fill,
							}
						}
						DrumFillEvent::Finish => {}
					}
				}
			}
			PlaybackState::PlayingFill {
				beat_tracker_sequence,
				loop_sequence,
				current_beat,
				drum_fill,
			} => {
				while let Some(beat) = beat_tracker_sequence.1.pop() {
					*current_beat = *beat;
				}
			}
			_ => {}
		}
		Ok(())
	}

	pub fn view(&mut self) -> iced::Element<'_, Message> {
		let mut column = Column::new()
			.push(
				Button::new(
					&mut self.play_button,
					Text::new(match &mut self.playback_state {
						PlaybackState::Stopped => "Play",
						_ => "Stop",
					}),
				)
				.on_press(match &mut self.playback_state {
					PlaybackState::Stopped => Message::Play,
					_ => Message::Stop,
				}),
			)
			.push({
				let mut button =
					Button::new(&mut self.play_drum_fill_button, Text::new("Play drum fill"));
				match &mut self.playback_state {
					PlaybackState::PlayingLoop {
						beat_tracker_sequence: _,
						loop_sequence: _,
						current_beat: _,
					} => {
						button = button.on_press(Message::PlayDrumFill);
					}
					_ => {}
				}
				button
			});
		match &mut self.playback_state {
			PlaybackState::PlayingLoop {
				beat_tracker_sequence: _,
				loop_sequence: _,
				current_beat,
			} => {
				column = column.push(Text::new(format!("Beat {}", current_beat.as_usize())));
			}
			_ => {}
		};
		column.into()
	}
}
