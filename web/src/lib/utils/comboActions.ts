/**
 * Combo action normalisation.
 *
 * The Rust backend serialises `ComboAction` as an internally-tagged
 * enum (`{"type":"bootloader"}`) while the frontend TypeScript types
 * model it as a flat string union (`'bootloader' | ...`). Layouts
 * written before the DTO change therefore carry the tagged shape,
 * but new ones can land in the flat shape. Both must render
 * correctly in the editor, so we accept either and fall back to a
 * safe default if the value is something we don't recognise.
 */

import type { ComboAction } from '$api/types';

const KNOWN_ACTIONS = ['disable_effects', 'disable_lighting', 'bootloader'] as const;

function isComboAction(value: unknown): value is ComboAction {
	return typeof value === 'string' && (KNOWN_ACTIONS as readonly string[]).includes(value);
}

/**
 * Convert any of the wire shapes (flat string, tagged object, or
 * something bogus) into the canonical `ComboAction` string. Falls
 * back to `'disable_effects'` for unknown / malformed input so the
 * UI always has a value to render.
 */
export function normaliseComboAction(action: unknown): ComboAction {
	if (isComboAction(action)) {
		return action;
	}
	if (action && typeof action === 'object' && 'type' in action) {
		const t = (action as { type: unknown }).type;
		if (isComboAction(t)) {
			return t;
		}
	}
	return 'disable_effects';
}
