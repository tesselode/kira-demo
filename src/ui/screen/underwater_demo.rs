use std::error::Error;

use iced::{Align, Button, Column, Length, Text};
use kira::{
	arrangement::{Arrangement, ArrangementId, LoopArrangementSettings},
	instance::{InstanceSettings, StopInstanceSettings},
	manager::AudioManager,
	mixer::effect::filter::{Filter, FilterSettings},
	parameter::Mapping,
	parameter::{ParameterId, Tween},
	playable::PlayableSettings,
	sequence::{Sequence, SequenceInstanceId},
	sound::Sound,
	Tempo, Value,
};

use crate::ui::{common::screen_wrapper::ScreenWrapper, style::AppStyles};

const EXPLANATION_TEXT: &str = "This demo uses a single \
parameter to control the cutoff frequency of a filter, \
the volume of the drums, and the volume of the pad.

Each of these values uses a different mapping to properly \
respond to the change in the \"underwater\" parameter.";

#[derive(Debug, Copy, Clone)]
pub enum Message {
	GoToDemoSelect,
	Play,
	Stop,
	Submerge,
	Resurface,
}

pub struct UnderwaterDemo {
	audio_manager: AudioManager,
	drums_loop_id: ArrangementId,
	bass_loop_id: ArrangementId,
	pad_loop_id: ArrangementId,
	lead_loop_id: ArrangementId,
	sequence_id: Option<SequenceInstanceId>,
	underwater_parameter_id: ParameterId,
	underwater: bool,
	screen_wrapper: ScreenWrapper<Message>,
	play_button: iced::button::State,
	underwater_button: iced::button::State,
}

impl UnderwaterDemo {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let mut audio_manager = AudioManager::new(Default::default())?;
		let underwater_parameter_id = audio_manager.add_parameter(0.0)?;
		let lead_track_id = audio_manager.add_sub_track(Default::default())?;
		audio_manager.add_effect_to_track(
			lead_track_id,
			Filter::new(FilterSettings::new().cutoff(Value::Parameter(
				underwater_parameter_id,
				Mapping {
					input_range: (0.0, 1.0),
					output_range: (8000.0, 2000.0),
					..Default::default()
				},
			))),
			Default::default(),
		)?;
		let assets_base_dir = std::env::current_dir()?.join("assets/underwater demo");
		let drums_sound_id = audio_manager.add_sound(Sound::from_file(
			assets_base_dir.join("drums.ogg"),
			PlayableSettings::new().semantic_duration(Tempo(85.0).beats_to_seconds(16.0)),
		)?)?;
		let drums_loop_id = audio_manager
			.add_arrangement(Arrangement::new_loop(drums_sound_id, Default::default()))?;
		let bass_sound_id = audio_manager.add_sound(Sound::from_file(
			assets_base_dir.join("bass.ogg"),
			PlayableSettings::new().semantic_duration(Tempo(85.0).beats_to_seconds(16.0)),
		)?)?;
		let bass_loop_id = audio_manager
			.add_arrangement(Arrangement::new_loop(bass_sound_id, Default::default()))?;
		let pad_sound_id = audio_manager.add_sound(Sound::from_file(
			assets_base_dir.join("pad.ogg"),
			PlayableSettings::new().semantic_duration(Tempo(85.0).beats_to_seconds(16.0)),
		)?)?;
		let pad_loop_id = audio_manager
			.add_arrangement(Arrangement::new_loop(pad_sound_id, Default::default()))?;
		let lead_sound_id = audio_manager.add_sound(Sound::from_file(
			assets_base_dir.join("lead.ogg"),
			PlayableSettings::new().semantic_duration(Tempo(85.0).beats_to_seconds(16.0)),
		)?)?;
		let lead_loop_id = audio_manager.add_arrangement(Arrangement::new_loop(
			lead_sound_id,
			LoopArrangementSettings::new().default_track(lead_track_id),
		))?;
		Ok(Self {
			audio_manager,
			drums_loop_id,
			bass_loop_id,
			pad_loop_id,
			lead_loop_id,
			sequence_id: None,
			underwater_parameter_id,
			underwater: false,
			screen_wrapper: ScreenWrapper::new("Underwater demo".into(), Message::GoToDemoSelect),
			play_button: iced::button::State::new(),
			underwater_button: iced::button::State::new(),
		})
	}

	pub fn update(&mut self, message: Message) -> Result<(), Box<dyn Error>> {
		match message {
			Message::Play => {
				let (sequence_id, _) = self.audio_manager.start_sequence(
					{
						let mut sequence = Sequence::<()>::new(Default::default());
						sequence.play(
							self.drums_loop_id,
							InstanceSettings::new().volume(Value::Parameter(
								self.underwater_parameter_id,
								Mapping {
									input_range: (0.0, 1.0),
									output_range: (1.0, 0.0),
									..Default::default()
								},
							)),
						);
						sequence.play(self.bass_loop_id, Default::default());
						sequence.play(
							self.pad_loop_id,
							InstanceSettings::new().volume(self.underwater_parameter_id),
						);
						sequence.play(self.lead_loop_id, Default::default());
						sequence
					},
					Default::default(),
				)?;
				self.sequence_id = Some(sequence_id)
			}
			Message::Stop => {
				if let Some(sequence_id) = self.sequence_id {
					self.audio_manager.stop_sequence_and_instances(
						sequence_id,
						StopInstanceSettings::new().fade_tween(Tween::linear(1.0)),
					)?;
					self.sequence_id = None;
				}
			}
			Message::Submerge => {
				self.audio_manager.set_parameter(
					self.underwater_parameter_id,
					1.0,
					Some(4.0.into()),
				)?;
				self.underwater = true;
			}
			Message::Resurface => {
				self.audio_manager.set_parameter(
					self.underwater_parameter_id,
					0.0,
					Some(4.0.into()),
				)?;
				self.underwater = false;
			}
			_ => {}
		}
		Ok(())
	}

	pub fn view(&mut self) -> iced::Element<'_, Message> {
		let play_button = Button::new(
			&mut self.play_button,
			Text::new(match self.sequence_id {
				Some(_) => "Stop",
				None => "Play",
			})
			.size(24),
		)
		.on_press(match self.sequence_id {
			Some(_) => Message::Stop,
			None => Message::Play,
		})
		.style(AppStyles);

		let underwater_button = Button::new(
			&mut self.underwater_button,
			Text::new(match self.underwater {
				false => "Submerge",
				true => "Resurface",
			})
			.size(24),
		)
		.on_press(match self.underwater {
			false => Message::Submerge,
			true => Message::Resurface,
		})
		.style(AppStyles);

		self.screen_wrapper.view(
			Column::new()
				.spacing(16)
				.align_items(Align::Center)
				.push(play_button)
				.push(underwater_button)
				.push(
					Column::new()
						.width(Length::Fill)
						.max_width(600)
						.push(Text::new(EXPLANATION_TEXT)),
				),
		)
	}
}
