# Dark games and AR hunting: design research for Reapers

Research date: 2026-10-01.

How to read this document:

- **EVIDENCE** marks a claim taken from a primary source, official documentation, a developer statement, peer-reviewed work or reputable reporting. The source is linked next to the claim.
- **CRAFT** marks a heuristic or my own synthesis. Treat these as proposals to test, not facts.
- Developer statements are paraphrased. There is one direct quotation from a source in this document (Miyazaki, in section 1). Other quotation marks are titles, game terms or my own labels.
- Where I only saw a search-result excerpt and could not open the page, the claim says so. Everything I could not verify is collected in [Not verified](#not-verified) at the end.

## What this changes in Reapers

1. **Design each Reaper class as a black shape before anything else.** Valve validated Team Fortress 2's nine classes by checking they could be told apart as pure silhouettes with no interior shading ([Mitchell, Francke and Eng, 2007](https://steamcdn-a.akamaihd.net/apps/valve/2007/NPAR07_IllustrativeRenderingInTeamFortress2.pdf)). For Reapers the class silhouette should be carried by three things only: head shape, shoulder line, and weapon line. (EVIDENCE for the method, CRAFT for the three carriers.)
2. **Randomise inside a class skeleton, not across it.** Blizzard's stated goal for Diablo IV was heavy customisation while every class stays recognisable ([Blizzard, June 2021](https://news.blizzard.com/en-us/article/23665024/diablo-iv-quarterly-updatejune-2021)). If hood, mask, horns, shoulders and weapon are all shuffled freely, the result is noise. Let the class lock the head, shoulder and weapon families, and let the mint shuffle vary parts within those families plus robe, aura and palette. (EVIDENCE for the goal, CRAFT for the rule.)
3. **"Darker" should mean the same outfit after something happened to it.** In Dark Souls the Black Knights are the Silver Knights after being burned, and the artists were pleased that players worked this out from the armour alone ([Dark Souls Design Works interview](http://soulslore.wikidot.com/das1-design-works)). Progression in Reapers should char, hollow and strip the starting outfit rather than bolt on more spikes. (EVIDENCE for the precedent, CRAFT for the rule.)
4. **Menace comes from dignity and concealment, not gore.** Miyazaki says he holds every design to a standard of refinement and elegance, and redirected an undead dragon concept away from maggots toward the sadness of a great creature dying out ([Design Works interview](http://soulslore.wikidot.com/das1-design-works)). Reapers should hide faces and keep clean, deliberate shapes. (EVIDENCE.)
5. **Never use neutral black, and always rim-light the figure.** Darkest Dungeon's artist kept pure black for the ink lines only and kept the palette warm ([GameSpot, 2015](https://www.gamespot.com/articles/the-gothic-sensibilities-of-darkest-dungeon/1100-6424880/)); TF2 adds a dedicated rim-light term so characters always separate from the background. In AR the background is whatever the camera sees, so a dark figure with no rim light will vanish against a dark room. (EVIDENCE for the two practices, CRAFT for the AR conclusion.)
6. **Because players cannot choose their Reaper, a class must be a lean, not a lock.** Souls classes only set starting stats and gear; Bloodborne origins set stats only ([Fextralife](https://bloodborne.wiki.fextralife.com/Classes)). Keep the stat gap between classes small, and let any class buy any weapon, so nobody feels they minted the wrong Reaper. (EVIDENCE for the precedent, CRAFT for the rule.)
7. **Tap-to-strike alone is the Pokémon GO raid loop.** Niantic's own help page describes raid combat as tap for a fast attack, tap a button when the meter fills, swipe to dodge ([Pokémon GO Help Center](https://niantic.helpshift.com/hc/en/6-pokemon-go/faq/2187-what-are-raid-battles/)). If Reapers ships that, it is a reskin. The opening for a twist is the phone itself: reward slow, steady aiming and stillness (see item 8). (EVIDENCE for the comparison, CRAFT for the judgement.)
8. **Make stillness the skill.** Pokémon GO's AR+ mode added an awareness meter that fills as you approach and makes the creature flee ([Niantic, December 2017](https://pokemongo.com/post/arplus/?hl=en)). Apple and Google both tell AR developers to avoid sudden, sweeping movement ([Apple HIG](https://developer.apple.com/design/human-interface-guidelines/augmented-reality), [ARCore](https://developers.google.com/ar/design/user/safety-comfort)). A Reaper that stalks by moving the phone slowly and strikes from a steady aim is both unusual and the safest possible AR input. (EVIDENCE for the inputs, CRAFT for the proposal.)
9. **Encounters should last about a minute and work one-handed while standing still.** Monster Hunter Now built its whole game around 75-second hunts and one-handed play ([Niantic/Capcom launch post](https://monsterhunternow.com/en/news/launch)). Apple recommends short levels with downtime between them. (EVIDENCE.)
10. **Ship raids with a remote path on day one.** Every comparable game added remote play after launch, and Pokémon GO's later price rise and daily cap on remote raids caused a player backlash ([Niantic, 2023](https://pokemongo.com/post/remote-raid-passes-update-2023?hl=en), [Monster Hunter Now Dimensional Links](https://monsterhunternow.com/news/dimensionallinks)). Design the remote slot and its limits up front so nothing has to be taken away later. (EVIDENCE for the history, CRAFT for the rule.)
11. **Use a two-minute lobby and squads of four inside bigger raids.** Monster Hunter Now auto-starts group hunts 120 seconds after recruitment opens; Pokémon GO's largest battles put up to 40 players in one lobby but split them into groups of four ([MHN Help Center](https://niantic.helpshift.com/hc/en/39-monster-hunter-now/faq/3943-hunting-monsters/), [Pokémon GO Max Battles](https://pokemongo.com/max-pokemon-battle)). (EVIDENCE.)
12. **Absorbing an NFT must be a sink with gameplay value only, never a faucet.** Axie Infinity's reward token was being produced about four times faster than it was burned before the team cut emissions ([Fortune](https://fortune.com/2022/03/31/axie-infinity-hack-coin-price-crypto-crisis/); the production-to-burn ratio comes from search excerpts of coverage such as [BitPinas](https://bitpinas.com/play-to-earn/axie-infinity-adventure-daily-quest-slp-rewards-zero/), pages not opened). Reapers should not pay out a tradeable token for play. (EVIDENCE for the collapse, CRAFT for the rule.)

Where the concept risks being generic: a hooded figure with a scythe is the most used death image there is, and plague-doctor masks are close behind. The six classes below deliberately move the silhouette away from "hood plus scythe" for five of the six.

## 1. Dark-fantasy character art direction

### Principles with sources

**Silhouette first.**

- EVIDENCE: Valve designed TF2's classes so each could be identified in silhouette with no lighting cues, and used that test during concept work. Interior clothing folds were drawn to echo the outline, and a rim-light term keeps characters separated from the scene ([paper](https://steamcdn-a.akamaihd.net/apps/valve/2007/NPAR07_IllustrativeRenderingInTeamFortress2.pdf), [TF2 wiki transcript](https://wiki.teamfortress.com/wiki/Illustrative_Rendering_in_Team_Fortress_2)).
- EVIDENCE: Team Cherry describe the Hollow Knight protagonist as the simplest possible way to show a bug that behaves like a knight. Ari Gibson notes that a cloak is useful because it hides the arms, and that a protagonist gets drawn a thousand times so there should not be much to it ([MCV/Develop, 2018](https://mcvuk.com/development-news/when-we-made-hollow-knight/)).
- EVIDENCE: Blizzard's associate art director for characters on Diablo IV lists readable detail, colours that group well and silhouettes that stand out as the requirements for creatures seen from the game camera ([Blizzard, June 2021](https://news.blizzard.com/en-us/article/23665024/diablo-iv-quarterly-updatejune-2021)).
- EVIDENCE: Discussing Big Hat Logan, the artist says the hat came out so big the face cannot be seen; Miyazaki replies that it made a good silhouette ([Design Works interview](http://soulslore.wikidot.com/das1-design-works)).
- CRAFT: A cloak is the cheapest way to get a strong silhouette and it also hides rigging and animation cost. For a modular Reaper this means the robe can be simple, and budget should go to the head, shoulders and weapon.

**Dignity instead of gore.**

- EVIDENCE: Miyazaki says he tries to keep refinement and elegance in every design and tells artists that muddy or messy is not acceptable. The undead dragon was first drawn covered in maggots; he asked instead for the sorrow of a great creature heading to extinction ([Design Works interview](http://soulslore.wikidot.com/das1-design-works)).
- EVIDENCE: On Dark Souls III he said the team wanted to show beauty that persists beyond withering and decay, and that sadness and loneliness are deliberate themes ([Game Developer, 2016, citing a Vice interview](https://www.gamedeveloper.com/business/director-miyazaki-speaks-to-what-drives-development-of-i-dark-souls-3-i-)).
- CRAFT: For Reapers, avoid blood, exposed organs and clutter. The figure should look like it has a job and takes it seriously.

**Hide the face.**

- EVIDENCE: Talking about a helmeted character whose face is never shown, Miyazaki says: "That's Dark Souls in a nutshell." ([Design Works interview](http://soulslore.wikidot.com/das1-design-works)).
- CRAFT (secondary analysis): A critical essay on Darkest Dungeon observes that hero eyes are always hidden by mask or shadow while monsters' eyes are visible, and reads that as deliberate ([Invisible Eyes, 2018](https://marywashicomics.net/article/the-art-of-darkest-dungeon/)). This is an observer's reading, not a developer statement.
- CRAFT: The Reaper face slot should be a mask, a skull, a void or a veil. Eyes, if present, are a light source and nothing else.

**Materials carry history, and gear must look usable.**

- EVIDENCE: Nakamura did not want to make a standard armour and swap parts; he aimed for designs that are not traditional but look like they could exist. Miyazaki briefed equipment with instructions such as making something you would trust with your life ([Design Works interview](http://soulslore.wikidot.com/das1-design-works)).
- EVIDENCE: For the Black Knights, Miyazaki wanted detail down to patterns carved into the armour, no cloth (because cloth would not fit a burned figure), and armour that looks too thick and heavy for a normal person, almost hollow ([Design Works interview](http://soulslore.wikidot.com/das1-design-works)).
- EVIDENCE: Diablo IV's art director said the team wanted characters to look hand-crafted and was wary of realism that would read as procedural or generic; the lead character artist describes materials that respond to light realistically ([Blizzard, June 2021](https://news.blizzard.com/en-us/article/23665024/diablo-iv-quarterly-updatejune-2021)).
- CRAFT: Limit each Reaper to three materials at a time (for example cloth, bone, iron). Progression changes the state of those materials, not their number.

**Restraint in palette.**

- EVIDENCE: Chris Bourassa says he used dead grey and black only in the ink lines, kept the Darkest Dungeon palette warm, gave each dungeon one dominant hue, and relied on strong linework so he could add unexpected hits of colour without losing readability ([GameSpot, 2015](https://www.gamespot.com/articles/the-gothic-sensibilities-of-darkest-dungeon/1100-6424880/), read via the [Yahoo mirror](https://www.yahoo.com/news/gothic-sensibilities-darkest-dungeon-164800337.html)).
- EVIDENCE: In 2013 he explained the style choice: stark shadows express encroaching darkness, hard edges match unforgiving gameplay, and the look should suggest an illuminated manuscript ([Mash Those Buttons, 2013](https://mashthosebuttons.com/2013/11/a-talk-with-darkest-dungeons-chris-bourassa/)).
- EVIDENCE: Jen Zee says Hades moved from a painterly style to pen and ink when the tone changed in preproduction, and that ink assets are also faster to produce ([MCV/Develop, 2021](https://mcvuk.com/business-news/behind-the-art-of-hades-we-value-artistic-integrity-and-excellence-in-artistic-craft-at-supergiant-however-were-first-and-foremost-a-game-design-lead-team/)).
- EVIDENCE (reporting on leaked art-book pages): Diablo IV's three art pillars were named Return to Darkness, Embrace the Legacy and Old Masters ([PCGamesN, 2019](https://www.pcgamesn.com/diablo-4/art-book-details)).
- CRAFT: One dominant hue per class, one accent, tinted blacks. The accent is the only saturated colour on the figure.

**Asymmetry and the hand-made.**

- EVIDENCE: Miyazaki describes Solaire's set as built around a large sun symbol and a mismatched, almost hand-made look, and says that design came together quickly ([Design Works interview](http://soulslore.wikidot.com/das1-design-works)).
- CRAFT: Give every class one asymmetric element (a single pauldron, a one-sided mantle, a lantern on one side). It breaks the "action figure" symmetry and gives the modular system an obvious slot to vary.

**Light sources on the figure.**

- EVIDENCE: Gravelord Nito was meant to be cloaked in shadow and surrounded by a deathly aura. Miyazaki says animators kept delivering smoke, and the problem was solved by telling them to make the aura behave more like cloth ([Design Works interview](http://soulslore.wikidot.com/das1-design-works)).
- CRAFT: Build Reaper auras as cloth-like ribbons or sheets with weight, not particle smoke. Smoke also reads badly over a live camera feed.
- CRAFT: One emissive point per Reaper (eyes, lantern, blade edge or collar). It tells the player where the "self" of the figure is and it survives dark real-world backgrounds.

**Brief with keywords, not specifications.**

- EVIDENCE: Miyazaki gave each artist a few simple keywords as a starting point and avoided dictating colour or shape, because overly specific instructions produce less creative designs ([Design Works interview](http://soulslore.wikidot.com/das1-design-works)).
- CRAFT: Write each class brief as five or six words plus a silhouette rule. That is also the right shape of brief for any generative or outsourced art pipeline.

**The view the player actually sees.**

- EVIDENCE: FromSoftware paid special attention to the back of the starting warrior armour because that is what the player looks at most of the time ([Design Works interview](http://soulslore.wikidot.com/das1-design-works)).
- CRAFT: Decide the hero view for a Reaper early. If it is mostly seen in a lobby, profile card or versus screen, the front three-quarter view matters most. If the weapon is seen in first person during AR, the weapon and forearm need the detail.

**Draw on a specific culture, not default medieval Europe.**

- EVIDENCE: Blasphemous' art director says its influences are mostly artists from southern Spain (Goya, Murillo, Velázquez) and that religion there is lived as culture and folklore ([PC Zone, 2023](https://www.pczone.co.uk/interview-blasphemous-2-art-director-enrique-cabeza/)).
- EVIDENCE: Death's Door started from the idea of a door to the afterlife and a bureaucracy that has taken over the flow of souls; the developers call the small crow the soul of the game ([Byteside, 2021](https://www.byteside.com/2021/08/deaths-door-developer-acid-nerve-on-how-it-made-its-sword-wielding-crow-game/)).
- CRAFT: Both games are remembered because the death imagery is specific. The psychopomp table in section 3 exists so Reapers can avoid the default hood and scythe.

**Bloodborne.** EVIDENCE: Miyazaki describes the setting as old gothic towns with Victorian-era elements layered on top ([interview compilation](https://www.bloodborne-wiki.com/2017/01/interviews.html)). I found no primary statement about how the hunter attire itself was designed.

## 2. Class systems in dark games

| Game | What a class is | How identity is made readable | Source |
| --- | --- | --- | --- |
| Dark Souls | 10 starting classes; a class sets starting stats and gear | Each has one lead stat and a distinct starting set. The Warrior was pushed toward "adventurer" rather than soldier, using keywords of leather with metal plating | EVIDENCE: [Fextralife class table](https://darksouls.wiki.fextralife.com/Classes), [Design Works interview](http://soulslore.wikidot.com/das1-design-works) |
| Elden Ring | 10 starting classes | One lead stat plus a signature weapon: Samurai (Dexterity, uchigatana and longbow), Hero (Strength, battle axe), Astrologer (Intelligence, staff), Prophet (Faith, spear and seal), Warrior (Dexterity, two scimitars), Wretch (level 1, a club, no armour) | EVIDENCE: [Fextralife class table](https://eldenring.wiki.fextralife.com/Classes), [Bandai Namco class page](https://en.bandainamcoent.eu/elden-ring/elden-ring/classes) |
| Bloodborne | 9 origins; stats and starting currency only, no gear difference | A one-line biography per origin (for example a violent past, a noble line, a lone survivor) and one raised stat | EVIDENCE: [Fextralife](https://bloodborne.wiki.fextralife.com/Classes) |
| Diablo IV | Fixed classes with heavy cosmetic customisation | Blizzard states each class should be instantly recognisable; armour pieces can each be dyed | EVIDENCE: [Blizzard, June 2021](https://news.blizzard.com/en-us/article/23665024/diablo-iv-quarterly-updatejune-2021) |
| Darkest Dungeon | 15 base classes plus 5 DLC; heroes arrive by stagecoach with a random name, class and quirks | Class weapons and armour are upgraded in tiers at the Blacksmith, gated by resolve level | EVIDENCE: [official wiki](https://darkestdungeon.wiki.gg/wiki/Heroes_(Darkest_Dungeon)) |

What makes a class readable (CRAFT synthesis from the table):

- One signature weapon per class. Elden Ring's classes can be told apart from the weapon pair alone.
- One lead stat that matches what the weapon looks like it does. A long weapon implies reach, a heavy one implies power.
- A one-line biography. Bloodborne's origins have no visual difference at all and still feel distinct because of a single sentence.
- A "nothing" option. Dark Souls (Deprived), Elden Ring (Wretch) and Bloodborne (Waste of Skin) each include a class defined by lacking everything. Players treat it as a badge.

Random assignment has a precedent: Darkest Dungeon gives the player a random roster and the game is about making the best of who shows up (EVIDENCE: [official wiki](https://darkestdungeon.wiki.gg/wiki/Heroes_(Darkest_Dungeon))). CRAFT: random assignment is accepted when classes are close in power and each has an obvious job in a group.

How armour communicates "darker and stronger":

- EVIDENCE: The Black Knight is the Silver Knight after burning. Same silhouette, new state ([Design Works interview](http://soulslore.wikidot.com/das1-design-works)).
- EVIDENCE: The Elite Knight set was made by taking an earlier knight armour and adding elements such as a surcoat to bring out nobility ([Design Works interview](http://soulslore.wikidot.com/das1-design-works)). Progression there is added rank, not added bulk.
- EVIDENCE (search excerpt, page not opened): Diablo IV transmogs are shared only between characters of the same class, which keeps class identity through cosmetic change ([Fextralife summary of the August 2022 update](https://fextralife.com/diablo-4-quarterly-update-august-2022-seasons-and-transmog-designs/)).
- CRAFT: Three levers signal progression without breaking the silhouette: value (the outfit gets darker), material state (cloth becomes charred, metal becomes pitted, parts go missing), and accent intensity (the one light on the figure gets brighter as everything else gets darker). Size and spike count are the weak levers because they blur class silhouettes together at high level.

## 3. Death figures and psychopomps

Britannica blocked automated access, so this table relies on Wikipedia articles and names the primary or scholarly sources those articles cite, plus one scholar interview.

| Figure | Origin and status | Signature motifs and tools | Notes for Reapers |
| --- | --- | --- | --- |
| Grim Reaper | Western Europe. Death as an animated skeleton dates from the Middle Ages; the English term is first attested in 1847 | Skeleton, robe, scythe | EVIDENCE: [Wikipedia, Death (personification)](https://en.wikipedia.org/wiki/Death_(personification)). Folk image, free to use. The most generic option |
| Ankou | Breton folklore. A servant of Death; the last person to die in a parish each year takes the role for the next year (Anatole Le Braz, 1893) | Tall thin man or skeleton, black robe, large hat hiding the face, scythe, a cart or a black coach with four black horses and two attendants on foot | EVIDENCE: [Wikipedia, Ankou](https://en.wikipedia.org/wiki/Ankou). The rotating role and the hat are strong, under-used hooks |
| Santa Muerte | Mexican folk saint. **Living devotion** with an estimated 10 to 12 million devotees; condemned by the Catholic Church | Female skeleton in a robe; scythe, globe, scales, hourglass, owl, oil lamp; robe and candle colours carry specific meanings | EVIDENCE: [R. Andrew Chesnut, OUP blog](https://blog.oup.com/2017/11/santa-muerte-saint-death-r-andrew-chesnut/), [Wikipedia](https://en.wikipedia.org/wiki/Santa_Muerte). Do not depict her. Scales, lamps and hourglasses are shared objects and can be used on their own |
| Shinigami | Japan. The word is rare in classical literature, appears in the Edo period, and the modern personified death god is shaped by Western ideas | No single canonical appearance | EVIDENCE: [Wikipedia, Shinigami](https://en.wikipedia.org/wiki/Shinigami). There is nothing specific to copy; modern looks belong to particular manga and anime, which are copyrighted |
| Thanatos | Ancient Greece | Winged youth, or a bearded winged man on the Euphronios krater carrying Sarpedon with Hypnos; inverted torch; sheathed sword; later a butterfly or poppy wreath | EVIDENCE: [Wikipedia, Thanatos](https://en.wikipedia.org/wiki/Thanatos). The inverted torch is a distinctive, rarely used tool |
| Charon | Ancient Greece. Ferryman of the dead | Boat, pole or oar; a rough, unkempt seaman on early vases; burning eyes in Virgil; a coin placed with the dead as fare. The Etruscan Charun is separate and carries a large hammer | EVIDENCE: [Wikipedia, Charon](https://en.wikipedia.org/wiki/Charon) |
| Anubis | Ancient Egypt | Black canine or jackal-headed man. Black refers to the colour of an embalmed body and to fertile Nile silt. Embalmer, guide of the dead, guardian of the scales; flail; imiut fetish | EVIDENCE: [Wikipedia, Anubis](https://en.wikipedia.org/wiki/Anubis). Use the weighing idea and the black-as-rebirth idea, not the jackal head |
| The Morrígan | Irish mythology (Ulster and Mythological Cycles) | War and fate; appears as a crow; sometimes a triple figure; washes the bloodied clothes of those about to die at a ford | EVIDENCE: [Wikipedia, The Morrígan](https://en.wikipedia.org/wiki/The_Morr%C3%ADgan) |
| Valkyries | Norse sources. The name means chooser of the slain | Helmet, spear, shield, horse; swan garments; ravens; drinking horns; in Darraðarljóð they weave on a loom made of human remains | EVIDENCE: [Wikipedia, Valkyrie](https://en.wikipedia.org/wiki/Valkyrie) |
| Baron Samedi | Haitian Vodou. A lwa, head of the Gede family. **Living religion** | Top hat, black tailcoat, dark glasses, cotton plugs in the nostrils (a body dressed for burial), rum, tobacco, a cemetery cross | EVIDENCE: [Wikipedia, Baron Samedi](https://en.wikipedia.org/wiki/Baron_Samedi) (the article carries a citation warning). Do not depict him. A formal funeral coat as a garment type is fine; the hat, glasses and cigar combination is his |
| Plague doctor | 17th-century Europe, not medieval. Costume credited to Charles de Lorme, 1619 | Beaked mask filled with aromatics, glass eye openings, long coat, hat, gloves, cane | EVIDENCE: [Wikipedia, Plague doctor costume](https://en.wikipedia.org/wiki/Plague_doctor_costume), which also notes evidence for wide use is thin. Heavily used in games already |
| Banshee | Irish folklore. A woman whose wailing foretells a death in certain families | Long loose hair, sometimes a comb; grey cloak over a green dress, or white; eyes red from weeping | EVIDENCE: [Wikipedia, Banshee](https://en.wikipedia.org/wiki/Banshee) |
| Dullahan | Irish folklore (Croker, 1828) | Headless rider on a black horse carrying his own head; a whip made from a spine (per modern commentators); a death coach; fear of gold | EVIDENCE: [Wikipedia, Dullahan](https://en.wikipedia.org/wiki/Dullahan) |

Guidance on respect (CRAFT):

- Santa Muerte and Baron Samedi belong to living practice. Take nothing that identifies them as individuals. Abstract objects (scales, a lamp, a funeral coat) are acceptable; names, full attribute sets and colour codes are not.
- I did not research current devotional practice around the Morrígan, the valkyries or Anubis. Treat them with the same care until that is checked: use roles and objects, not names or likenesses.
- Do not name a class after any of these figures. The class names below are job titles.

## Proposed Reaper classes

Shared rules (CRAFT):

- The class fixes the head family, shoulder family and weapon family. The mint shuffle varies parts within those families and varies robe, aura and palette freely.
- Four stats, each rated 1 to 5, every class totals 13: **Strike** (damage per hit), **Reach** (how far and how wide a strike lands), **Capture** (strength of the binding once a spirit is broken), **Resilience** (how much a Reaper can absorb before being pushed out of the encounter).
- Four outfit states shared by all classes, so "darker" is the same idea everywhere: **Shrouded** (undyed, rope and wood), **Sworn** (dyed dark, iron fittings, carved marks), **Scorched** (charred cloth, pitted metal, cloth starts to be replaced by plate or bone), **Hollowed** (parts of the figure are absent, the body is near-black, the accent light is at its brightest).

| Class | Strike | Reach | Capture | Resilience |
| --- | --- | --- | --- | --- |
| Harvester | 3 | 5 | 3 | 2 |
| Ferryman | 3 | 2 | 3 | 5 |
| Weigher | 4 | 2 | 4 | 3 |
| Chooser | 5 | 4 | 2 | 2 |
| Hollow | 4 | 3 | 2 | 4 |
| Mourner | 1 | 4 | 5 | 3 |

### Harvester

- **Fantasy:** the field hand of death, who cuts down many at once.
- **Inspiration:** the European Grim Reaper and the Breton Ankou, specifically the Ankou's wide hat that hides the face.
- **Silhouette hook:** a flat, wide-brimmed hat on a tall narrow body, with a scythe longer than the figure. The outline reads as a "T" crossed by a long diagonal.
- **Signature weapon:** long scythe. Wide, slow sweeps that hit several weak points in one pass.
- **Palette:** straw ochre and rust on brown-black; bone-white accent on the blade edge.
- **Stat lean:** Reach.
- **Darkening:** Shrouded has a straw hat and sackcloth. Sworn has a waxed black hat and an iron-shod haft. Scorched has a brim burned ragged and a blade pitted black. Hollowed has no head under the hat, only the brim floating over a lit blade edge.

### Ferryman

- **Fantasy:** the one who carries the dead across and cannot be moved.
- **Inspiration:** Charon's pole and fare; the rough seaman of early Greek vases.
- **Silhouette hook:** a hunched, heavy cowl with a rope coil over one shoulder and a vertical pole taller than the figure, with a lantern hanging from its top. The only class with a strong vertical line above head height.
- **Signature weapon:** lantern-pole (a ferryman's pole with a hooked lantern cage). Short hooking strikes; the lantern is the capture tool.
- **Palette:** verdigris and wet slate on green-black; amber lantern accent.
- **Stat lean:** Resilience.
- **Darkening:** Shrouded has wet rope and salt-stained wool. Sworn adds tarred cloth and brass fittings. Scorched fuses the rope into the body as blackened bands and cracks the lantern glass. Hollowed leaves the cowl empty with the lantern light showing through it from behind.

### Weigher

- **Fantasy:** the judge who measures a spirit before taking it.
- **Inspiration:** the weighing of the heart, and scales as a shared object across several traditions. No jackal head and no named deity.
- **Silhouette hook:** a tall, narrow two-pronged crest rising from a smooth faceless mask, and a balance beam held horizontally so the two pans hang like a second pair of shoulders.
- **Signature weapon:** balance-flail. The scale beam is swung so the pans strike in alternation; hitting marked weak points in the right order tips the scale and multiplies damage.
- **Palette:** tarnished gold on blue-black; a single cold white accent in the mask.
- **Stat lean:** Strike on weak points and Capture.
- **Darkening:** Shrouded has linen wraps and wooden pans. Sworn has gold leaf and inscribed pans. Scorched has gold flaked back to black metal and soot-stained wraps. Hollowed has one pan missing with a light in its place, so the scale is permanently tipped.

### Chooser

- **Fantasy:** the one who picks who falls, then strikes once.
- **Inspiration:** the meaning of "valkyrie" (chooser of the slain), swan garments and ravens; the crow form in Irish myth.
- **Silhouette hook:** a feather mantle on one side only, forming a single raised wing shape, with a spear held on the opposite diagonal. The outline is an asymmetric "X".
- **Signature weapon:** spear. A charged single thrust or throw; long wind-up, large hit.
- **Palette:** raven blue-black with red thread; red accent at the spear tip.
- **Stat lean:** Strike.
- **Darkening:** Shrouded has pale grey feathers and an ash spear. Sworn has black feathers and a helm with a nasal bar. Scorched has feathers burned to bare quills. Hollowed has a mantle of quills only, a helm with nothing inside, and a spear tip that glows.

### Hollow

- **Fantasy:** the rider with no head, who has already lost what you are afraid of losing.
- **Inspiration:** the Dullahan: the carried head and the spine whip.
- **Silhouette hook:** a high, empty collar with nothing above it, and a lantern-head carried low at the hip. The gap where the head should be is the recognisable part.
- **Signature weapon:** segmented chain-whip. Fast multi-hit strings; each tap extends the chain.
- **Palette:** gunmetal on red-black; ember-orange accent inside the collar and in the carried lantern-head.
- **Stat lean:** Strike tempo and Resilience.
- **Darkening:** Shrouded has a travelling coat and a wrapped bundle at the hip. Sworn has a riding cuirass and the bundle revealed as a lantern-skull. Scorched has the collar burned open and the chain glowing at the joints. Hollowed has no torso between collar and belt; the collar, arms and lantern hang around empty space.

### Mourner

- **Fantasy:** the voice that announces a death before it arrives.
- **Inspiration:** the banshee and the keening tradition; the washer at the ford.
- **Silhouette hook:** a floor-length veil that makes the whole figure a single tall triangle with no visible arms, and a handbell hanging below one edge.
- **Signature weapon:** tolling bell. Weak direct damage; each toll at range reveals weak points and lengthens the window in which a spirit can be captured.
- **Palette:** washed grey-green on grey-black; pale silver accent on the bell.
- **Stat lean:** Capture, and the group-support role in raids.
- **Darkening:** Shrouded has a grey-white veil and a wooden clapper. Sworn has a dyed veil with embroidered hems. Scorched has a soot-black veil burned away at the bottom, showing nothing underneath. Hollowed has a veil hanging from no head at all, with the bell lit from inside.

Not proposed as a class: the plague doctor. It is historically narrow (17th century), already the signature of a Darkest Dungeon hero, and common in dark games generally. It would work better as a purchasable outfit line than as a core class. (CRAFT.)

## Proposed NFT-spirit vessel archetypes

All six are original bodies that take one arbitrary 2D image as a texture. None reproduces any collection's art style. Shared rules (CRAFT):

- The image is shown unmodified on a flat or simply curved surface, so any square or near-square image works.
- Two or three dominant colours are sampled from the image and used for the vessel's glow and particles. This makes every vessel look related to its image without redrawing it.
- Each vessel has a physical weak point that is not on the image, so the art stays readable during the fight.
- Arbitrary images need a moderation and spam filter before they are shown to other players. This is a product requirement, not an art one.

| Vessel | Body | How the image is used | Weak point | Behaviour | Best for |
| --- | --- | --- | --- | --- | --- |
| **Frame Wraith** | A floating, cracked picture frame with torn cloth trailing from the bottom edge and two smoke hands gripping the sides | The image is the canvas | The four corner nails | Drifts, turns the canvas away to guard, lunges frame-first | The common, entry-level spirit |
| **Lantern Shade** | A paper lantern cylinder with a flame inside, slowly rotating | The image wraps the cylinder and is lit from within, so it glows | The flame, visible through tears that open as it takes damage | Spins faster before attacking; projects the image onto nearby surfaces | Night play; images with strong colour |
| **Standard Bearer** | A blank shrouded body carrying a tall processional pole | The image hangs as a banner at the top of the pole | The straps and rings holding the banner | Plants the pole to buff other spirits; marches toward the player | Mixed groups; a "leader" spirit |
| **Shard Swarm** | Eight to twenty mirror-like shards orbiting a dark core | The image is split across the shards and only reads as whole when they align | The core, exposed in the moment the shards align | Scatters to dodge, aligns to attack; alignment is the telegraph | Skill encounters; mid-tier bosses |
| **Scroll Eel** | A long ribbon that swims through the air, rolled at the head end | The image repeats along the ribbon like a printed strip | The unrolled segments nearest the tail | Circles the player, forcing a slow turn to track it | Encounters built around steady aiming |
| **Reliquary Stack** | A tower of cubes with mortar-like light between them | Each cube shows one image; many NFTs from one collection build one tower | The seams between cubes; breaking a seam drops a cube | Grows as more spirits join; collapses block by block | Raids. A dead collection becomes one boss |

The Reliquary Stack is the one to build first for raids (CRAFT): it turns "a collection too strong for one person" into a literal object, its height shows difficulty at a glance, and it scales with any number of contributed NFTs.

## 4. AR and location-based creature games

### How the catch or fight interaction works

| Game | Interaction | Source |
| --- | --- | --- |
| Pokémon GO (catch) | Hold the ball and a coloured ring appears on the creature; ring colour shows difficulty; the ring shrinks and the best catch chance is at its smallest; throw accuracy earns graded bonuses; a spun throw adds a curveball bonus; berries modify the attempt | EVIDENCE: [Help Center](https://niantic.helpshift.com/hc/en/6-pokemon-go/faq/102-finding-catching-wild-pokemon/) |
| Pokémon GO (AR+) | From December 2017, creatures are fixed to a point in space and can be approached. An awareness meter fills as you close in; if it fills, the creature flees. Catching up close gives an extra bonus | EVIDENCE: [Niantic, 2017](https://pokemongo.com/post/arplus/?hl=en) |
| Pokémon GO (raid) | Tap for fast attacks, trigger a charged attack when the meter fills, swipe to dodge. After a win, the boss is caught with a limited set of special balls | EVIDENCE: [Help Center](https://niantic.helpshift.com/hc/en/6-pokemon-go/faq/2187-what-are-raid-battles/) |
| Monster Hunter Now | Tap to attack, flick to evade. A red glow on the monster signals an attack. Evading at the last moment gives a brief advantage. Named weak points; parts can be broken for extra rewards; upgraded weapons add a special skill button. Hunts have a time limit; the launch post puts it at 75 seconds | EVIDENCE: [Help Center tips](https://niantic.helpshift.com/hc/en/39-monster-hunter-now/faq/3962-tips-slay-monster/), [launch post](https://monsterhunternow.com/en/news/launch) |
| The Witcher: Monster Slayer | One-on-one fights: fast or strong sword swipes build a critical meter; parry with the sword; cast signs; throw bombs. Oils and potions are prepared beforehand. Time of day changes what appears | EVIDENCE: [Wikipedia](https://en.wikipedia.org/wiki/The_Witcher:_Monster_Slayer) |
| Harry Potter: Wizards Unite | Drag the wand to draw a glyph and hold it over an enemy to cast. Three professions fill damage, balanced and support roles | EVIDENCE: [TheGamer, 2021](https://www.thegamer.com/harry-potter-wizards-unite-wizarding-challenges-tips/) |
| Jurassic World Alive | Pilot a drone to keep a reticle on a moving dinosaur and fire darts; accuracy sets the reward | EVIDENCE (search excerpts of community guides, pages not opened) |
| Peridot | Virtual pet rather than hunting. Launched 2023 | EVIDENCE: [mobilegamer.biz, 2026](https://mobilegamer.biz/niantic-is-closing-its-virtual-pet-game-peridot/) |

### What worked

- EVIDENCE: Short, one-handed sessions. Monster Hunter Now's launch pitch was 75-second hunts, one-handed play, and a Paintball item that lets a player mark a monster and fight it later ([launch post](https://monsterhunternow.com/en/news/launch)).
- EVIDENCE: A stealth approach using real movement. AR+ rewards getting close without alarming the creature ([Niantic, 2017](https://pokemongo.com/post/arplus/?hl=en)).
- EVIDENCE: Keeping the wider interaction radius. Pokémon GO doubled its interaction distance to 80 metres during the pandemic, tried to revert it, and made it permanent in August 2021 after player protest; accessibility advocates welcomed the decision ([Can I Play That, 2021](https://caniplaythat.com/2021/08/26/pokemon-go-will-keep-its-extended-80m-interaction-radius-going-forward/)).
- CRAFT: The Paintball idea maps directly to Reapers. "Mark this spirit, deal with it later" separates the walking part of the game from the fighting part, which is also the safest split.

### What failed, and the stated reasons

- EVIDENCE: **The Witcher: Monster Slayer** launched 21 July 2021 and closed 30 June 2023 ([Wikipedia](https://en.wikipedia.org/wiki/The_Witcher:_Monster_Slayer)). CD Projekt said it did not meet business expectations and tied the closure to a strategy change; the studio was folded into CD Projekt Red with layoffs ([Game Developer, 2022](https://www.gamedeveloper.com/business/layoffs-at-cd-projekt-mobile-studio-as-i-the-witcher-monster-slayer-i-shuts-down)).
- EVIDENCE: **Harry Potter: Wizards Unite** closed 31 January 2022. Niantic's statement said not all games are meant to last forever and that the two-year story arc was complete ([Engadget, 2021](https://www.engadget.com/harry-potter-wizards-unite-shut-down-january-31-2022-181008602.html)). Sensor Tower estimated about $37 million from about 20 million downloads, against roughly $1 billion for Pokémon GO in 2020 alone ([PocketGamer.biz, 2021](https://www.pocketgamer.biz/harry-potter-wizards-unite-failed-magic/)).
- CRAFT (commentary, not developer statement): The same PocketGamer.biz piece argues Wizards Unite was too close to a Pokémon GO clone and never built its own community. That is one journalist's analysis.
- EVIDENCE: **Peridot** was delisted on 14 May 2026 with servers closing 31 August 2026. Appmagic estimates put lifetime in-app revenue near $2.9 million on 1.9 million downloads ([mobilegamer.biz, 2026](https://mobilegamer.biz/niantic-is-closing-its-virtual-pet-game-peridot/)).
- CRAFT: The pattern across the three closures is that a known brand on top of the Pokémon GO template was not enough. None of the official statements blames AR itself. The lesson for Reapers is to make the core interaction different, not the theme.

### How raids coordinate players

| Game | Group size | Lobby and timing | Difficulty | Source |
| --- | --- | --- | --- | --- |
| Pokémon GO raids | Groups of up to 20 | The raid is open for 45 minutes after the egg hatches | Tiers 1, 3, 5, Mega and Mega Legendary; higher tiers need more players | EVIDENCE: [Help Center](https://niantic.helpshift.com/hc/en/6-pokemon-go/faq/2187-what-are-raid-battles/), [joining raids](https://niantic.helpshift.com/hc/en/6-pokemon-go/faq/2737-i-m-having-issues-joining-raids/) |
| Pokémon GO Max Battles | Up to 4 for standard battles; up to 40 for the largest, split into groups of four or fewer | At fixed map locations | The official post says the largest bosses take groups of 10 to 40 | EVIDENCE: [Max Battles page](https://pokemongo.com/max-pokemon-battle), [Niantic, 2024](https://pokemongo.com/en/news/go-bigger-gigantamax) |
| Monster Hunter Now group hunts | Up to four in total | Auto-start 120 seconds after recruitment begins; players tap ready | Monster star rating | EVIDENCE: [Help Center](https://niantic.helpshift.com/hc/en/39-monster-hunter-now/faq/3943-hunting-monsters/) |
| Monster Hunter Now Elder Dragon Interceptions | Up to four per group | Scheduled through the day; the event closes 25 minutes after you join; 180-minute cooldown, skippable with a paid ticket; minimum rank required | Reduced rewards for repelling without a kill | EVIDENCE: [Help Center](https://niantic.helpshift.com/hc/en/39-monster-hunter-now/faq/4653-elder-dragon-interception/) |
| Wizards Unite fortresses | Up to five per chamber | Chamber timers from 300 to 600 seconds | The level of an entry item (a runestone) raises difficulty. A community guide adds that player count also feeds the difficulty rating (search excerpt only) | EVIDENCE: [TheGamer, 2021](https://www.thegamer.com/harry-potter-wizards-unite-wizarding-challenges-tips/) |
| Jurassic World Alive raids | Friends and alliance members can be invited | Bosses stay on the map for 24 hours and move at dawn local time; you must be in range to create a lobby | Boss rarity; better rewards for the first win each week | EVIDENCE: [Ludia Help Center](https://ludia.helpshift.com/hc/en/21-jurassic-world-alive/faq/2169-what-are-raids/) |

### Players who cannot travel

- EVIDENCE: Pokémon GO's Remote Raid Pass lets a player join any raid visible on their nearby list or map. As of the current help page, up to five remote players can join one lobby and a player can do 10 remote raids a day ([Help Center](https://niantic.helpshift.com/hc/en/6-pokemon-go/faq/2737-i-m-having-issues-joining-raids/)).
- EVIDENCE: In April 2023 Niantic raised the price of remote passes, capped remote raids at five a day, and said remote raiding had come to dominate the game in a way it never intended. It added in-person rewards at the same time ([Niantic, 2023](https://pokemongo.com/post/remote-raid-passes-update-2023?hl=en)). The headline of TechRadar's coverage raised ableism concerns ([TechRadar](https://www.techradar.com/news/pokemon-go-raises-the-price-of-remote-raid-passes-inviting-concerns-of-ableism), headline only).
- EVIDENCE: In May 2025 the daily cap went back up to 10 and remote access was extended to more battle types ([Pokémon GO, 2025](https://pokemongo.com/en/news/shadow-raid-max-battle-update)).
- EVIDENCE: Monster Hunter Now added Dimensional Links in July 2024, which widen matchmaking for certain monsters to the whole world, and Friend Link in March 2026, which lets a player invite distant friends into hunts with a daily limit and a token cost ([Dimensional Links](https://monsterhunternow.com/news/dimensionallinks), [Friend Link](https://monsterhunternow.com/news/friendlink-mar2026)).
- EVIDENCE: Wizards Unite added the Knight Bus in April 2020 so players could reach group challenges from anywhere ([Wikipedia](https://en.wikipedia.org/wiki/Harry_Potter:_Wizards_Unite)). Jurassic World Alive states that invited players do not always need to be nearby ([Ludia Help Center](https://ludia.helpshift.com/hc/en/21-jurassic-world-alive/faq/2169-what-are-raids/)).
- CRAFT: Every one of these was added after launch. Remote access is not optional for this genre; the only choice is whether it is designed or retrofitted.

### Session length

- EVIDENCE: 75 seconds per hunt in Monster Hunter Now; 120-second lobbies; 300 to 600 seconds for a Wizards Unite chamber; a 25-minute window for Monster Hunter Now's large events (sources above).
- EVIDENCE: Apple recommends keeping AR game levels short with brief downtime between them; Google recommends avoiding long AR sessions and building in stopping points ([Apple HIG](https://developer.apple.com/design/human-interface-guidelines/augmented-reality), [ARCore](https://developers.google.com/ar/design/user/safety-comfort)).
- I found no published figure for average Pokémon GO session length.

### Ingress

- EVIDENCE (community documentation, search excerpt): Ingress uses a "speed lock": after an action, further actions fail if the player has moved faster than roughly 60 km/h, and the lock expires after 15 minutes. It is described as an anti-spoofing measure that also affects players in vehicles ([Fev Games guide](https://fevgames.net/ingress/ingress-guide/concepts/speedlock/)). I did not find the official statement of these numbers.

## Raid design recommendations

All CRAFT unless a source is given. Evidence for the underlying numbers is in section 4.

1. **Lobby: 120 seconds with ready-up.** This matches Monster Hunter Now's documented auto-start. Start early when everyone is ready.
2. **Squads of four inside a larger raid.** Follow the Max Battle structure: many players in one raid, but each player sees and coordinates with three others. Four is also what Monster Hunter Now uses throughout.
3. **Roles come from class.** With six classes, a squad of four should want variety: a Mourner extends capture windows, a Ferryman absorbs attacks, a Chooser or Harvester breaks parts, a Weigher finishes. Because classes are random, matchmaking should prefer mixed squads, and no raid should be impossible without a specific class.
4. **Scale the boss to the number present.** Wizards Unite reportedly set difficulty from the entry item plus the player count (the player-count part is from a search excerpt). Reapers should scale boss strength to the number of players in the raid so that a rural group of three can finish what a city group of thirty can.
5. **Three difficulty tiers at most to start.** Pokémon GO's readable tiers are effectively solo, small group, large group. Show the tier as the height of the Reliquary Stack.
6. **Raid battle length: three to five minutes.** Long enough for phases, short enough to hold a phone up. Include at least one enforced lull (for example the stack rebuilding) so arms can drop, following Apple's guidance on downtime.
7. **A raid window, not an instant.** Give each raid site a window of 30 to 60 minutes, as Pokémon GO (45 minutes) and Monster Hunter Now (25 minutes after joining) do, so people can arrive.
8. **Remote slots from day one, with a fixed share.** Reserve a set fraction of each raid for remote players and state the daily limit at launch. Do not rely on being able to tighten it later; Niantic's 2023 tightening is the cautionary case.
9. **Reward presence with something other than raw power.** In 2023 Niantic added in-person rewards alongside the remote restrictions. For Reapers, give in-person players the AR view of the full-scale boss, first choice of which cube to break, and cosmetic marks, while keeping the core reward the same for remote players.
10. **Global matchmaking for anything that needs a group.** Monster Hunter Now's Dimensional Links exist because many players never have three others nearby. A raid no one can fill is a dead feature.
11. **Players stand still during a raid.** No mechanic should ask raiders to walk, circle the boss, or back away. Turning on the spot is the maximum.
12. **Choose sites conservatively.** Niantic's 2019 settlement over trespass and nuisance complaints included removal requests for locations within 40 metres of a home and an in-app reminder when large groups gather ([Engadget, 2019](https://www.engadget.com/2019-02-15-niantic-pokemon-go-trespassing-lawsuit-settlement.html)). Raid sites should be public places with space to stand, away from housing and roads, with a removal request process from the start.
13. **A cooldown, not a paywall, on repeat raids.** Monster Hunter Now uses a 180-minute cooldown with a paid skip. A cooldown alone protects session length and avoids the pay-to-repeat criticism.

## AR safety and comfort

### Published guidance

- EVIDENCE: **Apple.** People using AR may not be aware of their surroundings, so rapid, sweeping or expansive motion can be dangerous; a game can avoid encouraging large or sudden movements. Holding a device at one distance or angle for a long time is tiring; keep levels short and mix in downtime. If the app encourages movement, introduce it gradually ([Apple HIG, Augmented reality](https://developer.apple.com/design/human-interface-guidelines/augmented-reality)).
- EVIDENCE: **Google.** Users absorbed in the camera view can bump into objects and miss hazards. Do not make users walk backward. Avoid long sessions and find natural stopping points. Let users pause and resume, including from a different location ([ARCore design guidelines, updated 2024-10-31](https://developers.google.com/ar/design/user/safety-comfort)).
- EVIDENCE: **Niantic.** Its safety guidance says the app shows an in-app message reminding players to stay aware of their surroundings, that the apps should not be used while driving and that this is reinforced by warnings and behaviour in the app, and that players should stay out of inappropriate areas ([Safety FAQs](https://niantic.helpshift.com/hc/en/6-pokemon-go/faq/114-safety-faqs/)).
- EVIDENCE: **Touch targets.** Apple's guidance is a hit region of at least 44 by 44 points for a control ([Apple HIG, Buttons](https://developer.apple.com/design/human-interface-guidelines/buttons)).
- EVIDENCE: **Flashing.** The W3C accessibility standard limits flashing to no more than three times in any one-second period ([WCAG 2.3.1](https://www.w3.org/WAI/WCAG21/Understanding/three-flashes-or-below-threshold.html)). It is a web standard, used here as the nearest published threshold.

### Research on harm

- EVIDENCE: A JAMA Internal Medicine study of tweets over ten days in July 2016 estimated that about a third of sampled tweets mentioning the game and driving indicated a driver, passenger or pedestrian was distracted by it (18% a person playing while driving), and found 14 crashes attributed to the game in news reports. The authors note that play was already restricted above 10 miles per hour and suggest making the game unavailable for a period after driving speed is detected ([Ayers et al., 2016](https://jamanetwork.com/journals/jamainternalmedicine/fullarticle/2553331)).
- EVIDENCE (search excerpt of the abstract, page blocked): A Purdue working paper using police reports from one Indiana county estimated a disproportionate rise in crashes near in-game locations in the 148 days after launch ([Faccio and McConnell, SSRN](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=3189603)). It is a working paper, not peer-reviewed as far as I could confirm.

### Recommendations for Reapers

All CRAFT, derived from the guidance above.

1. **Two modes that never overlap.** A map mode for moving (camera off, audio and haptic cues for nearby spirits) and an encounter mode for fighting (camera on, player stationary). Opening the camera view should require the phone to be roughly still.
2. **Speed gate encounters.** Block encounter start above walking pace, and add a short lockout after vehicle speed is detected, as the JAMA authors suggest. Pick and test the exact thresholds; I have no verified industry number to copy.
3. **Spirits come to the player.** Never place a spirit so the player must walk toward it while looking at the screen, and never behind them. Keep encounter action inside a forward arc the player can cover by turning their upper body.
4. **No backward steps, no dodging by moving the body.** Dodge with a flick or by lowering the phone, not by stepping.
5. **Reward slow movement.** A stalking meter that punishes fast phone motion (the AR+ awareness idea) turns the safest behaviour into the skilled behaviour.
6. **Sixty to ninety seconds per solo encounter**, with a clear end and a pause between encounters. Three to five minutes for raids with an enforced lull.
7. **One-handed, chest height.** Let the player calibrate a comfortable holding angle so the phone does not have to be held at eye level. Weak points must be at least 44 points across on screen at the intended distance.
8. **Full parity with AR off.** A non-camera encounter mode with the same rewards, for accessibility, battery, privacy and places where raising a camera is inappropriate.
9. **Awareness prompts on mode change**, not just at launch: when the camera opens, when a group of players gathers, and when it is late at night.
10. **The theme invites night play, so do not pay for it.** Do not give better rewards for being out in the dark or in isolated places. If time of day changes which spirits appear, make it cosmetic or mirror it with a daytime equivalent.
11. **Limit flashes.** Hit flashes and strobing auras must stay under three per second. Prefer hit-stop, sound and haptics over full-screen flashes.
12. **Do not shake the camera feed.** Screen shake on live passthrough moves the real world under the player's eyes. Shake the spirit and the interface instead.
13. **Bystander privacy.** Process camera frames on the device and do not upload or store them by default.
14. **Location hygiene.** No spawns on roads, rail, water, private housing or other excluded categories; a visible way for property owners to request removal.

## 5. Encounter feel for tap-to-attack combat

### What the sources say

- EVIDENCE: Steve Swink defines game feel as real-time control of virtual objects in a simulated space, with interactions emphasised by polish ([Wikipedia, Game feel](https://en.wikipedia.org/wiki/Game_feel)).
- EVIDENCE: A survey of over 200 sources groups game-feel design into three areas: tuning physicality, amplifying feedback ("juice"), and streamlining so the game acts on what the player meant to do ([Pichlmair and Johansen, 2020](https://arxiv.org/abs/2011.09201)).
- EVIDENCE: A 2022 conference study that ranked action games by player comments about impact concluded that hit stop, coherent sound and camera control are the features most strongly tied to how impactful hits feel ([Lin et al., IEEE GEM 2022](https://arxiv.org/abs/2208.06155)).
- EVIDENCE: A Naughty Dog designer analysing God of War (2018) describes hit stop as snapping the target to its hit pose and holding attacker and target for a moment, stresses how much impact sound contributes, and notes that exaggerated enemy reactions sell the player's power ([PlayStation Blog, 2022](https://blog.playstation.com/2022/10/04/game-developers-explain-what-makes-god-of-war-2018s-combat-tick/)).
- EVIDENCE: Mike Stout's widely cited article on telegraphing argues that an enemy needs a short delay and a visible wind-up before attacking so the player knows what is coming and what to do, which is what makes a hit feel fair ([Game Developer, 2015](https://www.gamedeveloper.com/design/enemy-attacks-and-telegraphing)).
- EVIDENCE: Monster Hunter Now implements exactly that on a phone: a red glow before an attack, a flick to evade, a reward for evading at the last moment, named weak points, and breakable parts with their own rewards ([Help Center](https://niantic.helpshift.com/hc/en/39-monster-hunter-now/faq/3962-tips-slay-monster/)).
- EVIDENCE: Infinity Blade built touch combat from swipes to attack and parry, taps to dodge and block, attacks that must be dodged rather than blocked, and a short vulnerable window on the enemy after a correct counter ([Wikipedia, Infinity Blade](https://en.wikipedia.org/wiki/Infinity_Blade)).
- EVIDENCE (search excerpt, page not opened): Sekiro's posture system lets attacks and well-timed deflections fill an enemy's posture meter until it breaks and a finishing blow is available; Miyazaki linked it to the aim of more active, ninja-like combat ([Red Bull interview](https://www.redbull.com/us-en/sekiro-shadows-die-twice-interview)).
- EVIDENCE: Two practitioner talks are the standard references for feedback layering: "Juice it or lose it" (Jonasson and Purho, GDC Europe 2012; [summary](https://www.richtaur.com/GameDevTreasure/post/juice-it-or-lose-it/)) and "The art of screenshake" (Jan Willem Nijman, 2013; [video](https://www.youtube.com/watch?v=AJdEqssNZ-U)). Both are demonstrations by developers, not studies.

### Proposed encounter structure for Reapers

All CRAFT.

1. **Stalk.** Bring the spirit into view with slow, steady phone movement. A steadiness meter fills. Jerky movement alerts the spirit and it starts the fight with an advantage.
2. **Strike.** Tap weak points as they open. Every tap gives a response within the same frame: a brief hit stop on the spirit, a sound that matches the weapon, a haptic tick, and a visible reaction from the vessel. The tap should never feel unanswered, even on a miss.
3. **Read.** The spirit telegraphs with one consistent signal across all vessels (a colour shift plus a sound), followed by a fixed delay. The player answers with a flick or by lowering the phone. A late, accurate answer opens extra weak points.
4. **Break.** Strikes on weak points fill a break meter. When it fills, the vessel cracks open and stops attacking. This is the stagger window.
5. **Capture.** During the window, a timing input closes the capture, in the same spirit as Pokémon GO's shrinking ring. Capture strength (class stat) sets how long the window lasts.
6. **Absorb.** A short, calm ending animation. This is the downtime Apple asks for.

Further heuristics, all CRAFT:

- Weak points should be few and large. Two or three at once, each at least 44 points across.
- Combo windows should be about rhythm, not speed. A steady beat rewards control; maximum tap rate rewards nothing and tires the hand.
- Each class should change one step. Harvester sweeps to hit several points; Chooser charges one large strike; Hollow chains taps; Weigher wants points hit in order; Mourner widens the capture window; Ferryman can hold through a telegraphed attack instead of dodging.
- In AR the camera belongs to the player, so camera control, one of the three features the 2022 study highlights, cannot be used. Sound and hit stop have to do more work.
- One counterpoint worth reading: a Game Developer piece argues against piling on feedback for its own sake ([video write-up](https://www.gamedeveloper.com/design/video-indies-resist-the-urge-to-juice-it-or-lose-it-)). Restraint also fits the art direction.

## 6. Lessons from NFT and crypto games

Kept short and factual. Nothing here is financial advice.

- EVIDENCE: **Axie Infinity.** Daily active users rose from about 38,000 in April 2021 to a peak near 2.5 million at the end of 2021. The reward token fell from about $0.36 in May 2021 to under $0.01 in February 2022. Sky Mavis said the inflation was not sustainable, removed token rewards from single-player play and cut others on 4 February 2022. On 23 March 2022 about $620 million was stolen from its bridge ([Fortune, 2022](https://fortune.com/2022/03/31/axie-infinity-hack-coin-price-crypto-crisis/)).
- EVIDENCE (search excerpt, page not opened): Before the cut, the token was being produced at about four times the rate it was burned ([BitPinas](https://bitpinas.com/play-to-earn/axie-infinity-adventure-daily-quest-slp-rewards-zero/)).
- EVIDENCE: Axie added "Releasing" in January 2022: players permanently give up an Axie in exchange for cosmetic land items and raffle entries. Sky Mavis described it as a lever for economic balance ([Axie Infinity blog](https://blog.axieinfinity.com/p/lny2022)). This is the closest precedent to absorbing a spirit: a permanent burn that pays out cosmetics, not currency.
- EVIDENCE: **STEPN.** On 26 May 2022 it announced it would stop GPS service for users in mainland China from 15 July; its governance token fell 38% within a day and the floor price of its sneaker NFTs dropped ([CoinDesk, 2022](https://www.coindesk.com/business/2022/05/26/stepn-to-bar-move-to-earn-for-china-based-users-in-july)). I did not verify later price history.
- EVIDENCE: **Star Atlas.** Its developer had roughly $15 million stuck on FTX when the exchange collapsed, which halved its runway; staff fell from a peak of 235 to about 100. In March 2025 the CEO said its browser game had about 1,500 daily active users ([The Block, 2025](https://www.theblock.co/post/345961/solana-based-mmo-video-game-star-atlas-aiming-to-launch-this-year-survive-shaky-market)).
- EVIDENCE: **Aurory.** In December 2023 an attacker exploited a race condition in its off-chain marketplace and withdrew about 600,000 tokens (around $830,000) from a team wallet; player assets were not lost ([Helius](https://www.helius.dev/blog/solana-hacks)).
- EVIDENCE: **DeGods and y00ts.** Both left Solana in 2023 (DeGods to Ethereum, y00ts to Polygon with a $3 million grant). In August 2023 y00ts announced a second move to Ethereum and the return of the grant ([Decrypt, 2023](https://decrypt.co/151959/degods-ditch-polygon-moving-y00ts-ethereum-returning-3-million-grant)). A collection's chain and community can move out from under a game that depends on it.
- EVIDENCE: **Burning on Solana returns rent.** Closing a standard token account returns its rent deposit, on the order of 0.002 SOL, to the owner ([Sol Incinerator](https://sol-incinerator.com/)). A third-party article claims about 0.01 SOL for a full NFT with metadata and nothing for compressed NFTs ([dev.to](https://dev.to/getblockapi/sol-incinerator-burning-unwanted-solana-nfts-56ib), search excerpt only).

What this suggests for Reapers (CRAFT):

- The collapses share one cause: rewards with cash value were issued faster than anything removed them. Reapers consumes NFTs and should issue nothing tradeable in return. Levels, outfits and cosmetics only.
- The choices credited with holding up are sinks (Axie's cuts and Releasing) and not keeping treasury on an exchange or in a hot team wallet (Star Atlas, Aurory). I did not find a rigorous study showing any crypto game economy that stayed stable; "held up" here means "was the fix the developers themselves reached for".
- Absorbing is irreversible. It needs a clear confirmation, and the rent refund is a small, honest benefit to state plainly.
- Do not depend on any one collection. Dead collections are the input, so the game needs a rule for which collections count that does not rely on a marketplace or a floor price.
- Wallets full of dead NFTs are also full of spam and scam NFTs. Filter by verified collection before anything is shown or absorbed.
- Showing someone's NFT image to other players raises image-rights and moderation questions that vary by collection. I did not research the legal position.

## Not verified

- **Design Works wording.** The Dark Souls Design Works quotes and paraphrases come from a fan translation hosted on Souls Lore, not the official English edition. Wording may differ.
- **Miyazaki's later comment on the undead dragon** (PC Gamer). I saw the headline only; the article body did not load. Not used.
- **Bloodborne hunter attire.** No primary source found on how the costume was designed. A claim that the hunter is slender to emphasise the weapons appeared only in a low-quality search result and is not used.
- **Hades.** I found no direct statement from Jen Zee about silhouette or colour rules; only the ink-style and process comments cited.
- **Diablo IV art pillars.** Taken from PCGamesN's report on leaked art-book pages. Two interviews that reportedly discuss them (TechRadar, GamesHub) would not load.
- **Diablo IV class-locked transmogs and two body types per class.** Search excerpts only.
- **TF2 paper.** I confirmed the silhouette and rim-light statements through the TF2 wiki transcript and a search excerpt; I could not parse the PDF directly.
- **Britannica.** All pages returned 403. The psychopomp table rests on Wikipedia and the sources it cites. The Baron Samedi article carries a citation warning. The hourglass as a Grim Reaper attribute is common knowledge but was not in the page I read, so it is not in the table.
- **Modern devotion to the Morrígan, valkyries or Anubis.** Not researched.
- **Pokémon GO lobby countdown.** A 120-second default appears on community wikis; I could not confirm it on a Niantic page. The 120-second figure used in this document is Monster Hunter Now's, which is official.
- **Remote Raid Pass launch date and original price** (April 2020). Search excerpts of news coverage only.
- **Share of Pokémon GO players who turn AR off.** No figure found.
- **Pokémon GO session length.** No published figure found.
- **Witcher: Monster Slayer AR mode.** Search excerpts of two guides say AR was mainly useful for screenshots and that turning it off saved time and battery; I could not open either page, so it is not used as a finding.
- **Jurassic World Alive.** The darting description and a raid team size of four come from search excerpts; the official forum would not resolve.
- **Ingress speed lock numbers.** Community documentation only.
- **Wizards Unite difficulty scaling by player count.** Search excerpt of a community guide only.
- **Niantic's sale of its games business to Scopely** (announced March 2025). Seen in search excerpts; not opened, and not relied on.
- **Faccio and McConnell crash study.** Abstract seen as a search excerpt; SSRN blocked access. Peer-review status not confirmed.
- **Sekiro posture interview.** Page did not load; description is from a search excerpt.
- **Axie production-to-burn ratio and the 56% emission cut.** Search excerpts only.
- **STEPN after May 2022.** Not verified.
- **Gods Unchained Forge** as a burn-to-upgrade sink. The official page returned 403; left out of the findings.
- **SOL returned for a full NFT burn and for compressed NFTs.** Third-party claim, search excerpt only.
- **Speed thresholds for AR games.** I found no official current figure from Niantic. The 10 mph figure is what the JAMA authors reported in 2016.
