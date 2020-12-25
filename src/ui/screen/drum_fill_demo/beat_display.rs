use iced::{mouse::Interaction, Background, Color, Length, Point, Rectangle, Size};
use iced_graphics::{Backend, Defaults, Primitive, Renderer};
use iced_native::{
	layout::{Limits, Node},
	Element, Layout, Widget,
};

use super::{Beat, DrumFill};

const DEFAULT_SIZE: Size = Size::new(200.0, 10.0);
const DEFAULT_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 0.25];
const CURRENT_BEAT_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const FILL_COLOR: [f32; 4] = [1.0, 0.0, 0.0, 0.5];

pub struct BeatDisplay {
	pub beat: Option<Beat>,
	pub fill: Option<DrumFill>,
}

impl BeatDisplay {
	fn does_drum_fill_occupy_beat(beat_index: usize, fill: DrumFill) -> bool {
		match fill {
			DrumFill::TwoBeat => beat_index > 2,
			DrumFill::ThreeBeat => beat_index > 1,
			DrumFill::FourBeat => true,
		}
	}

	fn circle_color(&self, beat_index: usize) -> [f32; 4] {
		if let Some(beat) = self.beat {
			if beat.as_usize() == beat_index {
				return CURRENT_BEAT_COLOR;
			}
		}
		if let Some(fill) = self.fill {
			if Self::does_drum_fill_occupy_beat(beat_index, fill) {
				return FILL_COLOR;
			}
		}
		DEFAULT_COLOR
	}
}

impl<Message, B: Backend> Widget<Message, Renderer<B>> for BeatDisplay {
	fn width(&self) -> Length {
		Length::Shrink
	}

	fn height(&self) -> Length {
		Length::Shrink
	}

	fn layout(&self, renderer: &Renderer<B>, limits: &Limits) -> Node {
		Node::new(DEFAULT_SIZE)
	}

	fn draw(
		&self,
		renderer: &mut Renderer<B>,
		defaults: &Defaults,
		layout: Layout<'_>,
		cursor_position: Point,
		viewport: &Rectangle,
	) -> (Primitive, Interaction) {
		let bounds = layout.bounds();
		let circle_radius = bounds.height / 2.0;
		(
			Primitive::Group {
				primitives: (1..=4)
					.map(|i| {
						let x = bounds.x
							+ (bounds.width - circle_radius * 2.0) * ((i - 1) as f32) / 3.0;
						let y = bounds.y;
						Primitive::Quad {
							bounds: Rectangle::new(
								Point::new(x, y),
								Size::new(circle_radius * 2.0, circle_radius * 2.0),
							),
							background: Background::Color(self.circle_color(i).into()),
							border_radius: circle_radius,
							border_width: 0.0,
							border_color: Color::TRANSPARENT,
						}
					})
					.collect(),
			},
			Interaction::default(),
		)
	}

	fn hash_layout(&self, state: &mut iced_native::Hasher) {
		use std::hash::Hash;
		0.0f32.to_bits().hash(state)
	}
}

impl<'a, Message, B: Backend> Into<Element<'a, Message, Renderer<B>>> for BeatDisplay {
	fn into(self) -> Element<'a, Message, Renderer<B>> {
		Element::new(self)
	}
}
