import net.minecraft.SharedConstants;
import net.minecraft.core.component.DataComponents;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.item.Item;

/**
 * Usage: java -cp <server libs> tools/DumpItemLootProperties.java OUTPUT.json
 * Dumps the item data components loot functions read: max_stack_size, max_damage, enchantable
 * and whether the item can carry enchantments / stored enchantments. */
public final class DumpItemLootProperties {
    public static void main(String[] args) {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        net.minecraft.core.registries.BuiltInRegistries.DATA_COMPONENT_INITIALIZERS
            .build(net.minecraft.data.registries.VanillaRegistries.createLookup())
            .forEach(net.minecraft.core.component.DataComponentInitializers.PendingComponents::apply);
        StringBuilder out = new StringBuilder("{\n");
        boolean first = true;
        for (Item item : BuiltInRegistries.ITEM) {
            String id = BuiltInRegistries.ITEM.getKey(item).toString();
            Integer stack = item.components().get(DataComponents.MAX_STACK_SIZE);
            Integer damage = item.components().get(DataComponents.MAX_DAMAGE);
            var enchantable = item.components().get(DataComponents.ENCHANTABLE);
            boolean enchantments = item.components().has(DataComponents.ENCHANTMENTS);
            boolean stored = item.components().has(DataComponents.STORED_ENCHANTMENTS);
            if (!first) out.append(",\n");
            first = false;
            out.append("  \"").append(id).append("\": {\"max_stack_size\": ").append(stack == null ? 1 : stack);
            if (damage != null) out.append(", \"max_damage\": ").append(damage);
            if (enchantable != null) out.append(", \"enchantable\": ").append(enchantable.value());
            if (enchantments) out.append(", \"enchantments\": true");
            if (stored) out.append(", \"stored_enchantments\": true");
            out.append("}");
        }
        try {
            java.nio.file.Files.writeString(java.nio.file.Path.of(args[0]), out.append("\n}\n").toString());
        } catch (java.io.IOException e) {
            throw new java.io.UncheckedIOException(e);
        }
    }
}
