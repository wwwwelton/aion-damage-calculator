# AION 2 Damage Calculator

Windows calculator with a local WebView2 interface. HTML, CSS and JavaScript are embedded in the executable. Damage calculations run locally; character search makes user-initiated requests to NotMeter's public character service and needs internet access. The [Microsoft Edge WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) must be installed on Windows. The app displays an error if the runtime is missing.

## Run

Open `dist/webview2/AION2-Damage-Calculator.exe` on Windows x64. This is a single self-contained executable; installing .NET is not required. It is not digitally signed.

## Build

On Windows with .NET SDK 10: `dotnet publish Aion2Calculator.csproj -c Release -o dist/webview2`. With Node.js, check the calculations using `node --test calculator.test.js recommendations.test.js characters.test.js`. Build requires access to NuGet for `Microsoft.Web.WebView2`.

Multipliers are entered as percentages: 100% = ×1, 150% = ×1.5. In the additive Damage Boost sum, the base field starts at 100%, and the PvE and Species bonuses start at 0%. Critical Hit, Double Chance procs and attack position apply their multipliers only when selected. The preset numbers are illustrative, not official game values. Pure Attack is a derived formula term, not a named in-game stat. The calculator neither clamps negative damage nor models in-game rounding, because the supplied formula does not specify either. The user's formatted formula omits `BossDamageBoost` from the final sum; that version is used here.

Numeric fields accept either `50,0` or `50.0` for fifty, including decimal values in all percentage and flat-stat fields. On leaving a field, the app removes redundant zeroes while retaining the entered decimal separator. Empty, incomplete or invalid entries are treated as zero during calculation and automatically replaced with `0` when the field loses focus. Grouping separators and scientific notation are not accepted.

## Enemy defenses

Red enemy fields start at zero and are paired with related offensive stats. Enemy Defense uses the original supplied `(Defense − Penetration) × 0.1` term. Weapon Damage Tolerance subtracts from Weapon Damage Boost before the weapon multiplier; Damage, PvE and Species Damage Tolerance subtract from the corresponding additive damage bucket; Critical Damage Tolerance subtracts from the critical multiplier on critical hits; Front/Back Attack Damage Tolerance subtracts from the selected directional multiplier. Enter only defenses applicable to this particular target. These are community-formula estimates: the provided equation does not specify resistance stacking, floors or rounding, so values can produce negative results.

The reference-only section collects Evasion versus Accuracy, Block versus Block Penetration, Critical Hit Resist versus Critical Hit, Double/Perfect/Multi-hit Resist versus their chance stats, Endurance versus Endurance Penetration, Boss Damage Tolerance, Elemental Resistance and flat Critical Damage/Front/Back Defense. These values **do not affect** the per-hit result: the supplied formula has no point-to-chance conversion, gear-grade-dependent block reduction, elemental terms or flat critical/position attack terms. Back attacks bypass Block; Endurance halves damage when it procs. Do not treat these resistance fields as percentages of guaranteed damage reduction. Boss Damage Tolerance is reference-only because the user's supplied final damage bucket omits Boss Damage Boost.

Stat names and interactions: [Wikily — Attack and defense](https://wikily.gg/aion-2/stats/attack-and-defense), [PvE and boss](https://wikily.gg/aion-2/stats/pve-and-boss), [Chances and resists](https://wikily.gg/aion-2/stats/chances-and-resists), [Creature types](https://wikily.gg/aion-2/stats/creature-types), and [Aion 2 Maps — How damage works](https://aion2maps.com/guides/how-damage-works/). These sources describe the stat interactions but do not establish exact enemy values for every target.

English stat terminology follows the [AION 2 damage formula guide](https://gameplay.tips/guides/aion-2-how-stats-work-damage-formula-guide.html). It is a community source, not official English localization; labels unavailable from an official English client are not certified as exact in-game UI strings.

## Character Search

Open **Character Search**, enter an exact character name, choose Korea, Taiwan or a Global region, and press **Search**. If multiple characters share that name, choose the correct server/class in the dropdown. The profile displays public attributes, equipped gear options and learned skills. **Load available stats into calculator** copies only explicit numeric bonuses from equipped items (for example, 5.6% Damage Boost becomes 105.6% in the calculator's base-plus-bonus field). Pure Attack, skill coefficients, target Defense and other unavailable totals remain manual. Equipment-only bonuses do not represent the complete character sheet, so verify the imported values before using the damage result. Public profiles may be delayed or incomplete.

The lookup follows NotMeter's public character search and profile endpoints (`https://notmeter.com/?view=character`). The service's current public API hostname is configured in `NotMeterClient.cs`; if it changes, search will report an error until that address is updated. The application does not store searched profiles.

## Upgrade tips

The tips recalculate after every input change. They rank a **+1 percentage-point** change to Weapon Damage Boost and Damage Boost; Critical Damage Boost is included only for a critical hit, and Front/Back Attack Damage Boost only for the selected position. Gains are calculated from the current damage formula and entered enemy tolerances rather than copied from a fixed stat tier list. They compare individual hit damage, **not** gear cost or expected DPS. Critical Hit rate, Double Chance proc rate, resistance-to-chance conversions, positional uptime and skill rotation are not modeled, so no DPS recommendation is made for those stats. For a non-positive damage baseline, percentage comparisons are omitted.

Research: [AION 2 damage formula and end-game stat efficiency](https://gameplay.tips/guides/aion-2-how-stats-work-damage-formula-guide.html) and [Aion 2 Maps — Stats explained](https://aion2maps.com/guides/stats-explained/). Their published rankings depend on build, boss and patch; the displayed numbers are **this formula's what-if estimates**, not measured in-game gains.
