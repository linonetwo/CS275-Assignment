extern crate amethyst;
#[macro_use]
extern crate log;

use amethyst::{
    assets::{PrefabLoader, PrefabLoaderSystem, RonFormat, Processor},
    audio::{output::init_output, Source},
    core::{transform::TransformBundle, Time},
    ecs::prelude::{Entity, System, Write},
    input::{is_close_requested, is_key_down, InputBundle},
    prelude::*,
    renderer::{
        DrawShaded, PosNormTex, VirtualKeyCode,
    },
    shrev::{EventChannel, ReaderId},
    ui::{UiBundle, UiCreator, UiEvent, UiFinder, UiText},
    utils::{
        fps_counter::{FPSCounter, FPSCounterBundle},
        scene::BasicScenePrefab,
    },
};

type MyPrefabData = BasicScenePrefab<Vec<PosNormTex>>;

struct EnterScene {
    fps_display: Option<Entity>,
}

impl<'a, 'b> SimpleState<'a, 'b> for EnterScene {
    fn on_start(&mut self, state_data: StateData<GameData>) {
        let StateData { world, .. } = state_data;
        // Initialise the scene with an object, a light and a camera.
        let prefab_path = format!("{}/resources/prefab/head.ron", env!("CARGO_MANIFEST_DIR"));
        let handle = world
            .exec(|loader: PrefabLoader<MyPrefabData>| loader.load(prefab_path, RonFormat, (), ()));
        world.create_entity().with(handle).build();

        init_output(&mut world.res);
        let ui_path = format!("{}/resources/ui/example.ron", env!("CARGO_MANIFEST_DIR"));
        world.exec(|mut creator: UiCreator| {
            creator.create(ui_path, ());
        });
    }

    fn handle_event(&mut self, _: StateData<GameData>, event: StateEvent) -> SimpleTrans<'a, 'b> {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                    Trans::Quit
                } else {
                    Trans::None
                }
            }
            StateEvent::Ui(ui_event) => {
                info!(
                    "[HANDLE_EVENT] You just interacted with a ui element: {:?}",
                    ui_event
                );
                Trans::None
            }
        }
    }

    fn update(&mut self, state_data: &mut StateData<GameData>) -> SimpleTrans<'a, 'b> {
        let StateData { world, data } = state_data;
        data.update(&world);
        if self.fps_display.is_none() {
            world.exec(|finder: UiFinder| {
                if let Some(entity) = finder.find("fps") {
                    self.fps_display = Some(entity);
                }
            });
        }
        let mut ui_text = world.write_storage::<UiText>();
        if let Some(fps_display) = self.fps_display.and_then(|entity| ui_text.get_mut(entity)) {
            if world.read_resource::<Time>().frame_number() % 20 == 0 {
                let fps = world.read_resource::<FPSCounter>().sampled_fps();
                fps_display.text = format!("FPS: {:.*}", 2, fps);
            }
        }

        Trans::None
    }
}

fn main() -> Result<(), amethyst::Error> {
    amethyst::start_logger(Default::default());

    let display_config_path = format!(
        "{}/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );
    let resources = format!("{}/resources/", env!("CARGO_MANIFEST_DIR"));

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(TransformBundle::new())?
        .with(Processor::<Source>::new(), "source_processor", &[])
        .with(UiEventHandlerSystem::new(), "ui_event_handler", &[])
        .with_bundle(FPSCounterBundle::default())?
        .with_bundle(InputBundle::<String, String>::new())?
        .with_bundle(UiBundle::<String, String>::new())?
        .with_basic_renderer(display_config_path, DrawShaded::<PosNormTex>::new(), true)?;
    let mut game = Application::new(resources, EnterScene { fps_display: None }, game_data)?;
    game.run();
    Ok(())
}

pub struct UiEventHandlerSystem {
    reader_id: Option<ReaderId<UiEvent>>,
}

impl UiEventHandlerSystem {
    pub fn new() -> Self {
        UiEventHandlerSystem { reader_id: None }
    }
}

impl<'a> System<'a> for UiEventHandlerSystem {
    type SystemData = Write<'a, EventChannel<UiEvent>>;

    fn run(&mut self, mut events: Self::SystemData) {
        if self.reader_id.is_none() {
            self.reader_id = Some(events.register_reader());
        }

        // Reader id was just initialized above if empty
        for ev in events.read(self.reader_id.as_mut().unwrap()) {
            info!("[SYSTEM] You just interacted with a ui element: {:?}", ev);
        }
    }
}
