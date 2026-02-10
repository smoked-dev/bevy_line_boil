//! # Bevy Line Boil
//!
//! A Bevy plugin that applies a classic cartoon "line boil" effect via turbulent vertex displacement.
//! The effect creates a hand-drawn animation look by jittering vertices at fixed frame intervals.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use bevy::prelude::*;
//! use bevy_line_boil::{LineBoil, LineBoilPlugin};
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(LineBoilPlugin)
//!         .add_systems(Startup, setup)
//!         .run();
//! }
//!
//! fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
//!     // Spawn a model with line boil effect
//!     commands.spawn((
//!         SceneRoot(asset_server.load("model.glb#Scene0")),
//!         LineBoil,
//!     ));
//! }
//! ```

use bevy::{
    asset::{load_internal_asset, uuid_handle},
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

/// Shader handle for the line boil vertex shader
pub const LINE_BOIL_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("89237458-9234-4589-a3ab-cdef12345678");

/// Plugin that adds line boil effect support.
///
/// Add this plugin to your app, then add the [`LineBoil`] component to any entity
/// with a glTF scene to apply the effect to all its meshes.
pub struct LineBoilPlugin;

impl Plugin for LineBoilPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, LineBoilMaterial>>::default(),
        );

        load_internal_asset!(
            app,
            LINE_BOIL_SHADER_HANDLE,
            "line_boil.wgsl",
            Shader::from_wgsl
        );

        app.add_systems(
            Update,
            (
                apply_line_boil_to_marked_entities,
                cleanup_old_materials.after(apply_line_boil_to_marked_entities),
            ),
        );
    }
}

/// The line boil material extension.
///
/// No uniforms — settings are hardcoded as shader constants for WebGL compatibility
/// and better performance (constant folding).
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone, Default)]
pub struct LineBoilMaterial {}

impl MaterialExtension for LineBoilMaterial {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Handle(LINE_BOIL_SHADER_HANDLE)
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }
}

/// Marker component to apply line boil effect to an entity and its mesh children.
///
/// Add this component to an entity (typically a glTF scene root) to apply the
/// line boil effect to all meshes within its hierarchy.
///
/// # Example
///
/// ```rust,ignore
/// commands.spawn((
///     SceneRoot(asset_server.load("character.glb#Scene0")),
///     LineBoil,
/// ));
/// ```
#[derive(Component, Default, Clone)]
pub struct LineBoil;

/// Marker component to track meshes that have already been processed.
#[derive(Component)]
struct LineBoilApplied;

fn cleanup_old_materials(
    mut commands: Commands,
    query: Query<Entity, (With<LineBoilApplied>, With<MeshMaterial3d<StandardMaterial>>)>,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>();
    }
}

fn apply_line_boil_to_marked_entities(
    mut commands: Commands,
    root_query: Query<Entity, With<LineBoil>>,
    children_query: Query<&Children>,
    mesh_query: Query<
        (Entity, &MeshMaterial3d<StandardMaterial>),
        Without<LineBoilApplied>,
    >,
    standard_materials: Res<Assets<StandardMaterial>>,
    mut line_boil_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, LineBoilMaterial>>>,
) {
    for root_entity in root_query.iter() {
        traverse_and_replace_materials(
            root_entity,
            &children_query,
            &mesh_query,
            &standard_materials,
            &mut line_boil_materials,
            &mut commands,
        );
    }
}

fn traverse_and_replace_materials(
    entity: Entity,
    children_query: &Query<&Children>,
    mesh_query: &Query<
        (Entity, &MeshMaterial3d<StandardMaterial>),
        Without<LineBoilApplied>,
    >,
    standard_materials: &Assets<StandardMaterial>,
    line_boil_materials: &mut Assets<ExtendedMaterial<StandardMaterial, LineBoilMaterial>>,
    commands: &mut Commands,
) {
    if let Ok((_, mat_handle)) = mesh_query.get(entity) {
        if let Some(std_mat) = standard_materials.get(&mat_handle.0) {
            let extended = ExtendedMaterial {
                base: std_mat.clone(),
                extension: LineBoilMaterial {},
            };
            let new_handle = line_boil_materials.add(extended);

            commands
                .entity(entity)
                .remove::<MeshMaterial3d<StandardMaterial>>()
                .insert(MeshMaterial3d(new_handle))
                .insert(LineBoilApplied);
        }
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            traverse_and_replace_materials(
                child,
                children_query,
                mesh_query,
                standard_materials,
                line_boil_materials,
                commands,
            );
        }
    }
}
