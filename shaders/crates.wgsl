// Vertex shader

struct CameraUniform {
  view: mat4x4<f32>,
  projection: mat4x4<f32>,
  view_projection: mat4x4<f32>,
  view_projection_inverse: mat4x4<f32>,
  view_projection_without_translation: mat4x4<f32>,
  position: vec3<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) tex_coords: vec2<f32>,
  @location(2) normal: vec3<f32>,
  @location(3) tangent: vec3<f32>,
  @location(4) bitangent: vec3<f32>,
};

struct InstanceInput {
    @location(5) transform_0: vec4<f32>,
    @location(6) transform_1: vec4<f32>,
    @location(7) transform_2: vec4<f32>,
    @location(8) transform_3: vec4<f32>,
    @location(9) disabled: i32,
};

struct VertexOutput {
  @location(0) disabled: i32,
  @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
  vertex: VertexInput,
  instance: InstanceInput,
) -> VertexOutput {
  let transform = mat4x4<f32>(
        instance.transform_0,
        instance.transform_1,
        instance.transform_2,
        instance.transform_3,
  );
  let world_position = transform * vec4<f32>(vertex.position, 1.0);

  var out: VertexOutput;
  out.disabled = instance.disabled;
  out.clip_position = camera.view_projection * world_position;
  return out;
}

// Fragment shader

struct ColorMaterial {
    color: vec4<f32>,
};
@group(0) @binding(0)
var<uniform> material: ColorMaterial;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
  if vertex.disabled != 0 {
    discard;
  }
  return material.color; 
}
