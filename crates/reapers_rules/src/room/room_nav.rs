//! The room seen as places a husk can walk and leap, built once per hunt.

use crate::math;
use crate::room::leap::Leap;
use crate::room::locomotion_profile::LocomotionProfile;
use crate::room::room_grid::RoomCell;
use crate::room::room_grid::RoomCellKind;
use crate::room::room_grid::RoomGrid;
use crate::vec3::Vec3;

/// The kinds of place a husk can make for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RoomSpotKind {
	/// An edge of something raised, facing the player, to sit and peer
	/// from.
	Perch,

	/// Floor behind cover, near enough to what hides it to look over.
	Peek,

	/// Floor behind cover, deep enough that the whole body is out of
	/// sight.
	Hide,

	/// Open floor the player can see, at a distance the husk keeps to.
	Open,
}

/// A connected surface at one level: a floor, a desk top, a chair seat.
#[derive(Clone, Debug, PartialEq)]
pub struct RoomRegion {
	/// The region's number, in the order its first cell was found.
	pub id: usize,

	/// The grid's level this surface stands at.
	pub level_index: u8,

	/// How high above the floor the surface is, in centimetres.
	pub height_centimetres: i64,

	/// Every cell of the surface, sorted by cell index.
	pub cells: Vec<usize>,

	/// The cells a husk's body fits on, with its foot radius clear of
	/// every edge. Empty when the surface is too narrow to stand on.
	pub standable_cells: Vec<usize>,
}

impl RoomRegion {
	/// Whether any part of the surface is big enough to stand on.
	pub fn is_standable(&self) -> bool {
		!self.standable_cells.is_empty()
	}

	/// How high the surface is, in metres above the floor.
	pub fn height_metres(&self) -> f64 {
		self.height_centimetres as f64 / 100.0
	}
}

/// A leap from the edge of one region to a cell of another.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JumpLink {
	/// The cell the leap leaves from.
	pub from_cell: usize,

	/// The cell the leap lands on.
	pub to_cell: usize,

	/// The region the leap leaves.
	pub from_region: usize,

	/// The region the leap lands in.
	pub to_region: usize,

	/// The ballistic arc, from the middle of the body to the middle of it.
	pub leap: Leap,

	/// How far apart the two cell centres are, in metres across the floor.
	pub gap_metres: f64,
}

impl JumpLink {
	/// The whole number of ticks the leap takes.
	pub fn ticks(&self) -> i64 {
		self.leap.ticks
	}

	/// The direction the leap goes, as a horizontal unit vector.
	pub fn direction(&self) -> Vec3 {
		Vec3::new(
			self.leap.to.x - self.leap.from.x,
			0.0,
			self.leap.to.z - self.leap.from.z,
		)
		.normalized()
	}
}

/// A place a husk might choose to be.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoomSpot {
	/// What kind of place it is.
	pub kind: RoomSpotKind,

	/// The cell to stand on.
	pub cell: usize,

	/// The grid's level this place stands at.
	pub level_index: u8,

	/// The middle of the surface, in the hunt's frame.
	pub point: Vec3,

	/// Which way the body should face on arrival, when the spot asks for
	/// it.
	pub facing: Option<Vec3>,
}

/// One place to walk to, and the leap from it when it is the edge of one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoomStep {
	/// The cell to stand on.
	pub cell: usize,

	/// The grid's level this place stands at.
	pub level_index: i8,

	/// The middle of the surface a foot stands on.
	pub point: Vec3,

	/// When set, the husk leaps from here to the next step instead of
	/// walking.
	pub leap: Option<JumpLink>,
}

/// A route through the room: places to walk to, with a leap between some.
#[derive(Clone, Debug, PartialEq)]
pub struct RoomPath {
	/// The places along the way, in the order they are visited.
	pub steps: Vec<RoomStep>,
}

impl RoomPath {
	/// Whether the route has no steps at all.
	pub fn is_empty(&self) -> bool {
		self.steps.is_empty()
	}

	/// The last place the route reaches.
	pub fn destination(&self) -> Option<&RoomStep> {
		self.steps.last()
	}

	/// The leaps the route makes, in order.
	pub fn leaps(&self) -> impl Iterator<Item = &JumpLink> {
		self.steps.iter().filter_map(|step| step.leap.as_ref())
	}
}

/// One boundary cell of a standable region, with the way out of it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Edge {
	/// The boundary cell.
	pub cell: usize,

	/// The region it belongs to.
	pub region: usize,

	/// The way out of the region, across the floor, as a unit vector.
	pub normal_x: f64,
	pub normal_z: f64,

	/// The normal rounded to one of nine directions, so a straight run of
	/// edge shares a bucket.
	pub bucket: i64,

	/// How far along the bucket's tangent the cell is, in cells.
	pub along: i64,
}

/// The eight neighbours of a cell, in index order.
pub(crate) const NEIGHBOURS: [(i64, i64); 8] = [
	(-1, -1),
	(0, -1),
	(1, -1),
	(-1, 0),
	(1, 0),
	(-1, 1),
	(0, 1),
	(1, 1),
];

/// The four orthogonal neighbours of a cell.
pub(crate) const ORTHOGONAL_NEIGHBOURS: [(i64, i64); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

/// The room's walkable surfaces, the leaps between them, and the places
/// worth standing, derived from a grid and a body.
///
/// It is built once per hunt in whole-number arithmetic. Every search
/// visits cells in index order and breaks ties by cell index, so the same
/// room and the same body always give the same answer, wherever it is
/// computed.
pub struct RoomNav {
	/// The room.
	pub grid: RoomGrid,

	/// The body and its jumping numbers.
	pub profile: LocomotionProfile,

	/// How far from the player the husk usually keeps, in metres. It
	/// widens the safety margin around every edge, because a depth
	/// reading is less certain farther away.
	pub range: f64,

	/// The nearest open floor is chosen no closer than this, in metres.
	pub near_metres: f64,

	/// Every surface big enough to matter, in the order its first cell
	/// was found.
	pub regions: Vec<RoomRegion>,

	/// Every leap the room offers, sorted by take-off cell.
	pub links: Vec<JumpLink>,

	/// Every perch, peek, hide and open place, sorted by kind and cell.
	pub spots: Vec<RoomSpot>,

	/// The level index of every cell, or -1 where it is not walkable.
	pub(crate) level: Vec<i8>,

	/// The region of every cell, or -1 where it has none.
	pub(crate) region: Vec<i64>,

	/// Whether a husk's body fits on each cell.
	pub(crate) standable: Vec<bool>,
}

impl RoomNav {
	/// Extra ticks a route pays for choosing a leap rather than staying
	/// on the same surface. It keeps a husk from hopping where walking is
	/// as quick.
	pub const JUMP_PENALTY_TICKS: i64 = 12;
	/// The smallest region worth keeping, in cells. Six cells is about a
	/// foot square at the sim grid's 10 centimetres.
	pub const MIN_REGION_CELLS: usize = 6;

	/// Derives the room for a body of `profile`, which keeps about
	/// `range` metres from the player and never closer than
	/// `near_metres`.
	pub fn new(grid: RoomGrid, profile: LocomotionProfile, range: f64, near_metres: f64) -> Self {
		let level = build_levels(&grid);
		let (region, region_cells) = build_regions(&grid, &level);
		let inset = build_inset(&grid, &region);
		let standable = build_standable(&grid, &profile, range, &region, &region_cells, &inset);
		let regions = assemble_regions(&grid, &region, &level, &region_cells, &standable);
		let edges = build_edges(&grid, &region, &standable);
		let links = build_links(&grid, &profile, &regions, &region, &standable, &edges);
		let spots = build_spots(
			&grid,
			&profile,
			&regions,
			&level,
			&standable,
			&edges,
			range,
			near_metres,
		);

		Self {
			grid,
			profile,
			range,
			near_metres,
			regions,
			links,
			spots,
			level,
			region,
			standable,
		}
	}

	/// Whether a husk's body fits on `cell`.
	pub fn is_standable(&self, cell: usize) -> bool {
		cell < self.standable.len() && self.standable[cell]
	}

	/// Whether a husk's body fits on `x`, `z`.
	pub fn is_standable_at(&self, x: i64, z: i64) -> bool {
		self.grid.contains(x, z) && self.standable[self.grid.index_of(x, z)]
	}

	/// The level index of `cell`, or -1 where it is not a walkable
	/// surface.
	pub fn level_of(&self, cell: usize) -> i8 {
		self.level.get(cell).copied().unwrap_or(-1)
	}

	/// The region `cell` belongs to, or nothing.
	pub fn region_of(&self, cell: usize) -> Option<&RoomRegion> {
		let id = *self.region.get(cell)?;

		self.regions.get(usize::try_from(id).ok()?)
	}

	/// The surface height of `cell` in the hunt's frame, or nothing where
	/// it is not walkable.
	pub fn support_height_of(&self, cell: usize) -> Option<f64> {
		if self.level_of(cell) < 0 {
			return None;
		}

		Some(
			self.grid
				.height_at(self.grid.x_of(cell), self.grid.z_of(cell)),
		)
	}

	/// The standable cell nearest `point`, or nothing when there is none
	/// within `max_radius` cells.
	///
	/// A husk standing near an edge is snapped to the middle of the
	/// nearest place its body fits, rather than being allowed to balance
	/// on the edge.
	pub fn standable_cell_near(&self, point: Vec3, max_radius: i64) -> Option<usize> {
		let start_x = self.grid.cell_x_at(point.x, point.z);
		let start_z = self.grid.cell_z_at(point.x, point.z);
		let mut best_distance = f64::INFINITY;
		let mut best: Option<usize> = None;

		for radius in 0..=max_radius {
			for dz in -radius..=radius {
				for dx in -radius..=radius {
					if dx.abs().max(dz.abs()) != radius {
						continue;
					}

					let x = start_x + dx;
					let z = start_z + dz;

					if !self.is_standable_at(x, z) {
						continue;
					}

					let centre = self.grid.centre_of(x, z);
					let away_x = centre.x - point.x;
					let away_z = centre.z - point.z;
					let distance = away_x * away_x + away_z * away_z;
					let cell = self.grid.index_of(x, z);

					// Both distances are squares of whole offsets, so the
					// tie is exact, and the lower cell index wins it.
					#[allow(
						clippy::float_cmp,
						reason = "squares of whole cell offsets compare exactly"
					)]
					if distance < best_distance
						|| (distance == best_distance && best.is_some_and(|found| cell < found))
					{
						best_distance = distance;
						best = Some(cell);
					}
				}
			}

			if best.is_some() {
				return best;
			}
		}

		None
	}

	/// Every spot of `kind`, in cell order.
	pub fn spots_of(&self, kind: RoomSpotKind) -> Vec<&RoomSpot> {
		self.spots.iter().filter(|spot| spot.kind == kind).collect()
	}
}

/// The level index of every cell, or -1 where it is not walkable.
fn build_levels(grid: &RoomGrid) -> Vec<i8> {
	let mut level = vec![-1_i8; grid.cell_count()];

	for index in 0..grid.cell_count() {
		if let Some(cell_level) = RoomCell::from_byte(grid.cells[index]).and_then(RoomCell::level) {
			level[index] = cell_level as i8;
		}
	}

	level
}

/// Flood-fills the walkable cells into regions, 8-connected within one
/// level, and drops the ones too small to matter.
///
/// Returns the region of every cell, -1 where it has none, and the cells
/// of every kept region in index order.
fn build_regions(grid: &RoomGrid, level: &[i8]) -> (Vec<i64>, Vec<Vec<usize>>) {
	let mut region = vec![-1_i64; grid.cell_count()];
	let mut raw: Vec<Vec<usize>> = Vec::new();

	for index in 0..grid.cell_count() {
		if level[index] < 0 || region[index] >= 0 {
			continue;
		}

		let mut cells = vec![index];
		region[index] = -2;

		let mut head = 0;

		while head < cells.len() {
			let cell = cells[head];
			head += 1;
			let (x, z) = (grid.x_of(cell), grid.z_of(cell));

			for (dx, dz) in NEIGHBOURS {
				let nx = x + dx;
				let nz = z + dz;

				if !grid.contains(nx, nz) {
					continue;
				}

				let next = grid.index_of(nx, nz);

				if level[next] != level[index] || region[next] != -1 {
					continue;
				}

				region[next] = -2;
				cells.push(next);
			}
		}

		raw.push(cells);
	}

	let mut kept: Vec<Vec<usize>> = Vec::new();

	for mut cells in raw {
		if cells.len() < RoomNav::MIN_REGION_CELLS {
			continue;
		}

		cells.sort_unstable();
		let id = kept.len() as i64;
		kept.push(cells.clone());

		for cell in cells {
			region[cell] = id;
		}
	}

	(region, kept)
}

/// How deep inside its region each cell sits: edges are 0, and each step
/// inward from an edge is one more.
fn build_inset(grid: &RoomGrid, region: &[i64]) -> Vec<i64> {
	let mut inset = vec![-1_i64; region.len()];
	let mut queue: Vec<usize> = Vec::new();

	for index in 0..grid.cell_count() {
		if region[index] < 0 {
			continue;
		}

		let (x, z) = (grid.x_of(index), grid.z_of(index));
		let mut edge = false;

		for (dx, dz) in NEIGHBOURS {
			let nx = x + dx;
			let nz = z + dz;

			if !grid.contains(nx, nz) || region[grid.index_of(nx, nz)] != region[index] {
				edge = true;
				break;
			}
		}

		if edge {
			inset[index] = 0;
			queue.push(index);
		}
	}

	let mut head = 0;

	while head < queue.len() {
		let cell = queue[head];
		head += 1;
		let (x, z) = (grid.x_of(cell), grid.z_of(cell));

		for (dx, dz) in NEIGHBOURS {
			let nx = x + dx;
			let nz = z + dz;

			if !grid.contains(nx, nz) {
				continue;
			}

			let next = grid.index_of(nx, nz);

			if region[next] == region[cell] && inset[next] < 0 {
				inset[next] = inset[cell] + 1;
				queue.push(next);
			}
		}
	}

	inset
}

/// Whether every cell within the foot radius of `cell` belongs to the
/// same surface. This is what keeps a foot from overhanging an edge.
///
/// The distance is measured to the near side of a neighbouring cell, not
/// to its middle: a cell boundary is half a cell beyond the middle, and a
/// foot that only just fits inside the cell centres still hangs over the
/// edge.
fn foot_clear(
	grid: &RoomGrid,
	profile: &LocomotionProfile,
	region: &[i64],
	cell: usize,
	foot_cells: i64,
) -> bool {
	let (x, z) = (grid.x_of(cell), grid.z_of(cell));

	for dz in -foot_cells..=foot_cells {
		for dx in -foot_cells..=foot_cells {
			if dx == 0 && dz == 0 {
				continue;
			}

			let nx = x + dx;
			let nz = z + dz;

			if !grid.contains(nx, nz) || region[grid.index_of(nx, nz)] == region[cell] {
				continue;
			}

			let near_x = math::max(0.0, dx.abs() as f64 - 0.5) * RoomGrid::CELL_METRES;
			let near_z = math::max(0.0, dz.abs() as f64 - 0.5) * RoomGrid::CELL_METRES;
			let distance = math::sqrt(near_x * near_x + near_z * near_z);

			if distance < profile.foot_radius {
				return false;
			}
		}
	}

	true
}

/// Which cells a husk's body fits on: the region eroded by the safety
/// margin, with the hard foot-radius clearance always kept.
///
/// A surface narrower than the safety margin keeps its middle cells
/// rather than losing the whole surface: a husk that fits on a chair can
/// stand in the middle of it, however far away the player is. The margin
/// grows with the range, because a depth reading is less certain farther
/// away, but it never grows past what the region itself allows.
fn build_standable(
	grid: &RoomGrid,
	profile: &LocomotionProfile,
	range: f64,
	region: &[i64],
	region_cells: &[Vec<usize>],
	inset: &[i64],
) -> Vec<bool> {
	let requested =
		math::ceil_int((profile.foot_radius + 0.05 * range) / RoomGrid::CELL_METRES).max(1);
	let foot_cells = math::ceil_int(profile.foot_radius / RoomGrid::CELL_METRES) + 1;
	let mut deepest = vec![0_i64; region_cells.len()];

	for (index, &cell_region) in region.iter().enumerate() {
		if cell_region >= 0 {
			deepest[cell_region as usize] = deepest[cell_region as usize].max(inset[index]);
		}
	}

	let mut standable = vec![false; grid.cell_count()];

	for index in 0..grid.cell_count() {
		let cell_region = region[index];

		if cell_region < 0 {
			continue;
		}

		let erosion = requested.min(deepest[cell_region as usize]);

		if inset[index] < erosion {
			continue;
		}

		standable[index] = foot_clear(grid, profile, region, index, foot_cells);
	}

	standable
}

/// Puts the kept regions together with their standable cells.
fn assemble_regions(
	grid: &RoomGrid,
	region: &[i64],
	level: &[i8],
	region_cells: &[Vec<usize>],
	standable: &[bool],
) -> Vec<RoomRegion> {
	let mut standable_by_region: Vec<Vec<usize>> = vec![Vec::new(); region_cells.len()];

	for (index, &fits) in standable.iter().enumerate() {
		if fits {
			standable_by_region[region[index] as usize].push(index);
		}
	}

	region_cells
		.iter()
		.enumerate()
		.map(|(id, cells)| {
			let level_index = level[cells[0]];

			RoomRegion {
				id,
				level_index: level_index as u8,
				height_centimetres: i64::from(grid.levels_centimetres[level_index as usize]),
				cells: cells.clone(),
				standable_cells: standable_by_region[id].clone(),
			}
		})
		.collect()
}

/// Every boundary cell of a standable region, with an outward normal
/// taken from its neighbours, ready to be sampled for links.
fn build_edges(grid: &RoomGrid, region: &[i64], standable: &[bool]) -> Vec<Edge> {
	let mut edges: Vec<Edge> = Vec::new();

	for index in 0..grid.cell_count() {
		if !standable[index] {
			continue;
		}

		let (x, z) = (grid.x_of(index), grid.z_of(index));
		let mut normal_x = 0.0;
		let mut normal_z = 0.0;

		for (dx, dz) in NEIGHBOURS {
			let nx = x + dx;
			let nz = z + dz;

			if !grid.contains(nx, nz)
				|| !standable[grid.index_of(nx, nz)]
				|| region[grid.index_of(nx, nz)] != region[index]
			{
				normal_x += dx as f64;
				normal_z += dz as f64;
			}
		}

		// The sums are whole numbers held in floats, so zero is
		// exact.
		#[allow(clippy::float_cmp, reason = "sums of ±1 as floats")]
		if normal_x == 0.0 && normal_z == 0.0 {
			continue;
		}

		let length = math::sqrt(normal_x * normal_x + normal_z * normal_z);
		normal_x /= length;
		normal_z /= length;

		let bucket_x = if normal_x.abs() < 0.25 {
			0
		} else {
			normal_x.signum() as i64
		};
		let bucket_z = if normal_z.abs() < 0.25 {
			0
		} else {
			normal_z.signum() as i64
		};
		let (tangent_x, tangent_z) = (-normal_z, normal_x);

		edges.push(Edge {
			cell: index,
			region: region[index] as usize,
			normal_x,
			normal_z,
			bucket: (bucket_x + 1) * 3 + (bucket_z + 1),
			along: math::round(x as f64 * tangent_x + z as f64 * tangent_z),
		});
	}

	edges.sort_unstable_by(|one, other| {
		one.bucket
			.cmp(&other.bucket)
			.then(one.along.cmp(&other.along))
			.then(one.cell.cmp(&other.cell))
	});

	edges
}

/// Whether the leap `one` would do better than `other` as a segment's
/// representative: nearer, then quicker, then by cell index.
fn is_better_link(one: &JumpLink, other: &JumpLink) -> bool {
	one.gap_metres
		.total_cmp(&other.gap_metres)
		.then(one.ticks().cmp(&other.ticks()))
		.then(one.from_cell.cmp(&other.from_cell))
		.then(one.to_cell.cmp(&other.to_cell))
		== core::cmp::Ordering::Less
}

/// Casts from one edge cell along `direction` for the body's leap reach,
/// and keeps the first standable cell of another region a leap may land
/// on.
///
/// The best cast per 30 centimetre segment per target region is kept, so
/// a long edge offers a husk one good leap per segment, not one per cell.
#[allow(
	clippy::too_many_arguments,
	reason = "the cast reads the grid, the regions and the standable mask"
)]
fn cast_link(
	grid: &RoomGrid,
	profile: &LocomotionProfile,
	regions: &[RoomRegion],
	region: &[i64],
	standable: &[bool],
	edge: &Edge,
	segment: usize,
	direction_x: f64,
	direction_z: f64,
	kept: &mut std::collections::BTreeMap<(usize, usize), JumpLink>,
) {
	let (from_x, from_z) = (grid.x_of(edge.cell), grid.z_of(edge.cell));
	let from_region = &regions[edge.region];
	let from_point = grid.centre_of(from_x, from_z) + Vec3::new(0.0, profile.half_height(), 0.0);
	let steps = math::ceil_int(profile.leaps_across / RoomGrid::CELL_METRES).max(1);

	for step in 1..=steps {
		let target_x = from_x + math::round(direction_x * step as f64);
		let target_z = from_z + math::round(direction_z * step as f64);

		if !grid.contains(target_x, target_z) {
			break;
		}

		let target = grid.index_of(target_x, target_z);
		let target_region = region[target];

		if target_region == region[edge.cell] {
			continue;
		}

		if target_region < 0 || !standable[target] {
			continue;
		}

		let target_region = &regions[target_region as usize];
		let drop = target_region.height_metres() - from_region.height_metres();

		if drop < -profile.drops_down || drop > profile.jumps_up {
			continue;
		}

		let target_point =
			grid.centre_of(target_x, target_z) + Vec3::new(0.0, profile.half_height(), 0.0);
		let Some(leap) = Leap::solve(from_point, target_point, grid, profile.half_height(), None)
		else {
			continue;
		};

		let away_x = target_point.x - from_point.x;
		let away_z = target_point.z - from_point.z;
		let gap = math::sqrt(away_x * away_x + away_z * away_z);

		if gap > profile.leaps_across + 1e-9 {
			continue;
		}

		let link = JumpLink {
			from_cell: edge.cell,
			to_cell: target,
			from_region: edge.region,
			to_region: target_region.id,
			leap,
			gap_metres: gap,
		};

		match kept.get(&(segment, target_region.id)) {
			Some(best) if !is_better_link(&link, best) => {}
			_ => {
				kept.insert((segment, target_region.id), link);
			}
		}

		break;
	}
}

/// Finds every jump link: each sampled edge cell casts along its normal
/// and 25 degrees either side, and a small surface with a diagonal normal
/// casts along the two sides as well, so a chair finds the desk beside it.
fn build_links(
	grid: &RoomGrid,
	profile: &LocomotionProfile,
	regions: &[RoomRegion],
	region: &[i64],
	standable: &[bool],
	edges: &[Edge],
) -> Vec<JumpLink> {
	let mut kept: std::collections::BTreeMap<(usize, usize), JumpLink> =
		std::collections::BTreeMap::new();
	let cosine = math::cos(crate::vec3::radians(25.0));
	let sine = math::sin(crate::vec3::radians(25.0));
	let mut run_start = 0;
	let mut previous: Option<&Edge> = None;

	for (index, edge) in edges.iter().enumerate() {
		let starts_run = previous.is_none_or(|last| {
			last.region != edge.region
				|| last.bucket != edge.bucket
				|| (edge.along - last.along).abs() > 1
		});

		if starts_run {
			run_start = index;
		}

		previous = Some(edge);

		if (index - run_start) % 3 != 0 {
			continue;
		}

		let segment = run_start + (index - run_start) / 3;

		for turn in -1..=1_i64 {
			// The normal, turned 25 degrees to the left, to the right,
			// or not at all.
			let (direction_x, direction_z) = match turn {
				-1 => {
					(
						edge.normal_x * cosine + edge.normal_z * sine,
						-edge.normal_x * sine + edge.normal_z * cosine,
					)
				}
				1 => {
					(
						edge.normal_x * cosine - edge.normal_z * sine,
						edge.normal_x * sine + edge.normal_z * cosine,
					)
				}
				_ => (edge.normal_x, edge.normal_z),
			};

			cast_link(
				grid,
				profile,
				regions,
				region,
				standable,
				edge,
				segment,
				direction_x,
				direction_z,
				&mut kept,
			);
		}

		// A small surface, such as a chair seat, has corners rather than
		// long straight edges, so its normal points diagonally. Cast
		// along the two sides as well, or a penguin would never find the
		// desk beside it.
		if edge.normal_x.abs() > 0.25 && edge.normal_z.abs() > 0.25 {
			cast_link(
				grid,
				profile,
				regions,
				region,
				standable,
				edge,
				segment,
				edge.normal_x.signum(),
				0.0,
				&mut kept,
			);
			cast_link(
				grid,
				profile,
				regions,
				region,
				standable,
				edge,
				segment,
				0.0,
				edge.normal_z.signum(),
				&mut kept,
			);
		}
	}

	let mut links: Vec<JumpLink> = kept.into_values().collect();

	links.sort_unstable_by(|one, other| {
		one.from_cell
			.cmp(&other.from_cell)
			.then(one.to_cell.cmp(&other.to_cell))
			.then(one.from_region.cmp(&other.from_region))
			.then(one.to_region.cmp(&other.to_region))
			.then(one.gap_metres.total_cmp(&other.gap_metres))
			.then(one.ticks().cmp(&other.ticks()))
	});

	links
}

/// Whether something stands within `radius` cells of `x`, `z`.
fn obstacle_within(grid: &RoomGrid, x: i64, z: i64, radius: i64) -> bool {
	for dz in -radius..=radius {
		for dx in -radius..=radius {
			let nx = x + dx;
			let nz = z + dz;

			if grid.contains(nx, nz) && grid.cell_at(nx, nz).kind == RoomCellKind::Obstacle {
				return true;
			}
		}
	}

	false
}

/// The first exposed cell within `radius` cells of `x`, `z` on the same
/// level, or nothing.
fn exposed_within(grid: &RoomGrid, level: &[i8], x: i64, z: i64, radius: i64) -> Option<usize> {
	let here = grid.index_of(x, z);

	for ring in 1..=radius {
		for dz in -ring..=ring {
			for dx in -ring..=ring {
				if dx.abs().max(dz.abs()) != ring {
					continue;
				}

				let nx = x + dx;
				let nz = z + dz;

				if !grid.contains(nx, nz) {
					continue;
				}

				let cell = grid.index_of(nx, nz);

				if level[cell] == level[here] && grid.cell_at(nx, nz).kind == RoomCellKind::Walkable
				{
					return Some(cell);
				}
			}
		}
	}

	None
}

/// The unit vector from one cell's middle towards another's.
fn facing_toward(grid: &RoomGrid, from_x: i64, from_z: i64, cell: usize) -> Option<Vec3> {
	let point = grid.centre_of(grid.x_of(cell), grid.z_of(cell));
	let here = grid.centre_of(from_x, from_z);
	let away = Vec3::new(point.x - here.x, 0.0, point.z - here.z);

	if away == Vec3::ZERO {
		None
	} else {
		Some(away.normalized())
	}
}

/// Whether a person standing at the eye can see the top of `point`.
fn seen_from_eye(grid: &RoomGrid, point: Vec3) -> bool {
	let away = math::sqrt(point.x * point.x + point.z * point.z);
	let steps = math::ceil_int(away / RoomGrid::CELL_METRES).max(1);

	for step in 0..=steps {
		let along = step as f64 / steps as f64;
		let x = point.x * along;
		let z = point.z * along;
		let line = point.y * along;
		let top = (grid.top_centimetres_at_world(x, z) - i64::from(grid.eye_height_centimetres))
			as f64
			/ 100.0;

		if top > line + 1e-9 {
			return false;
		}
	}

	true
}

/// The hidden cells in index order, and how deep into the hidden floor
/// each cell sits: a cell next to seen floor is one deep, and each
/// further step is one more.
fn hidden_depths(grid: &RoomGrid, level: &[i8]) -> (Vec<usize>, Vec<i64>) {
	let mut hidden: Vec<usize> = Vec::new();
	let mut depth = vec![0_i64; grid.cell_count()];
	let mut queue: Vec<usize> = Vec::new();

	for (index, &cell_level) in level.iter().enumerate() {
		if cell_level >= 0
			&& grid.cell_at(grid.x_of(index), grid.z_of(index)).kind == RoomCellKind::Hidden
		{
			hidden.push(index);
		}
	}

	for &cell in &hidden {
		let (x, z) = (grid.x_of(cell), grid.z_of(cell));

		for (dx, dz) in ORTHOGONAL_NEIGHBOURS {
			let nx = x + dx;
			let nz = z + dz;

			if !grid.contains(nx, nz) {
				continue;
			}

			let neighbour = grid.index_of(nx, nz);

			if level[neighbour] == level[cell]
				&& grid.cell_at(nx, nz).kind == RoomCellKind::Walkable
			{
				depth[cell] = 1;
				queue.push(cell);
				break;
			}
		}
	}

	let mut head = 0;

	while head < queue.len() {
		let cell = queue[head];
		head += 1;
		let (x, z) = (grid.x_of(cell), grid.z_of(cell));

		for (dx, dz) in NEIGHBOURS {
			let nx = x + dx;
			let nz = z + dz;

			if !grid.contains(nx, nz) {
				continue;
			}

			let neighbour = grid.index_of(nx, nz);

			if depth[neighbour] == 0
				&& level[neighbour] == level[cell]
				&& grid.cell_at(nx, nz).kind == RoomCellKind::Hidden
			{
				depth[neighbour] = depth[cell] + 1;
				queue.push(neighbour);
			}
		}
	}

	(hidden, depth)
}

/// Finds every place a husk might choose to be, sorted by kind and cell.
#[allow(
	clippy::too_many_arguments,
	reason = "the spots read the grid, the regions and the standable mask"
)]
fn build_spots(
	grid: &RoomGrid,
	profile: &LocomotionProfile,
	regions: &[RoomRegion],
	level: &[i8],
	standable: &[bool],
	edges: &[Edge],
	range: f64,
	near_metres: f64,
) -> Vec<RoomSpot> {
	let mut found: Vec<RoomSpot> = Vec::new();

	// A perch: an edge of a raised surface, turned towards the player.
	let facing_cosine = math::sqrt(0.5);

	for edge in edges {
		let raised = &regions[edge.region];

		if raised.height_centimetres == 0 {
			continue;
		}

		let (x, z) = (grid.x_of(edge.cell), grid.z_of(edge.cell));
		let toward = grid.toward_player(x, z);

		if edge.normal_x * toward.x + edge.normal_z * toward.z < facing_cosine {
			continue;
		}

		found.push(RoomSpot {
			kind: RoomSpotKind::Perch,
			cell: edge.cell,
			level_index: raised.level_index,
			point: grid.centre_of(x, z),
			facing: Some(grid.world_direction(toward)),
		});
	}

	// Hidden floor, and how deep into it each cell sits.
	let (hidden, hidden_depth) = hidden_depths(grid, level);

	let foot_depth = math::ceil_int(2.0 * profile.foot_radius / RoomGrid::CELL_METRES).max(1);
	let near_cover = math::ceil_int(profile.foot_radius / RoomGrid::CELL_METRES) + 2;

	// Peek and hide.
	for &cell in &hidden {
		if !standable[cell] {
			continue;
		}

		let (x, z) = (grid.x_of(cell), grid.z_of(cell));
		let point = grid.centre_of(x, z);
		let peek = exposed_within(grid, level, x, z, 2);

		if obstacle_within(grid, x, z, near_cover)
			&& let Some(exposed) = peek
		{
			found.push(RoomSpot {
				kind: RoomSpotKind::Peek,
				cell,
				level_index: level[cell] as u8,
				point,
				facing: facing_toward(grid, x, z, exposed),
			});
		}

		if hidden_depth[cell] >= foot_depth {
			found.push(RoomSpot {
				kind: RoomSpotKind::Hide,
				cell,
				level_index: level[cell] as u8,
				point,
				facing: None,
			});
		}
	}

	// Open floor: seen from the eye, inside the distances the husk keeps.
	for index in 0..grid.cell_count() {
		if !standable[index] || level[index] != 0 {
			continue;
		}

		let (x, z) = (grid.x_of(index), grid.z_of(index));

		if grid.cell_at(x, z).kind != RoomCellKind::Walkable {
			continue;
		}

		let point = grid.centre_of(x, z);
		let distance = math::sqrt(point.x * point.x + point.z * point.z);

		if distance < near_metres || distance > range || !seen_from_eye(grid, point) {
			continue;
		}

		found.push(RoomSpot {
			kind: RoomSpotKind::Open,
			cell: index,
			level_index: 0,
			point,
			facing: None,
		});
	}

	found.sort_unstable_by(|one, other| one.kind.cmp(&other.kind).then(one.cell.cmp(&other.cell)));

	found
}
