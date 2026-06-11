pub(crate) mod game;
pub(crate) mod spice;

use crate::{EphemerisError, FrameId, GameTime, ObjectDefinition, ObjectRegistry, StateVector};

#[allow(dead_code)]
pub(crate) trait EphemerisProvider {
    fn state(
        &self,
        object: &ObjectDefinition,
        epoch: &GameTime,
        frame: &FrameId,
        registry: &ObjectRegistry,
    ) -> Result<StateVector, EphemerisError>;
}
