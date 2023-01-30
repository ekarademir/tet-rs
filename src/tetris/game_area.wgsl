struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(@location(0) position: vec4<f32>) ->  VertexOutput {
  var out: VertexOutput;
  out.clip_position = position;

  return out;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
  return vec4<f32>(
     31.0 / 255.0,
     138.0 / 255.0,
     112.0 / 255.0,
    1.0
  );
}