use widget::WidgetKind;
use text::TextWidget;

pub mod widget;
pub mod text;

pub fn create_initial_ui_state(screen_size: [f32; 2]) -> Vec<WidgetKind> {
    fn white() -> [f32; 4] {
        [1.0, 1.0, 1.0, 1.0]
    }

    vec![
        WidgetKind::Text(TextWidget::new(0, String::default(), [20.0, 20.0], 20.0, white())),
        WidgetKind::Text(TextWidget::new(1, String::default(), [20.0, 40.0], 20.0, white())),
        WidgetKind::Text(TextWidget::new(2, String::default(), [20.0, 60.0], 20.0, white())),
    ]
}
