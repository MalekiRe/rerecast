# Rerecast
[![crates.io](https://img.shields.io/crates/v/rerecast)](https://crates.io/crates/rerecast)
[![docs.rs](https://docs.rs/rerecast/badge.svg)](https://docs.rs/rerecast)

![`rerecast` logo](https://raw.githubusercontent.com/janhohenheim/rerecast/refs/heads/main/media/logo.svg)

Rust port of [Recast](https://github.com/recastnavigation/recastnavigation), the industry-standard navigation mesh generator used
by Unreal, Unity, Godot, and other game engines.

## What's a Navmesh?!

A navmesh is a mesh that says where in the level a character can walk. This information is typically used for pathfinding.
Rerecast brings navmeshes in two flavors:

- **Polygon Mesh**: A simplified mesh made up of polygons that is quick to use for pathfinding
- **Detail Mesh**: A triangle mesh that is more detailed and can be optionally used to refine pathfinding, especially for vertical slopes and stairs.

A typical detail mesh looks like this:

![detail mesh](https://github.com/janhohenheim/rerecast/blob/main/media/editor.png?raw=true)

As you can see, it does not perfectly follow terrain, but is a very good approximation for pathfinding.

## Usage

### Raw Rerecast

Rerecast's API is fairly low level. As such, it's best if your game engine of choice provides an idiomatic interface to it.
If you want to build such an interface on your own, or want to use Rerecast directly in general, check out the [cpp comparison automated test](https://github.com/janhohenheim/rerecast/blob/main/crates/rerecast/tests/cpp_comparison.rs).

### Bevy Rerecast

To use `bevy_rerecast`, add it to your dependencies:
```bash
cargo add bevy_rerecast
```

Then add the plugin to your app:
```rust,no_run
use bevy::prelude::*;
use bevy_rerecast::prelude::*;

App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(NavmeshPlugins::default());
```

The next step is to provide a *backend*. Backends decide how the current Bevy scene should be translated into a list of trimeshes that rerecast can use. There's a builtin backend called the [`Mesh3dBackendPlugin`], which will use your entities holding a [`Mesh3d`] as obstacles. We will use it in this example, but your own code should usually use a physics engine's backend instead. See the section [Backends](#backends) for more.

To add a backend, add its plugin after the [`NavmeshPlugins`]:

```rust,no_run
use bevy::prelude::*;
use bevy_rerecast::prelude::*;
use bevy_rerecast::Mesh3dBackendPlugin;

App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(NavmeshPlugins::default())
    .add_plugins(Mesh3dBackendPlugin::default());
```

Now that you've added the backend, you're ready to create a navmesh. To do this, use the [`NavmeshGenerator`] query parameter:

```rust
use bevy::prelude::*;
use bevy_rerecast::prelude::*;

fn some_system_that_generates_your_navmesh(mut generator: NavmeshGenerator) {
    let agent_radius = 0.6;
    let agent_height = 1.8;
    let settings = NavmeshSettings::from_agent_3d(agent_radius, agent_height);
    let navmesh_handle = generator.generate(settings);

    // Now store the navmesh handle somewhere, like a resource,
    // so it doesn't get dropped again!
}
```

The navmesh will be generated in the background, so the `Handle<Navmesh>` you received will not immediately point to an available navmesh.
If you want to know exactly when the navmesh is ready, you can set up a [`NavmeshReady`] observer:

```rust
use bevy::prelude::*;
use bevy_rerecast::prelude::*;

fn on_navmesh_ready(trigger: Trigger<NavmeshReady>, navmeshes: Res<Assets<Navmesh>>) {
    let asset_id = trigger.event().0;

    // We can now safely fetch the navmesh from our assets:
    let navmesh = navmeshes.get(asset_id).unwrap();
}
```

If you need to regenerate a navmesh because the environment has changed, use [`NavmeshGenerator::regenerate`]. Once the navmesh was regenerated, you can observe a [`NavmeshReady`] trigger.

Take a look at the [`examples`](https://github.com/janhohenheim/rerecast/tree/main/examples/examples) directory to see all of this in action!

### Editor

![editor demo](https://github.com/janhohenheim/rerecast/raw/refs/heads/main/media/demo.mp4)


https://github.com/user-attachments/assets/f1c62fbe-cd76-4dc7-9c88-574e241e0a6d


Tweaking navmesh settings by hand and restarting the game to see the changes is a very inefficient way to iterate on your game.
Instead, navmeshes are often authored in advanced. To do this, the Bevy integration comes with an editor to help you out.
To use it, you must enable Bevy's BRP functionality, which is a way for Bevy processes to communicate over HTTP. To do this, enable Bevy's `remote` feature and add the [`RemotePlugin`] and [`RemoteHttpPlugin`] to your app:

```rust,no_run
use bevy::prelude::*;
use bevy::remote::{RemotePlugin, http::RemoteHttpPlugin};
use bevy_rerecast::prelude::*;
use bevy_rerecast::Mesh3dBackendPlugin;

App::new()
    .add_plugins(DefaultPlugins)
    // Enable BRP
    .add_plugins((RemotePlugin::default(), RemoteHttpPlugin::default()))
    // Enable Rerecast
    .add_plugins(NavmeshPlugins::default())
    // Also add some backend, for example the `Mesh3dBackendPlugin`
    .add_plugins(Mesh3dBackendPlugin::default());
```

Next, download the editor by entering the following command in your terminal:

```bash
cargo install bevy_rerecast_editor
```

And then run it:

```bash
bevy_rerecast_editor
```

Now, when you start your game, you can load the current level into the editor, tweak the navmesh, and save it into a `.nav` file that you can load into your game.

## Third-Party Integration

### Backends

The recommended way to use the navmesh generator is with a physics engine backend. That way, the generated navmesh will match the physics engine's collision geometry. Currently, the only supported physics engine is [Avian](https://github.com/Jondolf/avian). To use its backend, add the `avian_rerecast` crate to your project:

```bash
cargo add avian_rerecast
```

and then register its backend:

```rust,ignore
use bevy::prelude::*;
use bevy_rerecast::prelude::*;
use avian_rerecast::prelude::*;

App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(NavmeshPlugins::default())
    .add_plugins(AvianBackendPlugin::default());
```

The avian backend will consider colliders that are part of a static rigid body as obstacles.

Creating your own backend is *very* easy. Take a look at the implementation of the [`AvianBackendPlugin`] as an example.

### Pathfinding

When you have your navmesh, you'll need to feed it to some pathfinding library or algorithm to actually do anything.
The following pathfinding libraries are supported:

- [vleue_navigator](https://github.com/vleue/vleue_navigator)
- [landmass](https://github.com/andriydev/landmass)

Take a look at their repos for documentation on how to use them with rerecast.

## Features & Roadmap

### Rerecast

- [x] Generate polygon mesh
- [x] Generate detail meshes
- [ ] Generate tiles
- Partitioning
  - [x] Watershed
  - [ ] Monotone
  - [ ] Layer
- [x] `no_std` support
- [x] cross-platform determinism (use `libm` feature)

### Bevy Integration

- Editor
  - [x] Extract meshes from running game
  - [x] Configure navmesh generation
  - [ ] Advanced config
  - [x] Visualize navmesh
  - [x] Save navmesh
  - [ ] Load navmesh config
- API
  - [x] Optional editor communication
  - [x] Generate navmeshes on demand
  - [x] Fully regenerate navmeshes
  - [ ] Partially regenerate navmeshes
  - [ ] `no_std` support for `bevy_rerecast_core`
    - Technically available, but of limited use until `bevy_asset` supports `no_std`
  - [x] cross-platform determinism (use `libm` feature)

## Compatibility

| bevy  | bevy_rerecast |
|-------|---------------|
| 0.16  | 0.1           |


[`AvianBackendPlugin`]: https://docs.rs/avian_rerecast/latest/avian_rerecast/struct.AvianBackendPlugin.html
[`RemotePlugin`]: https://docs.rs/bevy/latest/bevy/remote/struct.RemotePlugin.html
[`RemoteHttpPlugin`]: https://docs.rs/bevy/latest/bevy/remote/http/struct.RemoteHttpPlugin.html
[`NavmeshReady`]: https://docs.rs/bevy_rerecast/latest/bevy_rerecast/generator/struct.NavmeshReady.html
[`NavmeshGenerator`]: https://docs.rs/bevy_rerecast/latest/bevy_rerecast/generator/struct.NavmeshGenerator.html
[`NavmeshGenerator::regenerate`]: https://docs.rs/bevy_rerecast/latest/bevy_rerecast/generator/struct.NavmeshGenerator.html#tymethod.regenerate
[`Mesh3d`]: https://docs.rs/bevy/latest/bevy/prelude/struct.Mesh3d.html
