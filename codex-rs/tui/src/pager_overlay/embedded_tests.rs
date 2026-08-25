use super::*;
use crate::history_cell::PlainHistoryCell;
use pretty_assertions::assert_eq;

fn overlay_with_rows(count: usize) -> TranscriptOverlay {
    let cells = (0..count)
        .map(|index| {
            Arc::new(PlainHistoryCell::new(vec![Line::from(format!(
                "message {index}"
            ))])) as Arc<dyn HistoryCell>
        })
        .collect();
    TranscriptOverlay::new(cells, crate::keymap::RuntimeKeymap::defaults().pager)
}

#[test]
fn embedded_navigation_claims_only_page_keys() {
    let mut overlay = overlay_with_rows(/*count*/ 20);
    let area = Rect::new(
        /*x*/ 0, /*y*/ 0, /*width*/ 40, /*height*/ 8,
    );
    let mut buf = Buffer::empty(area);
    overlay.render_embedded(area, &mut buf);
    let bottom_offset = overlay.view.scroll_offset;

    assert!(
        !overlay.handle_embedded_key_event(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE,))
    );
    assert_eq!(overlay.view.scroll_offset, bottom_offset);
    assert!(overlay.handle_embedded_key_event(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE,)));
    assert!(overlay.view.scroll_offset < bottom_offset);
}
