//! Custom WGSL shader effects for visual feedback.
//!
//! This module provides infrastructure for custom shader-based materials.
//! Currently, StandardMaterial with emissive is used for visual effects
//! due to pipeline binding issues with custom Materials in Bevy 0.17.
//!
//! The custom material definitions are kept for future use when the
//! binding issues are resolved (potentially in Bevy 0.18+).
//!
//! Available shader files in assets/shaders/:
//! - card_glow.wgsl: Pulsing edge glow for cards
//! - damage_flash.wgsl: Screen flash effect
//! - energy_field.wgsl: Flowing energy effect for buffs

use bevy::{prelude::*, render::render_resource::AsBindGroup, shader::ShaderRef};

use crate::{AppSystems, screens::Screen};

pub fn plugin(app: &mut App) {
    // Custom material plugins are disabled due to pipeline binding issues.
    // Error: "Storage class Storage doesn't match shader Uniform"
    // This appears to be a Bevy 0.17 issue with custom Material trait implementations.
    //
    // TODO: Re-enable when fixed (possibly Bevy 0.18+)
    // app.add_plugins((
    //     MaterialPlugin::<CardGlowMaterial>::default(),
    //     MaterialPlugin::<DamageFlashMaterial>::default(),
    //     MaterialPlugin::<EnergyFieldMaterial>::default(),
    // ));

    app.init_resource::<ShaderTime>();

    app.add_systems(
        Update,
        update_shader_time
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// Global shader time resource for synchronized animations.
#[derive(Resource, Default)]
pub struct ShaderTime {
    pub elapsed: f32,
}

fn update_shader_time(time: Res<Time>, mut shader_time: ResMut<ShaderTime>) {
    shader_time.elapsed += time.delta_secs();
}

// ============================================================================
// Color Presets
// ============================================================================

/// Glow color presets for different card states.
/// Used with StandardMaterial emissive for current implementation.
#[allow(dead_code)]
pub struct GlowColors;

#[allow(dead_code)]
impl GlowColors {
    /// Gold color for hovered cards
    pub const HOVERED: LinearRgba = LinearRgba::new(1.0, 0.9, 0.2, 1.0);
    /// Green color for playable cards
    pub const PLAYABLE: LinearRgba = LinearRgba::new(0.2, 1.0, 0.3, 1.0);
    /// Blue color for selected cards
    pub const SELECTED: LinearRgba = LinearRgba::new(0.3, 0.6, 1.0, 1.0);
    /// Red color for damage effects
    pub const DAMAGE: LinearRgba = LinearRgba::new(1.0, 0.2, 0.2, 1.0);
    /// Green color for heal effects
    pub const HEAL: LinearRgba = LinearRgba::new(0.2, 1.0, 0.5, 1.0);
    /// Gray for unplayable cards
    pub const UNPLAYABLE: LinearRgba = LinearRgba::new(0.4, 0.4, 0.4, 0.5);
}

/// Energy field color presets for different buff types.
#[allow(dead_code)]
pub struct EnergyColors;

#[allow(dead_code)]
impl EnergyColors {
    /// Orange/Yellow for acceleration buffs
    pub const ACCELERATION: (LinearRgba, LinearRgba) = (
        LinearRgba::new(1.0, 0.8, 0.2, 1.0),
        LinearRgba::new(1.0, 0.4, 0.0, 1.0),
    );
    /// Red for strength buffs
    pub const STRENGTH: (LinearRgba, LinearRgba) = (
        LinearRgba::new(1.0, 0.2, 0.2, 1.0),
        LinearRgba::new(0.8, 0.0, 0.3, 1.0),
    );
    /// Blue for block/defense
    pub const BLOCK: (LinearRgba, LinearRgba) = (
        LinearRgba::new(0.2, 0.5, 1.0, 1.0),
        LinearRgba::new(0.1, 0.3, 0.8, 1.0),
    );
    /// Green for thorns
    pub const THORNS: (LinearRgba, LinearRgba) = (
        LinearRgba::new(0.5, 0.8, 0.2, 1.0),
        LinearRgba::new(0.2, 0.5, 0.1, 1.0),
    );
}

// ============================================================================
// Custom Material Definitions (for future use)
// ============================================================================

/// Material for card edge glow effect.
/// Creates a pulsing glow around card edges.
///
/// NOTE: Currently not usable due to pipeline binding issues.
/// Use StandardMaterial with emissive instead.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[allow(dead_code)]
pub struct CardGlowMaterial {
    #[uniform(0)]
    pub glow_color: LinearRgba,
    #[uniform(0)]
    pub glow_intensity: f32,
    #[uniform(0)]
    pub time: f32,
    #[uniform(0)]
    pub _padding: Vec2,
}

impl Default for CardGlowMaterial {
    fn default() -> Self {
        Self {
            glow_color: LinearRgba::new(1.0, 0.8, 0.2, 1.0),
            glow_intensity: 1.0,
            time: 0.0,
            _padding: Vec2::ZERO,
        }
    }
}

#[allow(dead_code)]
impl CardGlowMaterial {
    pub fn new(color: LinearRgba, intensity: f32) -> Self {
        Self {
            glow_color: color,
            glow_intensity: intensity,
            time: 0.0,
            _padding: Vec2::ZERO,
        }
    }
}

impl Material for CardGlowMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/card_glow.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

/// Material for damage flash effect overlay.
///
/// NOTE: Currently not usable due to pipeline binding issues.
/// Use UI overlay with BackgroundColor instead.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[allow(dead_code)]
pub struct DamageFlashMaterial {
    #[uniform(0)]
    pub flash_color: LinearRgba,
    #[uniform(0)]
    pub flash_intensity: f32,
    #[uniform(0)]
    pub time: f32,
    #[uniform(0)]
    pub _padding: Vec2,
}

impl Default for DamageFlashMaterial {
    fn default() -> Self {
        Self {
            flash_color: LinearRgba::new(1.0, 0.0, 0.0, 1.0),
            flash_intensity: 1.0,
            time: 0.0,
            _padding: Vec2::ZERO,
        }
    }
}

impl Material for DamageFlashMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/damage_flash.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

/// Material for flowing energy field effect (for buffs/power cards).
///
/// NOTE: Currently not usable due to pipeline binding issues.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[allow(dead_code)]
pub struct EnergyFieldMaterial {
    #[uniform(0)]
    pub color_a: LinearRgba,
    #[uniform(0)]
    pub color_b: LinearRgba,
    #[uniform(0)]
    pub speed: f32,
    #[uniform(0)]
    pub time: f32,
    #[uniform(0)]
    pub scale: f32,
    #[uniform(0)]
    pub intensity: f32,
}

impl Default for EnergyFieldMaterial {
    fn default() -> Self {
        Self {
            color_a: LinearRgba::new(0.2, 0.5, 1.0, 1.0),
            color_b: LinearRgba::new(0.8, 0.2, 1.0, 1.0),
            speed: 1.0,
            time: 0.0,
            scale: 4.0,
            intensity: 0.8,
        }
    }
}

impl Material for EnergyFieldMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/energy_field.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
