use widget::WidgetKind;
use text::TextWidget;
use super::Renderer;
use widget::Widget;
use crate::editor::EditorState;

pub mod widget;
pub mod text;

pub fn create_initial_ui_state(screen_size: [f32; 2]) -> Vec<WidgetKind> {
    fn white() -> [f32; 4] { [1.0, 1.0, 1.0, 1.0] }

    vec![
        WidgetKind::Text(TextWidget::new(0, String::default(), [20.0, 20.0], 20.0, white())),
        WidgetKind::Text(TextWidget::new(1, String::default(), [20.0, 40.0], 20.0, white())),
        WidgetKind::Text(TextWidget::new(2, String::default(), [20.0, 60.0], 20.0, white())),
    ]
}

pub fn update_ui(editor_state: &mut EditorState, renderer: &mut Renderer, fps: f32) {
    fn format_slice(prefix: &str, vec: &[f32]) -> String {
        let mut text = String::from(prefix);
        let len = vec.len();
        text.push('[');

        for (i, v) in vec.iter().enumerate() {
            text.push_str(v.to_string().as_str());
            if i != len - 1 {
                text.push_str(", ");
            }
        }
        text.push(']');

        text
    }
 
    let mut new_content: [String; 3] = [
        String::with_capacity(32),
        String::with_capacity(32),
        String::with_capacity(32),
    ]; 

    let pos = String::from("Position: wooooooooop");
    new_content[0] = pos;
    
    let pos = String::from("Direction: unknown?");
    new_content[1] = pos;
    
    let mut fps_str = String::from("FPS: ");
    fps_str.push_str(fps.to_string().as_str());
    new_content[2] = fps_str;

    let mut i = 0;
    for w in editor_state.widgets.iter_mut() {
        match w {
            WidgetKind::Text(text_widget) => {
                text_widget.set_content(new_content[i].as_str());
                text_widget.draw(renderer);
            },
            _ => (),
        }

        i += 1;
    }
}
