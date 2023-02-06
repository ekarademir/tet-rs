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

  return out;
}

@group(0)
@binding(0)
var<uniform> colourx: vec4<f32>;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
  return colourx;
}