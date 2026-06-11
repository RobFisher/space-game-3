use crate::providers::EphemerisProvider;
use crate::{EphemerisError, FrameId, GameTime, ObjectDefinition, ObjectRegistry, StateVector};

#[allow(dead_code)]
#[derive(Debug, Default)]
pub(crate) struct SpiceProvider;

impl EphemerisProvider for SpiceProvider {
    fn state(
        &self,
        object: &ObjectDefinition,
        _epoch: &GameTime,
        _frame: &FrameId,
        _registry: &ObjectRegistry,
    ) -> Result<StateVector, EphemerisError> {
        Err(EphemerisError::Backend(format!(
            "SPICE provider is not implemented for object {}",
            object.id
        )))
    }
}
