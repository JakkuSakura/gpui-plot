use gpui::{div, IntoElement, ParentElement, Styled};

pub fn centered_text(text: impl Into<String>) -> impl IntoElement {
    div()
        .flex()
        .content_center()
        .justify_center()
        .child(text.into())
}
