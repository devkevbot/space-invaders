use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ExamplePlugin)
        .run();
}

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Age(u8);

fn add_people(mut commands: Commands) {
    commands.spawn((Person, Age(18)));
    commands.spawn((Person, Age(32)));
    commands.spawn((Person, Age(7)));
}

#[derive(Resource)]
struct SayAgeTimer(Timer);

fn say_age(time: Res<Time>, mut timer: ResMut<SayAgeTimer>, query: Query<&Age, With<Person>>) {
    if timer.0.tick(time.delta()).just_finished() {
        for age in &query {
            println!("My age is {}!", age.0)
        }
    }
}

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SayAgeTimer(Timer::from_seconds(5.0, TimerMode::Repeating)))
            .add_startup_system(add_people)
            .add_system(say_age);
    }
}
