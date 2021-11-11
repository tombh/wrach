/// This is tries to be a common way of using glam.
/// There are 2 different glam's, one from spirv::std that doesn't use Bytemuckable
/// traits, and the other native glam, which can be used to bytemuck.
/// Bytemucking only needs to happen when sending data *to* shaders, hence why it's
/// useful to have 2 different versions of glam.

#[cfg(not(target_arch = "spirv"))]
pub use glam;

#[cfg(target_arch = "spirv")]
pub use spirv_std::glam;
