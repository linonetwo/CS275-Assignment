extern crate amethyst;

use amethyst::{
    assets::{PrefabLoader, PrefabLoaderSystem, RonFormat},
    core::transform::TransformBundle,
    input::{is_close_requested, is_key_down},
    prelude::*,
    renderer::{
        DisplayConfig, DrawFlat, Event, Pipeline, PosNormTex, RenderBundle, Stage, VirtualKeyCode,
    },
    utils::scene::BasicScenePrefab,
};

type MyPrefabData = BasicScenePrefab<Vec<PosNormTex>>;

struct Example;

impl<'a, 'b> State<GameData<'a, 'b>> for Example {
    fn on_start(&mut self, data: StateData<GameData>) {
        // Initialise the scene with an object, a light and a camera.
        let prefab_path = format!("{}/resources/prefab/head.ron", env!("CARGO_MANIFEST_DIR"));
        let handle = data.world.exec(|loader: PrefabLoader<MyPrefabData>| {
            loader.load(prefab_path, RonFormat, (), ())
        });
        data.world.create_entity().with(handle).build();
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: Event) -> Trans<GameData<'a, 'b>> {
        if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
            Trans::Quit
        } else {
            Trans::None
        }
    }

    fn update(&mut self, data: StateData<GameData>) -> Trans<GameData<'a, 'b>> {
        data.data.update(&data.world);
        Trans::None
    }
}

fn main() -> Result<(), amethyst::Error> {
    amethyst::start_logger(Default::default());

    let config_path = format!(
        "{}/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let config = DisplayConfig::load(&config_path);
    let resources = format!("{}/resources/", env!("CARGO_MANIFEST_DIR"));

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.00196, 0.23726, 0.21765, 1.0], 1.0)
            .with_pass(DrawFlat::<PosNormTex>::new()),
    );

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new())?
        .with_bundle(RenderBundle::new(pipe, Some(config)))?;
    let mut game = Application::new(resources, Example, game_data)?;
    game.run();
    Ok(())
}
