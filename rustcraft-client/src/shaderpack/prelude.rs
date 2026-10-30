//! The GLSL Rustcraft wraps around a pack stage: the `rustcraft_*` stand-ins for
//! OpenGL's built-ins (see `glsl::translate::builtin`), filled from Rustcraft's
//! own vertex formats before the pack's `main` runs.
//!
//! Packs render in OpenGL's conventions: a GL projection (clip z in -1..1,
//! which the vertex `main` remaps) and images stored bottom row first, so
//! `gl_FragCoord` and texture coordinates both count up from the bottom as
//! the pack expects. Only `final` flips onto the upright swapchain.

use super::glsl::translate::Prelude;
use super::preprocess::render_stage;
use crate::renderer::chunk::mesher::{
    POS_BIAS, POS_RANGE, SPRITE_BITS, TERRAIN_UV_BITS, TERRAIN_UV_FIXED_SCALE,
};

/// How a pack wants terrain's `gl_Color`, from `shaders.properties`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TerrainLighting {
    /// `separateAo`: ambient occlusion in the alpha, not the colour.
    pub separate_ao: bool,
    /// `oldLighting`: vanilla's face shade kept, which Iris otherwise drops.
    pub old_lighting: bool,
}

/// The vertex formats a pack program can be fed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Geometry {
    /// `PackVertex` plus the section's `ChunkMeta`.
    Terrain,
    /// `PackVertex`, the section origin in a push constant (`draw_water`).
    Water,
    /// `EntityVertex` plus the entity renderer's per-instance data.
    Entity,
    /// `EntityVertex` with the block-entity push constants.
    BlockEntity,
    /// `SkyVertex` with the sky push block (`SkyPipeline::draws`).
    Sky,
    /// A full-screen triangle (`deferred`, `composite`).
    FullScreen,
    /// `final`, flipped onto the upright swapchain.
    Final,
}

/// Frame data the preludes read, set 0 binding 1; `PackFrame` on the Rust side.
const FRAME: &str = "
layout(std140, set = 0, binding = 1) uniform RustcraftFrame {
    mat4 rustcraft_view;
    mat4 rustcraft_projection;
    vec4 rustcraft_eye;
    ivec4 rustcraft_anchor;
    vec4 rustcraft_atlas_size;
    // x: fog start, y: fog end, in blocks; zw unused.
    vec4 rustcraft_fog;
    vec4 rustcraft_fog_color;
};
";

/// The built-in matrices and `renderStage`, both stages.
const MATRICES: &str = "
int rustcraft_render_stage;
mat4 rustcraft_ModelViewMatrix;
mat4 rustcraft_ModelViewMatrixInverse;
mat4 rustcraft_ProjectionMatrix;
mat4 rustcraft_ProjectionMatrixInverse;
mat4 rustcraft_ModelViewProjectionMatrix;
mat3 rustcraft_NormalMatrix;
mat4 rustcraft_TextureMatrix[2];

// The fixed-function fog `gl_Fog` reported.
struct rustcraft_FogParameters {
    vec4 color;
    float density;
    float start;
    float end;
    float scale;
};
rustcraft_FogParameters rustcraft_Fog;

void rustcraft_matrices(mat4 model_view, mat4 projection) {
    rustcraft_ModelViewMatrix = model_view;
    rustcraft_ModelViewMatrixInverse = inverse(model_view);
    rustcraft_ProjectionMatrix = projection;
    rustcraft_ProjectionMatrixInverse = inverse(projection);
    rustcraft_ModelViewProjectionMatrix = projection * model_view;
    rustcraft_NormalMatrix = transpose(inverse(mat3(model_view)));
    rustcraft_TextureMatrix[0] = mat4(1.0);
    // OptiFine's lightmap matrix: 0..240 light coordinates to texel centres.
    rustcraft_TextureMatrix[1] = mat4(
        vec4(1.0 / 256.0, 0.0, 0.0, 0.0),
        vec4(0.0, 1.0 / 256.0, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0 / 256.0, 0.0),
        vec4(1.0 / 32.0, 1.0 / 32.0, 1.0 / 32.0, 1.0));
    rustcraft_Fog = rustcraft_FogParameters(
        rustcraft_fog_color, 1.0, rustcraft_fog.x, rustcraft_fog.y, 1.0 / max(rustcraft_fog.y - rustcraft_fog.x, 1e-4));
}

// Iris's composite projection: the 0..1 square onto clip space.
const mat4 rustcraft_screen_projection = mat4(
    vec4(2.0, 0.0, 0.0, 0.0),
    vec4(0.0, 2.0, 0.0, 0.0),
    vec4(0.0, 0.0, 1.0, 0.0),
    vec4(-1.0, -1.0, 0.0, 1.0));
";

/// Built-in functions naga's GLSL frontend lacks, both stages.
const POLYFILLS: &str = "
float rustcraft_modf(float x, out float i) { i = trunc(x); return x - i; }
vec2 rustcraft_modf(vec2 x, out vec2 i) { i = trunc(x); return x - i; }
vec3 rustcraft_modf(vec3 x, out vec3 i) { i = trunc(x); return x - i; }
vec4 rustcraft_modf(vec4 x, out vec4 i) { i = trunc(x); return x - i; }
";

/// The vertex inputs' stand-ins.
const VERTEX_INPUTS: &str = "
vec4 rustcraft_Vertex;
vec3 rustcraft_Normal;
vec4 rustcraft_Color;
vec4 rustcraft_MultiTexCoord0;
vec4 rustcraft_MultiTexCoord1;
vec4 rustcraft_MultiTexCoord2;

vec4 rustcraft_ftransform() {
    return rustcraft_ModelViewProjectionMatrix * rustcraft_Vertex;
}

// Light lanes (`LightLevels::lane`): block + sky * 256, each 0..240.
vec4 rustcraft_light_coords(float lane) {
    return vec4(mod(lane, 256.0), floor(lane / 256.0), 0.0, 1.0);
}
";

/// The pack's `gl_Position` from GL clip space to Vulkan's 0..1 depth.
const GL_DEPTH: &str = "    gl_Position.z = (gl_Position.z + gl_Position.w) * 0.5;\n";

pub struct Preludes {
    vertex_decls: String,
    vertex_main: String,
    attributes: &'static [(&'static str, &'static str)],
    fragment_decls: String,
    fragment_main: String,
}

impl Geometry {
    /// Whether a `gbuffers_*` program draws it, rather than a full-screen pass.
    pub fn is_geometry(self) -> bool {
        !matches!(self, Self::FullScreen | Self::Final)
    }
}

impl Preludes {
    pub fn new(geometry: Geometry, lighting: TerrainLighting) -> Self {
        let (inputs, setup, model_view) = match geometry {
            Geometry::Terrain => (
                terrain_inputs(TERRAIN_ORIGIN),
                terrain_setup(lighting),
                "rustcraft_view",
            ),
            Geometry::Water => (
                terrain_inputs(WATER_ORIGIN),
                terrain_setup(lighting),
                "rustcraft_view",
            ),
            Geometry::Entity => (
                ENTITY_INPUTS.to_owned(),
                ENTITY_SETUP.to_owned(),
                ENTITY_MODEL_VIEW,
            ),
            Geometry::BlockEntity => (
                BLOCK_ENTITY_INPUTS.to_owned(),
                ENTITY_SETUP.to_owned(),
                ENTITY_MODEL_VIEW,
            ),
            Geometry::Sky => (
                format!("{SKY_DRAW}{SKY_INPUTS}"),
                SKY_SETUP.to_owned(),
                "rustcraft_view * rustcraft_sky_pose",
            ),
            Geometry::FullScreen | Geometry::Final => {
                (String::new(), FULL_SCREEN_SETUP.to_owned(), "mat4(1.0)")
            }
        };
        let stage = match geometry {
            Geometry::Sky => "int(rustcraft_sky_stage)".to_owned(),
            other => render_stage(match other {
                // TODO: cutout terrain reads TERRAIN_CUTOUT.
                Geometry::Terrain => "TERRAIN_SOLID",
                Geometry::Water => "TERRAIN_TRANSLUCENT",
                Geometry::Entity => "ENTITIES",
                Geometry::BlockEntity => "BLOCK_ENTITIES",
                _ => "NONE",
            })
            .to_string(),
        };
        let stage = format!("    rustcraft_render_stage = {stage};\n");
        let fragment_inputs = if geometry == Geometry::Sky {
            SKY_DRAW
        } else {
            ""
        };
        let (fragment_view, projection) = if geometry.is_geometry() {
            ("rustcraft_view", "rustcraft_projection")
        } else {
            ("mat4(1.0)", "rustcraft_screen_projection")
        };
        let flip = if geometry == Geometry::Final {
            "    gl_Position.y = -gl_Position.y;\n"
        } else {
            ""
        };
        Self {
            vertex_decls: format!(
                "{FRAME}{MATRICES}{POLYFILLS}{VERTEX_INPUTS}{EYE_OFFSET}{inputs}"
            ),
            // Iris reads the lightmap coordinate as TexCoord2 too.
            vertex_main: format!(
                "\nvoid main() {{\n{stage}{setup}    rustcraft_MultiTexCoord2 = rustcraft_MultiTexCoord1;\n    rustcraft_matrices({model_view}, {projection});\n    rustcraft_attributes();\n    pack_main();\n{GL_DEPTH}{flip}}}\n"
            ),
            attributes: if matches!(geometry, Geometry::Terrain | Geometry::Water) {
                &[("mc_Entity", "rustcraft_mc_Entity")]
            } else {
                &[]
            },
            fragment_decls: format!("{FRAME}{MATRICES}{POLYFILLS}{fragment_inputs}"),
            fragment_main: format!(
                "\nvoid main() {{\n{stage}    rustcraft_matrices({fragment_view}, {projection});\n    pack_main();\n}}\n"
            ),
        }
    }

    pub fn vertex(&self) -> Prelude<'_> {
        Prelude {
            decls: &self.vertex_decls,
            main: &self.vertex_main,
            attributes: self.attributes,
        }
    }

    pub fn fragment(&self) -> Prelude<'_> {
        Prelude {
            decls: &self.fragment_decls,
            main: &self.fragment_main,
            attributes: &[],
        }
    }
}

/// The translation from the render anchor to the eye: Rustcraft's positions are
/// anchor-relative, the pack's model-view matrix eye-relative.
const EYE_OFFSET: &str = "
mat4 rustcraft_eye_offset() {
    mat4 m = mat4(1.0);
    m[3] = vec4(-rustcraft_eye.xyz, 1.0);
    return m;
}
";

/// Where a terrain vertex's section origin comes from: the section meta
/// for the indirect passes, a push constant for `draw_water`.
const TERRAIN_ORIGIN: &str = "
layout(location = 5) in ivec3 rustcraft_in_origin;
layout(location = 6) in float rustcraft_in_visibility;

vec3 rustcraft_section_origin() {
    return vec3(rustcraft_in_origin - rustcraft_anchor.xyz);
}
";

const WATER_ORIGIN: &str = "
layout(push_constant) uniform RustcraftSection {
    vec4 rustcraft_origin_fade;
};

vec3 rustcraft_section_origin() {
    return rustcraft_origin_fade.xyz;
}
";

fn terrain_inputs(origin: &str) -> String {
    format!(
        "
const float RUSTCRAFT_POS_RANGE = {POS_RANGE:?};
const float RUSTCRAFT_POS_BIAS = {POS_BIAS:?};
const float RUSTCRAFT_UV_SCALE = {TERRAIN_UV_FIXED_SCALE:?};
const uint RUSTCRAFT_UV_BITS = {TERRAIN_UV_BITS}u;
const uint RUSTCRAFT_SPRITE_BITS = {SPRITE_BITS}u;

layout(location = 0) in vec2 rustcraft_in_pos_xy;
layout(location = 1) in float rustcraft_in_pos_z;
layout(location = 2) in uint rustcraft_in_uv;
layout(location = 3) in uint rustcraft_in_sprite;
layout(location = 4) in vec4 rustcraft_in_light_tint;
layout(location = 7) in vec4 rustcraft_in_shade;
layout(location = 8) in uint rustcraft_in_entity;
vec4 rustcraft_mc_Entity;
{origin}
// Level-0 sprite rectangles in atlas texels, as the atlas uploads them.
layout(std430, set = 2, binding = 0) readonly buffer RustcraftSprites {{
    uvec4 rustcraft_sprite_rects[];
}};

// `Direction` order (down, up, north, south, west, east), after 0 = none.
const vec3 RUSTCRAFT_FACE_NORMALS[7] = vec3[7](
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, -1.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, -1.0),
    vec3(0.0, 0.0, 1.0),
    vec3(-1.0, 0.0, 0.0),
    vec3(1.0, 0.0, 0.0));
"
    )
}

const TERRAIN_SETUP: &str = "
    vec3 local = vec3(rustcraft_in_pos_xy, rustcraft_in_pos_z) * RUSTCRAFT_POS_RANGE - RUSTCRAFT_POS_BIAS;
    vec3 rel = rustcraft_section_origin() - rustcraft_eye.xyz + local;
    rustcraft_Vertex = vec4(rel, 1.0);
    uint sprite = rustcraft_in_sprite & ((1u << RUSTCRAFT_SPRITE_BITS) - 1u);
    rustcraft_Normal = RUSTCRAFT_FACE_NORMALS[min(rustcraft_in_sprite >> RUSTCRAFT_SPRITE_BITS, 6u)];
    uint mask = (1u << RUSTCRAFT_UV_BITS) - 1u;
    vec2 sprite_uv = vec2(rustcraft_in_uv & mask, (rustcraft_in_uv >> RUSTCRAFT_UV_BITS) & mask) / RUSTCRAFT_UV_SCALE;
    uvec4 rect = rustcraft_sprite_rects[sprite];
    vec2 atlas_uv = (vec2(rect.xy) + sprite_uv * vec2(rect.zw)) / rustcraft_atlas_size.xy;
    rustcraft_MultiTexCoord0 = vec4(atlas_uv, 0.0, 1.0);
    rustcraft_MultiTexCoord1 = vec4(round(rustcraft_in_light_tint.r * 255.0), float(rustcraft_in_uv >> 24), 0.0, 1.0);
    // Iris's unpacking: the block id, then whether the quad is a fluid's.
    rustcraft_mc_Entity = vec4(float(int(rustcraft_in_entity >> 1u) - 1), float(rustcraft_in_entity & 1u), 0.0, 1.0);
";

/// [`TERRAIN_SETUP`] and Iris's terrain colour: the tint, with the ambient
/// occlusion and face shade (the flat one unless `oldLighting`) multiplied
/// in, or in the alpha under `separateAo`.
fn terrain_setup(lighting: TerrainLighting) -> String {
    let shade = if lighting.old_lighting {
        "rustcraft_in_shade.x * rustcraft_in_shade.y"
    } else {
        "rustcraft_in_shade.x * rustcraft_in_shade.z"
    };
    let color = if lighting.separate_ao {
        "vec4(rustcraft_in_light_tint.gba, rustcraft_shade)"
    } else {
        "vec4(rustcraft_in_light_tint.gba * rustcraft_shade, 1.0)"
    };
    format!("{TERRAIN_SETUP}    float rustcraft_shade = {shade};\n    rustcraft_Color = {color};\n")
}

const ENTITY_INPUTS: &str = "
layout(location = 0) in vec3 rustcraft_in_position;
layout(location = 1) in vec2 rustcraft_in_tex_coords;
layout(location = 2) in vec4 rustcraft_in_normal;
layout(location = 3) in vec4 rustcraft_in_model_0;
layout(location = 4) in vec4 rustcraft_in_model_1;
layout(location = 5) in vec4 rustcraft_in_model_2;
layout(location = 6) in vec4 rustcraft_in_model_3;
layout(location = 7) in vec4 rustcraft_in_tint;
layout(location = 8) in vec4 rustcraft_in_overlay;
layout(location = 9) in vec4 rustcraft_in_uv;

mat4 rustcraft_model() {
    return mat4(rustcraft_in_model_0, rustcraft_in_model_1, rustcraft_in_model_2, rustcraft_in_model_3);
}
";

const BLOCK_ENTITY_INPUTS: &str = "
layout(location = 0) in vec3 rustcraft_in_position;
layout(location = 1) in vec2 rustcraft_in_tex_coords;
layout(location = 2) in vec4 rustcraft_in_normal;

layout(push_constant) uniform RustcraftDraw {
    mat4 rustcraft_draw_model;
    vec4 rustcraft_in_tint;
    vec4 rustcraft_in_overlay;
    vec4 rustcraft_in_uv;
};

mat4 rustcraft_model() {
    return rustcraft_draw_model;
}
";

/// An entity's model-view: `rustcraft_model()` reads the instance attributes or
/// the block-entity push constants.
const ENTITY_MODEL_VIEW: &str = "rustcraft_view * rustcraft_eye_offset() * rustcraft_model()";

const ENTITY_SETUP: &str = "
    rustcraft_Vertex = vec4(rustcraft_in_position, 1.0);
    rustcraft_Normal = rustcraft_in_normal.xyz;
    rustcraft_Color = rustcraft_in_tint;
    rustcraft_MultiTexCoord0 = vec4(rustcraft_in_tex_coords + rustcraft_in_uv.xy, 0.0, 1.0);
    // Emissive layers come unlit; Iris draws them full bright.
    rustcraft_MultiTexCoord1 = rustcraft_in_uv.w < 0.0
        ? vec4(240.0, 240.0, 0.0, 1.0)
        : rustcraft_light_coords(rustcraft_in_uv.w);
";

/// The sky push block, both stages: `SkyPush` on the Rust side.
const SKY_DRAW: &str = "
layout(push_constant) uniform RustcraftSky {
    mat4 rustcraft_sky_pose;
    vec4 rustcraft_sky_color;
    uint rustcraft_sky_stage;
};
";

const SKY_INPUTS: &str = "
layout(location = 0) in vec3 rustcraft_in_position;
layout(location = 1) in vec2 rustcraft_in_tex_coords;
";

// Vanilla's sky formats carry no normal or light; the sunset fan's
// vertex alpha ramps from its centre (`build_sunrise_fan`'s uv.x).
const SKY_SETUP: &str = "
    rustcraft_Vertex = vec4(rustcraft_in_position, 1.0);
    rustcraft_Normal = vec3(0.0, 0.0, 1.0);
    rustcraft_Color = rustcraft_sky_color;
    // MC_RENDER_STAGE_SUNSET
    if (rustcraft_sky_stage == 2u) {
        rustcraft_Color.a *= rustcraft_in_tex_coords.x;
    }
    rustcraft_MultiTexCoord0 = vec4(rustcraft_in_tex_coords, 0.0, 1.0);
    rustcraft_MultiTexCoord1 = vec4(240.0, 240.0, 0.0, 1.0);
";

const FULL_SCREEN_SETUP: &str = "
    vec2 uv = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
    rustcraft_Vertex = vec4(uv, 0.0, 1.0);
    rustcraft_Normal = vec3(0.0, 0.0, 1.0);
    rustcraft_Color = vec4(1.0);
    rustcraft_MultiTexCoord0 = vec4(uv, 0.0, 1.0);
    rustcraft_MultiTexCoord1 = vec4(240.0, 240.0, 0.0, 1.0);
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_colour_follows_the_packs_lighting() {
        let color = |separate_ao, old_lighting| {
            let lighting = TerrainLighting {
                separate_ao,
                old_lighting,
            };
            Preludes::new(Geometry::Terrain, lighting).vertex_main
        };
        assert!(color(false, false).contains("rustcraft_in_shade.x * rustcraft_in_shade.z;"));
        assert!(color(false, false).contains("rustcraft_in_light_tint.gba * rustcraft_shade, 1.0)"));
        assert!(color(true, false).contains("vec4(rustcraft_in_light_tint.gba, rustcraft_shade)"));
        assert!(color(false, true).contains("rustcraft_in_shade.x * rustcraft_in_shade.y"));
    }
}
