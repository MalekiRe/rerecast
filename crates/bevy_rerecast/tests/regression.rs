#![allow(missing_docs)]

use std::time::Instant;

use bevy::{
    asset::AssetPlugin,
    camera::{primitives::Aabb, visibility::VisibilityPlugin},
    ecs::system::RunSystemOnce,
    gltf::GltfPlugin,
    log::LogPlugin,
    math::bounding::Aabb3d,
    prelude::*,
    render::mesh::MeshPlugin,
    scene::{SceneInstanceReady, ScenePlugin},
};
use bevy_rerecast::{Mesh3dBackendPlugin, debug::NavmeshDebugPlugin, prelude::*};
use bevy_rerecast_editor_integration::NavmeshEditorIntegrationPlugin;

#[test]
fn gltf_generation() {
    let mut app = App::new_test();
    let gltf_handle = app.world().load_asset("models/dungeon.glb#Scene0");
    app.world_mut().spawn(SceneRoot(gltf_handle)).observe(
        |_: On<SceneInstanceReady>, mut commands: Commands| {
            commands.insert_resource(GltfLoaded);
        },
    );

    let now = Instant::now();
    while app.world().get_resource::<GltfLoaded>().is_none() {
        app.update();
        if now.elapsed().as_secs() > 5 {
            panic!("Timeout waiting for glTF to load");
        }
    }
    let navmesh_handle = app.generate_navmesh(NavmeshSettings::default());
    let navmesh = app.get_navmesh(&navmesh_handle);
    let expected_navmesh = app.read_navmesh("test/dungeon/navmesh.nav");

    assert_eq!(
        expected_navmesh.polygon, navmesh.polygon,
        "Generated polygon navmesh does not match reference"
    );

    assert_eq!(
        expected_navmesh.detail, navmesh.detail,
        "Generated detail navmesh does not match reference"
    );
}

#[test]
fn primitive_2d_regeneration() {
    let mut app = App::new_test();
    let ground_handle = app
        .world_mut()
        .resource_mut::<Assets<Mesh>>()
        .add(Cuboid::new(1000.0, 1000.0, 1.0));
    let cube_handle = app
        .world_mut()
        .resource_mut::<Assets<Mesh>>()
        .add(Cuboid::new(10.0, 10.0, 10.0));
    app.world_mut().spawn(Mesh3d(ground_handle));
    let cube_entity = app.world_mut().spawn(Mesh3d(cube_handle)).id();

    let settings = NavmeshSettings {
        aabb: Some(Aabb3d::new(Vec3::ZERO, Vec3::new(100.0, 100.0, 5.0))),
        ..NavmeshSettings::from_agent_2d(5.0, 2.0)
    };
    let navmesh_handle = app.generate_navmesh(settings.clone());
    let navmesh = app.get_navmesh(&navmesh_handle);
    let expected_navmesh = app.read_navmesh("test/primitives/navmesh_1.nav");

    assert_eq!(
        expected_navmesh.polygon, navmesh.polygon,
        "Initial generated polygon navmesh does not match reference"
    );

    assert_eq!(
        expected_navmesh.detail, navmesh.detail,
        "Initial generated detail navmesh does not match reference"
    );

    app.world_mut().despawn(cube_entity);
    app.regenerate_navmesh(&navmesh_handle, settings);
    app.wait_for_navmesh_ready(&navmesh_handle);
    let navmesh = app.get_navmesh(&navmesh_handle);
    let expected_navmesh = app.read_navmesh("test/primitives/navmesh_2.nav");

    assert_eq!(
        expected_navmesh.polygon, navmesh.polygon,
        "Regenerated polygon navmesh does not match reference"
    );

    assert_eq!(
        expected_navmesh.detail, navmesh.detail,
        "Regenerated detail navmesh does not match reference"
    );
}

#[derive(Resource)]
struct GltfLoaded;

trait TestApp {
    fn generate_navmesh(&mut self, settings: NavmeshSettings) -> Handle<Navmesh>;
    fn get_navmesh(&mut self, handle: &Handle<Navmesh>) -> Navmesh;
    fn regenerate_navmesh(&mut self, handle: &Handle<Navmesh>, settings: NavmeshSettings) -> bool;
    fn wait_for_navmesh_ready(&mut self, handle: &Handle<Navmesh>);
    fn read_navmesh(&mut self, path: &str) -> Navmesh;
    fn new_test() -> App;
}

impl TestApp for App {
    fn generate_navmesh(&mut self, settings: NavmeshSettings) -> Handle<Navmesh> {
        self.world_mut()
            .run_system_once(move |mut genenerator: NavmeshGenerator| {
                genenerator.generate(settings.clone())
            })
            .unwrap()
    }

    fn get_navmesh(&mut self, handle: &Handle<Navmesh>) -> Navmesh {
        let now = Instant::now();
        loop {
            if let Some(navmesh) = self.world().resource::<Assets<Navmesh>>().get(handle) {
                break navmesh.clone();
            }
            self.update();
            if now.elapsed().as_secs() > 5 {
                panic!("Timeout waiting for generating initial navmesh");
            }
        }
    }
    fn read_navmesh(&mut self, path: &str) -> Navmesh {
        let expected_navmesh: Handle<Navmesh> = self
            .world()
            .resource::<AssetServer>()
            .load(path.to_string());
        let now = Instant::now();
        loop {
            self.update();
            if let Some(navmesh) = self
                .world()
                .resource::<Assets<Navmesh>>()
                .get(&expected_navmesh)
                .cloned()
            {
                self.world_mut().remove_resource::<NavmeshReadyResource>();
                break navmesh;
            }
            if now.elapsed().as_secs() > 5 {
                panic!("Timeout waiting for reading reference navmesh");
            }
        }
    }
    fn new_test() -> App {
        let mut app = App::new();
        app.add_plugins(headless_plugins);

        app.add_plugins((
            NavmeshPlugins::default()
                .build()
                .disable::<NavmeshDebugPlugin>()
                .disable::<NavmeshEditorIntegrationPlugin>(),
            Mesh3dBackendPlugin::default(),
        ));

        app.finish();
        app.cleanup();
        app.add_observer(|trigger: On<NavmeshReady>, mut commands: Commands| {
            commands.insert_resource(NavmeshReadyResource(trigger.event().0));
        });
        app
    }

    fn wait_for_navmesh_ready(&mut self, handle: &Handle<Navmesh>) {
        loop {
            if let Some(navmesh_ready_resource) = self
                .world_mut()
                .get_resource::<NavmeshReadyResource>()
                .cloned()
                && navmesh_ready_resource.0 == handle.id()
            {
                self.world_mut().remove_resource::<NavmeshReadyResource>();
                break;
            }
            self.update();
        }
    }

    fn regenerate_navmesh(&mut self, handle: &Handle<Navmesh>, settings: NavmeshSettings) -> bool {
        let handle = handle.clone();
        self.world_mut()
            .run_system_once(move |mut generator: NavmeshGenerator| {
                generator.regenerate(&handle, settings.clone())
            })
            .unwrap()
    }
}

#[derive(Debug, Resource, Clone, PartialEq, Eq, Hash, Deref, DerefMut)]
struct NavmeshReadyResource(AssetId<Navmesh>);

#[allow(dead_code)]
fn write_navmesh_to_file(navmesh: &Navmesh, file_path: &str) {
    let bincode = bincode::serde::encode_to_vec(navmesh, bincode::config::standard()).unwrap();
    std::fs::write(file_path, bincode).unwrap();
}

fn headless_plugins(app: &mut App) {
    app.add_plugins((
        MinimalPlugins,
        LogPlugin::default(),
        AssetPlugin {
            file_path: "../../assets".to_string(),
            ..default()
        },
        ScenePlugin,
        MeshPlugin,
        TransformPlugin,
        VisibilityPlugin,
        GltfPlugin::default(),
    ))
    .init_asset::<StandardMaterial>()
    .init_asset::<Mesh>()
    .register_type::<Visibility>()
    .register_type::<InheritedVisibility>()
    .register_type::<ViewVisibility>()
    .register_type::<Aabb>()
    .register_type::<MeshMaterial3d<StandardMaterial>>();
}
