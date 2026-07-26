use crate::model::Standing;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandingInputs {
    pub blocked: bool,
    pub required_rail_failure: bool,
    pub required_artifact_failure: bool,
    pub required_identity_failure: bool,
    pub optional_rail_failure: bool,
    pub optional_artifact_failure: bool,
    pub optional_identity_failure: bool,
}

pub const fn calculate(inputs: StandingInputs) -> Standing {
    if inputs.blocked
        || inputs.required_artifact_failure
        || inputs.required_identity_failure
    {
        Standing::Blocked
    } else if inputs.required_rail_failure {
        Standing::BuildBroken
    } else if inputs.optional_rail_failure
        || inputs.optional_artifact_failure
        || inputs.optional_identity_failure
    {
        Standing::PartialAlive
    } else {
        Standing::Alive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLEAN: StandingInputs = StandingInputs {
        blocked: false,
        required_rail_failure: false,
        required_artifact_failure: false,
        required_identity_failure: false,
        optional_rail_failure: false,
        optional_artifact_failure: false,
        optional_identity_failure: false,
    };

    #[test]
    fn admits_only_complete_success() {
        assert_eq!(calculate(CLEAN), Standing::Alive);
    }

    #[test]
    fn repository_refusal_has_highest_precedence() {
        let inputs = StandingInputs {
            blocked: true,
            required_rail_failure: true,
            optional_rail_failure: true,
            ..CLEAN
        };
        assert_eq!(calculate(inputs), Standing::Blocked);
    }

    #[test]
    fn required_artifact_and_identity_failures_block() {
        assert_eq!(
            calculate(StandingInputs {
                required_artifact_failure: true,
                ..CLEAN
            }),
            Standing::Blocked
        );
        assert_eq!(
            calculate(StandingInputs {
                required_identity_failure: true,
                ..CLEAN
            }),
            Standing::Blocked
        );
    }

    #[test]
    fn required_command_failure_is_build_broken() {
        assert_eq!(
            calculate(StandingInputs {
                required_rail_failure: true,
                ..CLEAN
            }),
            Standing::BuildBroken
        );
    }

    #[test]
    fn any_optional_failure_is_partial_alive() {
        for inputs in [
            StandingInputs {
                optional_rail_failure: true,
                ..CLEAN
            },
            StandingInputs {
                optional_artifact_failure: true,
                ..CLEAN
            },
            StandingInputs {
                optional_identity_failure: true,
                ..CLEAN
            },
        ] {
            assert_eq!(calculate(inputs), Standing::PartialAlive);
        }
    }
}
