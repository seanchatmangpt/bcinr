use crate::model::ReleaseProfile;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path};

const MAX_TIMEOUT_SECONDS: u64 = 7_200;

pub fn load_profile(path: &Path) -> Result<ReleaseProfile, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("failed to read profile {}: {error}", path.display()))?;
    let profile: ReleaseProfile = serde_json::from_slice(&bytes)
        .map_err(|error| format!("failed to parse profile {}: {error}", path.display()))?;
    validate_profile(&profile)?;
    Ok(profile)
}

pub fn validate_output_directory(path: &str) -> Result<(), String> {
    validate_repo_relative(path, "output_directory")?;
    let first = Path::new(path)
        .components()
        .next()
        .ok_or_else(|| "output_directory cannot be empty".to_owned())?;
    if first.as_os_str() != "target" {
        return Err("output_directory must be located under target/".to_owned());
    }
    Ok(())
}

fn validate_profile(profile: &ReleaseProfile) -> Result<(), String> {
    if profile.schema_version != 1 {
        return Err(format!(
            "unsupported profile schema version {}; expected 1",
            profile.schema_version
        ));
    }
    if profile.release.trim().is_empty() {
        return Err("release cannot be empty".to_owned());
    }
    if profile.repository.trim().is_empty() || !profile.repository.contains('/') {
        return Err("repository must use owner/name form".to_owned());
    }
    validate_output_directory(&profile.output_directory)?;
    if profile.rails.is_empty() {
        return Err("profile must contain at least one rail".to_owned());
    }

    let mut ids = BTreeSet::new();
    for rail in &profile.rails {
        validate_id(&rail.id, "rail id")?;
        if !ids.insert(format!("rail:{}", rail.id)) {
            return Err(format!("duplicate rail id {}", rail.id));
        }
        if rail.phase.trim().is_empty() {
            return Err(format!("rail {} has an empty phase", rail.id));
        }
        validate_program(&rail.program, &rail.id)?;
        validate_repo_relative(&rail.working_directory, "working_directory")?;
        if rail.timeout_seconds == 0 || rail.timeout_seconds > MAX_TIMEOUT_SECONDS {
            return Err(format!(
                "rail {} timeout must be between 1 and {MAX_TIMEOUT_SECONDS} seconds",
                rail.id
            ));
        }
        for (name, value) in &rail.environment {
            if name.trim().is_empty() || name.contains('=') || name.contains('\0') {
                return Err(format!("rail {} contains an invalid environment key", rail.id));
            }
            if value.contains('\0') {
                return Err(format!("rail {} contains a NUL environment value", rail.id));
            }
        }
    }

    for artifact in &profile.artifacts {
        validate_id(&artifact.id, "artifact id")?;
        if !ids.insert(format!("artifact:{}", artifact.id)) {
            return Err(format!("duplicate artifact id {}", artifact.id));
        }
        validate_repo_relative(&artifact.path, "artifact path")?;
    }

    for identity in &profile.byte_identity {
        validate_id(&identity.id, "byte identity id")?;
        if !ids.insert(format!("identity:{}", identity.id)) {
            return Err(format!("duplicate byte identity id {}", identity.id));
        }
        validate_repo_relative(&identity.left, "byte identity left path")?;
        validate_repo_relative(&identity.right, "byte identity right path")?;
    }

    Ok(())
}

fn validate_id(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(format!(
            "{label} must contain only ASCII letters, digits, '-', '_' or '.'"
        ));
    }
    Ok(())
}

fn validate_program(program: &str, rail_id: &str) -> Result<(), String> {
    if program.trim().is_empty() || program.contains('\0') {
        return Err(format!("rail {rail_id} has an invalid program"));
    }
    let path = Path::new(program);
    if path.is_absolute() || path.components().any(|part| matches!(part, Component::ParentDir)) {
        return Err(format!(
            "rail {rail_id} program must be a system command or repository-relative path"
        ));
    }
    Ok(())
}

fn validate_repo_relative(value: &str, label: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.contains('\0') {
        return Err(format!("{label} cannot be empty or contain NUL"));
    }
    let path = Path::new(value);
    if path.is_absolute() {
        return Err(format!("{label} must be repository-relative"));
    }
    if path
        .components()
        .any(|part| matches!(part, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
    {
        return Err(format!("{label} cannot escape the repository root"));
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
            schema_version: 1,
            release: "26.7.28".to_owned(),
            repository: "seanchatmangpt/bcinr".to_owned(),
            output_directory: "target/release-evidence/v26.7.28".to_owned(),
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
}
