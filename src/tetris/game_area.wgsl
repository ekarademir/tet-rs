struct VertexInput {
  @location(0) position: vec4<f32>,
  @location(1) colour: vec4<f32>,
}

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(1) colour: vec4<f32>,
}

@vertex
fn vs_main(in_data: VertexInput) ->  VertexOutput {
  var out: VertexOutput;
  out.clip_position = in_data.position;
  out.colour = in_data.colour;

  return out;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
  return vertex.colour;
}