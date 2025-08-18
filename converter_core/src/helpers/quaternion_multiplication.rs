pub fn multiply_quaternions(q1: [f32; 4], q2: [f32; 4]) -> [f32; 4] {
  let w1 = q1[0];
  let x1 = q1[1];
  let y1 = q1[2];
  let z1 = q1[3];
  let w2 = q2[0];
  let x2 = q2[1];
  let y2 = q2[2];
  let z2 = q2[3];

  let w = w1 * w2 - x1 * x2 - y1 * y2 - z1 * z2;
  let x = w1 * x2 + x1 * w2 + y1 * z2 - z1 * y2;
  let y = w1 * y2 - x1 * z2 + y1 * w2 + z1 * x2;
  let z = w1 * z2 + x1 * y2 - y1 * x2 + z1 * w2;

  [w, x, y, z]
}
