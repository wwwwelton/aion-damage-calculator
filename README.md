# AION 2 Damage Calculator

Windows x64 damage calculator with a local WebView2 interface, hosted by a Rust executable (tao + wry). HTML, CSS, JavaScript, the inference runtime and Qwen3-0.6B q8 weights are embedded in the executable. Calculations, the mob/boss catalog and local model do not require internet access. Character search requests public profiles from the official AION 2 site (aion2.plaync.com); the assistant can fetch a current AION 2 guide by topic.

## Get started

### Run the Windows app

Requirements: Windows x64, the [Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) and about 2.5 GB of free RAM while the local assistant is active (see [Memory](#avatarnoia-flow-engine-20)). Download `AION2-Damage-Calculator.exe` from the [GitHub Releases](https://github.com/wwwwelton/aion-damage-calculator/releases) page (verify it with the attached `.sha256` file), or run a locally built executable from `dist/v<version>/` (each local build is written to its own version folder, so earlier builds are kept). The app reports an error if WebView2 is unavailable. The executable is a native Rust binary with no runtime to install. It is not digitally signed.

### Build from source

Install [Rust](https://rustup.rs/) (stable, `x86_64-pc-windows-msvc`) and [Bun](https://bun.sh/) on Windows. From the repository root, run:

```powershell
bun install --frozen-lockfile
bun build-ai.mjs
cargo build --release
```

The executable is written to `target/release/AION2-Damage-Calculator.exe`. From WSL or Linux, `cargo build --release --target x86_64-pc-windows-gnu` cross-compiles the same executable with the MinGW toolchain (`x86_64-w64-mingw32-gcc`).

The first build downloads the pinned Qwen3-0.6B-ONNX ONNX q8 model from Hugging Face, verifies its SHA-256 checksum and rewrites it into the ONNX external-data layout (`model_quantized.onnx` plus `model_quantized.onnx_data`, see `onnx-external-data.mjs`); the converted files are checksum-pinned too. **Users running the published executable do not download a model.** The weight files are excluded from Git because they exceed GitHub's 100 MB file limit. `src/assets.rs` lists the files embedded with `rust-embed`; `cargo build` fails if a listed file is missing, and `packaging.test.js` fails if `index.html` references a file that is not listed. The window (`src/window.rs`) compiles only on Windows; `cargo test` runs the host's unit tests (asset lookup, official lookup helpers, guide extraction and message routing) on any platform. `AION2_SELFTEST=<file> AION2-Damage-Calculator.exe` runs a headless check that writes a JSON report (embedded assets, one official search and profile, one guide fetch) without opening a window.

### Continuous integration and releases

`.github/workflows/build.yml` runs on every push and pull request to `main`: the JavaScript tests on Ubuntu, then on Windows `npm run build:ai` (the converted model is cached between runs and re-verified by checksum), the real offline inference check, `cargo test`, `cargo build --release` and an uploaded artifact containing the executable and its SHA-256. Pushing a tag such as `v1.2.0` additionally creates a GitHub Release with those two files, generated release notes and the version baked into the executable (the `version` in `Cargo.toml`, which must match the tag). To release: update `Cargo.toml` and `Cargo.lock`, then `git tag v1.2.0 && git push origin v1.2.0`. Each tag creates its own release; earlier releases and their executables stay available.

### Development and checks

Node.js runs the JavaScript tests and builds the AI assets. The full local verification chain, in order, is:

```bash
npm ci
npm run build:ai          # download/convert the model, bundle the worker (idempotent)
node --test *.test.js     # 39 fast regression tests
node ai-model.smoke.mjs   # real offline CPU inference in English and Portuguese
cargo test                # Rust host unit tests (any platform)
cargo build --release     # Windows executable (on Windows, or with --target x86_64-pc-windows-gnu from Linux)
```

The executable is about 640 MB, almost all of it the embedded model and runtime files. `build:ai` reuses the converted model when its checksums match, so a rerun does not download again.

Changing `ai-worker.js` or updating Transformers.js requires rerunning `npm run build:ai` before publishing. `package-lock.json` pins the development dependencies; normal users do not need Node.js or npm. The smoke check is not part of the fast test suite. The UI can be previewed by serving the repository over HTTP (for example, `python -m http.server 8000` and <http://127.0.0.1:8000/>). Character search and online guide retrieval go through the Rust host bridge (`host-bridge.js`) and work only in the Windows app. Verify those interactions and desktop behavior in the published executable.

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
| `ai-worker.js`, `build-ai.mjs`, `onnx-external-data.mjs`, `ai-model.smoke.mjs`, `ai-worker.bundle.js`, `assets/ai/` | Transformers.js worker source, build script, ONNX external-data converter, inference smoke check, bundled worker and local inference runtime |

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

## Enemy character search

Open **Enemy Character**, enter a name and region, then select the correct character if several servers match. The profile shows explicit HP and defensive attributes from the public character sheet; where absent, it shows and labels equipped-item defenses as **partial** gear-only values. **Load enemy HP and defenses into calculator** copies available defenses and explicit HP into matching red fields; missing values reset to zero (unknown) when the opponent changes. A searched player character is not an NPC or boss: its defenses must not be mistaken for an encounter's actual stats. Block, Evasion and proc resistances remain reference-only for per-hit damage. The **Combat matchups** groups pair offensive stats with corresponding target defenses; PvP and elemental reference stats do not add unspecified multipliers to the supplied formula.

The lookup uses the official AION 2 site's own search (`api-search.plaync.com/aion2global/search/v2/character`, the API behind `aion2.plaync.com/en-us/characters/index`) and character APIs (`aion2.plaync.com/api/character/info`, `equipment` and `equipment/item`), requested with `region` and `lang=en-US`. Complete searches are cached in memory for two minutes and complete profiles for five minutes. Item details are requested with at most six concurrent requests; a failed item detail marks the profile partial. No profile cache is written to disk. The official character data does not include HP in its stat list, so **HP is not imported** from character lookups and remains a manual value; no HP is estimated from community formulas.

## Boss catalog and creature-type matchups

The **Mobs & Bosses** tab contains an offline snapshot of all **521** entries listed in [Aion2Data's global-client boss catalog](https://aion2data.com/bosses): 79 field bosses and 442 hero/legend monsters (client 2.0.6.0 hotfix 1). Filter bosses by name, available location or classification, or encounter group, select a boss and load it into the calculator. Entries include NPC ID, name, level or level range, rank and creature type; available location/classification is displayed along with the individual source URL (372 entries have a listed location). Missing locations are left blank rather than guessed. This is a snapshot of that catalog, not a live game database. The catalog and [individual boss pages](https://aion2data.com/monsters/2600097-executor-argo) do **not** publish verified Defense, damage tolerance or other numeric boss defenses. Import resets previously entered enemy defenses to zero and leaves them manual; zero means **unknown**, not measured immunity.

The **Mobs & Bosses** tab adds every entry in [Aion2Data's monster catalog](https://aion2data.com/monsters) for the same client snapshot: 3,593 unique NPC IDs across Cogni (2,114), Fera (571), Varian (557), Natura (334) and Special (17). Select **Boss** for the existing 521 boss records or **Mob** for the 3,072 remaining Normal/Elite monsters; entries are mutually exclusive, not double-counted. Search by name and choose the matching NPC ID, level and type. Importing a mob selects **PvE · Other monster**, imports its creature type and excludes Boss Attack Power. The mob catalog includes all records published by this source, not every NPC from other client versions or other sites; mob locations and numeric defenses are not available in its type listings.

The selected NPC's level, rank, creature type, encounter classification, source ID and source URL appear together in the catalog details. For the 521 bosses, `scrape-boss-details.mjs` additionally captured from each aion2data.com page the portrait, spawn-point count (374 bosses), basic attack type/reach/interval (493) and the client skill table with damage type, cast time, cooldown and range (163 bosses list skills); run `node scrape-boss-details.mjs` to refresh. HP, defenses and damage numbers are server-side and never appear on that site; they remain unknown and must be entered or measured manually instead of importing fabricated values.

For Cogni, Fera, Natura or Varian, importing selects the corresponding **Creature-type matchups** mode in the calculator. In this mode, the matching creature-specific Damage Boost and enemy Damage Tolerance fields replace the generic Species boost/tolerance terms of the supplied formula; they are not added twice. Other creature-specific fields remain reference-only because the formula has no hit-chance or separate creature-attack conversion. A target classified **Special** stays in manual Species mode. Select **Manual Species values** to restore the original generic fields. Creature mechanics: [Wikily — Creature types](https://wikily.gg/aion-2/stats/creature-types).

## Upgrade tips

The tips recalculate after every input change. They rank a **+1 percentage-point** change to Weapon Damage Boost and Damage Boost; Critical Damage Boost is included only for a critical hit, and Front/Back Attack Damage Boost only for the selected position. Gains are calculated from the current damage formula and entered enemy tolerances rather than copied from a fixed stat tier list. They compare individual hit damage, **not** gear cost or expected DPS. Critical Hit rate, Double Chance proc rate, resistance-to-chance conversions, positional uptime and skill rotation are not modeled, so no DPS recommendation is made for those stats. For a non-positive damage baseline, percentage comparisons are omitted.

Research: [AION 2 damage formula and end-game stat efficiency](https://gameplay.tips/guides/aion-2-how-stats-work-damage-formula-guide.html) and [Aion 2 Maps — Stats explained](https://aion2maps.com/guides/stats-explained/). Their published rankings depend on build, boss and patch; the displayed numbers are **this formula's what-if estimates**, not measured in-game gains.

### Avatarnoia Flow Engine 2.0

The assistant beneath the attacker and target names starts automatically with “Como melhorar minha build ou DPS” and refreshes advice two seconds after the last edit, and only when the build context actually changed, because each generation occupies a CPU core for tens of seconds. Questions are optional. Requests include the character and enemy classes (when available), the attributes that differ from the calculator defaults, active scenario, calculated hit, estimated DPS and top +1% upgrades. It runs the bundled [onnx-community/Qwen3-0.6B-ONNX](https://huggingface.co/onnx-community/Qwen3-0.6B-ONNX) q8 model locally on CPU through Transformers.js; WebGPU is not required. No model weights are downloaded when the program runs, and the embedded files are served with `no-store` so they are not duplicated in the WebView2 profile. Each request carries the selected UI language; the system prompt and every status message (loading, reading, generating, errors) follow it. Check real inference offline with `node ai-model.smoke.mjs` after `npm run build:ai`; the small model can produce inaccurate or off-topic responses even with full context, so prefer the deterministic comparisons for numeric advice.

**Memory.** The WebView2 renderer holds about 2 GB of RAM once the model has answered: ONNX Runtime keeps the 0.6 GB of int8 weights in a WebAssembly heap that never shrinks, and the exported graph computes logits for every prompt position, so each prompt token costs roughly 1.3 MB permanently. The worker therefore caps prompts at 320 tokens, dropping the oldest conversation turns, then the guide note, then part of the build context; measured usage stays flat across chat turns instead of growing to 3 GB. Generation runs on a single WASM thread and takes tens of seconds per request; machines with little free RAM or swap pressure may stall while the model is active.

The Windows app may automatically request one whitelisted [Aion 2 Maps combat guide](https://aion2maps.com/guides/stats-explained/) by topic and display its URL. Only a guide category is sent to the guide website; character names, calculator values and questions stay on the device. If the internet is unavailable, local advice still runs. This is guided lookup, not unrestricted web search.

**Apply suggested +1% upgrade** changes the highest-ranked applicable percentage field by one point and recalculates the result. It is an explicit what-if edit, not a claim that the character's equipment changed; imported character and enemy stats are never silently overwritten. The small model can be inaccurate or off-topic; the calculated comparisons and cited guide take precedence.

## Release history

| Version | Highlights |
| --- | --- |
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
