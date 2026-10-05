pub mod paths {
    use camino::Utf8PathBuf;
    use std::io;

    pub fn ensure_paths_exist() -> io::Result<()> {
        if let Err(err) = std::fs::create_dir_all(ssbusync()) {
            println!(
                "[ssbusync][config] failed to create config directory {}: {}",
                ssbusync(),
                err
            );
            return Err(err);
        }
        Ok(())
    }

    pub fn ssbusync() -> Utf8PathBuf {
        Utf8PathBuf::from("sd:/ultimate/ssbusync")
    }

    pub fn ssbusync_config() -> Utf8PathBuf {
        ssbusync().join("ssbusync.toml")
    }

    pub fn ssbusync_disablers() -> Utf8PathBuf {
        ssbusync().join("disablers.toml")
    }

    pub fn ssbusync_overclock() -> Utf8PathBuf {
        ssbusync().join("overclock.toml")
    }
}

pub mod config {
    use super::paths;
    use crate::SsbuSyncConfig;
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;
    use std::io;

    const DEFAULT_PROFILE_VERSION: f32 = 1.0;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DefaultProfileState {
        Created,
        Loaded,
    }

    #[derive(Debug, Serialize, Deserialize, Default)]
    #[serde(default)]
    struct ConfigFile {
        #[serde(rename = "SsbuSync")]
        ssbusync: BTreeMap<String, ProfileEntry>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(default)]
    struct ProfileEntry {
        profile_version: f32,
        #[serde(flatten)]
        config: SsbuSyncConfig,
        #[serde(flatten)]
        extra: BTreeMap<String, toml::Value>,
    }

    impl Default for ProfileEntry {
        fn default() -> Self {
            Self {
                profile_version: DEFAULT_PROFILE_VERSION,
                config: SsbuSyncConfig::default(),
                extra: BTreeMap::new(),
            }
        }
    }

    fn io_err(err: impl std::fmt::Display) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, err.to_string())
    }

    fn load_file(path: &camino::Utf8PathBuf) -> io::Result<ConfigFile> {
        if !path.exists() {
            return Ok(ConfigFile::default());
        }
        let data = match std::fs::read_to_string(path.as_std_path()) {
            Ok(data) => data,
            Err(err) => {
                println!(
                    "[ssbusync][config] failed reading {}: {}",
                    path,
                    err
                );
                return Err(err);
            }
        };

        match toml::from_str(&data) {
            Ok(parsed) => Ok(parsed),
            Err(err) => {
                println!(
                    "[ssbusync][config] failed parsing TOML {}: {}",
                    path,
                    err
                );
                Err(io_err(err))
            }
        }
    }

    fn write_file(path: &camino::Utf8PathBuf, data: &ConfigFile) -> io::Result<()> {
        let toml = match toml::to_string_pretty(data) {
            Ok(toml) => toml,
            Err(err) => {
                println!(
                    "[ssbusync][config] failed serializing TOML {}: {}",
                    path,
                    err
                );
                return Err(io_err(err));
            }
        };

        if let Err(err) = std::fs::write(path.as_std_path(), toml) {
            println!(
                "[ssbusync][config] failed writing {}: {}",
                path,
                err
            );
            return Err(err);
        }
        Ok(())
    }

    fn ensure_default_profile(data: &mut ConfigFile) -> bool {
        if data.ssbusync.contains_key("Default") {
            false
        } else {
            data.ssbusync.insert("Default".to_string(), ProfileEntry::default());
            true
        }
    }

    fn string_enabled(value: &str) -> bool {
        let trimmed = value.trim();
        !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case("none")
    }

    fn string_flag(entry: &ProfileEntry, key: &str) -> (Option<String>, bool) {
        let value = entry
            .extra
            .get(key)
            .and_then(|value| value.as_str())
            .map(|value| value.trim().to_string());
        let enabled = value.as_deref().map(string_enabled).unwrap_or(false);
        let filtered = if enabled { value } else { None };
        (filtered, enabled)
    }

    pub fn load_or_create() -> io::Result<(SsbuSyncConfig, DefaultProfileState)> {
        println!("[ssbusync] attempting config load");
        paths::ensure_paths_exist()?;
        let path = paths::ssbusync_config();
        let mut data = load_file(&path)?;

        let changed = ensure_default_profile(&mut data);
        if changed {
            write_file(&path, &data)?;
        }

        let mut config = data
            .ssbusync
            .get("Default")
            .map(|entry| entry.config)
            .unwrap_or_else(SsbuSyncConfig::default);
        if let Some(entry) = data.ssbusync.get("Default") {
            let (_, enabled) = string_flag(entry, "overclock_profile");
            config.has_overclock_profile = enabled;
        }

        let state = if changed {
            DefaultProfileState::Created
        } else {
            DefaultProfileState::Loaded
        };

        Ok((config, state))
    }

    pub fn load_or_create_with_overclock_profile(
    ) -> io::Result<(SsbuSyncConfig, DefaultProfileState, Option<String>)> {
        println!("[ssbusync] attempting config load");
        paths::ensure_paths_exist()?;
        let path = paths::ssbusync_config();
        let mut data = load_file(&path)?;

        let changed = ensure_default_profile(&mut data);
        if changed {
            write_file(&path, &data)?;
        }

        let mut config = SsbuSyncConfig::default();
        let mut profile = None;
        if let Some(entry) = data.ssbusync.get("Default") {
            let (profile_name, enabled) = string_flag(entry, "overclock_profile");
            config = entry.config;
            config.has_overclock_profile = enabled;
            profile = profile_name;
        }

        let state = if changed {
            DefaultProfileState::Created
        } else {
            DefaultProfileState::Loaded
        };

        Ok((config, state, profile))
    }

    pub fn get_or_make_profile(
        name: &str,
        defaults: &SsbuSyncConfig,
        version: f32,
    ) -> io::Result<SsbuSyncConfig> {
        paths::ensure_paths_exist()?;
        let path = paths::ssbusync_config();
        let mut data = load_file(&path)?;

        let mut changed = ensure_default_profile(&mut data);
        let entry = data.ssbusync.entry(name.to_string());
        let mut should_write = false;

        match entry {
            std::collections::btree_map::Entry::Occupied(mut occ) => {
                if occ.get().profile_version < version {
                    should_write = true;
                    let existing = occ.get_mut();
                    existing.profile_version = version;
                    existing.config = *defaults;
                }
            }
            std::collections::btree_map::Entry::Vacant(vac) => {
                should_write = true;
                vac.insert(ProfileEntry {
                    profile_version: version,
                    config: *defaults,
                    extra: BTreeMap::new(),
                });
            }
        }

        if should_write || changed {
            write_file(&path, &data)?;
        }

        let mut config = data
            .ssbusync
            .get(name)
            .map(|entry| entry.config)
            .unwrap_or_else(|| *defaults);
        if let Some(entry) = data.ssbusync.get(name) {
            let (_, enabled) = string_flag(entry, "overclock_profile");
            config.has_overclock_profile = enabled;
        }
        Ok(config)
    }

    pub fn get_or_make_profile_with_overclock_profile(
        name: &str,
        defaults: &SsbuSyncConfig,
        version: f32,
    ) -> io::Result<(SsbuSyncConfig, Option<String>)> {
        paths::ensure_paths_exist()?;
        let path = paths::ssbusync_config();
        let mut data = load_file(&path)?;

        let mut changed = ensure_default_profile(&mut data);
        let entry = data.ssbusync.entry(name.to_string());
        let mut should_write = false;

        match entry {
            std::collections::btree_map::Entry::Occupied(mut occ) => {
                if occ.get().profile_version < version {
                    should_write = true;
                    let existing = occ.get_mut();
                    existing.profile_version = version;
                    existing.config = *defaults;
                }
            }
            std::collections::btree_map::Entry::Vacant(vac) => {
                should_write = true;
                vac.insert(ProfileEntry {
                    profile_version: version,
                    config: *defaults,
                    extra: BTreeMap::new(),
                });
            }
        }

        if should_write || changed {
            write_file(&path, &data)?;
        }

        let mut config = data
            .ssbusync
            .get(name)
            .map(|entry| entry.config)
            .unwrap_or_else(|| *defaults);
        let mut profile = None;
        if let Some(entry) = data.ssbusync.get(name) {
            let (profile_name, enabled) = string_flag(entry, "overclock_profile");
            config.has_overclock_profile = enabled;
            profile = profile_name;
        }
        Ok((config, profile))
    }
}

mod profile_store {
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;

    pub trait ProfileConfig:
        Copy + Default + Serialize + for<'de> Deserialize<'de>
    {
    }

    impl<T> ProfileConfig for T where
        T: Copy + Default + Serialize + for<'de> Deserialize<'de>
    {
    }

    const DEFAULT_PROFILE_VERSION: f32 = 1.0;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(default, bound = "T: ProfileConfig")]
    pub struct ProfileEntry<T: ProfileConfig> {
        pub profile_version: f32,
        #[serde(flatten)]
        pub config: T,
        #[serde(default)]
        pub subprofiles: BTreeMap<String, SubProfileEntry<T>>,
        #[serde(flatten)]
        pub extra: BTreeMap<String, toml::Value>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(default, bound = "T: ProfileConfig")]
    pub struct SubProfileEntry<T: ProfileConfig> {
        pub profile_version: f32,
        #[serde(flatten)]
        pub config: T,
        #[serde(flatten)]
        pub extra: BTreeMap<String, toml::Value>,
    }

    impl<T: ProfileConfig> Default for ProfileEntry<T> {
        fn default() -> Self {
            Self {
                profile_version: DEFAULT_PROFILE_VERSION,
                config: T::default(),
                subprofiles: BTreeMap::new(),
                extra: BTreeMap::new(),
            }
        }
    }

    impl<T: ProfileConfig> Default for SubProfileEntry<T> {
        fn default() -> Self {
            Self {
                profile_version: DEFAULT_PROFILE_VERSION,
                config: T::default(),
                extra: BTreeMap::new(),
            }
        }
    }

    pub fn ensure_profile<T: ProfileConfig>(
        profiles: &mut BTreeMap<String, ProfileEntry<T>>,
        name: &str,
    ) -> bool {
        if profiles.contains_key(name) {
            false
        } else {
            profiles.insert(name.to_string(), ProfileEntry::default());
            true
        }
    }

    pub fn get_profile<T: ProfileConfig>(
        profiles: &BTreeMap<String, ProfileEntry<T>>,
        name: &str,
        defaults: &T,
    ) -> T {
        profiles
            .get(name)
            .map(|entry| entry.config)
            .unwrap_or_else(|| *defaults)
    }

    pub fn get_subprofile<T: ProfileConfig>(
        profiles: &BTreeMap<String, ProfileEntry<T>>,
        profile: &str,
        name: &str,
        defaults: &T,
    ) -> T {
        profiles
            .get(profile)
            .and_then(|entry| entry.subprofiles.get(name))
            .map(|entry| entry.config)
            .unwrap_or_else(|| *defaults)
    }

    pub fn get_first_subprofile<T: ProfileConfig>(
        profiles: &BTreeMap<String, ProfileEntry<T>>,
        profile: &str,
    ) -> Option<T> {
        profiles.get(profile).and_then(|entry| {
            entry
                .subprofiles
                .iter()
                .next()
                .map(|(_, sub)| sub.config)
        })
    }

    pub fn upsert_profile<T: ProfileConfig>(
        profiles: &mut BTreeMap<String, ProfileEntry<T>>,
        name: &str,
        defaults: &T,
        version: f32,
    ) -> bool {
        let entry = profiles.entry(name.to_string());
        match entry {
            std::collections::btree_map::Entry::Occupied(mut occ) => {
                if occ.get().profile_version < version {
                    let existing = occ.get_mut();
                    existing.profile_version = version;
                    existing.config = *defaults;
                    return true;
                }
            }
            std::collections::btree_map::Entry::Vacant(vac) => {
                vac.insert(ProfileEntry {
                    profile_version: version,
                    config: *defaults,
                    subprofiles: BTreeMap::new(),
                    extra: BTreeMap::new(),
                });
                return true;
            }
        }
        false
    }

    pub fn upsert_subprofile<T: ProfileConfig>(
        profiles: &mut BTreeMap<String, ProfileEntry<T>>,
        profile: &str,
        name: &str,
        defaults: &T,
        version: f32,
    ) -> bool {
        let profile_entry = profiles
            .entry(profile.to_string())
            .or_insert_with(ProfileEntry::default);
        let sub_entry = profile_entry.subprofiles.entry(name.to_string());
        match sub_entry {
            std::collections::btree_map::Entry::Occupied(mut occ) => {
                if occ.get().profile_version < version {
                    let existing = occ.get_mut();
                    existing.profile_version = version;
                    existing.config = *defaults;
                    return true;
                }
            }
            std::collections::btree_map::Entry::Vacant(vac) => {
                vac.insert(SubProfileEntry {
                    profile_version: version,
                    config: *defaults,
                    extra: BTreeMap::new(),
                });
                return true;
            }
        }
        false
    }

    pub fn set_profile<T: ProfileConfig>(
        profiles: &mut BTreeMap<String, ProfileEntry<T>>,
        name: &str,
        config: &T,
        version: f32,
    ) {
        let entry = profiles
            .entry(name.to_string())
            .or_insert_with(ProfileEntry::default);
        entry.profile_version = version;
        entry.config = *config;
    }

    pub fn set_subprofile<T: ProfileConfig>(
        profiles: &mut BTreeMap<String, ProfileEntry<T>>,
        profile: &str,
        name: &str,
        config: &T,
        version: f32,
    ) {
        let entry = profiles
            .entry(profile.to_string())
            .or_insert_with(ProfileEntry::default);
        let sub_entry = entry
            .subprofiles
            .entry(name.to_string())
            .or_insert_with(SubProfileEntry::default);
        sub_entry.profile_version = version;
        sub_entry.config = *config;
    }
}

pub mod disablers {
    use super::paths;
    use arcropolis_api::hash40;
    use serde::{Deserialize, Serialize};
    use skyline::nn::ro;
    use std::io;

    #[derive(Debug, Serialize, Deserialize, Default)]
    #[serde(default)]
    struct DisablersFile {
        #[serde(rename = "SsbuSync")]
        ssbusync: DisablersEntry,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(default)]
    struct DisablersEntry {
        mods: Vec<String>,
    }

    impl Default for DisablersEntry {
        fn default() -> Self {
            Self {
                mods: vec!["hdr".to_string(), "hdr-dev".to_string()],
            }
        }
    }

    fn io_err(err: impl std::fmt::Display) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, err.to_string())
    }

    fn arcropolis_api_available() -> bool {
        println!("[ssbusync] checking for arcropolis \n");
        let mut addr = 0usize;
        unsafe { ro::LookupSymbol(&mut addr, b"arcrop_api_version\0".as_ptr()) == 0 && addr != 0 }
    }

    type ArcropIsModEnabledFn = unsafe extern "C" fn(u64) -> bool;

    fn lookup_arcrop_is_mod_enabled() -> Option<ArcropIsModEnabledFn> {
        let mut addr = 0usize;
        let rc = unsafe { ro::LookupSymbol(&mut addr, b"arcrop_is_mod_enabled\0".as_ptr()) };
        if rc != 0 || addr == 0 {
            return None;
        }
        Some(unsafe { std::mem::transmute(addr) })
    }

    fn load_file(path: &camino::Utf8PathBuf) -> io::Result<DisablersFile> {
        if !path.exists() {
            return Ok(DisablersFile::default());
        }
        let data = match std::fs::read_to_string(path.as_std_path()) {
            Ok(data) => data,
            Err(err) => {
                println!("[ssbusync][disablers] failed reading {}: {}", path, err);
                return Err(err);
            }
        };

        match toml::from_str(&data) {
            Ok(parsed) => Ok(parsed),
            Err(err) => {
                println!(
                    "[ssbusync][disablers] failed parsing TOML {}: {}",
                    path, err
                );
                Err(io_err(err))
            }
        }
    }

    fn write_file(path: &camino::Utf8PathBuf, data: &DisablersFile) -> io::Result<()> {
        let toml = match toml::to_string_pretty(data) {
            Ok(toml) => toml,
            Err(err) => {
                println!(
                    "[ssbusync][disablers] failed serializing TOML {}: {}",
                    path, err
                );
                return Err(io_err(err));
            }
        };

        if let Err(err) = std::fs::write(path.as_std_path(), toml) {
            println!("[ssbusync][disablers] failed writing {}: {}", path, err);
            return Err(err);
        }
        Ok(())
    }

    fn normalize_mod_path(entry: &str) -> Option<String> {
        let entry = entry.trim();
        if entry.is_empty() {
            return None;
        }
        Some(format!(
            "sd:/ultimate/mods/{}",
            entry.trim_start_matches("sd:/")
                .trim_start_matches("ultimate/")
                .trim_start_matches("mods/")
                .trim_start_matches('/')
        ))
    }

    pub fn load_or_create() -> io::Result<Vec<String>> {
        paths::ensure_paths_exist()?;
        let path = paths::ssbusync_disablers();
        let created = !path.exists();
        let data = load_file(&path)?;
        if created {
            write_file(&path, &data)?;
            println!(
                "[ssbusync][disablers] created {} with default disablers: hdr, hdr-dev",
                path
            );
        }
        Ok(data.ssbusync.mods)
    }

    pub fn active_disabler_mod() -> io::Result<Option<String>> {
        let mods = load_or_create()?;
        if mods.is_empty() {
            return Ok(None);
        }

        if crate::emulator_status() || !arcropolis_api_available() {
            for entry in &mods {
                let Some(path) = normalize_mod_path(entry) else {
                    continue;
                };
                if std::path::Path::new(&path).is_dir() {
                    println!(
                        "[ssbusync][disablers] folder check matched disabler '{}'",
                        path
                    );
                    return Ok(Some(path));
                }
            }
            return Ok(None);
        }

        let Some(is_mod_enabled) = lookup_arcrop_is_mod_enabled() else {
            for entry in &mods {
                let Some(path) = normalize_mod_path(entry) else {
                    continue;
                };
                if std::path::Path::new(&path).is_dir() {
                    println!(
                        "[ssbusync][disablers] fallback folder check matched disabler '{}'",
                        path
                    );
                    return Ok(Some(path));
                }
            }
            return Ok(None);
        };

        for entry in &mods {
            let Some(path) = normalize_mod_path(entry) else {
                continue;
            };
            if unsafe { is_mod_enabled(hash40(path.as_str()).as_u64()) } {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    pub fn check_disabler_mods() -> bool {
        match active_disabler_mod() {
            Ok(Some(mod_path)) => {
                println!(
                    "[ssbusync] disablers.toml matched active mod '{}'; disabling ssbusync install",
                    mod_path
                );
                true
            }
            Ok(None) => false,
            Err(err) => {
                println!(
                    "[ssbusync] failed reading disablers.toml ({}); continuing install checks",
                    err
                );
                false
            }
        }
    }
}

pub mod overclock {
    use super::paths;
    use super::profile_store;
    use crate::OverclockConfig;
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;
    use std::io;

    #[derive(Debug, Serialize, Deserialize, Default)]
    #[serde(default)]
    struct OverclockFile {
        #[serde(rename = "NxOverclock")]
        overclock: BTreeMap<String, profile_store::ProfileEntry<OverclockConfig>>,
        #[serde(flatten)]
        extra: BTreeMap<String, toml::Value>,
    }

    fn io_err(err: impl std::fmt::Display) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, err.to_string())
    }

    fn load_file(path: &camino::Utf8PathBuf) -> io::Result<OverclockFile> {
        if !path.exists() {
            return Ok(OverclockFile::default());
        }
        let data = match std::fs::read_to_string(path.as_std_path()) {
            Ok(data) => data,
            Err(err) => {
                println!(
                    "[ssbusync][overclock] failed reading {}: {}",
                    path,
                    err
                );
                return Err(err);
            }
        };

        match toml::from_str(&data) {
            Ok(parsed) => Ok(parsed),
            Err(err) => {
                println!(
                    "[ssbusync][overclock] failed parsing TOML {}: {}",
                    path,
                    err
                );
                Err(io_err(err))
            }
        }
    }

    fn write_file(path: &camino::Utf8PathBuf, data: &OverclockFile) -> io::Result<()> {
        let toml = match toml::to_string_pretty(data) {
            Ok(toml) => toml,
            Err(err) => {
                println!(
                    "[ssbusync][overclock] failed serializing TOML {}: {}",
                    path,
                    err
                );
                return Err(io_err(err));
            }
        };

        if let Err(err) = std::fs::write(path.as_std_path(), toml) {
            println!(
                "[ssbusync][overclock] failed writing {}: {}",
                path,
                err
            );
            return Err(err);
        }
        Ok(())
    }

    fn ensure_default_profiles(data: &mut OverclockFile) -> bool {
        let mut changed = false;
        changed |= profile_store::ensure_profile(&mut data.overclock, "Default");
        changed |= profile_store::ensure_profile(&mut data.overclock, "HDR");
        changed
    }

    pub fn load_or_create() -> io::Result<()> {
        paths::ensure_paths_exist()?;
        let path = paths::ssbusync_overclock();
        let created = !path.exists();
        let mut data = load_file(&path)?;
        let changed = ensure_default_profiles(&mut data);
        if created || changed {
            write_file(&path, &data)?;
        }
        Ok(())
    }

    pub fn get_or_make_profile(
        name: &str,
        defaults: &OverclockConfig,
        version: f32,
    ) -> io::Result<OverclockConfig> {
        paths::ensure_paths_exist()?;
        let path = paths::ssbusync_overclock();
        let mut data = load_file(&path)?;

        let mut changed = ensure_default_profiles(&mut data);
        let should_write =
            profile_store::upsert_profile(&mut data.overclock, name, defaults, version);

        if should_write || changed {
            write_file(&path, &data)?;
        }

        Ok(profile_store::get_profile(
            &data.overclock,
            name,
            defaults,
        ))
    }

    pub fn get_or_make_subprofile(
        profile: &str,
        name: &str,
        defaults: &OverclockConfig,
        version: f32,
    ) -> io::Result<OverclockConfig> {
        paths::ensure_paths_exist()?;
        let path = paths::ssbusync_overclock();
        let mut data = load_file(&path)?;

        let mut changed = ensure_default_profiles(&mut data);
        let should_write = profile_store::upsert_subprofile(
            &mut data.overclock,
            profile,
            name,
            defaults,
            version,
        );

        if should_write || changed {
            write_file(&path, &data)?;
        }

        Ok(profile_store::get_subprofile(
            &data.overclock,
            profile,
            name,
            defaults,
        ))
    }

    pub fn get_first_subprofile(profile: &str) -> io::Result<Option<OverclockConfig>> {
        paths::ensure_paths_exist()?;
        let path = paths::ssbusync_overclock();
        let mut data = load_file(&path)?;

        let changed = ensure_default_profiles(&mut data);
        if changed {
            write_file(&path, &data)?;
        }

        Ok(profile_store::get_first_subprofile(
            &data.overclock,
            profile,
        ))
    }

    pub fn set_profile(
        name: &str,
        config: &OverclockConfig,
        version: f32,
    ) -> io::Result<()> {
        paths::ensure_paths_exist()?;
        let path = paths::ssbusync_overclock();
        let mut data = load_file(&path)?;

        ensure_default_profiles(&mut data);
        profile_store::set_profile(&mut data.overclock, name, config, version);

        write_file(&path, &data)?;
        Ok(())
    }

    pub fn set_subprofile(
        profile: &str,
        name: &str,
        config: &OverclockConfig,
        version: f32,
    ) -> io::Result<()> {
        paths::ensure_paths_exist()?;
        let path = paths::ssbusync_overclock();
        let mut data = load_file(&path)?;

        ensure_default_profiles(&mut data);
        profile_store::set_subprofile(&mut data.overclock, profile, name, config, version);

        write_file(&path, &data)?;
        Ok(())
    }
}
