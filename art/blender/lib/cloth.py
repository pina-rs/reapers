"""Robes, sleeves and hanging cloth.

Cloth here is drawn, not simulated: a tube whose folds deepen toward the hem,
cut ragged at the bottom. That keeps every robe deterministic, and lets the
same code make a narrow coat, a hunched cowl and a floor-length veil.
"""

import math
import random

import bmesh
from mathutils import Vector, noise

from .core import add_shape_key, object_from_bmesh


def _fold(angle, depth, folds, phases):
    """The ripple that turns a smooth tube into hanging cloth."""

    total = 0.0

    for count, weight, phase in zip(folds, (0.5, 0.32, 0.18), phases):
        total += weight * math.sin(angle * count + phase + depth * 1.4)

    return total


def robe(
    name,
    mat,
    parent=None,
    top=1.62,
    hem=0.14,
    profile=((0.0, 0.25, 0.14), (0.3, 0.2, 0.15), (1.0, 0.36, 0.3)),
    folds=(5, 9, 14),
    fold_depth=0.16,
    tatter=0.32,
    tears=9,
    seed=1,
    rings=54,
    segments=128,
    lean=0.0,
    slits=10,
    hunch=0.0,
):
    """A robe hanging from `top` to `hem`.

    `profile` lists (depth, half-width, half-depth) stops from top (0) to
    hem (1). `tatter` is how far up the tears reach, as a share of the robe's
    length, `tears` is how many deep tears are cut into the hem, and `slits`
    is how many narrow cuts split it into strips. `lean` pushes the hem back
    along +Y, for a figure drifting forward, and `hunch` carries the
    shoulders forward of the waist.
    """

    generator = random.Random(seed)
    phases = [generator.uniform(0, math.tau) for _ in range(3)]
    length = top - hem

    def radii(depth):
        for (d0, w0, t0), (d1, w1, t1) in zip(profile, profile[1:]):
            if depth <= d1:
                blend = (depth - d0) / max(d1 - d0, 1e-6)
                blend = blend * blend * (3 - 2 * blend)

                return w0 + (w1 - w0) * blend, t0 + (t1 - t0) * blend

        return profile[-1][1], profile[-1][2]

    # How far up each column of the robe has been torn away. A tear is a
    # sharp notch; between tears the hem is nibbled into small points.
    tear_angles = sorted(generator.uniform(0, math.tau) for _ in range(tears))
    tear_depths = [generator.uniform(0.4, 1.0) * tatter for _ in range(tears)]
    tear_widths = [generator.uniform(0.09, 0.26) for _ in range(tears)]
    nibble = generator.uniform(0, math.tau)

    def torn(angle):
        points = abs(math.sin(angle * 11 + nibble)) * abs(math.sin(angle * 4.3 + nibble * 2))
        cut = tatter * 0.16 * points

        for centre, depth, width in zip(tear_angles, tear_depths, tear_widths):
            distance = abs((angle - centre + math.pi) % math.tau - math.pi)

            if distance < width:
                cut = max(cut, depth * (1 - distance / width) ** 0.8)

        return cut

    # A slit is one column left out, running up from the hem, which is what
    # separates a ragged hem into hanging strips.
    slit_columns = {}

    for _ in range(slits):
        column = generator.randrange(segments)
        slit_columns[column] = generator.uniform(0.3, 1.0) * tatter * 1.25

    mesh = bmesh.new()
    grid = []

    for ring in range(rings + 1):
        # Rings bunch toward the hem, where the shape changes fastest.
        depth = (ring / rings) ** 0.8
        row = []

        for segment in range(segments):
            angle = math.tau * segment / segments
            reach = depth * (1 - torn(angle))
            width, thickness = radii(reach)
            ripple = fold_depth * reach**1.35
            fold = 1 + ripple * _fold(angle, reach, folds, phases)
            flutter = 0.03 * reach * reach * noise.noise(Vector((angle * 2.0, reach * 3.0, seed)))
            curl = hunch * math.sin(min(reach * 2.2, 1.0) * math.pi * 0.5) if hunch else 0.0

            row.append(
                mesh.verts.new(
                    (
                        math.cos(angle) * width * fold,
                        math.sin(angle) * thickness * fold + lean * reach * reach + flutter + curl - hunch,
                        top - reach * length,
                    )
                )
            )

        grid.append(row)

    for ring in range(rings):
        depth = ((ring + 1) / rings) ** 0.8

        for segment in range(segments):
            follow = (segment + 1) % segments

            if depth > 1 - slit_columns.get(segment, 0.0):
                continue

            mesh.faces.new((grid[ring][segment], grid[ring + 1][segment], grid[ring + 1][follow], grid[ring][follow]))

    loose = [vertex for vertex in mesh.verts if not vertex.link_faces]
    bmesh.ops.delete(mesh, geom=loose, context="VERTS")

    result = object_from_bmesh(name, mesh, mat, parent)
    _add_motion_keys(result, top, length)

    return result


def _add_motion_keys(obj, top, length):
    """Shape keys the game blends to make still cloth look alive."""

    def depth_of(co):
        return max(0.0, min(1.0, (top - co.z) / length))

    def sway(axis):
        def move(co):
            swing = depth_of(co) ** 2

            co[axis] += 0.2 * swing
            co.z += 0.035 * swing

            return co

        return move

    def billow(co):
        depth = depth_of(co)
        angle = math.atan2(co.y, co.x)
        puff = depth**2 * (0.16 + 0.1 * math.sin(angle * 3 + 1.0))

        co.x *= 1 + puff
        co.y *= 1 + puff
        co.z += 0.03 * depth * math.sin(angle * 5)

        return co

    def gaunt(co):
        depth = depth_of(co)

        co.x *= 1 - 0.2 * (0.35 + 0.65 * depth)
        co.y *= 1 - 0.1 * depth

        return co

    def long_hem(co):
        co.z -= 0.16 * depth_of(co) ** 3

        return co

    add_shape_key(obj, "sway_x", sway(0))
    add_shape_key(obj, "sway_y", sway(1))
    add_shape_key(obj, "billow", billow)
    add_shape_key(obj, "gaunt", gaunt)
    add_shape_key(obj, "long_hem", long_hem)


def sleeve(
    name,
    mat,
    parent,
    path,
    shoulder=0.085,
    cuff=0.13,
    droop=0.22,
    tatter=0.4,
    seed=3,
    rings=26,
    segments=40,
):
    """A sleeve swept along `path`, widening to a long, ragged cuff.

    `droop` is how far the underside of the cuff hangs below the wrist, which
    is what gives a robed arm its long pointed shape.
    """

    generator = random.Random(seed)
    phases = [generator.uniform(0, math.tau) for _ in range(3)]
    nibble = generator.uniform(0, math.tau)
    points = [Vector(point) for point in path]
    mesh = bmesh.new()
    grid = []
    normal = None

    def sample(depth):
        position = min(depth, 1.0) * (len(points) - 1)
        index = min(int(position), len(points) - 2)

        return points[index].lerp(points[index + 1], position - index)

    def torn(angle):
        return tatter * 0.3 * abs(math.sin(angle * 4 + nibble)) * abs(math.sin(angle * 1.5 + nibble * 3))

    for ring in range(rings + 1):
        depth = ring / rings
        centre = sample(depth)
        tangent = (sample(depth + 0.02) - sample(max(depth - 0.02, 0))).normalized()

        if normal is None:
            normal = tangent.cross(Vector((0, 1, 0))).normalized()
        else:
            normal = (normal - tangent * normal.dot(tangent)).normalized()

        binormal = tangent.cross(normal)
        row = []

        for segment in range(segments):
            angle = math.tau * segment / segments
            reach = depth * (1 - torn(angle))
            radius = shoulder + (cuff - shoulder) * reach**1.7
            fold = 1 + 0.16 * reach * _fold(angle, reach, (3, 5, 8), phases)
            around = normal * math.cos(angle) + binormal * math.sin(angle)
            point = sample(reach) + around * radius * fold

            # The underside of the cuff hangs: the lower a point sits on its
            # ring, the further it falls.
            under = max(0.0, -around.z)
            point.z -= droop * reach**3 * under**1.5

            row.append(mesh.verts.new(point))

        grid.append(row)

    for ring in range(rings):
        for segment in range(segments):
            follow = (segment + 1) % segments

            mesh.faces.new((grid[ring][segment], grid[ring + 1][segment], grid[ring + 1][follow], grid[ring][follow]))

    return object_from_bmesh(name, mesh, mat, parent)


def drape(
    name,
    mat,
    parent,
    top_edge,
    drop,
    width_gain=1.15,
    sag=0.05,
    tatter=0.35,
    folds=7,
    fold_depth=0.035,
    seed=5,
    rows=22,
    outward=None,
):
    """A sheet hanging from `top_edge`, a list of points: capes and banners.

    The sheet falls `drop` metres, widening by `width_gain`, and its bottom
    edge is cut ragged. `outward` is the direction the sheet's folds push
    toward; by default it is away from the middle of the top edge.
    """

    generator = random.Random(seed)
    edge = [Vector(point) for point in top_edge]
    centre = sum(edge, Vector()) / len(edge)
    phase = generator.uniform(0, math.tau)
    nibble = generator.uniform(0, math.tau)
    push = Vector(outward).normalized() if outward is not None else None
    mesh = bmesh.new()
    grid = []

    def torn(across):
        points = abs(math.sin(across * 17 + nibble)) * abs(math.sin(across * 6.1 + nibble * 2))
        notch = max(0.0, noise.noise(Vector((across * 5.0, seed, 2.0)))) ** 0.6

        return tatter * (0.25 * points + 0.75 * notch)

    for row in range(rows + 1):
        depth = row / rows
        line = []

        for index, point in enumerate(edge):
            across = index / (len(edge) - 1)
            reach = depth * (1 - torn(across))
            spread = centre + (point - centre) * (1 + (width_gain - 1) * reach)
            wave = fold_depth * reach * math.sin(across * folds * math.pi + phase + reach * 2)
            direction = push

            if direction is None:
                direction = point - centre
                direction.z = 0
                direction = direction.normalized() if direction.length > 1e-6 else Vector((0, 1, 0))

            line.append(
                mesh.verts.new(
                    (
                        spread.x + direction.x * (wave + sag * math.sin(reach * math.pi)),
                        spread.y + direction.y * (wave + sag * math.sin(reach * math.pi)),
                        point.z - reach * drop,
                    )
                )
            )

        grid.append(line)

    for row in range(rows):
        for index in range(len(edge) - 1):
            mesh.faces.new((grid[row][index], grid[row + 1][index], grid[row + 1][index + 1], grid[row][index + 1]))

    return object_from_bmesh(name, mesh, mat, parent)
