"""Shape-preserving cubic pose curves: velocity continues through breakdowns."""


def sample(keys, phase):
    """Monotone Hermite interpolation avoids overshoot and per-key braking."""
    joints = set().union(*(pose.keys() for _, pose in keys))
    result = {}
    for joint in joints:
        components = []
        for axis in range(3):
            values = [pose.get(joint, (0, 0, 0))[axis] for _, pose in keys]
            times = [time for time, _ in keys]
            slopes = [(b - a) / (end - start) for a, b, start, end in zip(values, values[1:], times, times[1:])]
            tangents = [0]
            for left, right in zip(slopes, slopes[1:]):
                tangents.append(0 if left * right <= 0 else 2 * left * right / (left + right))
            tangents.append(0)
            i = next((i for i in range(len(times) - 1) if phase <= times[i + 1]), len(times) - 2)
            dt = times[i + 1] - times[i]
            t = max(0, min(1, (phase - times[i]) / dt))
            components.append(
                (2 * t**3 - 3 * t * t + 1) * values[i]
                + (t**3 - 2 * t * t + t) * dt * tangents[i]
                + (-2 * t**3 + 3 * t * t) * values[i + 1]
                + (t**3 - t * t) * dt * tangents[i + 1]
            )
        result[joint] = tuple(components)
    return result
