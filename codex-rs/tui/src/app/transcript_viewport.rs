//! Interactive transcript viewport attached to the normal bottom pane.
//!
//! `Ctrl+T` keeps using the existing transcript overlay as its retained transcript model, but the
//! ordinary (non-backtrack) view renders the bottom pane below it and routes text input there.
//! PageUp/PageDown remain owned by the transcript so scrolling never moves or hides the composer.

use super::*;
use crate::pager_overlay::TranscriptHistoryState;
use crate::pager_overlay::TranscriptOverlay;
use ratatui::buffer::Buffer;
use ratatui::widgets::Clear;
use ratatui::widgets::Widget;

const MIN_TRANSCRIPT_VIEWPORT_HEIGHT: u16 = 3;

impl App {
    pub(super) fn embedded_transcript_active(&self) -> bool {
        matches!(self.overlay, Some(Overlay::Transcript(_)))
            && !self.backtrack.overlay_preview_active
    }

    pub(super) async fn handle_embedded_transcript_event(
        &mut self,
        tui: &mut tui::Tui,
        app_server: &mut AppServerSession,
        event: TuiEvent,
    ) -> Result<()> {
        match event {
            TuiEvent::Key(key_event) => {
                if self.keymap.app.open_transcript.is_pressed(key_event) {
                    self.close_transcript_overlay(tui);
                    return Ok(());
                }

                let (scrolls_transcript, should_load_older, should_load_from_start) =
                    match self.overlay.as_ref() {
                        Some(Overlay::Transcript(overlay)) => (
                            is_embedded_transcript_navigation_key(key_event),
                            overlay.should_load_older(key_event),
                            overlay.should_load_from_start(key_event),
                        ),
                        Some(Overlay::Static(_)) | None => (false, false, false),
                    };
                if scrolls_transcript {
                    if should_load_older
                        && let Some(thread_id) = self.chat_widget.thread_id()
                        && app_server.has_older_history(thread_id)
                        && self.request_older_history_page(app_server, thread_id)
                        && let Some(Overlay::Transcript(overlay)) = self.overlay.as_mut()
                    {
                        overlay.set_history_state(if should_load_from_start {
                            TranscriptHistoryState::LoadingBeginning
                        } else {
                            TranscriptHistoryState::LoadingOlder
                        });
                    }
                    if let Some(Overlay::Transcript(overlay)) = self.overlay.as_mut() {
                        overlay.handle_embedded_key_event(key_event);
                    }
                    tui.frame_requester().schedule_frame();
                    return Ok(());
                }

                // Escape belongs to the composer here (not transcript backtracking), which keeps
                // Vim insert-mode transitions and popup dismissal consistent with the main view.
                if matches!(key_event.code, KeyCode::Esc) {
                    self.chat_widget.handle_key_event(key_event);
                } else {
                    self.handle_key_event(tui, app_server, key_event).await;
                }
            }
            TuiEvent::Paste(pasted) => {
                let pasted = pasted.replace("\r\n", "\n").replace('\r', "\n");
                self.chat_widget.handle_paste(pasted);
            }
            TuiEvent::Draw | TuiEvent::Resume | TuiEvent::Resize(_) => {
                self.chat_widget.maybe_post_pending_notification(tui);
                if self
                    .chat_widget
                    .handle_paste_burst_tick(tui.frame_requester())
                {
                    return Ok(());
                }
                self.chat_widget.pre_draw_tick();
                self.render_embedded_transcript_frame(tui)?;
            }
        }
        Ok(())
    }

    fn render_embedded_transcript_frame(&mut self, tui: &mut tui::Tui) -> Result<Rect> {
        let active_key = self.chat_widget.active_cell_transcript_key();
        let chat_widget = &self.chat_widget;
        let bottom_pane = chat_widget.as_bottom_pane_renderable();
        let Some(Overlay::Transcript(overlay)) = self.overlay.as_mut() else {
            return Ok(Rect::default());
        };
        let mut rendered_area = Rect::default();
        tui.draw(u16::MAX, |frame| {
            let area = frame.area();
            rendered_area = area;
            let transcript_width = area.width.max(1);
            overlay.sync_live_tail(transcript_width, active_key, |width| {
                chat_widget.active_cell_transcript_hyperlink_lines(width)
            });
            let bottom_area =
                render_embedded_transcript_surface(overlay, &bottom_pane, area, frame.buffer);
            chat_widget.note_rendered_width(area.width);
            if let Some((x, y)) = bottom_pane.cursor_pos(bottom_area) {
                frame.set_cursor_style(bottom_pane.cursor_style(bottom_area));
                frame.set_cursor_position((x, y));
            }
        })?;

        if active_key.is_some_and(|key| key.animation_tick.is_some())
            && overlay.is_scrolled_to_bottom()
        {
            tui.frame_requester()
                .schedule_frame_in(std::time::Duration::from_millis(50));
        }
        Ok(rendered_area)
    }
}

fn is_embedded_transcript_navigation_key(key_event: KeyEvent) -> bool {
    matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat)
        && (key_event.modifiers.is_empty() || key_event.modifiers == KeyModifiers::SHIFT)
        && matches!(key_event.code, KeyCode::PageUp | KeyCode::PageDown)
}

fn render_embedded_transcript_surface(
    overlay: &mut TranscriptOverlay,
    bottom_pane: &dyn Renderable,
    area: Rect,
    buf: &mut Buffer,
) -> Rect {
    let bottom_height = if area.height > MIN_TRANSCRIPT_VIEWPORT_HEIGHT {
        bottom_pane
            .desired_height(area.width)
            .max(1)
            .min(area.height - MIN_TRANSCRIPT_VIEWPORT_HEIGHT)
    } else {
        area.height
    };
    let transcript_height = area.height.saturating_sub(bottom_height);
    let transcript_area = Rect::new(area.x, area.y, area.width, transcript_height);
    let bottom_area = Rect::new(
        area.x,
        area.y.saturating_add(transcript_height),
        area.width,
        bottom_height,
    );

    if transcript_area.height > 0 {
        overlay.render_embedded(transcript_area, buf);
    }
    if bottom_area.height > 0 {
        Clear.render(bottom_area, buf);
        bottom_pane.render(bottom_area, buf);
    }
    bottom_area
}

#[cfg(test)]
#[path = "transcript_viewport_tests.rs"]
mod tests;
