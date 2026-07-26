use crate::model::{ReleaseProfile, PROFILE_SCHEMA_VERSION};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path};

const MAX_PROFILE_BYTES: u64 = 1024 * 1024;
const MAX_TIMEOUT_SECONDS: u64 = 7_200;
const MAX_RAILS: usize = 128;
const MAX_ARTIFACTS: usize = 512;
const MAX_IDENTITIES: usize = 512;
const MAX_ARGUMENTS: usize = 256;
const MAX_ENVIRONMENT_ENTRIES: usize = 64;
const MAX_STRING_BYTES: usize = 16 * 1024;
const MIN_LOG_BYTES: u64 = 4 * 1024;
const MAX_LOG_BYTES: u64 = 256 * 1024 * 1024;

pub struct LoadedProfile {
    pub profile: ReleaseProfile,
    pub blake3: String,
}

pub fn load_profile(path: &Path) -> Result<LoadedProfile, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("failed to inspect profile {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "profile {} must be a regular non-symlink file",
            path.display()
        ));
    }
    if metadata.len() > MAX_PROFILE_BYTES {
        return Err(format!(
            "profile {} exceeds the {MAX_PROFILE_BYTES}-byte bound",
            path.display()
        ));
    }

    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read profile {}: {error}", path.display()))?;
    let profile: ReleaseProfile = serde_json::from_slice(&bytes)
        .map_err(|error| format!("failed to parse profile {}: {error}", path.display()))?;
    validate_profile(&profile)?;

    let mut hasher = blake3::Hasher::new_derive_key("bcinr.release.profile.v2");
    hasher.update(&bytes);
    Ok(LoadedProfile {
        profile,
        blake3: hasher.finalize().to_hex().to_string(),
    })
}

pub fn validate_output_directory(path: &str) -> Result<(), String> {
    validate_repo_relative(path, "output_directory")?;
    let components = Path::new(path).components().collect::<Vec<_>>();
    if components.len() < 2 || components[0].as_os_str() != "target" {
        return Err("output_directory must be located below target/".to_owned());
    }
    Ok(())
}

pub fn validate_repo_relative(value: &str, label: &str) -> Result<(), String> {
    validate_bounded_string(value, label)?;
    let path = Path::new(value);
    if path.is_absolute() {
        return Err(format!("{label} must be repository-relative"));
    }
    if path.components().any(|part| {
        matches!(
            part,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(format!("{label} cannot escape the repository root"));
    }
    Ok(())
}

fn validate_profile(profile: &ReleaseProfile) -> Result<(), String> {
    if profile.schema_version != PROFILE_SCHEMA_VERSION {
        return Err(format!(
            "unsupported profile schema version {}; expected {PROFILE_SCHEMA_VERSION}",
            profile.schema_version
        ));
    }
    validate_release(&profile.release)?;
    validate_repository(&profile.repository)?;
    validate_output_directory(&profile.output_directory)?;

    if !(MIN_LOG_BYTES..=MAX_LOG_BYTES).contains(&profile.max_log_bytes) {
        return Err(format!(
            "max_log_bytes must be between {MIN_LOG_BYTES} and {MAX_LOG_BYTES}"
        ));
    }
    if profile.rails.is_empty() || profile.rails.len() > MAX_RAILS {
        return Err(format!(
            "profile must contain between 1 and {MAX_RAILS} rails"
        ));
    }
    if profile.artifacts.len() > MAX_ARTIFACTS {
        return Err(format!("profile exceeds {MAX_ARTIFACTS} artifacts"));
    }
    if profile.byte_identity.len() > MAX_IDENTITIES {
        return Err(format!("profile exceeds {MAX_IDENTITIES} byte identities"));
    }

    let inherited_environment = validate_environment_names(&profile.inherited_environment)?;
    if !inherited_environment.contains("PATH") {
        return Err("inherited_environment must include PATH".to_owned());
    }

    let mut ids = BTreeSet::new();
    for rail in &profile.rails {
        validate_id(&rail.id, "rail id")?;
        if !ids.insert(rail.id.as_str()) {
            return Err(format!("duplicate admission id {}", rail.id));
        }
        validate_bounded_string(&rail.phase, "rail phase")?;
        validate_program(&rail.program, &rail.id)?;
        validate_repo_relative(&rail.working_directory, "working_directory")?;
        if rail.timeout_seconds == 0 || rail.timeout_seconds > MAX_TIMEOUT_SECONDS {
            return Err(format!(
                "rail {} timeout must be between 1 and {MAX_TIMEOUT_SECONDS} seconds",
                rail.id
            ));
        }
        if rail.args.len() > MAX_ARGUMENTS {
            return Err(format!(
                "rail {} exceeds the {MAX_ARGUMENTS}-argument bound",
                rail.id
            ));
        }
        for argument in &rail.args {
            validate_bounded_string(argument, "rail argument")?;
        }
        if rail.environment.len() > MAX_ENVIRONMENT_ENTRIES {
            return Err(format!(
                "rail {} exceeds the {MAX_ENVIRONMENT_ENTRIES}-variable bound",
                rail.id
            ));
        }
        for (name, value) in &rail.environment {
            validate_environment_name(name)?;
            refuse_loader_injection(name)?;
            validate_bounded_string(value, "environment value")?;
        }
    }

    for artifact in &profile.artifacts {
        validate_id(&artifact.id, "artifact id")?;
        if !ids.insert(artifact.id.as_str()) {
            return Err(format!("duplicate admission id {}", artifact.id));
        }
        validate_repo_relative(&artifact.path, "artifact path")?;
        if !(1..=4096).contains(&artifact.minimum_files) {
            return Err(format!(
                "artifact {} minimum_files must be between 1 and 4096",
                artifact.id
            ));
        }
        if let Some(suffix) = &artifact.suffix {
            validate_bounded_string(suffix, "artifact suffix")?;
            if !artifact.recursive || !suffix.starts_with('.') {
                return Err(format!(
                    "artifact {} suffix requires recursive=true and a leading '.'",
                    artifact.id
                ));
            }
        }
    }

    for identity in &profile.byte_identity {
        validate_id(&identity.id, "byte identity id")?;
        if !ids.insert(identity.id.as_str()) {
            return Err(format!("duplicate admission id {}", identity.id));
        }
        validate_repo_relative(&identity.left, "byte identity left path")?;
        validate_repo_relative(&identity.right, "byte identity right path")?;
        if identity.left == identity.right {
            return Err(format!(
                "byte identity {} must compare distinct paths",
                identity.id
            ));
        }
    }

    Ok(())
}

fn validate_release(release: &str) -> Result<(), String> {
    validate_bounded_string(release, "release")?;
    let components = release.split('.').collect::<Vec<_>>();
    if components.len() != 3
        || components
            .iter()
            .any(|component| component.is_empty() || !component.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return Err("release must have numeric YEAR.MONTH.DAY form".to_owned());
    }
    Ok(())
}

fn validate_repository(repository: &str) -> Result<(), String> {
    validate_bounded_string(repository, "repository")?;
    let mut parts = repository.split('/');
    let owner = parts.next().unwrap_or_default();
    let name = parts.next().unwrap_or_default();
    if parts.next().is_some()
        || owner.is_empty()
        || name.is_empty()
        || !owner.bytes().all(is_repository_byte)
        || !name.bytes().all(is_repository_byte)
    {
        return Err("repository must use exact owner/name form".to_owned());
    }
    Ok(())
}

const fn is_repository_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')
}

fn validate_id(value: &str, label: &str) -> Result<(), String> {
    validate_bounded_string(value, label)?;
    if !value.bytes().all(is_repository_byte) {
        return Err(format!(
            "{label} must contain only ASCII letters, digits, '-', '_' or '.'"
        ));
    }
    Ok(())
}

fn validate_program(program: &str, rail_id: &str) -> Result<(), String> {
    validate_bounded_string(program, "program")?;
    let path = Path::new(program);
    if path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
    {
        return Err(format!(
            "rail {rail_id} program must be a system command or repository-relative path"
        ));
    }
    Ok(())
}

fn validate_environment_names(values: &[String]) -> Result<BTreeSet<&str>, String> {
    if values.len() > MAX_ENVIRONMENT_ENTRIES {
        return Err(format!(
            "inherited_environment exceeds the {MAX_ENVIRONMENT_ENTRIES}-variable bound"
        ));
    }
    let mut names = BTreeSet::new();
    for value in values {
        validate_environment_name(value)?;
        refuse_loader_injection(value)?;
        if !names.insert(value.as_str()) {
            return Err(format!("duplicate inherited environment variable {value}"));
        }
    }
    Ok(names)
}

fn validate_environment_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.len() > 128
        || name.contains('=')
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(format!("invalid environment variable name {name:?}"));
    }
    Ok(())
}

fn refuse_loader_injection(name: &str) -> Result<(), String> {
    const REFUSED: &[&str] = &[
        "DYLD_INSERT_LIBRARIES",
        "DYLD_LIBRARY_PATH",
        "LD_AUDIT",
        "LD_LIBRARY_PATH",
        "LD_PRELOAD",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
    ];
    if REFUSED.contains(&name) {
        return Err(format!("environment variable {name} is refused"));
    }
    Ok(())
}

fn validate_bounded_string(value: &str, label: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.contains('\0') {
        return Err(format!("{label} cannot be empty or contain NUL"));
    }
    if value.len() > MAX_STRING_BYTES {
        return Err(format!("{label} exceeds {MAX_STRING_BYTES} bytes"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ReleaseProfile, RailSpec};
    use std::collections::BTreeMap;

    fn profile() -> ReleaseProfile {
        ReleaseProfile {
            schema_version: PROFILE_SCHEMA_VERSION,
            release: "26.7.28".to_owned(),
            repository: "seanchatmangpt/bcinr".to_owned(),
            output_directory: "target/release-evidence/v26.7.28".to_owned(),
            max_log_bytes: 1024 * 1024,
            inherited_environment: vec!["PATH".to_owned(), "HOME".to_owned()],
            rails: vec![RailSpec {
                id: "format".to_owned(),
                phase: "static".to_owned(),
                program: "cargo".to_owned(),
                args: vec!["fmt".to_owned()],
                working_directory: ".".to_owned(),
                timeout_seconds: 60,
                required: true,
                environment: BTreeMap::new(),
            }],
            artifacts: Vec::new(),
            byte_identity: Vec::new(),
        }
    }

    #[test]
    fn accepts_bounded_profile() {
        assert!(validate_profile(&profile()).is_ok());
    }

    #[test]
    fn refuses_output_outside_target() {
        let mut value = profile();
        value.output_directory = "release-evidence".to_owned();
        assert!(validate_profile(&value).is_err());
    }

    #[test]
    fn refuses_parent_traversal() {
        let mut value = profile();
        value.rails[0].working_directory = "../ggen".to_owned();
        assert!(validate_profile(&value).is_err());
    }

    #[test]
    fn refuses_loader_injection() {
        let mut value = profile();
        value.inherited_environment.push("LD_PRELOAD".to_owned());
        assert!(validate_profile(&value).is_err());
    }

    #[test]
    fn refuses_duplicate_ids_across_kinds() {
        let mut value = profile();
        value.artifacts.push(crate::model::ArtifactSpec {
            id: "format".to_owned(),
            path: "Cargo.lock".to_owned(),
            required: true,
            recursive: false,
            suffix: None,
            minimum_files: 1,
        });
        assert!(validate_profile(&value).is_err());
    }
}
