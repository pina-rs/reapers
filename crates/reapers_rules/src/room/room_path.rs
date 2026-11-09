//! Choosing a route: leaps between regions, walking inside one.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::hunt::hunt_stats::HuntRules;
use crate::math;
use crate::room::room_nav::JumpLink;
use crate::room::room_nav::RoomNav;
use crate::room::room_nav::RoomPath;
use crate::room::room_nav::RoomStep;
use crate::vec3::Vec3;

/// A cell on the way to a goal, ordered by its estimate and then by its
/// index, so ties break the same way every time.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Node {
	estimate: i64,
	cell: usize,
}

impl RoomNav {
	/// A route from `from` to `to`, or nothing when the room has no way
	/// between them for this body.
	///
	/// Walking legs are found with A* inside one region, then
	/// string-pulled so the husk walks in straight lines rather than
	/// along the grid. Leaps are chosen by a search over the links, so a
	/// penguin can go floor, chair, desk when it cannot reach the desk in
	/// one bound.
	pub fn path(&self, from: Vec3, to: Vec3) -> Option<RoomPath> {
		let from_cell = self.standable_cell_near(from, 20)?;
		let to_cell = self.standable_cell_near(to, 20)?;
		let from_region = self.region[from_cell];
		let to_region = self.region[to_cell];

		if from_region < 0 || to_region < 0 {
			return None;
		}

		if from_region == to_region {
			let walking = self.astar(from_cell, to_cell, from_region)?;

			return Some(RoomPath {
				steps: self.steps_of(self.pull(walking)),
			});
		}

		let route = self.route(from_cell, to_cell, from_region, to_region)?;
		let mut steps: Vec<RoomStep> = Vec::new();
		let mut current = from_cell;

		for link in route {
			let walking = self.astar(current, link.from_cell, self.region[current])?;
			Self::add_steps(&mut steps, self.steps_of(self.pull(walking)));

			if steps.last().is_none_or(|last| last.cell != link.from_cell) {
				return None;
			}

			let take_off = steps.pop().expect("checked above");

			steps.push(RoomStep {
				leap: Some(*link),
				..take_off
			});
			steps.push(self.step_of(link.to_cell));
			current = link.to_cell;
		}

		let walking = self.astar(current, to_cell, self.region[current])?;
		Self::add_steps(&mut steps, self.steps_of(self.pull(walking)));

		Some(RoomPath { steps })
	}

	/// The cheapest chain of links from one region to another, in ticks,
	/// with the walk between them counted too.
	fn route(
		&self,
		from_cell: usize,
		to_cell: usize,
		from_region: i64,
		to_region: i64,
	) -> Option<Vec<&JumpLink>> {
		let count = self.links.len();

		if count == 0 {
			return None;
		}

		let unreached = 1_i64 << 40;
		let mut distance = vec![unreached; count];
		let mut previous = vec![usize::MAX; count];
		let mut visited = vec![false; count];

		for (index, link) in self.links.iter().enumerate() {
			if link.from_region as i64 == from_region {
				distance[index] =
					self.walk_ticks(from_cell, link.from_cell) + Self::link_cost(link);
			}
		}

		loop {
			let mut best = usize::MAX;

			for index in 0..count {
				if visited[index] || distance[index] >= unreached {
					continue;
				}

				if best == usize::MAX || distance[index] < distance[best] {
					best = index;
				}
			}

			if best == usize::MAX {
				break;
			}

			visited[best] = true;
			let chosen = &self.links[best];

			for index in 0..count {
				if visited[index] || self.links[index].from_region != chosen.to_region {
					continue;
				}

				let step = distance[best]
					+ self.walk_ticks(chosen.to_cell, self.links[index].from_cell)
					+ Self::link_cost(&self.links[index]);

				if step < distance[index] {
					distance[index] = step;
					previous[index] = best;
				}
			}
		}

		let mut end = usize::MAX;
		let mut end_cost = unreached;

		for index in 0..count {
			if self.links[index].to_region as i64 != to_region || distance[index] >= unreached {
				continue;
			}

			let cost = distance[index] + self.walk_ticks(self.links[index].to_cell, to_cell);

			if cost < end_cost {
				end_cost = cost;
				end = index;
			}
		}

		if end == usize::MAX {
			return None;
		}

		let mut route: Vec<&JumpLink> = Vec::new();
		let mut at = Some(end);

		while let Some(index) = at {
			route.push(&self.links[index]);
			at = (previous[index] != usize::MAX).then_some(previous[index]);
		}

		route.reverse();
		Some(route)
	}

	/// A link's cost in ticks: its flight plus the penalty for leaving
	/// the ground.
	fn link_cost(link: &JumpLink) -> i64 {
		link.ticks() + Self::JUMP_PENALTY_TICKS
	}

	/// How long a walk from one cell to another takes, in ticks, as the
	/// octile distance walks at the body's own pace.
	fn walk_ticks(&self, from_cell: usize, to_cell: usize) -> i64 {
		if from_cell == to_cell {
			return 0;
		}

		let dx = (self.grid.x_of(from_cell) - self.grid.x_of(to_cell)).abs();
		let dz = (self.grid.z_of(from_cell) - self.grid.z_of(to_cell)).abs();
		let octile = 10 * dx.max(dz) + 4 * dx.min(dz);
		let speed = math::max(self.profile.walk_speed, 0.1);

		// The octile count is tenths of a cell, so a hundredth of it is
		// metres.
		math::round(octile as f64 / 100.0 / speed * HuntRules::TICK_RATE as f64)
	}

	/// The shortest way across one region, 8-connected with octile costs.
	fn astar(&self, start: usize, goal: usize, region: i64) -> Option<Vec<usize>> {
		if start == goal {
			return Some(vec![start]);
		}

		let unreached = 1_i64 << 40;
		let mut distance = vec![unreached; self.grid.cell_count()];
		let mut previous = vec![usize::MAX; self.grid.cell_count()];
		let mut closed = vec![false; self.grid.cell_count()];
		let mut open: BinaryHeap<Reverse<Node>> = BinaryHeap::new();

		distance[start] = 0;
		open.push(Reverse(Node {
			cell: start,
			estimate: self.heuristic(start, goal),
		}));

		while let Some(Reverse(node)) = open.pop() {
			let cell = node.cell;

			if closed[cell] {
				continue;
			}

			if distance[cell] + self.heuristic(cell, goal) != node.estimate {
				continue;
			}

			if cell == goal {
				return Some(Self::reconstruct(&previous, cell));
			}

			closed[cell] = true;
			let (x, z) = (self.grid.x_of(cell), self.grid.z_of(cell));

			for (dx, dz) in crate::room::room_nav::NEIGHBOURS {
				let nx = x + dx;
				let nz = z + dz;

				if !self.grid.contains(nx, nz) {
					continue;
				}

				let next = self.grid.index_of(nx, nz);

				if !self.standable[next] || self.region[next] != region {
					continue;
				}

				// A diagonal step cuts a corner unless both sides of the
				// corner are open too.
				if dx != 0 && dz != 0 {
					let side_x = self.grid.index_of(x + dx, z);
					let side_z = self.grid.index_of(x, z + dz);

					if !self.standable[side_x]
						|| self.region[side_x] != region
						|| !self.standable[side_z]
						|| self.region[side_z] != region
					{
						continue;
					}
				}

				let step = distance[cell] + if dx != 0 && dz != 0 { 14 } else { 10 };

				if step >= distance[next] {
					continue;
				}

				distance[next] = step;
				previous[next] = cell;
				open.push(Reverse(Node {
					cell: next,
					estimate: step + self.heuristic(next, goal),
				}));
			}
		}

		None
	}

	/// The octile distance between two cells, in the A* cost's units.
	fn heuristic(&self, from: usize, to: usize) -> i64 {
		let width = usize::from(self.grid.width);
		let dx = (from % width).abs_diff(to % width) as i64;
		let dz = (from / width).abs_diff(to / width) as i64;

		10 * dx.max(dz) + 4 * dx.min(dz)
	}

	/// The way back from the A* search, start first.
	fn reconstruct(previous: &[usize], goal: usize) -> Vec<usize> {
		let mut path = vec![goal];
		let mut at = goal;

		while previous[at] != usize::MAX {
			at = previous[at];
			path.push(at);
		}

		path.reverse();
		path
	}

	/// Drops the waypoints a straight walk makes redundant: whenever one
	/// grid line of sight reaches past a waypoint, that waypoint goes.
	fn pull(&self, cells: Vec<usize>) -> Vec<usize> {
		if cells.len() <= 2 {
			return cells;
		}

		let mut kept = vec![cells[0]];
		let mut anchor = 0;

		for index in 2..cells.len() {
			if !self.line_of_sight(cells[anchor], cells[index]) {
				kept.push(cells[index - 1]);
				anchor = index - 1;
			}
		}

		if kept.last() != cells.last() {
			kept.push(cells[cells.len() - 1]);
		}

		kept
	}

	/// Whether every cell the grid walk between two cells crosses stays
	/// standable inside one region, corners included.
	fn line_of_sight(&self, from: usize, to: usize) -> bool {
		let mut x = self.grid.x_of(from);
		let mut z = self.grid.z_of(from);
		let (target_x, target_z) = (self.grid.x_of(to), self.grid.z_of(to));
		let dx = (target_x - x).abs();
		let dz = (target_z - z).abs();
		let step_x = if x < target_x { 1 } else { -1 };
		let step_z = if z < target_z { 1 } else { -1 };
		let mut error = dx - dz;
		let region = self.region[from];

		loop {
			let here = self.grid.index_of(x, z);

			if !self.standable[here] || self.region[here] != region {
				return false;
			}

			if x == target_x && z == target_z {
				return true;
			}

			let doubled = 2 * error;
			let mut moved_x = false;
			let mut moved_z = false;

			if doubled > -dz {
				error -= dz;
				x += step_x;
				moved_x = true;
			}

			if doubled < dx {
				error += dx;
				z += step_z;
				moved_z = true;
			}

			if moved_x
				&& moved_z
				&& (!self.is_standable_at(x - step_x, z)
					|| !self.is_standable_at(x, z - step_z)
					|| self.region[self.grid.index_of(x - step_x, z)] != region
					|| self.region[self.grid.index_of(x, z - step_z)] != region)
			{
				return false;
			}
		}
	}

	/// The route's cells as steps, each with its place and level.
	fn steps_of(&self, cells: Vec<usize>) -> Vec<RoomStep> {
		cells.into_iter().map(|cell| self.step_of(cell)).collect()
	}

	/// One cell as a step.
	pub(crate) fn step_of(&self, cell: usize) -> RoomStep {
		let (x, z) = (self.grid.x_of(cell), self.grid.z_of(cell));

		RoomStep {
			cell,
			level_index: self.level[cell],
			point: self.grid.centre_of(x, z),
			leap: None,
		}
	}

	/// Adds steps to a route, skipping where one ends where the next
	/// begins.
	fn add_steps(into: &mut Vec<RoomStep>, steps: Vec<RoomStep>) {
		for step in steps {
			if into.last().is_some_and(|last| last.cell == step.cell) {
				continue;
			}

			into.push(step);
		}
	}
}
