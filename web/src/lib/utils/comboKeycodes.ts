/**
 * Parses a QMK keycode into its parts for split editing.
 *
 * Mirrors `src/tui/editor/key_editor.rs:46` (ComboKeycodeType) but as a
 * lightweight frontend helper. Returns the prefix and the typed values,
 * allowing the WebUI to swap just the hold or tap part without losing
 * the other side.
 *
 * Recognised shapes:
 *   - LT(layer, key)            — layer-tap
 *   - MT(modifier, key)         — mod-tap (named)
 *   - LM(layer, modifier)       — layer-mod
 *   - LCG/MEH/HYPR(key)         — modifier-combo (modifier is in prefix)
 *   - TD(name)                  — tap dance
 *   - LCTL_T/KC_…                — non-parameterized, returns null
 *
 * Layer references can be numeric ("2") or UUID-based ("@abc-def").
 */

export type ComboKind =
	| 'layer-tap'
	| 'mod-tap'
	| 'layer-mod'
	| 'mod-combo'
	| 'tap-dance';

export interface ParsedCombo {
	kind: ComboKind;
	prefix: string;
	hold: string;
	tap: string;
	/** True if the "hold" param is a layer reference (e.g. for LT/LM). */
	holdIsLayer: boolean;
	/** True if the "hold" param is a modifier expression. */
	holdIsModifier: boolean;
}

const MOD_TAP_PREFIXES = [
	'LCTL_T',
	'RCTL_T',
	'CTL_T',
	'LSFT_T',
	'RSFT_T',
	'SFT_T',
	'LALT_T',
	'RALT_T',
	'ALT_T',
	'LOPT_T',
	'ROPT_T',
	'OPT_T',
	'LGUI_T',
	'RGUI_T',
	'GUI_T',
	'LCMD_T',
	'RCMD_T',
	'CMD_T',
	'LWIN_T',
	'RWIN_T',
	'WIN_T',
	'LSG_T',
	'RSG_T',
	'SGUI_T',
	'LCA_T',
	'RCA_T',
	'LCS_T',
	'RCS_T',
	'LCAG_T',
	'RCAG_T',
	'LSA_T',
	'RSA_T',
	'SAGR_T',
	'LSAG_T',
	'RSAG_T',
	'MEH_T',
	'HYPR_T',
	'ALL_T'
];

const MOD_COMBO_PREFIXES = [
	'LCTL',
	'RCTL',
	'LSFT',
	'RSFT',
	'LALT',
	'RALT',
	'LOPT',
	'ROPT',
	'LGUI',
	'RGUI',
	'LCMD',
	'RCMD',
	'LWIN',
	'RWIN',
	'LCA',
	'RCA',
	'LSA',
	'RSA',
	'LAG',
	'RAG',
	'LCAG',
	'RCAG',
	'LSAG',
	'RSAG',
	'LSG',
	'RSG',
	'LCS',
	'RCS',
	'MEH',
	'HYPR',
	'LCG',
	'RCG'
];

/**
 * Split `2, KC_SPC` or `1, LCTL_T(KC_A)` into top-level args. Tracks nested
 * parentheses so commas inside nested parens (e.g. tap arg) don't split.
 */
function splitTopLevelArgs(input: string): string[] {
	const args: string[] = [];
	let depth = 0;
	let current = '';
	for (const ch of input) {
		if (ch === '(') {
			depth++;
			current += ch;
		} else if (ch === ')') {
			depth--;
			current += ch;
		} else if (ch === ',' && depth === 0) {
			args.push(current.trim());
			current = '';
		} else {
			current += ch;
		}
	}
	if (current.trim().length > 0) args.push(current.trim());
	return args;
}

export function parseComboKeycode(keycode: string): ParsedCombo | null {
	const trimmed = keycode.trim();
	if (!trimmed.includes('(')) return null;

	const open = trimmed.indexOf('(');
	const close = trimmed.lastIndexOf(')');
	if (open === -1 || close === -1 || close <= open) return null;

	const prefix = trimmed.slice(0, open);
	const argsRaw = trimmed.slice(open + 1, close);
	const args = splitTopLevelArgs(argsRaw).map((a) => a.trim());

	if (prefix === 'LT' && args.length === 2) {
		return {
			kind: 'layer-tap',
			prefix,
			hold: args[0],
			tap: args[1],
			holdIsLayer: true,
			holdIsModifier: false
		};
	}

	if (MOD_TAP_PREFIXES.includes(prefix) && args.length === 1) {
		return {
			kind: 'mod-tap',
			prefix,
			hold: prefix,
			tap: args[0],
			holdIsLayer: false,
			holdIsModifier: true
		};
	}

	if (prefix === 'MT') {
		// MT(modifier, key) — modifier is full expression
		if (args.length === 2) {
			return {
				kind: 'mod-tap',
				prefix,
				hold: args[0],
				tap: args[1],
				holdIsLayer: false,
				holdIsModifier: true
			};
		}
	}

	if (prefix === 'LM' && args.length === 2) {
		return {
			kind: 'layer-mod',
			prefix,
			hold: args[0],
			tap: args[1],
			holdIsLayer: true,
			holdIsModifier: true
		};
	}

	if (MOD_COMBO_PREFIXES.includes(prefix) && args.length === 1) {
		return {
			kind: 'mod-combo',
			prefix,
			hold: prefix,
			tap: args[0],
			holdIsLayer: false,
			holdIsModifier: true
		};
	}

	if (prefix === 'TD' && args.length === 1) {
		return {
			kind: 'tap-dance',
			prefix,
			hold: args[0],
			tap: args[0],
			holdIsLayer: false,
			holdIsModifier: false
		};
	}

	return null;
}

/**
 * Rebuild a keycode after editing one of its parts.
 */
export function reassembleCombo(combo: ParsedCombo, updated: { hold?: string; tap?: string }): string {
	const hold = updated.hold ?? combo.hold;
	const tap = updated.tap ?? combo.tap;
	return `${combo.prefix}(${hold}, ${tap})`;
}

/**
 * Returns a friendly display string for the hold part.
 */
export function describeHold(combo: ParsedCombo): string {
	if (combo.holdIsLayer) {
		const ref = combo.hold.startsWith('@') ? combo.hold.slice(1, 9) : combo.hold;
		return `Layer ${ref}`;
	}
	if (combo.holdIsModifier) {
		return combo.hold;
	}
	return combo.hold;
}