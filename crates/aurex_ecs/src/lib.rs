use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(pub u32);

#[derive(Debug, Clone, PartialEq)]
pub struct Transform2p5D {
    pub position: [f32; 3],
    pub rotation_yaw_pitch_roll: [f32; 3],
    pub scale: [f32; 3],
}

impl Default for Transform2p5D {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation_yaw_pitch_roll: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EcsCommand {
    SpawnEntity {
        entity: EntityId,
    },
    DespawnEntity {
        entity: EntityId,
    },
    SetTransform {
        entity: EntityId,
        transform: Transform2p5D,
    },
}

#[derive(Debug, Default)]
pub struct CommandBuffer {
    commands: Vec<EcsCommand>,
}

impl CommandBuffer {
    pub fn push(&mut self, command: EcsCommand) {
        self.commands.push(command);
    }

    pub fn drain_sorted(&mut self) -> Vec<EcsCommand> {
        let mut commands = std::mem::take(&mut self.commands);
        commands.sort_by_key(command_order_key);
        commands
    }
}

fn command_order_key(command: &EcsCommand) -> (u8, u32) {
    match command {
        EcsCommand::SpawnEntity { entity } => (0, entity.0),
        EcsCommand::SetTransform { entity, .. } => (1, entity.0),
        EcsCommand::DespawnEntity { entity } => (2, entity.0),
    }
}

#[derive(Debug, Default)]
pub struct EcsWorld {
    transforms: BTreeMap<EntityId, Transform2p5D>,
}

impl EcsWorld {
    pub fn apply_commands(&mut self, command_buffer: &mut CommandBuffer) {
        for command in command_buffer.drain_sorted() {
            match command {
                EcsCommand::SpawnEntity { entity } => {
                    self.transforms.entry(entity).or_default();
                }
                EcsCommand::DespawnEntity { entity } => {
                    self.transforms.remove(&entity);
                }
                EcsCommand::SetTransform { entity, transform } => {
                    if let Some(current) = self.transforms.get_mut(&entity) {
                        *current = transform;
                    }
                }
            }
        }
    }

    pub fn entity_count(&self) -> usize {
        self.transforms.len()
    }

    pub fn ordered_entities(&self) -> Vec<EntityId> {
        self.transforms.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_commands() -> CommandBuffer {
        let mut buffer = CommandBuffer::default();
        buffer.push(EcsCommand::SpawnEntity {
            entity: EntityId(3),
        });
        buffer.push(EcsCommand::SpawnEntity {
            entity: EntityId(1),
        });
        buffer.push(EcsCommand::SetTransform {
            entity: EntityId(1),
            transform: Transform2p5D {
                position: [1.0, 2.0, 3.0],
                ..Transform2p5D::default()
            },
        });
        buffer.push(EcsCommand::SpawnEntity {
            entity: EntityId(2),
        });
        buffer
    }

    #[test]
    fn deterministic_order_and_counts() {
        let mut world = EcsWorld::default();
        let mut commands = sample_commands();
        world.apply_commands(&mut commands);

        assert_eq!(world.entity_count(), 3);
        assert_eq!(
            world.ordered_entities(),
            vec![EntityId(1), EntityId(2), EntityId(3)]
        );
    }

    #[test]
    fn same_commands_produce_same_result() {
        let mut world_a = EcsWorld::default();
        let mut world_b = EcsWorld::default();

        let mut commands_a = sample_commands();
        let mut commands_b = sample_commands();

        world_a.apply_commands(&mut commands_a);
        world_b.apply_commands(&mut commands_b);

        assert_eq!(world_a.entity_count(), world_b.entity_count());
        assert_eq!(world_a.ordered_entities(), world_b.ordered_entities());
    }
}
