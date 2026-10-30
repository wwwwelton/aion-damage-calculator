# AION 2 Damage Calculator

Windows x64 damage calculator with an embedded WebView2 interface, hosted by Rust (tao + wry). Calculations and the mob/boss catalog work offline. Character search uses the official AION 2 site; optional AI advice uses the [Google Gemini API](https://ai.google.dev/gemini-api/docs/openai) and may incur usage charges.

## Get started

### Run the Windows app

Requirements: Windows x64 and the [Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/). Download `AION2-Damage-Calculator.exe` from [GitHub Releases](https://github.com/wwwwelton/aion-damage-calculator/releases) and verify its SHA-256. Local builds are kept in `dist/v<version>/`. Windows-script and CI builds carry a fresh self-signed Authenticode signature with a SHA-256 RFC3161 timestamp; this verifies signed-file integrity, not a trusted publisher identity. Windows/SmartScreen warnings can remain.

### Build from source

Install [Rust](https://rustup.rs/) (stable, `x86_64-pc-windows-msvc`) on Windows. The executable embeds the WebAssembly interface from `ui/dist`, so first run `bash scripts/build-ui.sh` (needs the `wasm32-unknown-unknown` target and `wasm-bindgen-cli` 0.2.130). Then, from the repository root, run:

```powershell
cargo build --release -j 1
```

To build the interface in WSL, copy the WSL project (`/home/avatar/gui-dps-aion2`) to `C:\dev\gui-dps-aion2`, build it with one Cargo job, and keep each version's executable in `dist/v<version>/`, run `pwsh -NoProfile -File C:\dev\gui-dps-aion2\build-windows.ps1` on Windows. The script preserves downloaded model files and existing Windows builds; it does not remove files deleted from the WSL source.

The Windows script and CI share `scripts/sign-windows.ps1`; install the Windows SDK Signing Tools. Each signing run creates a fresh non-exportable code-signing key in the current user's personal certificate store, signs with SHA-256 and a DigiCert RFC3161 timestamp, checks the signature and timestamp, exports only `AION2-Damage-Calculator.crt` beside the executable, then removes the certificate and private key. No PFX or repository private key is used, and no trusted-root/publisher store is modified. Signature verification accepts only success or Windows' specific untrusted-self-signed-root result; missing timestamps and integrity errors fail the build. `-WslSource /path/to/clean/worktree/` selects a different WSL checkout for `build-windows.ps1`. A direct `cargo build` or Linux cross-build remains unsigned; to sign an existing Windows binary, run:

```powershell
pwsh -NoProfile -File scripts/sign-windows.ps1 -BinaryPath target/release/AION2-Damage-Calculator.exe -CertificatePath dist/AION2-Damage-Calculator.crt
```

From WSL, `bash build.sh` builds the interface, cross-compiles with MinGW (`x86_64-w64-mingw32-gcc`), then generates a fresh certificate and signs using Windows PowerShell and the Windows SDK. The executable and public certificate are written to `target/x86_64-pc-windows-gnu/release/`. Plain Linux without Windows interop cannot run the signing step.

No model download or AI bundling step is needed. `src/assets.rs` lists the embedded UI assets; `packaging.test.js` checks that the built page's stylesheet and WebAssembly files are included. The window compiles only on Windows; host unit tests run on any platform. `AION2_SELFTEST=<file>` runs the asset, official-profile and guide smoke check without opening a window.

### Continuous integration and releases

`.github/workflows/build.yml` runs JavaScript and Rust tests on Ubuntu, then builds and signs the Windows executable and publishes it with its public CRT and a checksum computed after signing. A `v<version>` tag matching `Cargo.toml` additionally creates a GitHub Release under that same v-prefixed tag. Each tag keeps previous releases available.

### Development and checks

Run the fast checks with Bun or Node.js, then build on Windows:

```bash
bash scripts/build-ui.sh
bun run test
cargo test
cargo build --release -j 1
```

The UI can be previewed over HTTP. Character lookup, guide retrieval, API key storage and AI requests require the Windows host bridge. Do not put a key in source control.

### Project layout

| Path | Purpose |
| --- | --- |
| `Cargo.toml`, `src/main.rs`, `src/window.rs`, `src/assets.rs` | Rust Windows host: tao window, wry WebView2 surface, embedded files served from `https://aion2.localhost` |
| `src/official.rs`, `src/guides.rs`, `src/protocol.rs` | Public character lookup from the official site, opt-in wiki-guide retrieval and page↔host message routing |
| `host-bridge.js` | Injected by the host; the only API the pages use to reach it (`aion2Host.post` / `aion2Host.onMessage`) |
| `index.html`, `style.css`, `i18n.js`, `renderer.js`, `profile-ui.js`, `ai-tips.js` | Bilingual interface, character dossier and contextual assistant |
| `attribute-trees.js`, `stat-tooltips.js` | Attribute tree nodes with imported sheet values and bilingual stat explanations |
| `calculator.js`, `matchups.js`, `recommendations.js`, `numeric-input.js` | Damage calculation, matchup rules, tips and number parsing |
| `characters.js`, `bosses.js`, `boss-data.js`, `mob-data.js`, `scrape-boss-details.mjs` | Profile imports, offline NPC catalog and the boss-detail refresh script |
| `*.test.js` | Node.js regression tests |
| `src/gemini.rs` | Native Google Gemini client (OpenAI-compatible endpoint) |

## Calculation scope

Multipliers are entered as percentages: 100% = ×1, 150% = ×1.5. The additive Damage Boost bucket starts at 100%; PvE, Boss and Species bonuses start at zero. Critical Hit, Double Chance procs and attack position apply only when selected. Presets are illustrative, not official values. Pure Attack is a derived formula term. This community-derived model follows the tested attack → skill → Defense → boost → critical/Double/position order, caps effective Defense and each damage stage at zero, uses Defense ×0.1 in PvE and ×0.01 in PvP, uses a ×2 Double proc in PvE versus ×1.5 in PvP, and changes the base critical multiplier from ×1.5 in PvE to ×1.25 in PvP. Internal game rounding is not modeled.

The header language selector switches the interface between Portuguese and English and persists the choice locally. Static labels, dynamic calculation messages, profile views, matchup sections, stat tips and future assistant replies follow the selected language; game-provided character, item, skill and NPC names remain as published by their source.
The **Attribute Trees** tab shows two four-level binary maps of circular icon nodes with eight leaves each. Damage: base attack (Pure Attack, Attack Increase), landing hits (Penetration, Accuracy), damage boosts (Damage Boost, Weapon Damage Boost) and critical hits (Critical Hit, Critical Damage Boost). Defense: damage reduction (Defense, Damage Tolerance), critical defense (Critical Hit Resist, Critical Damage Tolerance), avoiding hits (Evasion, Block) and health (Player HP, Endurance). Every node carries a one-line hint of what it does; offensive leaf values follow the calculator, while defensive leaves show the imported character’s own sheet after **Load attributes into calculator** in Character Search (not the enemy’s defense) and read “Not available” otherwise. Calculator defaults are examples. The imported sheet list explains each primary stat and divinity, and a **Sheet modifiers (display only)** block lists the summed percent increases that are not applied to calculator fields. Below the trees, **All combat attributes** lists every one of the 120 attributes the calculator models (hit/evasion/block, attack, skills, boosts and tolerances, criticals, procs, positional, elemental, the four creature genera and HP/cadence) with its current calculator value; enemy-side attributes are prefixed “Enemy”. Hovering or focusing any attribute button shows a bilingual tooltip with the full explanation; clicking pins it, and Escape or clicking elsewhere closes it. The trees shrink to the window width without horizontal scrolling and switch to an indented vertical layout below 600 px. The trees are category maps, not a universal stat priority; use the calculator's upgrade comparisons for your current target.

The **Attribute Trees** tab also includes five static BIS Genus Board tables with bundled Aion2Data creature symbols. The community reference lists recommended attributes by slot, preserves the Crit/Accuracy cap note, and does not change calculator values.

## Icons

The eight class emblems are bundled locally from [MetaBot's AION 2 class icon gallery](https://metabot.gg/en/aion-2/classes), which credits the game client. Known class names show the emblem beside characters in the encounter header and public profiles. Public character dossiers display remote item and skill artwork when available; offline, their generic category icons remain. Bosses show their aion2data.com portrait when online and a generic boss symbol otherwise; mobs use the generic mob symbol. Text labels remain visible when an icon is unavailable.

## PvE and PvP scenarios

At the top of **Damage Calculator**, choose **PvE · Boss**, **PvE · Other monster** or **PvP · Player**. The header has editable attacker and target names/classes or creature type. Importing a character, enemy player, boss or mob fills its header; opponent imports select the corresponding mode. Switching modes preserves offensive values but clears the previous opponent's identity and defenses. Boss Attack Power and Boss Damage Boost/Tolerance apply only to bosses; PvE and Species terms do not apply to players. Creature-type matchups replace generic Species Boost/Tolerance, not add to them. PvP-specific Attack/Boost/Tolerance and elemental modifiers remain reference-only: their exact placement and stacking are not established by the sources; the PvP result is **not** a complete damage prediction. Unknown opponent defenses stay manual.

Numeric fields accept either `50,0` or `50.0` for fifty, including decimal values in all percentage and flat-stat fields. On leaving a field, the app removes redundant zeroes while retaining the entered decimal separator. Empty, incomplete or invalid entries are treated as zero during calculation and automatically replaced with `0` when the field loses focus. Grouping separators and scientific notation are not accepted.

## Estimated DPS and health

The result card is labeled **Damage of one landed hit · deterministic**: given the entered values, it is the formula's exact output. The estimate card is labeled **Combat estimate · probabilistic assumptions** and lists, below the numbers, what the current toggles take for granted: that a selected Critical Hit lands, that a selected Double proc triggers, positional uptime, a constant landed-hit rate, unmodeled resistances and in-game rounding. **Estimated DPS = estimated hit damage × landed hits per second**. Enter an observed or assumed positive hit rate; zero is unknown, so no DPS is shown. **Time to defeat = target HP ÷ estimated DPS** only when both are positive. These are constant-hit what-if values, not a measured skill rotation, expected proc average, or a complete PvP simulation; misses, healing, downtime and encounter mechanics are excluded. The **Player HP** field shows the attacker's HP for context and does not change outgoing damage. The target field is labeled **Boss HP**, **Mob HP** or **Enemy player HP** according to the selected opponent; zero means unknown and target changes clear it. Public player profiles import only an explicit positive character-sheet **HP** stat (no equipment inference). [Aion2Data explains](https://aion2data.com/monsters/2200208-basement-zirka) that NPC HP and combat stats are held by the server, not the client database, so mob and boss HP must be entered manually when known. [Wikily describes HP as a character stat](https://wikily.gg/aion-2/stats/basic).

## Enemy defenses

Red enemy fields start at zero and pair with related offensive stats. Effective Defense is `max(0, Defense − Penetration)`; subtract 10% of it in PvE or 1% in PvP. Weapon Damage Tolerance subtracts from Weapon Damage Boost; Damage, PvE, Boss and Species Tolerances subtract from the additive boost bucket when relevant. Critical Damage Tolerance subtracts from the critical multiplier on critical hits; Front/Back Attack Damage Tolerance subtracts from the selected positional multiplier. Enter only defenses applicable to this target. Tolerance stacking and damage floors outside Defense remain community estimates and may yield negative values.

The reference-only section collects Evasion versus Accuracy, Block versus Block Penetration, Critical Hit Resist versus Critical Hit, Double/Perfect/Multi-hit Resist versus their chance stats, Endurance versus Endurance Penetration, elemental stats and flat Critical Damage/Front/Back Defense. These do not affect a selected hit: the model has no verified point-to-chance conversion, gear-grade-dependent block reduction, elemental skill designation or flat critical/position attack conversion. Back attacks bypass Block; Endurance halves damage when it procs. Do not treat chance resistances as percentages of guaranteed damage reduction.

Stat names and interactions: [Wikily — Basic](https://wikily.gg/aion-2/stats/basic), [Attack and defense](https://wikily.gg/aion-2/stats/attack-and-defense), [PvE and boss](https://wikily.gg/aion-2/stats/pve-and-boss), [PvP](https://wikily.gg/aion-2/stats/pvp), [Chances and resists](https://wikily.gg/aion-2/stats/chances-and-resists), [Creature types](https://wikily.gg/aion-2/stats/creature-types), [Aion 2 Maps — Stats explained](https://aion2maps.com/guides/stats-explained/) and [How damage works](https://aion2maps.com/guides/how-damage-works/). These sources describe interactions but do not establish exact enemy values for every target.

English stat terminology follows the [AION 2 damage formula guide](https://gameplay.tips/guides/aion-2-how-stats-work-damage-formula-guide.html). It is a community source, not official English localization; labels unavailable from an official English client are not certified as exact in-game UI strings.

## Character Search

Open **Character Search**, enter an exact character name, choose a Global region, and press **Search**. Korean and Taiwanese servers are not available in this lookup. If multiple characters share that name, choose the correct server/class in the dropdown. The profile displays public attributes, equipped gear options and learned skills. **Load available stats into calculator** copies explicit numeric bonuses from equipped items and fills offensive matchup fields from the character sheet or explicit **partial** equipment options. Gear options are matched by their stable API id (for example `AmplifyAllDamage`, `DefensePierce`), with the English name as a fallback, so renamed or localized labels still import. The official sheet publishes only primary stats (Might, Dexterity, Constitution, Intelligence, Precision, Willpower), the ten divinities and item level, each with its published secondary effects. From those, the language-independent stat types provide Might **and Destruction [Zikel]** → Attack Increase (1 point = 0.1%), Justice [Nezekan] → Perfect Chance, Wisdom → Double Chance, Illusion → Endurance Penetration, Death → Regeneration Penetration, and the defensive Time → Double Resist, Destruction → Perfect Resist, Destiny → Endurance and Life → Regeneration. Percent-of-base effects (Accuracy, Critical Hit, Evasion, Block, Defense and HP increases from Precision, Dexterity, Constitution, Freedom, Justice, Death, Life and Space) are shown as **sheet modifiers** but never converted into flat values, because the base totals they multiply are not public. Primary stats and divinities printed on equipment are already included in the sheet totals and are marked **already counted in the character sheet** instead of being summed again. Explicit Penetration and Skill Specialty Enhancement options are imported when present. After import, the status reports how many fields came from the character sheet and how many are equipment-only partials. Pure Attack, skill coefficients, rotation cadence, target Defense and other unavailable totals remain manual and are listed after import instead of being inferred. Gear-only bonuses do not represent the complete character sheet. Public profiles may be delayed or incomplete.

The character dossier has **Equipment**, **Skills**, **Attributes** and **Progression** views. The summary shows level, race, gender, server, Combat Power, item level, the equipped title and how many titles the character owns. Equipment shows published base/bonus and Soul Bind options, stones and attached skills, with percentage values styled apart from flat values; **Attributes** groups the sheet into primary stats, divinities and other values and tags every published effect as **feeds the calculator** or **display only**; Skills groups learned active, passive and Stigma abilities with their levels; Progression shows published titles (equipped effects plus the bonus granted by owning the title), Daevanion boards, Arcana, pet with its level, and wings with grade and enchant level. The same views are available for the selected enemy player. Portrait and item/skill images require internet access; hidden or absent profile details are not inferred.

The profile summary reads the legion from the official character profile and shows overall, class and server Combat Power ranks when available. Rankings are an optional Atool community lookup; unavailable rankings do not block the official profile and may be stale or absent.

Equipped title names and published equip bonuses appear in the profile header. Wings and mounts appear with equipment. Skills are separated into equipped, acquired and not acquired, with their published levels. The response does not expose selected specialties, level-scaled descriptions, cooldowns or wing stat effects; these are not inferred.

## Enemy character search

Open **Enemy Character**, enter a name and region, then select the correct character if several servers match. The profile shows explicit HP and defensive attributes from the public character sheet; where absent, it shows and labels equipped-item defenses as **partial** gear-only values. **Load enemy HP and defenses into calculator** copies available defenses and explicit HP into matching red fields; missing values reset to zero (unknown) when the opponent changes. A searched player character is not an NPC or boss: its defenses must not be mistaken for an encounter's actual stats. Block, Evasion and proc resistances remain reference-only for per-hit damage. The **Combat matchups** groups pair offensive stats with corresponding target defenses; PvP and elemental reference stats do not add unspecified multipliers to the supplied formula.

The lookup uses the official site's search (`api-search.plaync.com/aion2global/search/v2/character`) and character APIs (`aion2.plaync.com/api/character/info`, `equipment`, `equipment/item`) with `lang=en-US`. Item detail requests are limited to six concurrently; missing details mark the profile partial. The last 20 consulted profiles persist in `%LOCALAPPDATA%/AION2-Calculadora/profiles.json`, with atomic replacement and a successful-lookup timestamp. Saved profiles open immediately; the host fetches fresh data without serving its five-minute memory cache, then updates the dossier only. Network failures retain the last displayed data. Periodic background synchronization defaults to 600 seconds, can be disabled or set to 600–3600 seconds, and runs in Rust even when the window has no focus. In-flight requests may finish after it is disabled. **Refresh profile** explicitly fetches new data. Updates never re-import calculator values or invoke AI. Favorites are local browser preferences and sort saved profiles first; they do not exempt a profile from the 20-entry cache limit. Timestamps describe a successful API lookup, not when the character last changed in-game. The official profile has no HP total; HP remains manual.

Atool rank data is fetched from its public Global `combat-score-rankings` endpoint with a five-second timeout. It is supplementary third-party data, not an official game ranking; when the service is unavailable or has no match, the dossier labels ranking as unavailable.

## Timers, Map, Builds & Verified Database

The **Timers & Map** tab adds regional event timers (Fenda, Shugo Festa, Invasão Dimensional, Vigilante Kairah) synchronized to regional schedules, interactive regional map coordinates and metadata (under CC BY-NC 4.0; Copyright © 2025 Yihao Liu), an October 2026 meta builds reference, a verified top-10 Combat Power leaderboard snapshot, and a fast game database directory.

### Live Combat Telemetry & Prerequisites

Live combat measurement (real-time DPS meter, combat replay, in-game task reading and live pet radar) requires passive packet capture from the game client. On Windows, the Npcap driver (`wpcap.dll` / `Packet.dll`) is detected and interface handles open without elevation. However, full live telemetry and opcode parsing require a running packet capture session and continuous packet decoding. Without an active capture decoder, these features remain explicitly documented as requiring external game event streams; they are not simulated with fake data.

## Boss catalog and creature-type matchups

The **Mobs & Bosses** tab contains an offline snapshot of all **521** entries listed in [Aion2Data's global-client boss catalog](https://aion2data.com/bosses): 79 field bosses and 442 hero/legend monsters (client 2.0.6.0 hotfix 1). Filter bosses by name, available location or classification, or encounter group, select a boss and load it into the calculator. Entries include NPC ID, name, level or level range, rank and creature type; available location/classification is displayed along with the individual source URL (372 entries have a listed location). Missing locations are left blank rather than guessed. This is a snapshot of that catalog, not a live game database. The catalog and [individual boss pages](https://aion2data.com/monsters/2600097-executor-argo) do **not** publish verified Defense, damage tolerance or other numeric boss defenses. Import resets previously entered enemy defenses to zero and leaves them manual; zero means **unknown**, not measured immunity.

The **Mobs & Bosses** tab adds every entry in [Aion2Data's monster catalog](https://aion2data.com/monsters) for the same client snapshot: 3,593 unique NPC IDs across Cogni (2,114), Fera (571), Varian (557), Natura (334) and Special (17). Select **Boss** for the existing 521 boss records or **Mob** for the 3,072 remaining Normal/Elite monsters; entries are mutually exclusive, not double-counted. Search by name and choose the matching NPC ID, level and type. Importing a mob selects **PvE · Other monster**, imports its creature type and excludes Boss Attack Power. The mob catalog includes all records published by this source, not every NPC from other client versions or other sites; mob locations and numeric defenses are not available in its type listings.

The selected NPC's level, rank, creature type, encounter classification, source ID and source URL appear together in the catalog details. For the 521 bosses, `scrape-boss-details.mjs` additionally captured from each aion2data.com page the portrait, spawn-point count (374 bosses), basic attack type/reach/interval (493) and the client skill table with damage type, cast time, cooldown and range (163 bosses list skills); run `node scrape-boss-details.mjs` to refresh. HP, defenses and damage numbers are server-side and never appear on that site; they remain unknown and must be entered or measured manually instead of importing fabricated values.

For Cogni, Fera, Natura or Varian, importing selects the corresponding **Creature-type matchups** mode in the calculator. In this mode, the matching creature-specific Damage Boost and enemy Damage Tolerance fields replace the generic Species boost/tolerance terms of the supplied formula; they are not added twice. Other creature-specific fields remain reference-only because the formula has no hit-chance or separate creature-attack conversion. A target classified **Special** stays in manual Species mode. Select **Manual Species values** to restore the original generic fields. Creature mechanics: [Wikily — Creature types](https://wikily.gg/aion-2/stats/creature-types).
## Upgrade tips

The tips recalculate after every input change. They rank a **+1 percentage-point** change to Weapon Damage Boost and Damage Boost; Critical Damage Boost is included only for a critical hit, and Front/Back Attack Damage Boost only for the selected position. Gains are calculated from the current damage formula and entered enemy tolerances rather than copied from a fixed stat tier list. They compare individual hit damage, **not** gear cost or expected DPS. Critical Hit rate, Double Chance proc rate, resistance-to-chance conversions, positional uptime and skill rotation are not modeled, so no DPS recommendation is made for those stats. For a non-positive damage baseline, percentage comparisons are omitted.

Research: [AION 2 damage formula and end-game stat efficiency](https://gameplay.tips/guides/aion-2-how-stats-work-damage-formula-guide.html) and [Aion 2 Maps — Stats explained](https://aion2maps.com/guides/stats-explained/). Their published rankings depend on build, boss and patch; the displayed numbers are **this formula's what-if estimates**, not measured in-game gains.

### Combat report

The report at the bottom of the calculator uses the same effective scenario and results as the main calculation: attacker/target, landed-hit damage, estimated DPS and time to defeat. Missing hit rate or target HP is explained rather than guessed; zero hit damage stays zero. Its local +1 percentage-point comparisons are not a meta build or gear-cost ranking. Editing, importing or changing language refreshes the report offline without requesting AI.

### Gemini assistant

Open **Configure AI**, save your API key in Windows Credential Manager and select a model. Only the model ID is stored in browser local storage; the saved key is not returned to the page. The default is `gemini-3.1-flash-lite`. The local inference worker, weights and runtime are no longer distributed.

Only pressing **Ask** or Enter sends a request. It includes the current question, recent conversation, character/target names, non-default calculator values, calculated hit/DPS, marginal upgrades and an optional guide excerpt. These data leave the device and are processed by Google under your account. Editing the calculator does not initiate paid requests. Without network access or an API key, formula calculations remain available.

A successful Gemini answer also appears in the combat report at the bottom of the calculator. Editing the scenario or starting another request hides the previous AI analysis; a response for an outdated scenario is not published there. Failed requests leave the local calculated comparison available.

The model is not an authoritative game database: it must not infer exact DPS from combat power, invent proc/Teostone formulas or equate player defenses with boss defenses. The prompt asks for missing evidence and separates estimates from measured rotations, but cannot guarantee correctness.

**Apply suggested +1% upgrade** is a local formula-based what-if edit, not an AI recommendation or a change to actual equipment. Calculated comparisons take precedence over generated text.

## Release history

| Version | Highlights |
| --- | --- |
| 1.5.1 | Update the final combat report after successful Gemini answers; accept decimal-string legion averages; generate fresh certificates and sign local/CI executables with SHA-256 timestamping and public certificate export; preserve the release source tag. |
| 1.5.0 | Resolve public legion server regions and use embedded Google DNS for official lookups and guides. |
| 1.4.0 | Switch the AI assistant from AionLabs to Google Gemini (default `gemini-3.1-flash-lite`; the API key is stored XOR-obfuscated as the embedded fallback), render assistant answers as Markdown, and show attacker-vs-target attribute matchups as bordered tables. |
| 1.3.1 | Fix desktop view container width, add interactive legion roster modal with member inspection, and update release distribution. |
| 1.3.0 | Replace local SLM with AionLabs API, add persistent profile cache & background sync, Authenticode SHA-256 signing with DigiCert timestamp, event timers, map reference, and dynamic PvP/PvE meta builds. |
| 1.2.5 | PowerShell script copies and builds the Windows app; Teostones names and long effect text now have separate lines. |
| 1.2.4 | Equipment cards switch to one column at narrower widths. |
| 1.2.3 | Version-only rebuild; application behavior is unchanged from 1.2.2. |
| 1.2.2 | Windows source-build instructions use Bun; packaged application behavior is unchanged. |
| 1.2.1 | Five bilingual BIS Genus Board tables with bundled creature symbols, attribute alternatives and the Crit/Accuracy cap note. |
| 1.2.0 | Character import matches gear options by stable API id, adds Destruction → Attack Increase and Justice → Perfect Chance, marks sheet-included gear stats and display-only percent modifiers, groups the Attributes view into primary stats, divinities and other values with "feeds the calculator" / "display only" tags, and splits the result into a deterministic hit card and a combat estimate card that lists its assumptions. |
| 1.1.0 | Windows host ported from C# to Rust (tao + wry); web UI, ONNX runtime and model embedded in a single executable. |
| 1.0.2 | Character lookup from the official AION 2 site; Qwen3-0.6B q8 local assistant. |
| 1.0.1 | Attribute trees fit the window; GitHub Actions build and release pipeline. |
| 1.0.0 | First public release. |

Every GitHub Release keeps its own executable and SHA-256; publishing a new version never removes earlier downloads.

## Contributing

Contributions are welcome. Create a working branch, keep changes focused and update the relevant tests and documentation. For calculation changes, state the formula assumption and cite the game data or guide used; do not treat unknown NPC defenses or HP as verified values. For profile changes, distinguish published character-sheet totals from partial equipment options.

Run the Node.js checks in [Development and checks](#development-and-checks), publish the Windows executable and exercise the affected UI. In a pull request, explain the behavior changed, the evidence for game-data changes and the checks you ran. Include a screenshot for visible UI changes when possible.

## License

No license file is present in this repository. Do not assume the GPL license from the reference README applies to this project.
