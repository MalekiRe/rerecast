//! The editor for the Navmesh plugin.

use bevy::{
    ecs::error::warn, feathers::FeathersPlugins, input_focus::InputDispatchPlugin, prelude::*,
};
use bevy_rerecast::prelude::*;
use bevy_ui_text_input::TextInputPlugin;

extern crate alloc;

mod backend;
mod camera;
mod get_navmesh_input;
mod load;
mod save;
mod theme;
mod ui;
mod visualization;

fn main() -> AppExit {
    App::new()
        .set_error_handler(warn)
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Rerecast Navmesh Editor".to_string(),
                    ..default()
                }),
                ..default()
            }),
            // InputDispatchPlugin is also added by TextInputPlugin
            FeathersPlugins.build().disable::<InputDispatchPlugin>(),
        ))
        .add_plugins((NavmeshPlugins::default(), TextInputPlugin))
        .add_plugins((
            camera::plugin,
            get_navmesh_input::plugin,
            ui::plugin,
            theme::plugin,
            visualization::plugin,
            backend::plugin,
            save::plugin,
            load::plugin,
        ))
        .run()
}
