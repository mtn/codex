//! Embedded transcript rendering and navigation.
//!
//! The full transcript pager owns every key while it is modal. The embedded transcript instead
//! keeps the bottom pane interactive, so it deliberately claims only physical PageUp/PageDown
//! events and leaves text-editing bindings to the composer.

use super::*;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;

impl TranscriptOverlay {
    /// Render transcript content with compact navigation hints for the composer-attached view.
    pub(crate) fn render_embedded(&mut self, area: Rect, buf: &mut Buffer) {
        self.view.render(area, buf);
        self.render_history_state(area, buf);
        if area.height == 0 {
            return;
        }

        let footer = Rect::new(
            area.x,
            area.bottom().saturating_sub(1),
            area.width,
            /*height*/ 1,
        );
        render_key_hints(
            footer,
            buf,
            &[
                (
                    vec![
                        key_hint::plain(KeyCode::PageUp).into(),
                        key_hint::plain(KeyCode::PageDown).into(),
                    ],
                    "to scroll transcript",
                ),
                (
                    first_or_empty(
                        &self.view.keymap,
                        "close_transcript",
                        &self.view.keymap.close_transcript,
                    ),
                    "to close",
                ),
            ],
        );
    }

    /// Scroll one transcript page without claiming ordinary composer input.
    pub(crate) fn handle_embedded_key_event(&mut self, key_event: KeyEvent) -> bool {
        if !matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat)
            || !(key_event.modifiers.is_empty() || key_event.modifiers == KeyModifiers::SHIFT)
        {
            return false;
        }

        let page_height = self
            .view
            .last_content_height
            .unwrap_or(/*default*/ 1)
            .max(1);
        match key_event.code {
            KeyCode::PageUp => {
                self.view.scroll_offset = self.view.scroll_offset.saturating_sub(page_height);
            }
            KeyCode::PageDown => {
                self.view.scroll_offset = self.view.scroll_offset.saturating_add(page_height);
            }
            _ => return false,
        }
        true
    }
}

#[cfg(test)]
#[path = "embedded_tests.rs"]
mod tests;
