#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ImpossibleTriggerModel;

impl ImpossibleTriggerModel {
    pub fn add_player_listener(
        &self,
        _player: &mut PlayerAdvancementsModel,
        _listener: ImpossibleTriggerListenerModel,
    ) {
    }

    pub fn remove_player_listener(
        &self,
        _player: &mut PlayerAdvancementsModel,
        _listener: ImpossibleTriggerListenerModel,
    ) {
    }

    pub fn remove_player_listeners(&self, _player: &mut PlayerAdvancementsModel) {}

    pub fn codec(&self) -> ImpossibleTriggerInstanceCodecModel {
        ImpossibleTriggerInstanceCodecModel
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ImpossibleTriggerInstanceModel;

impl ImpossibleTriggerInstanceModel {
    pub fn validate(&self) -> Vec<String> {
        Vec::new()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ImpossibleTriggerInstanceCodecModel;

impl ImpossibleTriggerInstanceCodecModel {
    pub fn decode_unit(&self) -> ImpossibleTriggerInstanceModel {
        ImpossibleTriggerInstanceModel
    }

    pub fn encode_unit(&self, _instance: ImpossibleTriggerInstanceModel) -> UnitMapModel {
        UnitMapModel
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UnitMapModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerAdvancementsModel {
    listeners: Vec<ImpossibleTriggerListenerModel>,
}

impl PlayerAdvancementsModel {
    pub fn with_listener(listener: ImpossibleTriggerListenerModel) -> Self {
        Self {
            listeners: vec![listener],
        }
    }

    pub fn listeners(&self) -> &[ImpossibleTriggerListenerModel] {
        &self.listeners
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImpossibleTriggerListenerModel {
    id: u32,
}

impl ImpossibleTriggerListenerModel {
    pub fn new(id: u32) -> Self {
        Self { id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn listener_registration_methods_are_no_ops() {
        let trigger = ImpossibleTriggerModel;
        let existing = ImpossibleTriggerListenerModel::new(1);
        let added = ImpossibleTriggerListenerModel::new(2);
        let mut player = PlayerAdvancementsModel::with_listener(existing);

        trigger.add_player_listener(&mut player, added);
        assert_eq!(player.listeners(), &[existing]);

        trigger.remove_player_listener(&mut player, existing);
        assert_eq!(player.listeners(), &[existing]);

        trigger.remove_player_listeners(&mut player);
        assert_eq!(player.listeners(), &[existing]);
    }

    #[test]
    fn codec_is_unit_codec_for_single_empty_trigger_instance() {
        let codec = ImpossibleTriggerModel.codec();
        let instance = codec.decode_unit();

        assert_eq!(instance, ImpossibleTriggerInstanceModel);
        assert_eq!(codec.encode_unit(instance), UnitMapModel);
        assert_eq!(codec.decode_unit(), ImpossibleTriggerInstanceModel);
    }

    #[test]
    fn trigger_instance_validation_is_empty() {
        assert!(ImpossibleTriggerInstanceModel.validate().is_empty());
    }
}
