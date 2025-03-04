//! This examples illustrates the different ways you can employ component lifecycle hooks

use bevy::ecs::component::{ComponentInfo, TableStorage};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
struct MyComponent(KeyCode);

impl Component for MyComponent {
    type Storage = TableStorage;

    /// Hooks can also be registered during component initialisation by
    /// implementing `init_component_info`
    fn init_component_info(_info: &mut ComponentInfo) {
        // Register hooks...
    }
}

#[derive(Resource, Default, Debug, Deref, DerefMut)]
struct MyComponentIndex(HashMap<KeyCode, Entity>);

#[derive(Event)]
struct MyEvent;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, trigger_hooks)
        .init_resource::<MyComponentIndex>()
        .add_event::<MyEvent>()
        .run();
}

fn setup(world: &mut World) {
    // In order to register component hooks the component must:
    // - not belong to any created archetypes
    // - not already have a hook of that kind registered
    // This is to prevent overriding hooks defined in plugins and other crates as well as keeping things fast
    world
        .register_component_hooks::<MyComponent>()
        // There are 3 component lifecycle hooks: `on_add`, `on_insert` and `on_remove`
        // A hook has 3 arguments:
        // - a `DeferredWorld`, this allows access to resource and component data as well as `Commands`
        // - the entity that triggered the hook
        // - the component id of the triggering component, this is mostly used for dynamic components
        //
        // `on_add` will trigger when a component is inserted onto an entity without it
        .on_add(|mut world, entity, component_id| {
            // You can access component data from within the hook
            let value = world.get::<MyComponent>(entity).unwrap().0;
            println!(
                "Component: {:?} added to: {:?} with value {:?}",
                component_id, entity, value
            );
            // Or access resources
            world
                .resource_mut::<MyComponentIndex>()
                .insert(value, entity);
            // Or send events
            world.send_event(MyEvent);
        })
        // `on_insert` will trigger when a component is inserted onto an entity,
        // regardless of whether or not it already had it and after `on_add` if it ran
        .on_insert(|world, _, _| {
            println!("Current Index: {:?}", world.resource::<MyComponentIndex>());
        })
        // `on_remove` will trigger when a component is removed from an entity,
        // since it runs before the component is removed you can still access the component data
        .on_remove(|mut world, entity, component_id| {
            let value = world.get::<MyComponent>(entity).unwrap().0;
            println!(
                "Component: {:?} removed from: {:?} with value {:?}",
                component_id, entity, value
            );
            world.resource_mut::<MyComponentIndex>().remove(&value);
            // You can also issue commands through `.commands()`
            world.commands().entity(entity).despawn();
        });
}

fn trigger_hooks(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    index: Res<MyComponentIndex>,
) {
    for (key, entity) in index.iter() {
        if !keys.pressed(*key) {
            commands.entity(*entity).remove::<MyComponent>();
        }
    }
    for key in keys.get_just_pressed() {
        commands.spawn(MyComponent(*key));
    }
}
