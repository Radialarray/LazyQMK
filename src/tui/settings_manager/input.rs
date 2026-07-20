//! Input handling for SettingsManager.
//!
//! These are inherent impl methods on SettingsManager, with pub(super) visibility
//! so the dispatch in handle_input_with_context (in mod.rs) can call them.

use crossterm::event::{KeyCode, KeyEvent};

use crate::models::{
    HoldDecisionMode, PaletteFxEffect, PaletteFxPalette, RgbMatrixEffect, RippleColorMode,
    TapHoldPreset,
};

use super::SettingItem;
use super::SettingsManager;
use super::SettingsManagerContext;
use super::SettingsManagerEvent;

impl SettingsManager {
    pub(super) fn handle_browsing_input(
        &mut self,
        key: KeyEvent,
        _context: &SettingsManagerContext,
    ) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => Some(SettingsManagerEvent::Cancelled),
            KeyCode::Up | KeyCode::Char('k') => {
                let count = SettingItem::all().len();
                self.state.select_previous(count);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = SettingItem::all().len();
                self.state.select_next(count);
                None
            }
            KeyCode::Enter => {
                // Note: Actual setting editing is complex and involves opening
                // sub-dialogs or other popups. This is handled by the parent.
                // We just signal that Enter was pressed on a setting.
                None
            }
            _ => None,
        }
    }

    pub(super) fn handle_preset_selection(
        &mut self,
        key: KeyEvent,
        _context: &SettingsManagerContext,
    ) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let count = TapHoldPreset::all().len();
                self.state.option_previous(count);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = TapHoldPreset::all().len();
                self.state.option_next(count);
                None
            }
            KeyCode::Enter => {
                // Signal that a selection was made
                // Parent will extract the value from state
                Some(SettingsManagerEvent::SettingsUpdated)
            }
            _ => None,
        }
    }

    pub(super) fn handle_hold_mode_selection(
        &mut self,
        key: KeyEvent,
        _context: &SettingsManagerContext,
    ) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let count = HoldDecisionMode::all().len();
                self.state.option_previous(count);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = HoldDecisionMode::all().len();
                self.state.option_next(count);
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }

    pub(super) fn handle_numeric_editing(&mut self, key: KeyEvent) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.increment_numeric(10);
                None
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.state.increment_numeric(1);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.decrement_numeric(10);
                None
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.state.decrement_numeric(1);
                None
            }
            KeyCode::Home => {
                self.state.reset_numeric_to_default();
                None
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                self.state.handle_char_input(c);
                None
            }
            KeyCode::Backspace => {
                self.state.handle_backspace();
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }

    pub(super) fn handle_signed_numeric_editing(
        &mut self,
        key: KeyEvent,
    ) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.increment_signed_numeric(10);
                None
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.state.increment_signed_numeric(1);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.decrement_signed_numeric(10);
                None
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.state.decrement_signed_numeric(1);
                None
            }
            KeyCode::Home => {
                self.state.reset_signed_numeric_to_default();
                None
            }
            KeyCode::Char(c) if c.is_ascii_digit() || c == '-' => {
                self.state.handle_signed_char_input(c);
                None
            }
            KeyCode::Backspace => {
                self.state.handle_backspace();
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }

    pub(super) fn handle_boolean_toggle(&mut self, key: KeyEvent) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Up | KeyCode::Down | KeyCode::Char('k') | KeyCode::Char('j') => {
                self.state.option_previous(2);
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }

    pub(super) fn handle_string_editing(&mut self, key: KeyEvent) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Char(c) => {
                self.state.handle_string_char_input(c);
                None
            }
            KeyCode::Backspace => {
                self.state.handle_string_backspace();
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }

    pub(super) fn handle_output_format_selection(
        &mut self,
        key: KeyEvent,
    ) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.option_previous(3);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.option_next(3);
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }

    pub(super) fn handle_theme_mode_selection(
        &mut self,
        key: KeyEvent,
    ) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.option_previous(3);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.option_next(3);
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }

    pub(super) fn handle_path_editing(&mut self, key: KeyEvent) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Char(c) => {
                self.state.handle_string_char_input(c);
                None
            }
            KeyCode::Backspace => {
                self.state.handle_string_backspace();
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }

    pub(super) fn handle_idle_effect_mode_selection(
        &mut self,
        key: KeyEvent,
    ) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let count = RgbMatrixEffect::all().len();
                self.state.option_previous(count);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = RgbMatrixEffect::all().len();
                self.state.option_next(count);
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }

    pub(super) fn handle_ripple_color_mode_selection(
        &mut self,
        key: KeyEvent,
    ) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let count = RippleColorMode::all().len();
                self.state.option_previous(count);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = RippleColorMode::all().len();
                self.state.option_next(count);
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }

    pub(super) fn handle_palette_fx_effect_selection(
        &mut self,
        key: KeyEvent,
    ) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let count = PaletteFxEffect::all().len();
                self.state.option_previous(count);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = PaletteFxEffect::all().len();
                self.state.option_next(count);
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }

    pub(super) fn handle_palette_fx_palette_selection(
        &mut self,
        key: KeyEvent,
    ) -> Option<SettingsManagerEvent> {
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let count = PaletteFxPalette::all().len();
                self.state.option_previous(count);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let count = PaletteFxPalette::all().len();
                self.state.option_next(count);
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }

    pub(super) fn handle_key_action_palette_selection(
        &mut self,
        key: KeyEvent,
    ) -> Option<SettingsManagerEvent> {
        // +1 for "Default" option at position 0
        let count = PaletteFxPalette::all().len() + 1;
        match key.code {
            KeyCode::Esc => {
                self.state.cancel();
                None
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.option_previous(count);
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.option_next(count);
                None
            }
            KeyCode::Enter => Some(SettingsManagerEvent::SettingsUpdated),
            _ => None,
        }
    }
}
