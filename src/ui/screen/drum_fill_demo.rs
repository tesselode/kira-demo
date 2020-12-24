use std::error::Error;

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

impl Beat {
	fn as_usize(self) -> usize {
		match self {
			Beat::One => 1,
			Beat::Two => 2,
			Beat::Three => 3,
			Beat::Four => 4,
		}
	}
}

#[derive(Debug, Clone, Copy)]
enum FillLength {
	TwoBeats,
	ThreeBeats,
	FourBeats,
}

impl FillLength {
	fn as_usize(self) -> usize {
		match self {
			FillLength::TwoBeats => 2,
			FillLength::ThreeBeats => 3,
			FillLength::FourBeats => 4,
		}
	}
}

enum PlaybackState {
	Stopped,
	PlayingLoop {
		beat_tracker_sequence: (SequenceInstanceId, EventReceiver<Beat>),
		loop_sequence: (SequenceInstanceId, EventReceiver<()>),
		current_beat: Beat,
	},
	QueueingFill {
		beat_tracker_sequence: (SequenceInstanceId, EventReceiver<Beat>),
		loop_sequence: (SequenceInstanceId, EventReceiver<()>),
		current_beat: Beat,
		fill_length: FillLength,
	},
	PlayingFill {
		beat_tracker_sequence: (SequenceInstanceId, EventReceiver<Beat>),
		loop_sequence: (SequenceInstanceId, EventReceiver<()>),
		current_beat: Beat,
		fill_length: FillLength,
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

	fn start_loop_sequence(&mut self) -> AudioResult<(SequenceInstanceId, EventReceiver<()>)> {
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

	pub fn update(&mut self, message: Message) -> Result<(), Box<dyn Error>> {
		match message {
			Message::Play => {
				self.playback_state = PlaybackState::PlayingLoop {
					beat_tracker_sequence: self.start_beat_tracker()?,
					loop_sequence: self.start_loop_sequence()?,
					current_beat: Beat::One,
				};
				self.audio_manager.start_metronome()?;
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
			_ => {}
		}
		Ok(())
	}

	pub fn view(&mut self) -> iced::Element<'_, Message> {
		let mut column = Column::new().push(
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
		);
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
