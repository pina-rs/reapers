//! Benches for development: a room drawn by hand to hunt in where no room
//! can be seen, the depth bench, the arms bench and the kit page. Debug
//! builds only; a release never shows them. The older benches' words are
//! plain English and stay out of the translations, as the Flutter game's
//! did: a player never reads them. The arms bench's and the kit page's are
//! in the translations, so they can be shown to anyone.

pub mod arms;
pub mod drawn_room;
pub mod kit;
pub mod probe;
pub mod room_overlay;
