use crate::call::AsyncCaller;
use candid::de::ArgumentDecoder;
use candid::ser::IDLBuilder;
use candid::CandidType;
use ic_agent::Agent;
use ic_types::{Principal, PrincipalError};
use std::convert::TryInto;
use thiserror::Error;

/// An error happened while building a canister.
#[derive(Debug, Error)]
pub enum CanisterBuilderError {
    #[error("Getting the Canister ID returned an error: {0}")]
    PrincipalError(#[from] PrincipalError),

    #[error("Must specify an Agent")]
    MustSpecifyAnAgent(),

    #[error("Must specify a Canister ID")]
    MustSpecifyCanisterId(),
}

/// A canister builder, which can be used to create a canister abstraction.
pub struct CanisterBuilder<'agent, T = ()> {
    agent: Option<&'agent Agent>,
    canister_id: Option<Result<Principal, PrincipalError>>,
    interface: T,
}

impl<'agent, T> CanisterBuilder<'agent, T> {
    /// Attach a canister ID to this canister.
    pub fn with_canister_id<E, P>(self, canister_id: P) -> Self
    where
        E: std::error::Error,
        P: TryInto<Principal, Error = E>,
    {
        Self {
            canister_id: Some(
                canister_id
                    .try_into()
                    .map_err(|e| PrincipalError::ExternalError(format!("{}", e))),
            ),
            ..self
        }
    }

    /// Assign an agent to the canister being built.
    pub fn with_agent(self, agent: &'agent Agent) -> Self {
        CanisterBuilder {
            agent: Some(agent),
            ..self
        }
    }

    /// Create this canister abstraction after passing in all the necessary state.
    pub fn build(self) -> Result<Canister<'agent, T>, CanisterBuilderError> {
        let canister_id = if let Some(cid) = self.canister_id {
            cid?
        } else {
            return Err(CanisterBuilderError::MustSpecifyCanisterId());
        };

        let agent = self
            .agent
            .ok_or(CanisterBuilderError::MustSpecifyAnAgent())?;
        Ok(Canister {
            agent,
            canister_id,
            interface: self.interface,
        })
    }
}

impl Default for CanisterBuilder<'static, ()> {
    fn default() -> Self {
        CanisterBuilder {
            agent: None,
            canister_id: None,
            interface: (),
        }
    }
}

impl<'agent> CanisterBuilder<'agent, ()> {
    /// Create a canister builder with no value.
    pub fn new() -> CanisterBuilder<'static, ()> {
        Default::default()
    }

    /// Apply an interface to this canister. An interface can add methods to the canister's
    /// type. For example, see the Management Canister.
    pub fn with_interface<T>(self, interface: T) -> CanisterBuilder<'agent, T> {
        CanisterBuilder {
            agent: self.agent,
            canister_id: self.canister_id,
            interface,
        }
    }
}

/// Create an encapsulation of a Canister running on the Internet Computer.
/// This supports making calls to methods, installing code if needed, and various
/// utilities related to a canister.
///
/// This is the higher level construct for talking to a canister on the Internet
/// Computer.
pub struct Canister<'agent, T = ()> {
    pub(super) agent: &'agent Agent,
    pub(super) canister_id: Principal,
    interface: T,
}

impl<'agent, T> Canister<'agent, T> {
    /// Get the interface object from this canister. Sometimes those interfaces might have
    /// custom methods that are useful.
    pub fn interface_(&self) -> &T {
        &self.interface
    }

    /// Create an AsyncCallBuilder to do an update call.
    pub fn update_<'canister>(
        &'canister self,
        method_name: &str,
    ) -> AsyncCallBuilder<'agent, 'canister, T> {
        AsyncCallBuilder::new(self, method_name)
    }
}

impl<'agent> Canister<'agent, ()> {
    /// Create a CanisterBuilder instance to build a canister abstraction.
    pub fn builder() -> CanisterBuilder<'agent, ()> {
        Default::default()
    }
}

/// A builder for an asynchronous call (ie. update) to the Internet Computer.
///
/// See [AsyncCaller] for a description of this structure.
pub struct AsyncCallBuilder<'agent, 'canister: 'agent, T> {
    canister: &'canister Canister<'agent, T>,
    method_name: String,
    arg: Result<candid::ser::IDLBuilder, candid::Error>,
}

impl<'agent, 'canister: 'agent, T> AsyncCallBuilder<'agent, 'canister, T> {
    /// Create a new instance of an AsyncCallBuilder.
    pub(super) fn new(
        canister: &'canister Canister<'agent, T>,
        method_name: &str,
    ) -> AsyncCallBuilder<'agent, 'canister, T> {
        Self {
            canister,
            method_name: method_name.to_string(),
            arg: Ok(IDLBuilder::new()),
        }
    }
}

impl<'agent, 'canister: 'agent, Interface> AsyncCallBuilder<'agent, 'canister, Interface> {
    /// Add an argument to the list. This requires Candid arguments, as well as
    pub fn with_arg<Argument>(
        mut self,
        arg: Argument,
    ) -> AsyncCallBuilder<'agent, 'canister, Interface>
    where
        Argument: CandidType + Sync + Send,
    {
        if let Ok(ref mut idl_builder) = self.arg {
            let result = idl_builder.arg(&arg);
            if let Err(e) = result {
                self.arg = Err(e)
            }
        }
        self
    }

    /// Builds an [AsyncCaller] from this builder's state.
    pub fn build<Output>(self) -> AsyncCaller<'canister, Output>
    where
        Output: for<'de> ArgumentDecoder<'de> + Send + Sync,
    {
        let c = self.canister;
        AsyncCaller {
            agent: c.agent,
            canister_id: c.canister_id.clone(),
            method_name: self.method_name.clone(),
            arg: self.arg.and_then(|mut idl| idl.serialize_to_vec()),
            phantom_out: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::interfaces::ManagementCanister;
    use crate::call::AsyncCall;
    use ic_agent::BasicIdentity;

    #[ignore]
    #[tokio::test]
    async fn simple() {
        use super::Canister;
        use delay::Delay;

        let rng = ring::rand::SystemRandom::new();
        let key_pair = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng)
            .expect("Could not generate a key pair.");

        let identity = BasicIdentity::from_key_pair(
            ring::signature::Ed25519KeyPair::from_pkcs8(key_pair.as_ref())
                .expect("Could not read the key pair."),
        );

        let agent = ic_agent::Agent::builder()
            .with_url("http://localhost:8001")
            .with_identity(identity)
            .build()
            .unwrap();

        let management_canister = Canister::builder()
            .with_agent(&agent)
            .with_canister_id("aaaaa-aa")
            .with_interface(ManagementCanister)
            .build()
            .unwrap();

        let (new_canister_id,) = management_canister
            .create_canister()
            .call_and_wait(Delay::throttle(std::time::Duration::from_secs(1)))
            .await
            .unwrap();

        let (status,) = management_canister
            .canister_status(&new_canister_id)
            .call_and_wait(Delay::throttle(std::time::Duration::from_secs(1)))
            .await
            .unwrap();

        assert_eq!(format!("{}", status), "Running");

        let canister_wasm = b"\0asm\x01\0\0\0";
        management_canister
            .install_code(&new_canister_id, canister_wasm)
            .call_and_wait(Delay::throttle(std::time::Duration::from_secs(1)))
            .await
            .unwrap();

        let canister = Canister::builder()
            .with_agent(&agent)
            .with_canister_id(new_canister_id)
            .build()
            .unwrap();

        assert!(canister
            .update_("hello")
            .build::<()>()
            .call_and_wait(Delay::throttle(std::time::Duration::from_secs(1)))
            .await
            .is_err());
    }
}