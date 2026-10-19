// Hides the part of a husk that is behind something in the room.
//
// Drawn as a mask over the 3D scene: where the room has something standing
// nearer than the husk, the scene is cut away and the camera's own picture
// of the counter, the sofa or the table shows through in its place. The
// room is an OcclusionMap: how far the nearest thing is in every direction
// from where the player stands, laid out as an image by yaw and pitch.

#version 460 core

#include <flutter/runtime_effect.glsl>

precision highp float;

// The size of the view, in the same units as FlutterFragCoord.
uniform vec2 uSize;

// Which way the phone points: the middle of the view, and the view's right
// and up, in the hunt's space.
uniform vec3 uForward;
uniform vec3 uRight;
uniform vec3 uUp;

// The tangents of half the view across and up.
uniform vec2 uTanHalf;

// The husk, taken as an upright ellipsoid: the middle of its body, in
// metres from the eye, and its half width and half height. Over how many
// metres either side of its surface the cut fades in.
uniform vec3 uHusk;
uniform vec2 uHuskRadii;
uniform float uSoftness;

// The OcclusionMap. Centimetres in red (high byte) and green (low byte);
// blue is 1 where something standing was seen, a half where only floor was,
// and 0 where nothing was.
uniform sampler2D uRoom;

out vec4 fragColor;

const float CELL = 0.25;
const float HIGHEST = 48.0;
const vec2 GRID = vec2(1440.0, 480.0);

// How far along the unit [ray] the husk's surface is: where the ray enters
// its ellipsoid, or, for a ray that passes it by (its shadow, a leg held
// out), how far it is when the ray passes nearest it.
float huskAlong(vec3 ray) {
  vec3 scale = vec3(uHuskRadii.x, uHuskRadii.y, uHuskRadii.x);
  vec3 direction = ray / scale;
  vec3 centre = uHusk / scale;
  float a = dot(direction, direction);
  float b = dot(direction, centre);
  float c = dot(centre, centre) - 1.0;
  float inside = b * b - a * c;

  if (inside < 0.0) {
    return max(0.0, dot(ray, uHusk));
  }

  return max(0.0, (b - sqrt(inside)) / a);
}

// How much of the husk shows through one cell of the map, [husk] metres
// out along the ray.
float visibleAt(vec2 cell, float husk) {
  if (cell.y < 0.0 || cell.y >= GRID.y) {
    return 1.0;
  }

  vec4 texel = texture(uRoom, (vec2(mod(cell.x, GRID.x), cell.y) + 0.5) / GRID);

  if (texel.b < 0.75) {
    return 1.0;
  }

  float metres = (texel.r * 65280.0 + texel.g * 255.0) / 100.0;

  return smoothstep(-uSoftness, uSoftness, metres - husk);
}

void main() {
  vec2 at = FlutterFragCoord().xy / uSize;
  vec2 view = vec2(at.x * 2.0 - 1.0, 1.0 - at.y * 2.0);
  vec3 ray = normalize(
    uForward + uRight * (view.x * uTanHalf.x) + uUp * (view.y * uTanHalf.y)
  );
  float yaw = degrees(atan(ray.x, ray.z));
  float pitch = degrees(asin(clamp(ray.y, -1.0, 1.0)));
  float husk = huskAlong(ray);

  // Read between the four nearest cells, so the edge of the counter is a
  // line and not a staircase.
  vec2 cell = vec2((yaw + 180.0) / CELL, (HIGHEST - pitch) / CELL) - 0.5;
  vec2 base = floor(cell);
  vec2 within = cell - base;
  float top = mix(
    visibleAt(base, husk),
    visibleAt(base + vec2(1.0, 0.0), husk),
    within.x
  );
  float bottom = mix(
    visibleAt(base + vec2(0.0, 1.0), husk),
    visibleAt(base + vec2(1.0, 1.0), husk),
    within.x
  );

  fragColor = vec4(mix(top, bottom, within.y));
}
