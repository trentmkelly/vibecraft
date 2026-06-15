#![allow(dead_code)]

#[cfg(test)]
mod tests {
    #[cfg(vibecraft_has_decompiled_sources)]
    const RIDE_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/RideCommand.java");

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn ride_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"ride\")",
            "Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)",
            "Commands.argument(\"target\", EntityArgument.entity())",
            "Commands.literal(\"mount\")",
            "Commands.argument(\"vehicle\", EntityArgument.entity())",
            "Commands.literal(\"dismount\")",
            "target.getVehicle()",
            "ERROR_ALREADY_RIDING.create(target.getDisplayName(), currentVehicle.getDisplayName())",
            "vehicle.is(EntityType.PLAYER)",
            "ERROR_MOUNTING_PLAYER.create()",
            "target.getSelfAndPassengers().anyMatch(e -> e == vehicle)",
            "ERROR_MOUNTING_LOOP.create()",
            "target.level() != vehicle.level()",
            "ERROR_WRONG_DIMENSION.create()",
            "target.startRiding(vehicle, true, true)",
            "ERROR_MOUNT_FAILED.create(target.getDisplayName(), vehicle.getDisplayName())",
            "commands.ride.mount.success",
            "target.stopRiding()",
            "commands.ride.dismount.success",
            "return 1;",
        ] {
            assert!(
                RIDE_COMMAND_JAVA.contains(sentinel),
                "RideCommand.java is missing sentinel: {sentinel}"
            );
        }
    }
}
