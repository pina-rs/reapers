//! A model whose parts the game moves itself, tick by tick, sending the
//! host only what changed: part offsets (eased over the tick on the host's
//! clock), part visibility, material overrides and morph weights.

use std::collections::HashMap;

use leptoskit::scene3d::MaterialOverride;
use leptoskit::scene3d::NodeId;
use leptoskit::scene3d::Op;
use leptoskit::scene3d::Transform;
use leptoskit::scene3d::client::Scene;

/// One spawned model and what was last sent for each of its parts.
#[derive(Debug)]
pub struct Puppet {
	pub node: NodeId,
	offsets: HashMap<&'static str, Transform>,
	shown: HashMap<&'static str, bool>,
	looks: HashMap<&'static str, MaterialOverride>,
	morphs: HashMap<(&'static str, &'static str), f32>,
}

impl Puppet {
	/// Loads `model` and spawns it in `scene` under `parent`.
	pub fn spawn(scene: &Scene, model: &str, parent: Option<NodeId>) -> Self {
		let asset = scene.load(model);
		let node = scene.spawn_in(Some(asset), parent);
		scene.push(Op::SetShadow { node, cast: false });
		Self {
			node,
			offsets: HashMap::new(),
			shown: HashMap::new(),
			looks: HashMap::new(),
			morphs: HashMap::new(),
		}
	}

	/// Moves `part` to `offset` from where it rests, over `seconds`.
	pub fn offset(&mut self, scene: &Scene, part: &'static str, offset: Transform, seconds: f32) {
		if self
			.offsets
			.get(part)
			.is_some_and(|sent| close(sent, &offset))
		{
			return;
		}
		self.offsets.insert(part, offset);
		scene.push(Op::SetPartOffset {
			node: self.node,
			part: part.to_owned(),
			offset,
			seconds,
		});
	}

	pub fn visible(&mut self, scene: &Scene, part: &'static str, visible: bool) {
		if self.shown.get(part) == Some(&visible) {
			return;
		}
		self.shown.insert(part, visible);
		scene.set_part_visible(self.node, part, visible);
	}

	/// Whether `part` was last shown.
	pub fn shown(&self, part: &'static str) -> bool {
		self.shown.get(part).copied().unwrap_or(true)
	}

	pub fn material(&mut self, scene: &Scene, part: &'static str, look: MaterialOverride) {
		if self
			.looks
			.get(part)
			.is_some_and(|sent| same_look(sent, &look))
		{
			return;
		}
		self.looks.insert(part, look);
		scene.set_material(self.node, Some(part), look);
	}

	pub fn morph(&mut self, scene: &Scene, part: &'static str, target: &'static str, weight: f32) {
		if self
			.morphs
			.get(&(part, target))
			.is_some_and(|sent| (sent - weight).abs() < 1.0e-3)
		{
			return;
		}
		self.morphs.insert((part, target), weight);
		scene.push(Op::SetMorphWeight {
			node: self.node,
			part: Some(part.to_owned()),
			target: target.to_owned(),
			weight,
		});
	}

	pub fn remove(&self, scene: &Scene) {
		scene.remove(self.node);
	}
}

fn near(a: &[f32], b: &[f32], by: f32) -> bool {
	a.iter().zip(b).all(|(a, b)| (a - b).abs() < by)
}

fn close(a: &Transform, b: &Transform) -> bool {
	near(&a.translation, &b.translation, 1.0e-4)
		&& near(&a.rotation, &b.rotation, 1.0e-4)
		&& near(&a.scale, &b.scale, 1.0e-4)
}

fn same_look(a: &MaterialOverride, b: &MaterialOverride) -> bool {
	let pair = |a: Option<&[f32]>, b: Option<&[f32]>| {
		match (a, b) {
			(Some(a), Some(b)) => near(a, b, 2.0e-3),
			(None, None) => true,
			_ => false,
		}
	};
	pair(
		a.color.as_ref().map(|c| &c[..]),
		b.color.as_ref().map(|c| &c[..]),
	) && pair(
		a.emissive.as_ref().map(|c| &c[..]),
		b.emissive.as_ref().map(|c| &c[..]),
	) && pair(
		a.emissive_strength.as_ref().map(std::slice::from_ref),
		b.emissive_strength.as_ref().map(std::slice::from_ref),
	) && pair(
		a.opacity.as_ref().map(std::slice::from_ref),
		b.opacity.as_ref().map(std::slice::from_ref),
	) && a.rim == b.rim
}
