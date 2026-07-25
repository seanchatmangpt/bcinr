/// Compiler-style plan/process transformation for CMD pipelines.
///
/// `WorkflowApplication` is assembled from the same `PlanPass::then` algebra exposed
/// to custom consumers: planning, envelope manufacture, and command binding form one
/// root-continuous production chain. `Then` refuses any stage boundary whose declared
/// input root does not equal the prior output root. Custom pipelines may substitute
/// passes or semantic rails, but they inherit the same continuity and typed-refusal
/// obligations; there is no parallel direct facade path.
pub trait PlanPass<I> {
    type Output;
    type Witness;
    type Refusal;

    fn apply(&self, input: I) -> Result<PassOutput<Self::Output, Self::Witness>, Self::Refusal>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassOutput<T, W> {
    pub value: T,
    pub witness: W,
    pub input_root: PassRoot,
    pub output_root: PassRoot,
}

#[derive(Debug, Clone)]
pub struct Then<A, B> {
    first: A,
    second: B,
}

pub trait PlanPassExt<I>: PlanPass<I> + Sized {
    fn then<B>(self, second: B) -> Then<Self, B> {
        Then {
            first: self,
            second,
        }
    }
}

impl<I, P> PlanPassExt<I> for P where P: PlanPass<I> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PassChainRefusal<A, B> {
    First(A),
    Second(B),
    RootDiscontinuity {
        first_output: PassRoot,
        second_input: PassRoot,
    },
}

impl<I, A, B> PlanPass<I> for Then<A, B>
where
    A: PlanPass<I>,
    B: PlanPass<A::Output>,
{
    type Output = B::Output;
    type Witness = (A::Witness, B::Witness);
    type Refusal = PassChainRefusal<A::Refusal, B::Refusal>;

    fn apply(&self, input: I) -> Result<PassOutput<Self::Output, Self::Witness>, Self::Refusal> {
        let first = self.first.apply(input).map_err(PassChainRefusal::First)?;
        let first_input = first.input_root;
        let first_output = first.output_root;
        let first_witness = first.witness;
        let second = self
            .second
            .apply(first.value)
            .map_err(PassChainRefusal::Second)?;
        if second.input_root != first_output {
            return Err(PassChainRefusal::RootDiscontinuity {
                first_output,
                second_input: second.input_root,
            });
        }
        Ok(PassOutput {
            value: second.value,
            witness: (first_witness, second.witness),
            input_root: first_input,
            output_root: second.output_root,
        })
    }
}

/// Language-independent semantic rail contract. Implementations must publish
/// their own capability/refusal types and may not hide non-capability failures
/// behind fallback success.
pub trait SemanticRail<Request> {
    type Candidate;
    type Standing;
    type Refusal;

    fn rail_root(&self) -> SearchPolicyRoot;
    fn admit_and_plan(
        &mut self,
        request: &Request,
    ) -> Result<Artifact<Self::Candidate, Self::Standing>, Self::Refusal>;
}

/// Stable refusal categories for application policy and recovery routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowRefusalCode {
    SourceParse,
    Canonicalization,
    UnsupportedCapability,
    InconsistentTheory,
    InvalidObservation,
    InvalidGoal,
    BoundExhaustion,
    SearchExhaustion,
    PlanValidation,
    CausalAnalysis,
    ConcurrencyWitness,
    ProcessProjection,
    ProcessValidation,
    SchedulerDeadlock,
    ReplayMismatch,
    ActionLabel,
    CommandBinding,
    Policy,
    StaleObservation,
    BrokerAdmission,
    IdempotencyConflict,
    Actuation,
    EffectObservation,
    CursorMismatch,
    Replan,
    ReceiptMismatch,
    TransportTrust,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roots_are_domain_separated_and_roundtrip() {
        let bytes = b"same material";
        let plan = PlanRoot::hash(bytes);
        let effect = EffectRoot::hash(bytes);
        assert_ne!(plan.as_bytes(), effect.as_bytes());
        assert_eq!(plan.to_string().parse::<PlanRoot>().unwrap(), plan);
    }

    #[test]
    fn pass_chain_refuses_root_discontinuity() {
        struct First;
        struct Second;

        impl PlanPass<&'static str> for First {
            type Output = String;
            type Witness = &'static str;
            type Refusal = ();

            fn apply(
                &self,
                input: &'static str,
            ) -> Result<PassOutput<Self::Output, Self::Witness>, Self::Refusal> {
                Ok(PassOutput {
                    value: input.to_uppercase(),
                    witness: "upper",
                    input_root: PassRoot::hash(input.as_bytes()),
                    output_root: PassRoot::hash(input.to_uppercase().as_bytes()),
                })
            }
        }

        impl PlanPass<String> for Second {
            type Output = usize;
            type Witness = &'static str;
            type Refusal = ();

            fn apply(
                &self,
                input: String,
            ) -> Result<PassOutput<Self::Output, Self::Witness>, Self::Refusal> {
                Ok(PassOutput {
                    value: input.len(),
                    witness: "length",
                    input_root: PassRoot::hash(b"wrong"),
                    output_root: PassRoot::hash(&input.len().to_le_bytes()),
                })
            }
        }

        let result = First.then(Second).apply("hello");
        assert!(matches!(
            result,
            Err(PassChainRefusal::RootDiscontinuity { .. })
        ));
    }
}
