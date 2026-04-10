use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::render::render_resource::Face;

pub struct GpmoOutlinePlugin {
    default_style: OutlineStyle,
}

impl Default for GpmoOutlinePlugin {
    fn default() -> Self {
        Self {
            default_style: OutlineStyle::default(),
        }
    }
}

impl GpmoOutlinePlugin {
    pub fn with_default_style(default_style: OutlineStyle) -> Self {
        Self { default_style }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct OutlineAssets {
    default_material: Handle<StandardMaterial>,
    default_style: OutlineStyle,
}

impl OutlineAssets {
    pub fn default_material(&self) -> Handle<StandardMaterial> {
        self.default_material.clone()
    }

    pub fn default_style(&self) -> OutlineStyle {
        self.default_style
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct OutlineShell;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutlineStyle {
    pub color: Color,
    pub emissive_strength: f32,
    pub scale: Vec3,
}

impl Default for OutlineStyle {
    fn default() -> Self {
        Self {
            color: Color::srgb(0.43, 0.94, 0.98),
            emissive_strength: 2.0,
            scale: Vec3::splat(1.08),
        }
    }
}

impl Plugin for GpmoOutlinePlugin {
    fn build(&self, app: &mut App) {
        let default_style = self.default_style;
        app.insert_resource(PendingOutlineStyle(default_style))
            .add_systems(Startup, setup_outline_assets);
    }
}

#[derive(Resource, Debug, Clone, Copy)]
struct PendingOutlineStyle(OutlineStyle);

pub fn create_outline_material(
    materials: &mut Assets<StandardMaterial>,
    style: OutlineStyle,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: style.color,
        emissive: style.color.to_linear() * style.emissive_strength,
        unlit: true,
        cull_mode: Some(Face::Front),
        ..default()
    })
}

pub fn outline_shell_transform(style: OutlineStyle) -> Transform {
    Transform::from_scale(style.scale)
}

pub fn spawn_outline_mesh<'a>(
    parent: &'a mut ChildSpawnerCommands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    style: OutlineStyle,
    visibility: Visibility,
) -> EntityCommands<'a> {
    parent.spawn((
        OutlineShell,
        Mesh3d(mesh),
        MeshMaterial3d(material),
        outline_shell_transform(style),
        visibility,
    ))
}

pub fn spawn_default_outline_mesh<'a>(
    parent: &'a mut ChildSpawnerCommands,
    outline_assets: &OutlineAssets,
    mesh: Handle<Mesh>,
    visibility: Visibility,
) -> EntityCommands<'a> {
    spawn_outline_mesh(
        parent,
        mesh,
        outline_assets.default_material(),
        outline_assets.default_style(),
        visibility,
    )
}

fn setup_outline_assets(
    mut commands: Commands,
    pending_style: Res<PendingOutlineStyle>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let style = pending_style.0;
    let default_material = create_outline_material(&mut materials, style);
    commands.insert_resource(OutlineAssets {
        default_material,
        default_style: style,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_style_has_positive_scale() {
        let style = OutlineStyle::default();

        assert!(style.scale.x > 1.0);
        assert!(style.scale.y > 1.0);
        assert!(style.scale.z > 1.0);
    }

    #[test]
    fn outline_transform_uses_style_scale() {
        let style = OutlineStyle {
            scale: Vec3::splat(1.25),
            ..default()
        };

        let transform = outline_shell_transform(style);

        assert_eq!(transform.scale, Vec3::splat(1.25));
    }
}
