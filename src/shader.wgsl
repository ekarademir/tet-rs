struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(1) sq_type: i32,
}

@vertex
fn vs_main(@location(0) position: vec4<f32>) ->  VertexOutput {
  var out: VertexOutput;
  out.clip_position = position;
  out.sq_type = i32(1);

  return out;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
  if(vertex.sq_type == 0) {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
  } else if (vertex.sq_type == 1) {
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
  } else if (vertex.sq_type == 2) {
    return vec4<f32>(0.0, 0.0, 1.0, 1.0);
  } else {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
  }
}