use std::borrow::Cow;

use bevy::{ecs::system::EntityCommands, prelude::*};

pub trait TransformExt {
	fn translate(self, delta: Vec3) -> Self;
}
impl TransformExt for Transform {
	fn translate(mut self, delta: Vec3) -> Self {
		self.translation += delta;
		self
	}
}

pub trait EntityCommandsExt {
	fn name<T: Into<Cow<'static, str>>>(&mut self, name: T) -> &mut Self;
}

impl EntityCommandsExt for EntityCommands<'_, '_, '_> {
	fn name<T: Into<Cow<'static, str>>>(&mut self, name: T) -> &mut Self {
		self.insert(Name::new(name))
	}
}
