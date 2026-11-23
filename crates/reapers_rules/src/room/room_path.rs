//! Choosing a route: leaps between regions, walking inside one.

use std::cell::RefCell;
use std::collections::BinaryHeap;

use crate::hunt::hunt_stats::HuntRules;
use crate::math;
use crate::room::room_nav::JumpLink;
use crate::room::room_nav::RoomNav;
use crate::room::room_nav::RoomPath;
use crate::room::room_nav::RoomStep;
use crate::vec3::Vec3;

/// Reusable scratch tables for A* and link Dijkstra, avoiding ~170 KB of
/// heap allocation and zeroing per path query.
#[derive(Default)]
struct PathScratch {
	stamp: u32,
	seen: Vec<u32>,
	closed: Vec<u32>,
	distance: Vec<u32>,
	previous: Vec<u16>,
	/// Packed min-heap entries: `u64::MAX - ((estimate << 16) | cell)` so a
	/// standard max-heap pops smallest `(estimate, cell)` in a single register.
	open: BinaryHeap<u64>,
	route_distance: Vec<i64>,
	route_previous: Vec<usize>,
	route_visited: Vec<bool>,
}

impl PathScratch {
	const NONE: u16 = u16::MAX;

	fn prepare_astar(&mut self, cell_count: usize) -> u32 {
		if self.seen.len() < cell_count {
			self.seen.resize(cell_count, 0);
			self.closed.resize(cell_count, 0);
			self.distance.resize(cell_count, 0);
			self.previous.resize(cell_count, Self::NONE);
		}
		self.stamp = self.stamp.wrapping_add(1);
		if self.stamp == 0 {
			self.seen.fill(0);
			self.closed.fill(0);
			self.stamp = 1;
		}
		self.open.clear();
		self.stamp
	}
}

thread_local! {
	static PATH_SCRATCH: RefCell<PathScratch> = RefCell::new(PathScratch::default());
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
		let from_cell = self.start_cell(from)?;
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

		if !self
			.reachable_slice(from_region as usize)
			.is_some_and(|row| row[to_region as usize])
		{
			return None;
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

	/// Whether a route exists from `from` to `to`, without running A* or
	/// Dijkstra. Regions are connected components of standable cells, so
	/// reachability in the region link graph is exact.
	pub(crate) fn has_path(&self, from: Vec3, to: Vec3) -> bool {
		let Some(from_cell) = self.start_cell(from) else {
			return false;
		};
		let Some(to_cell) = self.standable_cell_near(to, 20) else {
			return false;
		};
		let from_region = self.region[from_cell];
		let to_region = self.region[to_cell];
		if from_region < 0 || to_region < 0 {
			return false;
		}
		from_region == to_region
			|| self
				.reachable_slice(from_region as usize)
				.is_some_and(|row| row[to_region as usize])
	}

	/// The standable cell a route from a body at `from` starts on: the
	/// nearest on the surface its feet are on, so a body standing off its
	/// footing, where a charge left it, walks back onto the floor it is on
	/// rather than up onto the desk beside it. Where that surface has none
	/// near, the nearest of any.
	fn start_cell(&self, from: Vec3) -> Option<usize> {
		let feet = from.y - self.profile.half_height();

		self.standable_cell_near_where(from, 20, |cell| {
			self.support_height_of(cell)
				.is_some_and(|height| (height - feet).abs() < 0.005)
		})
		.or_else(|| self.standable_cell_near(from, 20))
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

		let from_reg = usize::try_from(from_region).ok()?;
		if self.links_by_region.get(from_reg).is_none_or(Vec::is_empty) {
			return None;
		}

		PATH_SCRATCH.with_borrow_mut(|scratch| {
			let unreached = 1_i64 << 40;
			scratch.route_distance.clear();
			scratch.route_distance.resize(count, unreached);
			scratch.route_previous.clear();
			scratch.route_previous.resize(count, usize::MAX);
			scratch.route_visited.clear();
			scratch.route_visited.resize(count, false);

			let distance = &mut scratch.route_distance;
			let previous = &mut scratch.route_previous;
			let visited = &mut scratch.route_visited;

			for &index in &self.links_by_region[from_reg] {
				let link = &self.links[index];
				distance[index] =
					self.walk_ticks(from_cell, link.from_cell) + Self::link_cost(link);
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
				let base = distance[best];

				for &index in &self.links_by_region[chosen.to_region] {
					if visited[index] {
						continue;
					}

					let next_link = &self.links[index];
					let step = base
						+ self.walk_ticks(chosen.to_cell, next_link.from_cell)
						+ Self::link_cost(next_link);

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
		})
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

		let width = usize::from(self.grid.width);
		let dx = (from_cell % width).abs_diff(to_cell % width) as i64;
		let dz = (from_cell / width).abs_diff(to_cell / width) as i64;
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

		let cell_count = self.grid.cell_count();
		let width = usize::from(self.grid.width);
		let depth = usize::from(self.grid.depth);
		let goal_x = goal % width;
		let goal_z = goal / width;
		let target_reg = i16::try_from(region).ok()?;
		let sr = self.standable_region.as_slice();

		let heuristic_at = |x: usize, z: usize| -> u32 {
			let dx = x.abs_diff(goal_x) as u32;
			let dz = z.abs_diff(goal_z) as u32;
			10 * dx.max(dz) + 4 * dx.min(dz)
		};
		let pack = |estimate: u32, cell: usize| -> u64 {
			u64::MAX - ((u64::from(estimate) << 16) | (cell as u64))
		};

		PATH_SCRATCH.with_borrow_mut(|scratch| {
			let stamp = scratch.prepare_astar(cell_count);
			let start_x = start % width;
			let start_z = start / width;

			scratch.seen[start] = stamp;
			scratch.distance[start] = 0;
			scratch.previous[start] = PathScratch::NONE;
			scratch
				.open
				.push(pack(heuristic_at(start_x, start_z), start));

			while let Some(packed) = scratch.open.pop() {
				let raw = u64::MAX - packed;
				let cell = (raw & 0xFFFF) as usize;

				if scratch.closed[cell] == stamp {
					continue;
				}

				if cell == goal {
					return Some(Self::reconstruct(&scratch.previous, cell));
				}

				scratch.closed[cell] = stamp;
				let cell_dist = scratch.distance[cell];
				let x = cell % width;
				let z = cell / width;

				for (dx, dz) in crate::room::room_nav::NEIGHBOURS {
					let nx = x as isize + dx as isize;
					let nz = z as isize + dz as isize;

					if nx < 0 || nx >= width as isize || nz < 0 || nz >= depth as isize {
						continue;
					}

					let nxu = nx as usize;
					let nzu = nz as usize;
					let next = nzu * width + nxu;

					if sr[next] != target_reg {
						continue;
					}

					// A diagonal step cuts a corner unless both sides of the
					// corner are open too.
					if dx != 0
						&& dz != 0
						&& (sr[z * width + nxu] != target_reg || sr[nzu * width + x] != target_reg)
					{
						continue;
					}

					let step = cell_dist + if dx != 0 && dz != 0 { 14 } else { 10 };

					if scratch.seen[next] == stamp && step >= scratch.distance[next] {
						continue;
					}

					scratch.seen[next] = stamp;
					scratch.distance[next] = step;
					scratch.previous[next] = cell as u16;
					scratch.open.push(pack(step + heuristic_at(nxu, nzu), next));
				}
			}

			None
		})
	}

	/// The way back from the A* search, start first.
	fn reconstruct(previous: &[u16], goal: usize) -> Vec<usize> {
		let mut path = vec![goal];
		let mut at = goal;

		while previous[at] != PathScratch::NONE {
			at = usize::from(previous[at]);
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
		let width = usize::from(self.grid.width);
		let mut x = (from % width) as i64;
		let mut z = (from / width) as i64;
		let target_x = (to % width) as i64;
		let target_z = (to / width) as i64;
		let dx = (target_x - x).abs();
		let dz = (target_z - z).abs();
		let step_x = if x < target_x { 1 } else { -1 };
		let step_z = if z < target_z { 1 } else { -1 };
		let mut error = dx - dz;
		let sr = self.standable_region.as_slice();
		let target_reg = sr[from];
		if target_reg < 0 {
			return false;
		}

		loop {
			let here = (z as usize) * width + (x as usize);

			if sr[here] != target_reg {
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
				&& (sr[(z as usize) * width + ((x - step_x) as usize)] != target_reg
					|| sr[((z - step_z) as usize) * width + (x as usize)] != target_reg)
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
