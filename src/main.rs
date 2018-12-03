extern crate amethyst;
#[macro_use]
extern crate amethyst_editor_sync;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

use amethyst_editor_sync::*;
use amethyst::{
    animation::{
        get_animation_set, AnimationBundle, AnimationCommand, AnimationSet, AnimationSetPrefab,
        DeferStartRelation, EndControl, StepDirection,
    },
    assets::{PrefabLoader, PrefabLoaderSystem, Processor, RonFormat},
    audio::{output::init_output, Source},
    core::{Time, Transform, TransformBundle},
    ecs::prelude::*,
    input::{get_key, is_close_requested, is_key_down, InputBundle},
    prelude::*,
    renderer::{DrawShaded, ElementState, PosNormTex, VirtualKeyCode},
    shrev::{EventChannel, ReaderId},
    ui::{UiBundle, UiCreator, UiEvent, UiFinder, UiText},
    utils::{
        fps_counter::{FPSCounter, FPSCounterBundle},
        scene::BasicScenePrefab,
    },
};

#[derive(Eq, PartialOrd, PartialEq, Hash, Debug, Copy, Clone, Deserialize, Serialize)]
enum AnimationId {
    Scale,
    Rotate,
    Translate,
}

type MyPrefabData = (
    BasicScenePrefab<Vec<PosNormTex>>,
    Option<AnimationSetPrefab<AnimationId, Transform>>,
);

struct EnterScene {
    fps_display: Option<Entity>,
    head: Option<Entity>,
    rate: f32,
    current_animation: AnimationId,
}

impl Default for EnterScene {
    fn default() -> Self {
        EnterScene {
            head: None,
            fps_display: None,
            rate: 1.0,
            current_animation: AnimationId::Rotate,
        }
    }
}

impl<'a, 'b> SimpleState<'a, 'b> for EnterScene {
    fn on_start(&mut self, state_data: StateData<GameData>) {
        let StateData { world, .. } = state_data;
        // Initialise the scene with an object, a light and a camera.
        let prefab_path = format!("{}/resources/prefab/head.ron", env!("CARGO_MANIFEST_DIR"));
        let head_prefab_handle = world
            .exec(|loader: PrefabLoader<MyPrefabData>| loader.load(prefab_path, RonFormat, (), ()));
        self.head = Some(world.create_entity().with(head_prefab_handle).build());

        init_output(&mut world.res);
        let keyframe_control_path = format!(
            "{}/resources/ui/keyframe_control.ron",
            env!("CARGO_MANIFEST_DIR")
        );
        world.exec(|mut creator: UiCreator| {
            creator.create(keyframe_control_path, ());
        });
    }

    fn handle_event(
        &mut self,
        state_data: StateData<GameData>,
        event: StateEvent,
    ) -> SimpleTrans<'a, 'b> {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                    return Trans::Quit;
                }
                let StateData { world, .. } = state_data;
                match get_key(&event) {
                    Some((VirtualKeyCode::Space, ElementState::Pressed)) => {
                        add_animation(
                            world,
                            self.head.unwrap(),
                            self.current_animation,
                            self.rate,
                            None,
                            true,
                        );
                    }

                    Some((VirtualKeyCode::D, ElementState::Pressed)) => {
                        add_animation(
                            world,
                            self.head.unwrap(),
                            AnimationId::Translate,
                            self.rate,
                            None,
                            false,
                        );
                        add_animation(
                            world,
                            self.head.unwrap(),
                            AnimationId::Rotate,
                            self.rate,
                            Some((AnimationId::Translate, DeferStartRelation::End)),
                            false,
                        );
                        add_animation(
                            world,
                            self.head.unwrap(),
                            AnimationId::Scale,
                            self.rate,
                            Some((AnimationId::Rotate, DeferStartRelation::Start(0.666))),
                            false,
                        );
                    }

                    Some((VirtualKeyCode::F, ElementState::Pressed)) => {
                        print!("{:?}", self.head.unwrap().clone());
                    }

                    Some((VirtualKeyCode::Left, ElementState::Pressed)) => {
                        get_animation_set::<AnimationId, Transform>(
                            &mut world.write_storage(),
                            self.head.unwrap().clone(),
                        ).unwrap()
                        .step(self.current_animation, StepDirection::Backward);
                    }

                    Some((VirtualKeyCode::Right, ElementState::Pressed)) => {
                        get_animation_set::<AnimationId, Transform>(
                            &mut world.write_storage(),
                            self.head.unwrap().clone(),
                        ).unwrap()
                        .step(self.current_animation, StepDirection::Forward);
                    }

                    _ => {}
                };
                Trans::None
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

    // devtool setup
    let components = type_set![];
    let resources = type_set![];
    let editor_bundle = SyncEditorBundle::new()
        .sync_default_types()
        .sync_components(&components)
        .sync_resources(&resources);

    let game_data = GameDataBuilder::default()
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "", &[])
        .with_bundle(AnimationBundle::<AnimationId, Transform>::new(
            "head_animation_control_system",
            "head_sampler_interpolation_system",
        ))?.with_bundle(TransformBundle::new().with_dep(&["head_sampler_interpolation_system"]))?
        .with_bundle(FPSCounterBundle::default())?
        .with_bundle(InputBundle::<String, String>::new())?
        .with_bundle(UiBundle::<String, String>::new())?
        .with(Processor::<Source>::new(), "source_processor", &[])
        .with(UiEventHandlerSystem::new(), "ui_event_handler", &[])
        .with_basic_renderer(display_config_path, DrawShaded::<PosNormTex>::new(), true)?;
    let mut game = Application::new(resources, EnterScene::default(), game_data)?;
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

fn add_animation(
    world: &mut World,
    entity: Entity,
    id: AnimationId,
    rate: f32,
    defer: Option<(AnimationId, DeferStartRelation)>,
    toggle_if_exists: bool,
) {
    let animation = world
        .read_storage::<AnimationSet<AnimationId, Transform>>()
        .get(entity)
        .and_then(|s| s.get(&id))
        .cloned()
        .unwrap();
    let mut sets = world.write_storage();
    let control_set = get_animation_set::<AnimationId, Transform>(&mut sets, entity).unwrap();
    match defer {
        None => {
            if toggle_if_exists && control_set.has_animation(id) {
                control_set.toggle(id);
            } else {
                control_set.add_animation(
                    id,
                    &animation,
                    EndControl::Normal,
                    rate,
                    AnimationCommand::Start,
                );
            }
        }

        Some((defer_id, defer_relation)) => {
            control_set.add_deferred_animation(
                id,
                &animation,
                EndControl::Normal,
                rate,
                AnimationCommand::Start,
                defer_id,
                defer_relation,
            );
        }
    }
}
