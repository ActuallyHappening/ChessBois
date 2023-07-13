use bevy::prelude::*;

pub trait TransformExt {
	fn translate(self, delta: Vec3) -> Self;
}
impl TransformExt for Transform {
	fn translate(mut self, delta: Vec3) -> Self {
		self.translation += delta;
		self
	}
}