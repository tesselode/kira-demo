use std::error::Error;

use iced::{Align, Button, Column, Container, Length, Row, Text};
use kira::{
	arrangement::{Arrangement, ArrangementId},
	instance::InstanceSettings,
	manager::AudioManager,
	mixer::effect::filter::{Filter, FilterSettings},
	mixer::TrackIndex,
	parameter::Mapping,
	parameter::ParameterId,
	playable::PlayableSettings,
	sequence::{Sequence, SequenceId},
	sound::Sound,
	Tempo, Value,
};

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
	sequence_id: Option<SequenceId>,
	underwater_parameter_id: ParameterId,
	underwater: bool,
	back_button: iced::button::State,
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
		let sound_settings =
			PlayableSettings::new().semantic_duration(Tempo(85.0).beats_to_seconds(16.0));
		let drums_sound_id = audio_manager.add_sound(Sound::from_file(
			assets_base_dir.join("drums.ogg"),
			sound_settings,
		)?)?;
		let drums_loop_id = audio_manager.add_arrangement(Arrangement::new_loop(drums_sound_id))?;
		let bass_sound_id = audio_manager.add_sound(Sound::from_file(
			assets_base_dir.join("bass.ogg"),
			sound_settings,
		)?)?;
		let bass_loop_id = audio_manager.add_arrangement(Arrangement::new_loop(bass_sound_id))?;
		let pad_sound_id = audio_manager.add_sound(Sound::from_file(
			assets_base_dir.join("pad.ogg"),
			sound_settings,
		)?)?;
		let pad_loop_id = audio_manager.add_arrangement(Arrangement::new_loop(pad_sound_id))?;
		let lead_sound_id = audio_manager.add_sound(Sound::from_file(
			assets_base_dir.join("lead.ogg"),
			sound_settings,
		)?)?;
		let lead_loop_id = audio_manager.add_arrangement({
			let mut arrangement = Arrangement::new_loop(lead_sound_id);
			arrangement.settings.default_track = TrackIndex::Sub(lead_track_id);
			arrangement
		})?;
		Ok(Self {
			audio_manager,
			drums_loop_id,
			bass_loop_id,
			pad_loop_id,
			lead_loop_id,
			sequence_id: None,
			underwater_parameter_id,
			underwater: false,
			back_button: iced::button::State::new(),
			play_button: iced::button::State::new(),
			underwater_button: iced::button::State::new(),
		})
	}

	pub fn update(&mut self, message: Message) -> Result<(), Box<dyn Error>> {
		match message {
			Message::Play => {
				self.sequence_id = Some(self.audio_manager.start_sequence({
					let mut sequence = Sequence::new();
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
				})?)
			}
			Message::Stop => {
				if let Some(sequence_id) = self.sequence_id {
					self.audio_manager
						.stop_sequence_and_instances(sequence_id, Some(0.1.into()))?;
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
			}),
		)
		.on_press(match self.sequence_id {
			Some(_) => Message::Stop,
			None => Message::Play,
		});

		let underwater_button = Button::new(
			&mut self.underwater_button,
			Text::new(match self.underwater {
				false => "Submerge",
				true => "Resurface",
			}),
		)
		.on_press(match self.underwater {
			false => Message::Submerge,
			true => Message::Resurface,
		});

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
					.push(Text::new("Underwater demo")),
			)
			.push(
				Container::new(
					Column::new()
						.spacing(16)
						.align_items(Align::Center)
						.push(play_button)
						.push(underwater_button),
				)
				.width(Length::Fill)
				.height(Length::Fill)
				.center_x()
				.center_y(),
			)
			.into()
	}
}
