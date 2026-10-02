# Music

Nothing here was composed or generated for the game. Each piece is a recording somebody made and released for anyone to use. `tool/build_music.dart` fetches the originals and re-encodes them; the only change made to the sound is its level.

| File | Piece | By | Licence |
| --- | --- | --- | --- |
| `sanctum.ogg` | J.S. Bach, Prelude No. 8 in E-flat minor, BWV 853, from The Well-Tempered Clavier, Book 1 | Kimiko Ishizaka, piano. From the Open Well-Tempered Clavier, [welltemperedclavier.org](https://welltemperedclavier.org) | Public domain (CC0), by the performer's dedication |
| `hunt.ogg` | Anabasis I, from The Weight of Air | Scott Buckley, [scottbuckley.com.au](https://www.scottbuckley.com.au/library/anabasis-i/) | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) |
| `rift.ogg` | Katabasis I, from The Weight of Air | Scott Buckley, [scottbuckley.com.au](https://www.scottbuckley.com.au/library/katabasis-i/) | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) |

## Credit lines

The two pieces under CC BY 4.0 have to be credited wherever the game is published, and in the description of any video of it. The composer asks for these words:

- 'Anabasis I' by Scott Buckley - released under CC-BY 4.0. www.scottbuckley.com.au
- 'Katabasis I' by Scott Buckley - released under CC-BY 4.0. www.scottbuckley.com.au

The Bach recording asks for no credit. It is given one anyway: one page that hosts it marks it CC BY 3.0, and naming the performer satisfies that reading too.

The game's colophon screen carries all three.

## Changes

Each piece was re-encoded as Ogg Vorbis at about 160 kbit/s. The Bach was raised by 6 dB. In play, the game changes how loud a piece is and passes it through a low-pass filter; it does not cut, loop inside, or mix the recordings together.
