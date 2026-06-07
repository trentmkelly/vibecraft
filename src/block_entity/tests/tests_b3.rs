use super::super::container_decorative::{ContainerOpenersCounterModel, ContainerUserOpenState};
use super::super::ender_chest::{EnderChestBlockEntityModel, EnderChestBlockEvent};
use super::*;

#[test]
fn ender_chest_lid_openers_and_menu_match_java() {
    let mut ender_chest = EnderChestBlockEntityModel::new(BlockPos { x: -2, y: 65, z: 9 });
    assert_ender_chest_constants_and_validity(&mut ender_chest);
    assert_ender_chest_openers_and_recheck(&mut ender_chest);
    assert_ender_chest_lid_event_and_menu(&mut ender_chest);
}

fn assert_ender_chest_constants_and_validity(ender_chest: &mut EnderChestBlockEntityModel) {
    assert_eq!(EnderChestBlockEntityModel::MENU_TYPE, "generic_9x3");
    assert_eq!(
        EnderChestBlockEntityModel::CONTAINER_TITLE,
        "container.enderchest"
    );
    assert_eq!(
        EnderChestBlockEntityModel::OPEN_STAT,
        "minecraft:open_enderchest"
    );
    assert!(ender_chest.still_valid(true, 64.0));
    assert!(!ender_chest.still_valid(true, 64.01));
    assert!(!ender_chest.still_valid(false, 1.0));
    assert_eq!(ender_chest.start_open(true, 4.5), None);
    ender_chest.remove = true;
    assert_eq!(ender_chest.start_open(false, 4.5), None);
    ender_chest.remove = false;
}

fn assert_ender_chest_openers_and_recheck(ender_chest: &mut EnderChestBlockEntityModel) {
    let first_open = ender_chest.start_open(false, 5.0).unwrap();
    assert_eq!(
        first_open.block_event,
        EnderChestBlockEvent {
            pos: ender_chest.world_position,
            block: EnderChestBlockEntityModel::BLOCK_ID,
            action: EnderChestBlockEntityModel::BLOCK_EVENT_ACTION,
            param: 1,
        }
    );
    assert_eq!(
        first_open.sound_event,
        Some(EnderChestBlockEntityModel::OPEN_SOUND)
    );
    assert_eq!(first_open.counter_effect.opener_count_changed, (0, 1));
    assert_eq!(
        first_open.counter_effect.schedule_recheck_delay,
        Some(ContainerOpenersCounterModel::CHECK_TICK_DELAY)
    );

    let second_open = ender_chest.start_open(false, 3.0).unwrap();
    assert_eq!(second_open.sound_event, None);
    assert_eq!(second_open.block_event.param, 2);
    assert_eq!(ender_chest.stop_open(false).unwrap().block_event.param, 1);
    let close = ender_chest.stop_open(false).unwrap();
    assert_eq!(
        close.sound_event,
        Some(EnderChestBlockEntityModel::CLOSE_SOUND)
    );
    assert_eq!(close.block_event.param, 0);
    assert_eq!(close.counter_effect.opener_count_changed, (1, 0));

    let users = [ContainerUserOpenState {
        has_container_open: true,
        spectator: false,
        interaction_range: 6.0,
    }];
    let recheck = ender_chest.recheck_open(&users).unwrap();
    assert_eq!(
        recheck.sound_event,
        Some(EnderChestBlockEntityModel::OPEN_SOUND)
    );
    assert_eq!(recheck.block_event.param, 1);
    assert_eq!(
        recheck.counter_effect.schedule_recheck_delay,
        Some(ContainerOpenersCounterModel::CHECK_TICK_DELAY)
    );
}

fn assert_ender_chest_lid_event_and_menu(ender_chest: &mut EnderChestBlockEntityModel) {
    assert!(!ender_chest.trigger_event(2, 1));
    assert!(ender_chest.trigger_event(EnderChestBlockEntityModel::BLOCK_EVENT_ACTION, 1));
    ender_chest.lid_animate_tick();
    assert!((ender_chest.get_openness(1.0) - 0.1).abs() < 0.001);
    assert!(ender_chest.trigger_event(EnderChestBlockEntityModel::BLOCK_EVENT_ACTION, 0));
    ender_chest.lid_animate_tick();
    assert!((ender_chest.get_openness(1.0) - 0.0).abs() < 0.001);

    let slots = vec![Some(stack("minecraft:ender_pearl", 16)); 27];
    assert_eq!(ender_chest.open_menu(9, slots.clone(), true), None);
    let menu = ender_chest.open_menu(9, slots.clone(), false).unwrap();
    assert_eq!(menu.container_id, 9);
    assert_eq!(menu.menu_type, EnderChestBlockEntityModel::MENU_TYPE);
    assert_eq!(menu.initial_slots, slots);
}
