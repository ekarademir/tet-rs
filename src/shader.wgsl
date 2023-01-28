struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) sq_type: i32,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) ->  VertexOutput {
  var out: VertexOutput;
  let x = f32(i32(in_vertex_index) - 1);
  let y = f32(i32(in_vertex_index & 1u) * 2 - 1);

  out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
  out.sq_type = 2;

  return out;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
  var out: vec4<f32>;

  if(vertex.sq_type == 0) {
    out = vec4<f32>(1.0, 0.0, 0.0, 1.0);
  } else if (vertex.sq_type == 1) {
    out = vec4<f32>(0.0, 1.0, 0.0, 1.0);
  } else if (vertex.sq_type == 2) {
    out = vec4<f32>(0.0, 0.0, 1.0, 1.0);
  } else {
    out = vec4<f32>(1.0, 1.0, 1.0, 1.0);
  }
  return out;
}