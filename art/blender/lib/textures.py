"""Deterministic procedural PBR textures (normal, ORM, subtle albedo grain).

Every texture is generated from pure math in numpy inside Blender, tiled
seamlessly at 128x128, and packed as PNG into the exported GLB. Because
reapers and husks receive their final colours via `leptoskit_3d` instance
tints, surface textures give each material real tactile relief (woven cloth,
hammered iron, chased gold, wood grain, porous bone, coarse fur, curly wool,
brushed scrap metal) without altering colourways or requiring external assets.
"""

import math

import bpy
import numpy as np

SIZE = 128


def _periodic_noise(size, freq, seed):
    """Smooth tileable 2D value noise with `freq` cells across `[0, 1)`."""

    rng = np.random.default_rng(seed)
    grid = rng.random((freq, freq), dtype=np.float32)

    coords = np.linspace(0.0, freq, size, endpoint=False, dtype=np.float32)
    i0 = np.floor(coords).astype(np.int32) % freq
    i1 = (i0 + 1) % freq
    frac = coords - np.floor(coords)
    # Quintic fade for C2 continuity (no grid ridges in normal maps).
    fade = frac * frac * frac * (frac * (frac * 6.0 - 15.0) + 10.0)

    v00 = grid[np.ix_(i0, i0)]
    v10 = grid[np.ix_(i0, i1)]
    v01 = grid[np.ix_(i1, i0)]
    v11 = grid[np.ix_(i1, i1)]

    fx = fade[None, :]
    fy = fade[:, None]
    top = v00 * (1.0 - fx) + v10 * fx
    bot = v01 * (1.0 - fx) + v11 * fx
    return top * (1.0 - fy) + bot * fy


def _fbm(size, octaves, seed):
    """Multi-octave seamless periodic fBm normalized to `[0, 1]`."""

    total = np.zeros((size, size), dtype=np.float32)
    weight_sum = 0.0
    for idx, (freq, weight) in enumerate(octaves):
        total += weight * _periodic_noise(size, freq, seed + idx * 97)
        weight_sum += weight
    return total / max(weight_sum, 1e-6)


def _uv_grids(size):
    t = np.linspace(0.0, 1.0, size, endpoint=False, dtype=np.float32)
    u, v = np.meshgrid(t, t)
    return u, v


def _height_to_normal(height, strength):
    """Converts a periodic `[0, 1]` heightfield into an OpenGL tangent-space normal map."""

    # Periodic central differences (Blender pixel row 0 is v=0 at the bottom).
    du = (np.roll(height, -1, axis=1) - np.roll(height, 1, axis=1)) * 0.5
    dv = (np.roll(height, -1, axis=0) - np.roll(height, 1, axis=0)) * 0.5

    nx = -du * strength
    ny = -dv * strength
    nz = np.ones_like(height)
    inv_len = 1.0 / np.sqrt(nx * nx + ny * ny + nz * nz)

    rgb = np.empty((height.shape[0], height.shape[1], 3), dtype=np.float32)
    rgb[:, :, 0] = nx * inv_len * 0.5 + 0.5
    rgb[:, :, 1] = ny * inv_len * 0.5 + 0.5
    rgb[:, :, 2] = nz * inv_len * 0.5 + 0.5
    return np.clip(rgb, 0.0, 1.0)


def _heightfield(kind, size=SIZE):
    """Returns `(height, ao, rough_mod, albedo_mod)` arrays in `[0, 1]`."""

    u, v = _uv_grids(size)
    tau = 2.0 * math.pi

    if kind == "cloth":
        # Heavy medieval diagonal twill wool weave + soft slub.
        warp = 0.5 + 0.5 * np.sin(tau * (u * 24.0 + v * 24.0))
        weft = 0.5 + 0.5 * np.sin(tau * (u * 24.0 - v * 24.0))
        twill = 0.5 + 0.5 * np.sin(tau * (u * 16.0 + v * 8.0))
        slub = _fbm(size, ((4, 0.5), (8, 0.35), (16, 0.15)), 101)
        weave = 0.42 * warp * weft + 0.33 * twill + 0.25 * slub
        ao = 0.80 + 0.20 * weave
        rough = 0.86 + 0.14 * (1.0 - weave)
        albedo = 0.86 + 0.14 * weave
        return weave, ao, rough, albedo

    if kind == "cloth_alt":
        # Finer woven linen/brocade drape.
        warp = 0.5 + 0.5 * np.sin(tau * u * 32.0)
        weft = 0.5 + 0.5 * np.sin(tau * v * 32.0)
        linen = np.maximum(warp, weft) * 0.65 + (warp * weft) * 0.35
        slub = _fbm(size, ((4, 0.4), (12, 0.4), (24, 0.2)), 137)
        h = 0.72 * linen + 0.28 * slub
        ao = 0.84 + 0.16 * h
        rough = 0.84 + 0.16 * (1.0 - h)
        albedo = 0.88 + 0.12 * h
        return h, ao, rough, albedo

    if kind == "metal":
        # Hand-forged blackened iron: hammered peen dimples + fine brushed marks.
        dimples = _fbm(size, ((4, 0.45), (8, 0.35), (16, 0.2)), 211)
        dimples = np.power(np.clip(dimples, 0.0, 1.0), 1.3)
        brush_v = _periodic_noise(size, 32, 223)
        brush = 0.5 * brush_v + 0.5 * np.roll(brush_v, 1, axis=1)
        micro = _periodic_noise(size, 24, 229)
        h = 0.60 * dimples + 0.22 * brush + 0.18 * micro
        ao = 0.78 + 0.22 * dimples
        rough = 0.64 + 0.36 * (1.0 - h)
        albedo = 0.84 + 0.16 * h
        return h, ao, rough, albedo

    if kind == "gold":
        # Chased antique ceremonial gold: warm hammered sheen + subtle patina.
        hammer = _fbm(size, ((4, 0.55), (8, 0.3), (16, 0.15)), 307)
        fine = _periodic_noise(size, 20, 311)
        h = 0.75 * hammer + 0.25 * fine
        ao = 0.84 + 0.16 * hammer
        rough = 0.60 + 0.40 * (1.0 - hammer)
        albedo = 0.90 + 0.10 * h
        return h, ao, rough, albedo

    if kind == "wood":
        # Aged ash/oak vertical wood grain with wavy growth fibers.
        warp = _fbm(size, ((2, 0.6), (4, 0.4)), 401)
        rings = 0.5 + 0.5 * np.sin(tau * (u * 10.0 + warp * 1.4))
        fibers = _periodic_noise(size, 32, 409)
        # Stretch fibers vertically by averaging along v.
        for shift in (1, 2, 3):
            fibers = 0.7 * fibers + 0.3 * np.roll(fibers, shift, axis=0)
        h = 0.55 * rings + 0.45 * fibers
        ao = 0.76 + 0.24 * h
        rough = 0.78 + 0.22 * (1.0 - h)
        albedo = 0.78 + 0.22 * h
        return h, ao, rough, albedo

    if kind == "bone":
        # Aged porous skeletal bone with subtle longitudinal osteon striations.
        striation = _periodic_noise(size, 16, 503)
        for shift in (1, 2):
            striation = 0.75 * striation + 0.25 * np.roll(striation, shift, axis=0)
        pores = _fbm(size, ((8, 0.4), (16, 0.35), (32, 0.25)), 509)
        h = 0.48 * striation + 0.52 * pores
        ao = 0.84 + 0.16 * h
        rough = 0.76 + 0.24 * (1.0 - h)
        albedo = 0.88 + 0.12 * h
        return h, ao, rough, albedo

    if kind == "rope":
        # Tightly twisted braided hemp/cord strands.
        strand = 0.5 + 0.5 * np.sin(tau * (u * 14.0 + v * 7.0))
        sub = 0.5 + 0.5 * np.sin(tau * (u * 28.0 - v * 14.0))
        fiber = _periodic_noise(size, 24, 601)
        h = 0.62 * np.power(strand, 0.75) + 0.20 * sub + 0.18 * fiber
        ao = 0.70 + 0.30 * strand
        rough = 0.84 + 0.16 * (1.0 - strand)
        albedo = 0.80 + 0.20 * strand
        return h, ao, rough, albedo

    if kind == "feather":
        # Fine angled raven feather barbules.
        chevron = 0.5 + 0.5 * np.sin(tau * (np.abs(u - 0.5) * 28.0 + v * 16.0))
        micro = _fbm(size, ((8, 0.5), (24, 0.5)), 701)
        h = 0.70 * chevron + 0.30 * micro
        ao = 0.84 + 0.16 * h
        rough = 0.68 + 0.32 * (1.0 - h)
        albedo = 0.86 + 0.14 * h
        return h, ao, rough, albedo

    if kind == "fur":
        # Coarse directional gorilla fur coat.
        tufts = _periodic_noise(size, 24, 809)
        for shift in (1, 2, 3):
            tufts = 0.65 * tufts + 0.35 * np.roll(tufts, shift, axis=0)
        clump = _fbm(size, ((6, 0.5), (12, 0.5)), 811)
        h = 0.65 * tufts + 0.35 * clump
        ao = 0.74 + 0.26 * h
        rough = 0.82 + 0.18 * (1.0 - h)
        albedo = 0.86 + 0.14 * h
        return h, ao, rough, albedo

    if kind == "skin":
        # Organic leathery skin pores.
        cells = _fbm(size, ((8, 0.4), (16, 0.35), (32, 0.25)), 821)
        h = np.power(cells, 0.9)
        ao = 0.88 + 0.12 * h
        rough = 0.74 + 0.26 * (1.0 - h)
        albedo = 0.92 + 0.08 * h
        return h, ao, rough, albedo

    if kind == "wool":
        # Curly puffed llama wool fleece.
        c1 = _fbm(size, ((6, 0.45), (12, 0.35), (20, 0.20)), 907)
        swirl = 0.5 + 0.5 * np.sin(tau * (u * 8.0 + c1 * 1.8)) * np.cos(tau * (v * 8.0 - c1 * 1.6))
        h = 0.55 * c1 + 0.45 * swirl
        ao = 0.74 + 0.26 * h
        rough = 0.88 + 0.12 * (1.0 - h)
        albedo = 0.88 + 0.12 * h
        return h, ao, rough, albedo

    if kind == "textile":
        # Crisp Andean woven blanket textile.
        warp = 0.5 + 0.5 * np.sin(tau * u * 28.0)
        weft = 0.5 + 0.5 * np.sin(tau * v * 28.0)
        diamond = 0.5 + 0.5 * np.cos(tau * (u * 8.0 + v * 8.0)) * np.cos(tau * (u * 8.0 - v * 8.0))
        h = 0.55 * (warp * 0.5 + weft * 0.5) + 0.45 * diamond
        ao = 0.80 + 0.20 * h
        rough = 0.84 + 0.16 * (1.0 - h)
        albedo = 0.88 + 0.12 * h
        return h, ao, rough, albedo

    if kind == "plumage":
        # Sleek velvety micro-down feathers for penguin & street bird.
        row = np.floor(v * 14.0)
        stagger = (u * 14.0 + 0.5 * (row % 2)) % 1.0
        local_v = (v * 14.0) % 1.0
        scallop = np.clip(1.0 - np.hypot(stagger - 0.5, local_v * 0.7) * 1.5, 0.0, 1.0)
        micro = _fbm(size, ((8, 0.45), (16, 0.35), (32, 0.20)), 953)
        h = 0.22 * scallop + 0.78 * micro
        ao = 0.90 + 0.10 * h
        rough = 0.78 + 0.22 * (1.0 - h)
        albedo = 0.93 + 0.07 * h
        return h, ao, rough, albedo

    if kind == "machined":
        # Brushed & slightly scuffed scrapyard steel plates.
        brush = _periodic_noise(size, 32, 971)
        for shift in (1, 2, 3):
            brush = 0.68 * brush + 0.32 * np.roll(brush, shift, axis=1)
        scuff = _fbm(size, ((4, 0.5), (12, 0.5)), 977)
        h = 0.52 * brush + 0.48 * scuff
        ao = 0.80 + 0.20 * scuff
        rough = 0.60 + 0.40 * (1.0 - h)
        albedo = 0.86 + 0.14 * h
        return h, ao, rough, albedo

    # "stone" / carved reliquary bronze-stone relief.
    crag = _fbm(size, ((4, 0.4), (8, 0.35), (16, 0.25)), 991)
    chisel = 0.5 + 0.5 * np.sin(tau * (u * 12.0 + crag * 0.6))
    h = 0.68 * crag + 0.32 * chisel
    ao = 0.76 + 0.24 * crag
    rough = 0.68 + 0.32 * (1.0 - h)
    albedo = 0.84 + 0.16 * h
    return h, ao, rough, albedo


# Per-preset normal bump scale and glTF NormalMap node strength.
NORMAL_STRENGTH = {
    "cloth": (3.2, 0.65),
    "cloth_alt": (2.8, 0.55),
    "metal": (3.8, 0.75),
    "gold": (2.6, 0.55),
    "wood": (3.8, 0.75),
    "bone": (2.4, 0.50),
    "rope": (4.8, 0.85),
    "feather": (3.0, 0.60),
    "fur": (4.5, 0.80),
    "skin": (2.2, 0.45),
    "wool": (4.2, 0.78),
    "textile": (3.4, 0.68),
    "plumage": (1.4, 0.30),
    "machined": (3.2, 0.65),
    "stone": (4.0, 0.78),
}


def _pack_image(name, rgb_array, colorspace="Non-Color"):
    """Creates (or returns cached) packed PNG Blender image from HxWx3 float array."""

    existing = bpy.data.images.get(name)
    if existing is not None:
        return existing

    h, w, _ = rgb_array.shape
    rgba = np.ones((h, w, 4), dtype=np.float32)
    rgba[:, :, :3] = np.clip(rgb_array, 0.0, 1.0)

    img = bpy.data.images.new(name, w, h, alpha=False)
    img.colorspace_settings.name = colorspace
    img.pixels.foreach_set(rgba.ravel())
    img.file_format = "PNG"
    img.pack()
    return img


def _gltf_output_group():
    """Returns Blender's special 'glTF Material Output' group for glTF occlusion export."""

    group_name = "glTF Material Output"
    group = bpy.data.node_groups.get(group_name)
    if group is None:
        group = bpy.data.node_groups.new(group_name, "ShaderNodeTree")
        group.interface.new_socket(name="Occlusion", in_out="INPUT", socket_type="NodeSocketFloat")
    return group


def get_maps(kind, roughness=1.0, metallic=0.0):
    """Returns packed `(normal_img, orm_img, albedo_img, normal_strength)` for `kind`.

    `roughness` and `metallic` scale the ORM green/blue channels directly so
    material exports carry exact PBR response (for husks where `roughnessFactor`
    is 1.0 when textured, or for reapers/weapons pass `roughness=1.0, metallic=1.0`
    so `build.rs`'s `roughnessFactor` / `metallicFactor` multiplies cleanly).
    """

    height, ao, rough_mod, albedo_mod = _heightfield(kind)
    grad_scale, map_strength = NORMAL_STRENGTH.get(kind, (3.0, 0.6))

    normal_rgb = _height_to_normal(height, grad_scale)
    normal_img = _pack_image(f"tex_normal_{kind}", normal_rgb, "Non-Color")

    orm_key = f"tex_orm_{kind}_{int(round(roughness * 100))}_{int(round(metallic * 100))}"
    orm_rgb = np.empty((SIZE, SIZE, 3), dtype=np.float32)
    orm_rgb[:, :, 0] = ao
    orm_rgb[:, :, 1] = np.clip(rough_mod * roughness, 0.05, 1.0)
    orm_rgb[:, :, 2] = np.full((SIZE, SIZE), metallic, dtype=np.float32)
    orm_img = _pack_image(orm_key, orm_rgb, "Non-Color")

    albedo_rgb = np.repeat(albedo_mod[:, :, None], 3, axis=2)
    albedo_img = _pack_image(f"tex_albedo_{kind}", albedo_rgb, "sRGB")

    return normal_img, orm_img, albedo_img, map_strength


def apply_pbr(mat, kind, roughness=1.0, metallic=0.0, use_albedo=True, use_orm=True):
    """Wires procedural normal (+ optional ORM and subtle grayscale albedo) into `mat`."""

    mat.use_nodes = True
    tree = mat.node_tree
    nodes = tree.nodes
    links = tree.links
    shader = nodes.get("Principled BSDF")
    if shader is None:
        return mat

    # Avoid double-wiring if the shared material is requested more than once.
    if mat.get("_pina_pbr_kind") == kind:
        return mat
    mat["_pina_pbr_kind"] = kind

    normal_img, orm_img, albedo_img, strength = get_maps(kind, roughness=roughness, metallic=metallic)

    tex_norm = nodes.new("ShaderNodeTexImage")
    tex_norm.image = normal_img
    norm_map = nodes.new("ShaderNodeNormalMap")
    norm_map.inputs["Strength"].default_value = strength
    links.new(tex_norm.outputs["Color"], norm_map.inputs["Color"])
    links.new(norm_map.outputs["Normal"], shader.inputs["Normal"])

    if use_orm:
        tex_orm = nodes.new("ShaderNodeTexImage")
        tex_orm.image = orm_img
        sep = nodes.new("ShaderNodeSeparateColor")
        links.new(tex_orm.outputs["Color"], sep.inputs["Color"])
        links.new(sep.outputs["Green"], shader.inputs["Roughness"])
        if metallic > 0.0:
            links.new(sep.outputs["Blue"], shader.inputs["Metallic"])

        gltf_out = nodes.new("ShaderNodeGroup")
        gltf_out.node_tree = _gltf_output_group()
        links.new(sep.outputs["Red"], gltf_out.inputs["Occlusion"])

    if use_albedo:
        tex_albedo = nodes.new("ShaderNodeTexImage")
        tex_albedo.image = albedo_img
        links.new(tex_albedo.outputs["Color"], shader.inputs["Base Color"])

    return mat


def write_box_uvs(obj, scale=3.5):
    """Writes clean world-scale box-projected UVs into `TEXCOORD_0` (first UV layer).

    Avoids smeared/degenerate UV derivatives for screen-space normal mapping
    (`leptoskit_3d::shader.wgsl::perturb`), without touching meshes that carry
    authored image/shadow/palette coordinates.
    """

    mesh = obj.data
    for layer in list(mesh.uv_layers):
        mesh.uv_layers.remove(layer)

    uv_layer = mesh.uv_layers.new(name="UVMap")
    uv_data = uv_layer.data
    verts = mesh.vertices

    for poly in mesh.polygons:
        nx, ny, nz = abs(poly.normal.x), abs(poly.normal.y), abs(poly.normal.z)
        if nz >= nx and nz >= ny:
            axis = 2
        elif ny >= nx:
            axis = 1
        else:
            axis = 0

        for loop_idx in poly.loop_indices:
            co = verts[mesh.loops[loop_idx].vertex_index].co
            if axis == 2:
                uv_data[loop_idx].uv = (co.x * scale, co.y * scale)
            elif axis == 1:
                uv_data[loop_idx].uv = (co.x * scale, co.z * scale)
            else:
                uv_data[loop_idx].uv = (co.y * scale, co.z * scale)
