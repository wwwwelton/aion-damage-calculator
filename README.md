# AION 2 Damage Calculator

Windows x64 damage calculator with a local WebView2 interface. HTML, CSS, JavaScript, the inference runtime and SmolLM2 Q4F16 weights are embedded in the executable. Calculations, the mob/boss catalog and local model do not require internet access. Character search requests public profiles from NotMeter; the assistant can fetch a current AION 2 guide by topic.

## Get started

### Run the Windows app

Requirements: Windows x64 and the [Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/). Run `dist/webview2/AION2-Damage-Calculator.exe` after publishing, or use a distributed executable. The app reports an error if WebView2 is unavailable. The executable includes the .NET runtime, so users do not need to install the .NET SDK. It is not digitally signed.

### Build from source

Install the [.NET SDK 10](https://dotnet.microsoft.com/download/dotnet/10.0) and [Node.js](https://nodejs.org/) on Windows. From the repository root, run:

```powershell
npm ci
npm run build:ai
dotnet restore Aion2Calculator.csproj
dotnet publish Aion2Calculator.csproj -c Release -o dist/webview2
```

The first build downloads the pinned 118 MB Q4F16 model from Hugging Face and verifies its SHA-256 checksum; **users running the published executable do not download a model**. The ONNX weight file is excluded from Git because it exceeds GitHub's 100 MB file limit. The build requires NuGet for `Microsoft.Web.WebView2`; `NuGet.Config` uses nuget.org. The project targets `net10.0-windows` and `win-x64` and publishes a self-contained, single-file executable (about 250 MB). Open `dist/webview2/AION2-Damage-Calculator.exe` to check the desktop UI. Building from WSL with the Windows SDK requires invoking Windows `dotnet.exe`; publishing does not make the app runnable on Linux.

### Development and checks

Node.js runs the JavaScript tests and builds the AI assets. From the repository root:

```bash
node --test calculator.test.js numeric-input.test.js recommendations.test.js characters.test.js
```

To change `ai-worker.js` or update Transformers.js, regenerate the bundled worker and local WebAssembly runtime before publishing:

```bash
npm ci
npm run build:ai
```

`package-lock.json` pins the development dependencies; normal users do not need Node.js or npm. The UI can be previewed by serving the repository over HTTP (for example, `python -m http.server 8000` and <http://127.0.0.1:8000/>). Character search and online guide retrieval use the WebView2-to-.NET bridge and work only in the Windows app. Verify those interactions and desktop behavior in the published executable.

### Project layout

| Path | Purpose |
| --- | --- |
| `Aion2Calculator.csproj`, `Program.cs` | Windows build, embedded files and WebView2 host |
| `NotMeterClient.cs`, `GuideResearch.cs` | Public character lookup and opt-in wiki-guide retrieval through the native host |
| `index.html`, `style.css`, `renderer.js`, `profile-ui.js`, `ai-tips.js` | Interface, character dossier and contextual assistant |
| `calculator.js`, `matchups.js`, `recommendations.js`, `numeric-input.js` | Damage calculation, matchup rules, tips and number parsing |
| `characters.js`, `bosses.js`, `boss-data.js`, `mob-data.js` | Profile imports and offline NPC catalog |
| `*.test.js` | Node.js regression tests |
| `ai-worker.js`, `build-ai.mjs`, `ai-worker.bundle.js`, `assets/ai/` | Transformers.js worker source, build script, bundled worker and local inference runtime |

## Calculation scope

Multipliers are entered as percentages: 100% = ×1, 150% = ×1.5. The additive Damage Boost bucket starts at 100%; PvE, Boss and Species bonuses start at zero. Critical Hit, Double Chance procs and attack position apply only when selected. Presets are illustrative, not official values. Pure Attack is a derived formula term. This community-derived model adds Boss Damage Boost to the bucket (unlike the originally supplied formula), caps effective Defense at zero after Penetration, uses Defense ×0.1 in PvE and ×0.01 in PvP, and uses a ×2 Double proc in PvE versus ×1.5 in PvP. Other damage floors and game rounding are not modeled.

## Icons

The eight class emblems are bundled locally from [MetaBot's AION 2 class icon gallery](https://metabot.gg/en/aion-2/classes), which credits the game client. Known class names show the emblem beside characters in the encounter header and public profiles. Public character dossiers display remote item and skill artwork when available; offline, their generic category icons remain. Boss and mob icons are distinct generic symbols, not individual NPC portraits. Text labels remain visible when an icon is unavailable.

## PvE and PvP scenarios

At the top of **Damage Calculator**, choose **PvE · Boss**, **PvE · Other monster** or **PvP · Player**. The header has editable attacker and target names/classes or creature type. Importing a character, enemy player, boss or mob fills its header; opponent imports select the corresponding mode. Switching modes preserves offensive values but clears the previous opponent's identity and defenses. Boss Attack Power and Boss Damage Boost/Tolerance apply only to bosses; PvE and Species terms do not apply to players. Creature-type matchups replace generic Species Boost/Tolerance, not add to them. PvP-specific Attack/Boost/Tolerance and elemental modifiers remain reference-only: their exact placement and stacking are not established by the sources; the PvP result is **not** a complete damage prediction. Unknown opponent defenses stay manual.

Numeric fields accept either `50,0` or `50.0` for fifty, including decimal values in all percentage and flat-stat fields. On leaving a field, the app removes redundant zeroes while retaining the entered decimal separator. Empty, incomplete or invalid entries are treated as zero during calculation and automatically replaced with `0` when the field loses focus. Grouping separators and scientific notation are not accepted.

## Estimated DPS and health

**Estimated DPS = estimated hit damage × landed hits per second**. Enter an observed or assumed positive hit rate; zero is unknown, so no DPS is shown. **Time to defeat = target HP ÷ estimated DPS** only when both are positive. These are constant-hit what-if values, not a measured skill rotation, expected proc average, or a complete PvP simulation; misses, healing, downtime and encounter mechanics are excluded. The **Player HP** field shows the attacker's HP for context and does not change outgoing damage. The target field is labeled **Boss HP**, **Mob HP** or **Enemy player HP** according to the selected opponent; zero means unknown and target changes clear it. Public player profiles import only an explicit positive character-sheet **HP** stat (no equipment inference). [Aion2Data explains](https://aion2data.com/monsters/2200208-basement-zirka) that NPC HP and combat stats are held by the server, not the client database, so mob and boss HP must be entered manually when known. [Wikily describes HP as a character stat](https://wikily.gg/aion-2/stats/basic).

## Enemy defenses

Red enemy fields start at zero and pair with related offensive stats. Effective Defense is `max(0, Defense − Penetration)`; subtract 10% of it in PvE or 1% in PvP. Weapon Damage Tolerance subtracts from Weapon Damage Boost; Damage, PvE, Boss and Species Tolerances subtract from the additive boost bucket when relevant. Critical Damage Tolerance subtracts from the critical multiplier on critical hits; Front/Back Attack Damage Tolerance subtracts from the selected positional multiplier. Enter only defenses applicable to this target. Tolerance stacking and damage floors outside Defense remain community estimates and may yield negative values.

The reference-only section collects Evasion versus Accuracy, Block versus Block Penetration, Critical Hit Resist versus Critical Hit, Double/Perfect/Multi-hit Resist versus their chance stats, Endurance versus Endurance Penetration, elemental stats and flat Critical Damage/Front/Back Defense. These do not affect a selected hit: the model has no verified point-to-chance conversion, gear-grade-dependent block reduction, elemental skill designation or flat critical/position attack conversion. Back attacks bypass Block; Endurance halves damage when it procs. Do not treat chance resistances as percentages of guaranteed damage reduction.

Stat names and interactions: [Wikily — Basic](https://wikily.gg/aion-2/stats/basic), [Attack and defense](https://wikily.gg/aion-2/stats/attack-and-defense), [PvE and boss](https://wikily.gg/aion-2/stats/pve-and-boss), [PvP](https://wikily.gg/aion-2/stats/pvp), [Chances and resists](https://wikily.gg/aion-2/stats/chances-and-resists), [Creature types](https://wikily.gg/aion-2/stats/creature-types), [Aion 2 Maps — Stats explained](https://aion2maps.com/guides/stats-explained/) and [How damage works](https://aion2maps.com/guides/how-damage-works/). These sources describe interactions but do not establish exact enemy values for every target.

English stat terminology follows the [AION 2 damage formula guide](https://gameplay.tips/guides/aion-2-how-stats-work-damage-formula-guide.html). It is a community source, not official English localization; labels unavailable from an official English client are not certified as exact in-game UI strings.

## Character Search

Open **Character Search**, enter an exact character name, choose Korea, Taiwan or a Global region, and press **Search**. If multiple characters share that name, choose the correct server/class in the dropdown. The profile displays public attributes, equipped gear options and learned skills. **Load available stats into calculator** copies explicit numeric bonuses from equipped items (for example, 5.6% Damage Boost becomes 105.6% in the calculator's base-plus-bonus field), and fills offensive matchup fields from the character sheet or, when missing, explicit **partial** equipment options. Pure Attack, skill coefficients, target Defense and other unavailable totals remain manual. Gear-only bonuses do not represent the complete character sheet. Public profiles may be delayed or incomplete.

The character dossier has **Equipment**, **Skills**, **Attributes** and **Progression** views. Equipment shows published base/bonus and Soul Bind options, stones and attached skills; Skills groups learned active, passive and Stigma abilities with their levels; Progression shows published titles, Daevanion boards, Arcana, pet and wing. The same views are available for the selected enemy player. Portrait and item/skill images require internet access; hidden or absent profile details are not inferred.

## Enemy character search

Open **Enemy Character**, enter a name and region, then select the correct character if several servers match. The profile shows explicit HP and defensive attributes from the public character sheet; where absent, it shows and labels equipped-item defenses as **partial** gear-only values. **Load enemy HP and defenses into calculator** copies available defenses and explicit HP into matching red fields; missing values reset to zero (unknown) when the opponent changes. A searched player character is not an NPC or boss: its defenses must not be mistaken for an encounter's actual stats. Block, Evasion and proc resistances remain reference-only for per-hit damage. The **Combat matchups** groups pair offensive stats with corresponding target defenses; PvP and elemental reference stats do not add unspecified multipliers to the supplied formula.

The lookup follows NotMeter's public character search and profile endpoints (`https://notmeter.com/?view=character`). The service's current public API hostname is configured in `NotMeterClient.cs`; if it changes, search will report an error until that address is updated. The application does not store searched profiles.

## Boss catalog and creature-type matchups

The **Mobs & Bosses** tab contains an offline snapshot of all **521** entries listed in [Aion2Data's global-client boss catalog](https://aion2data.com/bosses): 79 field bosses and 442 hero/legend monsters (client 2.0.6.0 hotfix 1). Filter bosses by name, available location or classification, or encounter group, select a boss and load it into the calculator. Entries include NPC ID, name, level or level range, rank and creature type; available location/classification is displayed along with the individual source URL (372 entries have a listed location). Missing locations are left blank rather than guessed. This is a snapshot of that catalog, not a live game database. The catalog and [individual boss pages](https://aion2data.com/monsters/2600097-executor-argo) do **not** publish verified Defense, damage tolerance or other numeric boss defenses. Import resets previously entered enemy defenses to zero and leaves them manual; zero means **unknown**, not measured immunity.

The **Mobs & Bosses** tab adds every entry in [Aion2Data's monster catalog](https://aion2data.com/monsters) for the same client snapshot: 3,593 unique NPC IDs across Cogni (2,114), Fera (571), Varian (557), Natura (334) and Special (17). Select **Boss** for the existing 521 boss records or **Mob** for the 3,072 remaining Normal/Elite monsters; entries are mutually exclusive, not double-counted. Search by name and choose the matching NPC ID, level and type. Importing a mob selects **PvE · Other monster**, imports its creature type and excludes Boss Attack Power. The mob catalog includes all records published by this source, not every NPC from other client versions or other sites; mob locations and numeric defenses are not available in its type listings.

The selected NPC's level, rank, creature type, encounter classification, source ID and source URL appear together in the catalog details. This catalog does not publish reliable NPC health, defense or skill lists; these remain unknown and must be entered or measured manually instead of importing fabricated values.

For Cogni, Fera, Natura or Varian, importing selects the corresponding **Creature-type matchups** mode in the calculator. In this mode, the matching creature-specific Damage Boost and enemy Damage Tolerance fields replace the generic Species boost/tolerance terms of the supplied formula; they are not added twice. Other creature-specific fields remain reference-only because the formula has no hit-chance or separate creature-attack conversion. A target classified **Special** stays in manual Species mode. Select **Manual Species values** to restore the original generic fields. Creature mechanics: [Wikily — Creature types](https://wikily.gg/aion-2/stats/creature-types).

## Upgrade tips

The tips recalculate after every input change. They rank a **+1 percentage-point** change to Weapon Damage Boost and Damage Boost; Critical Damage Boost is included only for a critical hit, and Front/Back Attack Damage Boost only for the selected position. Gains are calculated from the current damage formula and entered enemy tolerances rather than copied from a fixed stat tier list. They compare individual hit damage, **not** gear cost or expected DPS. Critical Hit rate, Double Chance proc rate, resistance-to-chance conversions, positional uptime and skill rotation are not modeled, so no DPS recommendation is made for those stats. For a non-positive damage baseline, percentage comparisons are omitted.

Research: [AION 2 damage formula and end-game stat efficiency](https://gameplay.tips/guides/aion-2-how-stats-work-damage-formula-guide.html) and [Aion 2 Maps — Stats explained](https://aion2maps.com/guides/stats-explained/). Their published rankings depend on build, boss and patch; the displayed numbers are **this formula's what-if estimates**, not measured in-game gains.

### Avatarnoia Flow Engine 2.0

The assistant beneath the attacker and target names starts automatically and refreshes advice as the calculator or opponent changes. Questions are optional. It runs the bundled [HuggingFaceTB/SmolLM2-135M-Instruct](https://huggingface.co/HuggingFaceTB/SmolLM2-135M-Instruct) Q4F16 model locally through Transformers.js. WebGPU with `shader-f16` support and a current WebView2 Runtime are required for generation; the deterministic damage ranking remains available if the GPU lacks that feature. No model weights are downloaded when the program runs.

The Windows app may automatically request one whitelisted [Aion 2 Maps combat guide](https://aion2maps.com/guides/stats-explained/) by topic and display its URL. Only a guide category is sent to the guide website; character names, calculator values and questions stay on the device. If the internet is unavailable, local advice still runs. This is guided lookup, not unrestricted web search.

**Apply suggested +1% upgrade** changes the highest-ranked applicable percentage field by one point and recalculates the result. It is an explicit what-if edit, not a claim that the character's equipment changed; imported character and enemy stats are never silently overwritten. The small model can be inaccurate or off-topic; the calculated comparisons and cited guide take precedence.

## Contributing

Contributions are welcome. Create a working branch, keep changes focused and update the relevant tests and documentation. For calculation changes, state the formula assumption and cite the game data or guide used; do not treat unknown NPC defenses or HP as verified values. For profile changes, distinguish published character-sheet totals from partial equipment options.

Run the Node.js checks in [Development and checks](#development-and-checks), publish the Windows executable and exercise the affected UI. In a pull request, explain the behavior changed, the evidence for game-data changes and the checks you ran. Include a screenshot for visible UI changes when possible.

## License

No license file is present in this repository. Do not assume the GPL license from the reference README applies to this project.
