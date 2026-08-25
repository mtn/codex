use super::*;
use crate::app::test_support::make_test_app;
use crate::history_cell::PlainHistoryCell;
use pretty_assertions::assert_eq;
use ratatui::text::Line;
use std::sync::Arc;

#[tokio::test]
async fn transcript_scroll_and_composer_input_are_independent() {
    let mut app = make_test_app().await;
    app.chat_widget
        .apply_external_edit("keep typing here".to_string());
    let cells = (0..18)
        .map(|index| {
            Arc::new(PlainHistoryCell::new(vec![Line::from(format!(
                "message {index}"
            ))])) as Arc<dyn HistoryCell>
        })
        .collect();
    let mut overlay = TranscriptOverlay::new(cells, app.keymap.pager.clone());
    let area = Rect::new(
        /*x*/ 0, /*y*/ 0, /*width*/ 72, /*height*/ 18,
    );
    let mut buffer = Buffer::empty(area);

    {
        let bottom_pane = app.chat_widget.as_bottom_pane_renderable();
        render_embedded_transcript_surface(&mut overlay, &bottom_pane, area, &mut buffer);
    }
    assert!(overlay.handle_embedded_key_event(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE,)));
    assert!(!overlay.is_scrolled_to_bottom());

    app.chat_widget
        .handle_key_event(KeyEvent::new(KeyCode::Char('é'), KeyModifiers::NONE));
    assert_eq!(
        app.chat_widget.composer_text_with_pending(),
        "keep typing hereé"
    );
    assert!(!overlay.is_scrolled_to_bottom());

    buffer.reset();
    let bottom_pane = app.chat_widget.as_bottom_pane_renderable();
    let bottom_area =
        render_embedded_transcript_surface(&mut overlay, &bottom_pane, area, &mut buffer);
    let cursor = bottom_pane
        .cursor_pos(bottom_area)
        .expect("interactive composer cursor");
    assert!(cursor.0 >= bottom_area.left() && cursor.0 < bottom_area.right());
    assert!(cursor.1 >= bottom_area.top() && cursor.1 < bottom_area.bottom());

    insta::assert_snapshot!(
        "interactive_transcript_with_active_composer",
        buffer_text(&buffer, area)
    );
}

#[tokio::test]
async fn backtrack_transcript_remains_modal() {
    let mut app = make_test_app().await;
    app.overlay = Some(Overlay::Transcript(TranscriptOverlay::new(
        Vec::new(),
        app.keymap.pager.clone(),
    )));

    assert!(app.embedded_transcript_active());

    app.backtrack.overlay_preview_active = true;
    assert!(!app.embedded_transcript_active());
}

fn buffer_text(buffer: &Buffer, area: Rect) -> String {
    (area.top()..area.bottom())
        .map(|y| {
            let line = (area.left()..area.right())
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>();
            line.trim_end().to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}
