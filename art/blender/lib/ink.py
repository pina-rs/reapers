"""Ink weight: how heavy the key line is at each vertex (art bible 4.3).

The game draws every mesh's key line as an inverted hull, pushed out along
smoothed normals by `weight x side x wobble x width`. The build bakes the
first factor into every model; the game supplies the rest:

    weight = part x contact x paint, clamped by thickness

- `part`, from the size of the connected piece of mesh the vertex is on:
  fine, interior or silhouette, by its largest dimension (the thresholds
  are per kind of model: a first-person hand is seen far closer than a
  reaper). Floored so a part's line is never under `minimum_pt`.
- `contact`, heavier within a few centimetres of the floor, so a figure
  stands on its line.
- `paint`, an artist's multiplier from 0 to 2, painted as a vertex group
  named `ink_weight` (`paint`): a vertex at weight w in the group is x2w,
  and one outside it x1.
- the thickness clamp: the push may never be more than a share of the
  local thickness, measured by a ray cast inward along -N, so fingers,
  chains and ears never balloon.

The light-side term and the wobble are the game's: they depend on the light
and the surface, which the game knows and the build does not.

The weight is stored on the mesh as a float point attribute, `_INK_W`
(`ATTRIBUTE`), which the review sheets draw with (`lib/cel.py`), so what a
sheet shows is what the game will draw. It reaches the game in the red of
the second colour set, `COLOR_1` (`CARRIER`), as weight / `COLOR_SCALE`,
written by `carry` and read back by `read`: the one place that decides it.
See art/blender/README.md, "Ink weight".

Every number is in `[weight]` of art/blender/cel.toml.
"""

import os
import tomllib

import bpy
from mathutils import Vector
from mathutils.bvhtree import BVHTree

# The weight's name on a Blender mesh: a FLOAT attribute on the POINT domain.
ATTRIBUTE = "_INK_W"

# The vertex group an artist paints the multiplier in.
PAINT_GROUP = "ink_weight"

# Where the weight travels in a glTF file: the red of the second colour set.
# The game's renderer reads its weight there, as 2 x red; gltfpack 0.25
# keeps `COLOR_1` with the flags in crates/xtask/src/art.rs (`-kv` is not
# needed), where it drops a second UV set and every custom attribute.
CARRIER = "COLOR_1.r"

# Every exported mesh has two colour sets, in this order: `COLOUR`
# (`COLOR_0`, the active one) is constant white, which gltfpack strips, and
# `INK_COLOUR` (`COLOR_1`) is (weight / `COLOR_SCALE`, char, wear, hollow),
# the outfit masks `core.paint_masks` painted, or 0 where a mesh has none.
COLOUR = "Color"
INK_COLOUR = "Ink"
COLOR_SCALE = 2.0

# Families that draw no line: their weight is 0 (lib/cel.py draws them, and
# cel.toml's `line = 0` says which).
CONFIG = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "cel.toml")


def settings(path=CONFIG):
    """`[weight]` of cel.toml, with every kind's numbers checked."""

    with open(path, "rb") as source:
        raw = tomllib.load(source)

    weight = raw["weight"]

    for kind, numbers in weight["kinds"].items():
        fine, interior = numbers["sizes"]

        if not 0 < fine < interior:
            raise ValueError(f"{path}: [weight.kinds.{kind}] sizes must rise: {numbers['sizes']}")

        if numbers["push_m"] <= 0 or numbers["line_pt"] <= 0:
            raise ValueError(f"{path}: [weight.kinds.{kind}] needs a positive push_m and line_pt")

    unlined = {name for name, family in raw["families"].items() if isinstance(family, dict) and family.get("line", 1.0) == 0}

    return {**weight, "unlined": unlined}


def kind_of(root):
    """What kind of model `root` heads, from its name: `reaper`, `husk`,
    `weapon` or `first_person`."""

    name = root.name.split(".")[0]

    if name == "reaper":
        return "reaper"

    if name == "husk":
        return "husk"

    if name == "weapons" or name.startswith("weapon_"):
        return "weapon"

    if name in ("viewmodel", "capture_orb", "vmfx"):
        return "first_person"

    raise ValueError(f"no ink kind for a model whose root is {root.name!r}; pass kind=")


def floor_of(root, kind):
    """Where the floor is under `root`, or None for a model that does not
    stand on one: a reaper stands on z = 0, a husk that walks (it casts a
    `shadow`) on its lowest point."""

    if kind == "reaper":
        return 0.0

    if kind != "husk":
        return None

    meshes = [node for node in [root, *root.children_recursive] if node.type == "MESH"]

    if not any(node.name.split(".")[0] == "shadow" for node in meshes):
        return None

    bpy.context.view_layer.update()

    return min(
        (node.matrix_world @ Vector(corner)).z for node in meshes if node.name.split(".")[0] != "shadow" and not node.name.startswith("seal_") for corner in node.bound_box
    )


def paint(obj, multiplier):
    """Paints the artist's multiplier on `obj`: `multiplier(co)` returns 0 to
    2 for a vertex at object position `co`, or None to leave it at 1."""

    group = obj.vertex_groups.get(PAINT_GROUP) or obj.vertex_groups.new(name=PAINT_GROUP)

    for vertex in obj.data.vertices:
        value = multiplier(vertex.co)

        if value is not None:
            group.add([vertex.index], max(0.0, min(2.0, value)) / 2, "REPLACE")

    return obj


def meshes(root):
    return [node for node in [root, *root.children_recursive] if node.type == "MESH" and node.data.vertices]


def weigh(root, kind=None, floor="auto", config=None):
    """Bakes `ATTRIBUTE` onto every mesh under `root`. Returns them.

    `kind` picks the size thresholds and the clamp's scale (`kind_of` when
    None); `floor` is the floor's height, None for none, or "auto" to find it
    (`floor_of`)."""

    config = config or settings()
    kind = kind or kind_of(root)
    numbers = config["kinds"][kind]
    floor = floor_of(root, kind) if floor == "auto" else floor
    bpy.context.view_layer.update()
    weighed = []

    for obj in meshes(root):
        _weigh(obj, numbers, floor, config)
        weighed.append(obj)

    return weighed


def _weigh(obj, numbers, floor, config):
    mesh = obj.data
    world = obj.matrix_world
    normal_world = world.to_3x3().inverted().transposed()
    positions = [world @ vertex.co for vertex in mesh.vertices]
    normals = [(normal_world @ vertex.normal).normalized() for vertex in mesh.vertices]
    count = len(positions)
    weights = [1.0] * count

    # Every vertex of a mesh drawn only in families with no line weighs 0.
    unlined = config["unlined"]
    families = [slot.material.name.split(".")[0] if slot.material else "" for slot in obj.material_slots]

    if families and all(name.startswith(tuple(unlined)) for name in families):
        _store(mesh, [0.0] * count)
        return

    # Part: the largest dimension of each connected piece, in metres.
    fine, interior = numbers["sizes"]
    part = config["part"]
    lowest = config["minimum_pt"] / numbers["line_pt"]

    for island in _islands(mesh):
        low = Vector(map(min, *(positions[index] for index in island))) if len(island) > 1 else positions[island[0]]
        high = Vector(map(max, *(positions[index] for index in island))) if len(island) > 1 else positions[island[0]]
        size = max(high - low)
        share = part["fine"] if size < fine else part["interior"] if size < interior else part["silhouette"]

        for index in island:
            weights[index] = max(share, lowest)

    # Contact: heavier near the floor.
    if floor is not None:
        reach = config["contact_reach"]

        for index, position in enumerate(positions):
            if position.z - floor < reach:
                weights[index] *= config["contact"]

    # Paint: the artist's multiplier.
    group = obj.vertex_groups.get(PAINT_GROUP)

    if group is not None:
        for vertex in mesh.vertices:
            for element in vertex.groups:
                if element.group == group.index:
                    weights[vertex.index] *= 2 * element.weight

    # The thickness clamp, from a ray cast inward along -N.
    tree = BVHTree.FromPolygons(positions, [polygon.vertices for polygon in mesh.polygons], epsilon=0.0)
    share = config["clamp"]
    push = numbers["push_m"] * config["headroom"]

    for index, (position, normal) in enumerate(zip(positions, normals)):
        if weights[index] <= 0 or normal.length == 0:
            continue

        thickness = _thickness(tree, position, normal)

        if thickness is not None:
            weights[index] = min(weights[index], share * thickness / push)

    _store(mesh, weights)


def _thickness(tree, position, normal):
    """How far the surface at `position` is from the far side of its part,
    along -`normal`: the first face the ray meets from inside, or None."""

    travelled = 1e-5
    origin = position - normal * travelled

    # A face met from inside faces along the ray's way out. A face that does
    # not (a neighbour in a crease, another part crossing in front) is
    # stepped through, a few times at most.
    for _ in range(3):
        hit, face_normal, _, distance = tree.ray_cast(origin, -normal)

        if hit is None:
            return None

        travelled += distance

        if face_normal.dot(-normal) > 0:
            return travelled

        origin = hit - normal * 1e-5
        travelled += 1e-5

    return None


def _islands(mesh):
    """The connected pieces of `mesh`, as lists of vertex indices."""

    parent = list(range(len(mesh.vertices)))

    def root(index):
        while parent[index] != index:
            parent[index] = parent[parent[index]]
            index = parent[index]

        return index

    for edge in mesh.edges:
        a, b = root(edge.vertices[0]), root(edge.vertices[1])

        if a != b:
            parent[a] = b

    groups = {}

    for index in range(len(parent)):
        groups.setdefault(root(index), []).append(index)

    return list(groups.values())


def _store(mesh, weights):
    attribute = mesh.attributes.get(ATTRIBUTE)

    if attribute is None or attribute.domain != "POINT" or attribute.data_type != "FLOAT":
        if attribute is not None:
            mesh.attributes.remove(attribute)

        attribute = mesh.attributes.new(ATTRIBUTE, "FLOAT", "POINT")

    attribute.data.foreach_set("value", weights)


def weights(obj):
    """`obj`'s weights by vertex, or None when it has none."""

    attribute = obj.data.attributes.get(ATTRIBUTE)

    if attribute is None:
        return None

    values = [0.0] * len(obj.data.vertices)
    attribute.data.foreach_get("value", values)

    return values


# The carrier ----------------------------------------------------------------


def carry(root, kind=None):
    """Weighs every mesh under `root` that has no weights yet, and writes the
    weights into `CARRIER` for export. Returns what `export_options` adds to
    the glTF exporter's options."""

    unweighed = [obj for obj in meshes(root) if obj.data.attributes.get(ATTRIBUTE) is None]

    if unweighed:
        weigh(root, kind)

    for obj in meshes(root):
        _carry_colour(obj.data, weights(obj))

    return export_options()


def export_options():
    """What the glTF exporter needs to write `CARRIER`: the active colour set
    as `COLOR_0` and the next as `COLOR_1`, on every mesh, material or none."""

    return {
        "export_vertex_color": "ACTIVE",
        "export_all_vertex_colors": True,
        "export_active_vertex_color_when_no_material": True,
    }


def _carry_colour(mesh, values):
    """Gives `mesh` exactly two colour sets on its points: `COLOUR`, white
    and active, then `INK_COLOUR`, the weight in red over the outfit masks
    `COLOUR` held (char, wear and hollow, in green, blue and alpha)."""

    count = len(mesh.vertices)
    unknown = [attribute.name for attribute in mesh.color_attributes if attribute.name not in (COLOUR, INK_COLOUR)]

    if unknown:
        raise ValueError(f"{mesh.name}: the ink weight needs the colour sets to be {COLOUR!r} and {INK_COLOUR!r} alone, but it also has {unknown}")

    masks = [0.0] * (count * 4)
    painted = mesh.color_attributes.get(COLOUR)

    if painted is not None:
        if painted.domain != "POINT":
            raise ValueError(f"{mesh.name}: the outfit masks must be on points to travel with the ink weight")

        painted.data.foreach_get("color", masks)

    for name in (INK_COLOUR, COLOUR):
        if name in mesh.color_attributes:
            mesh.color_attributes.remove(mesh.color_attributes[name])

    white = mesh.color_attributes.new(COLOUR, "FLOAT_COLOR", "POINT")
    white.data.foreach_set("color", [1.0] * (count * 4))
    ink = mesh.color_attributes.new(INK_COLOUR, "FLOAT_COLOR", "POINT")
    colours = [0.0] * (count * 4)

    for index, value in enumerate(values):
        colours[4 * index] = max(0.0, min(1.0, value / COLOR_SCALE))

        # Grime (red) gives way to the weight: no outfit reads it.
        if painted is not None:
            colours[4 * index + 1 : 4 * index + 4] = masks[4 * index + 1 : 4 * index + 4]

    ink.data.foreach_set("color", colours)
    mesh.color_attributes.active_color = white
    mesh.color_attributes.render_color_index = 0


def read(obj, kind="reaper"):
    """Puts `CARRIER`'s weights back into `ATTRIBUTE` on an imported mesh,
    or weighs it afresh when the model was built without them. Returns the
    weights.

    Blender's importer names `COLOR_1` `Color.001` and stores it on the
    corners in 8 bits, which is close enough for a review sheet; it cannot
    read a gltfpack-packed file at all."""

    mesh = obj.data

    if mesh.attributes.get(ATTRIBUTE) is not None:
        return weights(obj)

    values = None
    sets = mesh.color_attributes
    carried = sets.get(INK_COLOUR) or (sets[1] if len(sets) >= 2 else None)

    if carried is not None:
        colours = [0.0] * (len(carried.data) * 4)
        carried.data.foreach_get("color", colours)

        if carried.domain == "POINT":
            values = [colours[4 * index] * COLOR_SCALE for index in range(len(mesh.vertices))]
        elif carried.domain == "CORNER":
            values = [0.0] * len(mesh.vertices)

            for index, loop in enumerate(mesh.loops):
                values[loop.vertex_index] = colours[4 * index] * COLOR_SCALE

    if values is None:
        _weigh(obj, settings()["kinds"][kind], None, settings())

        return weights(obj)

    _store(mesh, values)

    return values
