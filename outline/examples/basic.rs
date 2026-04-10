use bevy::prelude::*;
use bevy_shell_outline::{
    GpmoOutlinePlugin, OutlineAssets, OutlineShell, spawn_default_outline_mesh,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, GpmoOutlinePlugin::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, pulse_outline)
        .run();
}

fn setup(
    mut commands: Commands,
    outline_assets: Res<OutlineAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.8, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        PointLight {
            intensity: 2_500_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    let cube_mesh = meshes.add(Cuboid::default());
    let cube_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.65, 0.69, 0.76),
        perceptual_roughness: 0.85,
        ..default()
    });

    commands
        .spawn((Transform::default(), Visibility::Visible))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(cube_mesh.clone()),
                MeshMaterial3d(cube_material),
                Transform::default(),
            ));
            spawn_default_outline_mesh(parent, &outline_assets, cube_mesh, Visibility::Visible);
        });
}

fn pulse_outline(time: Res<Time>, mut outlines: Query<&mut Visibility, With<OutlineShell>>) {
    let visible = (time.elapsed_secs() * 2.0).sin() > 0.0;

    for mut visibility in &mut outlines {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
