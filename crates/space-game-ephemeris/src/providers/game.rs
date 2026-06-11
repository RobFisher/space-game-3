use crate::providers::EphemerisProvider;
use crate::{
    resolution, EphemerisError, FrameId, GameTime, ObjectDefinition, ObjectRegistry, StateVector,
};

#[allow(dead_code)]
#[derive(Debug, Default)]
pub(crate) struct GameObjectProvider;

impl EphemerisProvider for GameObjectProvider {
    fn state(
        &self,
        object: &ObjectDefinition,
        epoch: &GameTime,
        _frame: &FrameId,
        registry: &ObjectRegistry,
    ) -> Result<StateVector, EphemerisError> {
        resolution::resolve_global_state(registry, object.id.as_str(), epoch)
    }
}
