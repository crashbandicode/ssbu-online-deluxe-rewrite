#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
struct NsTuffClockTriple {
    cpu_hz: u32,
    gpu_hz: u32,
    mem_hz: u32,
}

use crate::util::file::overclock as overclock_file;
use crate::OverclockConfig;
use std::io;

// [NxOverclock.Default]
// [Safe.Default]
// [CPU = 1224, GPU = 844, MEM = 1728]
// [Performance.Singles]
// [CPU = 1581, GPU = 921, MEM = 1728]
// [Performance.FFA]
// [CPU = 1683, GPU = 921, MEM = 1728]

// [NxOverclock.Hdr]
// [Safe.Default]
// [CPU = 1581, GPU = 844, MEM = 1728]
// [Performance.Singles]
// [CPU = 1581, GPU = 921, MEM = 1728]
// [Performance.FFA]
// [CPU = 1683, GPU = 921, MEM = 1728]

pub fn load_or_create() -> io::Result<()> {
    overclock_file::load_or_create()
}

pub fn get_or_make_profile(
    name: &str,
    defaults: &OverclockConfig,
    version: f32,
) -> io::Result<OverclockConfig> {
    overclock_file::get_or_make_profile(name, defaults, version)
}

pub fn get_or_make_subprofile(
    profile: &str,
    name: &str,
    defaults: &OverclockConfig,
    version: f32,
) -> io::Result<OverclockConfig> {
    overclock_file::get_or_make_subprofile(profile, name, defaults, version)
}

pub fn set_profile(name: &str, config: &OverclockConfig, version: f32) -> io::Result<()> {
    overclock_file::set_profile(name, config, version)
}

pub fn set_subprofile(
    profile: &str,
    name: &str,
    config: &OverclockConfig,
    version: f32,
) -> io::Result<()> {
    overclock_file::set_subprofile(profile, name, config, version)
}

pub fn get_profile_or_subprofile(
    profile: &str,
    subprofile: Option<&str>,
    defaults: &OverclockConfig,
    version: f32,
) -> io::Result<OverclockConfig> {
    match subprofile {
        Some(name) => get_or_make_subprofile(profile, name, defaults, version),
        None => get_or_make_profile(profile, defaults, version),
    }
}

pub fn get_overclock_profile(
    defaults: &OverclockConfig,
    version: f32,
) -> io::Result<Option<OverclockConfig>> {
    let (_, _, profile_name) = crate::Config::load_or_create_with_overclock_profile()?;
    let Some(profile) = profile_name else {
        return Ok(None);
    };

    let _ = get_or_make_profile(profile.as_str(), defaults, version)?;
    overclock_file::get_first_subprofile(profile.as_str())
}
