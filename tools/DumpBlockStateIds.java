import java.util.List;

import net.minecraft.SharedConstants;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.resources.Identifier;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockState;

public final class DumpBlockStateIds {
    public static void main(String[] args) {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        List<String> names = args.length == 0
                ? List.of(
                        "minecraft:air",
                        "minecraft:stone",
                        "minecraft:granite",
                        "minecraft:diorite",
                        "minecraft:andesite",
                        "minecraft:grass_block",
                        "minecraft:dirt",
                        "minecraft:bedrock",
                        "minecraft:water",
                        "minecraft:sand",
                        "minecraft:red_sand",
                        "minecraft:sandstone",
                        "minecraft:red_sandstone",
                        "minecraft:white_terracotta",
                        "minecraft:orange_terracotta",
                        "minecraft:terracotta",
                        "minecraft:yellow_terracotta",
                        "minecraft:brown_terracotta",
                        "minecraft:red_terracotta",
                        "minecraft:light_gray_terracotta",
                        "minecraft:oak_log",
                        "minecraft:oak_leaves",
                        "minecraft:birch_log",
                        "minecraft:birch_leaves",
                        "minecraft:deepslate",
                        "minecraft:short_grass",
                        "minecraft:dandelion",
                        "minecraft:poppy",
                        "minecraft:sunflower")
                : List.of(args);
        for (String name : names) {
            Block block = BuiltInRegistries.BLOCK.getValue(Identifier.parse(name));
            BlockState state = block.defaultBlockState();
            System.out.println(name + "=" + Block.getId(state) + " state=" + state);
        }
        System.out.println("block_state_registry_size=" + Block.BLOCK_STATE_REGISTRY.size());
    }
}
