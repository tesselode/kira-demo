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

#[derive(Debug, Clone, Copy)]
pub enum DrumFill {
	TwoBeat,
	ThreeBeat,
	FourBeat,
}

impl DrumFill {
	fn length(self) -> usize {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Beat {
	One,
	Two,
	Three,
	Four,
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

	fn fill(self) -> DrumFill {
		match self {
			Beat::One => DrumFill::ThreeBeat,
			Beat::Two => DrumFill::TwoBeat,
			_ => DrumFill::FourBeat,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrumFillEvent {
	Start,
	Finish,
}

#[derive(Debug, Clone, Copy)]
enum PlaybackState {
	Stopped,
	PlayingLoop(Beat),
	QueueingFill(Beat, DrumFill),
	PlayingFill(Beat, DrumFill),
}

pub struct DrumFillDemo {
	audio_manager: AudioManager,
	group_id: GroupId,
	loop_sound_id: SoundId,
	fill_2b_sound_id: SoundId,
	fill_3b_sound_id: SoundId,
	fill_4b_sound_id: SoundId,
	playback_state: PlaybackState,
	beat_tracker_sequence: Option<(SequenceInstanceId, EventReceiver<Beat>)>,
	loop_sequence: Option<(SequenceInstanceId, EventReceiver<DrumFillEvent>)>,
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
			beat_tracker_sequence: None,
			loop_sequence: None,
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

	fn start_beat_tracker(&mut self) -> AudioResult<(SequenceInstanceId, EventReceiver<Beat>)> {
		self.audio_manager.start_sequence(
			{
				let mut sequence = Sequence::new(SequenceSettings::new().groups([self.group_id]));
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
		&mut self,
	) -> AudioResult<(SequenceInstanceId, EventReceiver<DrumFillEvent>)> {
		self.audio_manager.start_sequence(
			{
				let mut sequence = Sequence::new(SequenceSettings::new().groups([self.group_id]));
				sequence.wait_for_interval(1.0);
				sequence.start_loop();
				sequence.play(self.loop_sound_id, Default::default());
				sequence.wait(Duration::Beats(4.0));
				sequence
			},
			Default::default(),
		)
	}

	fn start_fill_and_loop_sequence(
		&mut self,
		fill: DrumFill,
	) -> AudioResult<(SequenceInstanceId, EventReceiver<DrumFillEvent>)> {
		let previous_loop_sequence = self.loop_sequence.take().unwrap();
		let fill_sound = self.fill_sound(fill);
		self.audio_manager.start_sequence(
			{
				let mut sequence = Sequence::new(SequenceSettings::new().groups([self.group_id]));
				sequence.wait_for_interval(fill.start_interval());
				sequence.emit(DrumFillEvent::Start);
				sequence.stop_sequence_and_instances(previous_loop_sequence.0, Default::default());
				sequence.play(fill_sound, Default::default());
				sequence.wait_for_interval(4.0);
				sequence.emit(DrumFillEvent::Finish);
				sequence.start_loop();
				sequence.play(self.loop_sound_id, Default::default());
				sequence.wait(Duration::Beats(4.0));
				sequence
			},
			Default::default(),
		)
	}

	pub fn update(&mut self, message: Message) -> Result<(), Box<dyn Error>> {
		match message {
			Message::Play => {
				self.playback_state = PlaybackState::PlayingLoop(Beat::One);
				self.beat_tracker_sequence = Some(self.start_beat_tracker()?);
				self.loop_sequence = Some(self.start_loop_sequence()?);
				self.audio_manager.start_metronome()?;
			}
			Message::PlayDrumFill => match self.playback_state {
				PlaybackState::PlayingLoop(beat) => {
					let fill = beat.fill();
					self.playback_state = PlaybackState::QueueingFill(beat, fill);
					self.loop_sequence = Some(self.start_fill_and_loop_sequence(fill)?);
				}
				_ => unreachable!(),
			},
			Message::Stop => {
				self.audio_manager
					.stop_group(self.group_id, Default::default())?;
				self.audio_manager.stop_metronome()?;
				self.playback_state = PlaybackState::Stopped;
				self.beat_tracker_sequence = None;
				self.loop_sequence = None;
			}
			_ => {}
		}
		Ok(())
	}

	pub fn check_for_events(&mut self) -> Result<(), Box<dyn Error>> {
		if let Some(sequence) = &mut self.beat_tracker_sequence {
			while let Some(new_beat) = sequence.1.pop() {
				match &mut self.playback_state {
					PlaybackState::PlayingLoop(beat) => {
						*beat = *new_beat;
					}
					PlaybackState::QueueingFill(beat, _) | PlaybackState::PlayingFill(beat, _) => {
						*beat = *new_beat;
					}
					_ => {}
				}
			}
		}
		if let Some(sequence) = &mut self.loop_sequence {
			while let Some(event) = sequence.1.pop() {
				match event {
					DrumFillEvent::Start => {
						if let PlaybackState::QueueingFill(beat, fill) = self.playback_state {
							self.playback_state = PlaybackState::PlayingFill(beat, fill);
						}
					}
					DrumFillEvent::Finish => {
						if let PlaybackState::PlayingFill(beat, _) = self.playback_state {
							self.playback_state = PlaybackState::PlayingLoop(beat);
						}
					}
				}
			}
		}
		Ok(())
	}

	pub fn view(&mut self) -> iced::Element<'_, Message> {
		let mut column = Column::new()
			.push(
				Button::new(
					&mut self.play_button,
					Text::new(match self.playback_state {
						PlaybackState::Stopped => "Play",
						_ => "Stop",
					}),
				)
				.on_press(match self.playback_state {
					PlaybackState::Stopped => Message::Play,
					_ => Message::Stop,
				}),
			)
			.push({
				let mut button =
					Button::new(&mut self.play_drum_fill_button, Text::new("Play drum fill"));
				match self.playback_state {
					PlaybackState::PlayingLoop(_) => {
						button = button.on_press(Message::PlayDrumFill);
					}
					_ => {}
				}
				button
			});
		match self.playback_state {
			PlaybackState::PlayingLoop(beat) => {
				column = column.push(Text::new(format!("Beat {}", beat.as_usize())));
			}
			PlaybackState::QueueingFill(beat, _) | PlaybackState::PlayingFill(beat, _) => {
				column = column.push(Text::new(format!("Beat {}", beat.as_usize())));
			}
			_ => {}
		}
		column.into()
	}
}
