use spirv_std::memory::{Scope, Semantics};

#[cfg(not(target_arch = "spirv"))]
use crevice::std140::AsStd140;

use crate::compute;
use crate::neighbours;
use crate::particle;
use crate::workgroup;

#[cfg_attr(not(target_arch = "spirv"), derive(AsStd140, Debug))]
#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct Params {
    pub stage: u32,
}

pub struct GPUIdentifiers {
    pub global: u32,
    pub workgroup: u32,
    pub local: u32,
}

impl GPUIdentifiers {
    pub fn global_id_to_particle_id(&self) -> particle::ParticleIDGlobal {
        particle::ParticleIDGlobal { id: self.global }
    }
}

/// Prevents the work item continuing until all work items in workgroup have reached it
pub fn execution_barrier() {
    unsafe {
        spirv_std::arch::control_barrier::<
            { Scope::Workgroup as u32 },
            { Scope::Workgroup as u32 },
            { Semantics::NONE.bits() },
        >();
    }
}

pub fn memory_barrier() {
    unsafe {
        spirv_std::arch::memory_barrier::<{ Scope::Workgroup as u32 }, { Scope::Workgroup as u32 }>(
        );
    }
}

pub fn entry(
    ids: GPUIdentifiers,
    _params: &Params,
    particles_src: &mut particle::ParticlesGlobal,
    _particles_dst: &mut particle::ParticlesGlobal,
    workgroup_data: &mut workgroup::WorkGroupData,
    pixel_grid: &mut neighbours::PixelGridGlobal,
) {
    workgroup_data.workgroup_id = ids.workgroup;
    workgroup_data.populate(ids.local, pixel_grid, particles_src);
    compute::execution_barrier();

    let mut stage = 0;
    while stage < 3 {
        let mut job = 0;
        loop {
            let particle = workgroup_data.particle_for_work_item(ids.local, job);
            if particle.id_global.id == particle::ParticleIDLocal::null() {
                break;
            }
            //neighbours::NeighbouringParticles::find(particle_local, workgroup_data);
            // let mut current_particle = particle::CurrentParticle::new(ids.local as usize, particle);
            // match stage {
            //     0 => current_particle.predict(),
            //     // 1 => current_particle.compute(workgroup_data),
            //     // 2 => current_particle.propogate(workgroup_data),
            //     _ => (),
            // }
            job += 1;
        }
        //compute::execution_barrier();
        stage += 1;
    }
}
