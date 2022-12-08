use bevy::{prelude::*, text::Text2dBounds};
use std::marker::PhantomData;

use crate::{
    assets::DefaultAssets,
    connection::ConnectionEvent,
    node::{Node, NodeSet},
    widget::{SlotWidget, Widget, WidgetPlugin},
};

pub trait InputWidgetValue {
    fn backspace(&mut self);
    fn on_input(&mut self, c: char);
    fn peek(&self) -> String;
}

#[derive(Default)]
pub struct InputWidgetPlugin<N: NodeSet>(PhantomData<N>);

impl<N: NodeSet> Plugin for InputWidgetPlugin<N>
where
    N: SlotWidget<N, InputWidget<N>>,
    N::NodeIO: InputWidgetValue,
{
    fn build(&self, app: &mut App) {
        app.add_plugin(WidgetPlugin::<N, InputWidget<N>>::default())
            .add_system(input_widget_input::<N>)
            .add_system(input_widget_value::<N>);
    }
}

#[derive(Clone, Component, Default)]
pub struct InputWidget<N: NodeSet> {
    pub active: bool,
    pub dirty: bool,
    pub size: Vec2,
    pub text_entity: Option<Entity>,
    pub value: N::NodeIO,
}

impl<N: NodeSet> Widget<N> for InputWidget<N> {
    fn build(
        &mut self,
        entity: Entity,
        commands: &mut Commands,
        area: Vec2,
        assets: &Res<DefaultAssets>,
    ) {
        let text_style_title = TextStyle {
            font: assets.font.clone(),
            font_size: 16.0,
            color: Color::BLACK,
        };

        self.size = area;

        let child = commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(self.size),
                    ..default()
                },
                ..default()
            })
            .with_children(|parent| {
                let text_entity = parent
                    .spawn(Text2dBundle {
                        text: Text::from_section("", text_style_title),
                        text_2d_bounds: Text2dBounds { size: self.size },
                        transform: Transform::from_xyz(-self.size.x / 2.0, self.size.y / 2.0, 2.0),
                        ..default()
                    })
                    .id();

                self.text_entity = Some(text_entity);
            })
            .id();

        commands.entity(entity).push_children(&[child]);
    }

    fn blur(&mut self) {
        self.active = false;
    }

    fn can_click(&self) -> bool {
        true
    }

    fn clean(&mut self) {
        self.dirty = false;
    }

    fn dirty(&self) -> bool {
        self.dirty
    }

    fn focus(&mut self) {
        self.active = true;
    }

    fn get_value(&self) -> N::NodeIO {
        self.value.clone()
    }

    fn set_value(&mut self, value: N::NodeIO) {
        self.value = value.clone();
    }

    fn size(&self) -> Vec2 {
        self.size
    }
}

fn input_widget_input<N: NodeSet>(
    mut ev_char: EventReader<ReceivedCharacter>,
    mut query: Query<&mut InputWidget<N>>,
) where
    N::NodeIO: InputWidgetValue,
{
    const BACKSPACE: char = '\u{0008}';

    for ev in ev_char.iter() {
        for mut widget in query.iter_mut() {
            if widget.active {
                widget.dirty = true;

                if ev.char.is_ascii_graphic() {
                    widget.value.on_input(ev.char);
                } else if ev.char == BACKSPACE {
                    widget.value.backspace();
                }
            }
        }
    }
}

fn input_widget_value<N: NodeSet>(
    mut ev_conn: EventWriter<ConnectionEvent>,
    mut q_node: Query<&mut Node<N>>,
    mut q_widget: Query<(&Parent, &mut InputWidget<N>)>,
    mut q_text: Query<&mut Text>,
) where
    N: SlotWidget<N, InputWidget<N>>,
    N::NodeIO: InputWidgetValue,
{
    for (parent, mut widget) in q_widget.iter_mut() {
        if widget.dirty() {
            widget.clean();

            if let Some(entity) = widget.text_entity {
                if let Ok(mut text) = q_text.get_mut(entity) {
                    text.sections[0].value = widget.get_value().peek();
                }
            }

            if let Ok(mut node) = q_node.get_mut(parent.get()) {
                (*node).set_value(widget.get_value());
                ev_conn.send(ConnectionEvent::Propagate);
            }
        }
    }
}