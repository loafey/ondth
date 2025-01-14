use bevy::{
    color::Color,
    prelude::{Bundle, Button, Component, Query, Text, With},
    text::TextFont,
    ui::{BackgroundColor, BorderColor, Interaction, Node, UiRect, Val},
};
use i_cant_believe_its_not_bsn::WithChild;

#[derive(Component, Default)]
pub struct MenuButtonMarker;

#[derive(Bundle, Default)]
pub struct MenuButton<Event: Component> {
    node: Node,
    button: Button,
    background_color: BackgroundColor,
    event: Event,
    marker: MenuButtonMarker,
    border: BorderColor,
    children: WithChild<(TextFont, Text)>,
}
impl<Event: Component> MenuButton<Event> {
    pub fn new(
        text: impl Into<String>,
        font_size: Option<f32>,
        border: Option<f32>,
        padding: Option<f32>,
        event: Event,
    ) -> Self {
        Self {
            node: Node {
                padding: UiRect::all(Val::Px(padding.unwrap_or(10.0))),
                border: UiRect::left(Val::Px(border.unwrap_or(5.0))),
                ..Default::default()
            },
            button: Button,
            event,
            border: BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            marker: MenuButtonMarker,
            children: WithChild((
                TextFont {
                    font_size: font_size.unwrap_or(20.0),
                    ..Default::default()
                },
                Text(text.into()),
            )),
        }
    }
}
pub fn update(
    mut query: Query<
        (&Interaction, &mut BorderColor, &mut BackgroundColor),
        With<MenuButtonMarker>,
    >,
) {
    for (int, mut border, mut bc) in &mut query {
        match int {
            Interaction::Pressed => {
                border.0 = Color::WHITE;
                bc.0 = Color::srgba(1.0, 1.0, 1.0, 0.05);
            }
            Interaction::Hovered => {
                border.0 = Color::WHITE;
                bc.0 = Color::srgba(1.0, 1.0, 1.0, 0.025);
            }
            Interaction::None => {
                border.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
                bc.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
            }
        }
    }
}
