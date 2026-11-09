//! Benches for development: a room drawn by hand to hunt in where no room
//! can be seen, the depth bench and the arms bench. Debug builds only; a
//! release never shows them. The older benches' words are plain English
//! and stay out of the translations, as the Flutter game's did: a player
//! never reads them. The arms bench's are in the translations, so it can
//! be shown to anyone.

pub mod arms;
pub mod drawn_room;
pub mod probe;
