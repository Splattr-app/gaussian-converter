use crate::{ConversionError, GaussianSplat, Importer, Scene};
use spz_rs::{UnpackedGaussian, load_packed_gaussians_from_spz_buffer};

pub struct SpzImporter;
pub struct SpzExporter;

fn unpack_sh_rest(unpacked: &UnpackedGaussian) -> Vec<f32> {
  let mut sh_rest = vec![0.0f32; 45]; // preallocate all 45 slots

  for i in 0..15 {
    sh_rest[i] = unpacked.sh_r[i] as f32;
    sh_rest[i + 15] = unpacked.sh_g[i] as f32;
    sh_rest[i + 30] = unpacked.sh_b[i] as f32;
  }

  sh_rest
}

impl Importer for SpzImporter {
  fn import(reader: &mut impl std::io::Read) -> Result<Scene, ConversionError> {
    let packed_gaussians = load_packed_gaussians_from_spz_buffer(reader)?;

    let mut splats: Vec<GaussianSplat> = Vec::with_capacity(packed_gaussians.num_points);

    for i in 0..packed_gaussians.num_points {
      let unpacked_gaussian = packed_gaussians.unpack(i);

      let splat = GaussianSplat {
        position: unpacked_gaussian.position,
        normal: [0f32, 0f32, 0f32],
        spherical_harmonics_dc: unpacked_gaussian.color,
        spherical_harmonics_rest: unpack_sh_rest(&unpacked_gaussian),
        opacity: unpacked_gaussian.alpha,
        scale: unpacked_gaussian.scale,
        rotation: unpacked_gaussian.rotation,
      };

      splats.push(splat);
    }

    Ok(Scene { splats })
  }
}
