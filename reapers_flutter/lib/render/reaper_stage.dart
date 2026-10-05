// Keep Flutter GPU out of the browser compilation graph.
export 'reaper_stage_native.dart'
    if (dart.library.js_interop) 'reaper_stage_web.dart';
