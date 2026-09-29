import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.mojang.serialization.JsonOps;
import it.unimi.dsi.fastutil.objects.Object2IntMap;
import java.lang.reflect.Field;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.TreeMap;
import net.minecraft.SharedConstants;
import net.minecraft.core.Holder;
import net.minecraft.core.Registry;
import net.minecraft.core.RegistryAccess;
import net.minecraft.resources.RegistryDataLoader;
import net.minecraft.server.packs.PackType;
import net.minecraft.server.packs.repository.ServerPacksSource;
import net.minecraft.server.packs.resources.MultiPackResourceManager;
import net.minecraft.server.packs.resources.ResourceManager;
import net.minecraft.tags.TagLoader;
import net.minecraft.core.HolderLookup;
import net.minecraft.core.component.DataComponentInitializers;
import net.minecraft.core.component.DataComponents;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.data.registries.VanillaRegistries;
import net.minecraft.server.Bootstrap;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.component.SuspiciousStewEffects;
import net.minecraft.world.item.enchantment.ItemEnchantments;
import net.minecraft.world.level.storage.loot.LootContext;
import net.minecraft.world.level.storage.loot.LootParams;
import net.minecraft.world.level.storage.loot.LootTable;
import net.minecraft.world.level.storage.loot.parameters.LootContextParamSets;
import net.minecraft.world.level.storage.loot.parameters.LootContextParams;
import net.minecraft.world.phys.Vec3;
import sun.misc.Unsafe;

/**
 * Runs vanilla chest loot tables through the real 26.1.2 loot code and dumps the
 * resulting stacks, to pin the Rust loot model's RNG-sequence-sensitive behaviour.
 *
 * Usage: java -cp SERVER_LIBS DumpLootRuns.java VANILLA_DATA_DIR OUTPUT.json TABLE...
 * where VANILLA_DATA_DIR contains data/minecraft/{loot_table,tags} (tag options such as
 * "#minecraft:on_random_loot" are expanded to explicit id lists before decoding, because
 * datagen registries carry no tags) and each TABLE is a loot table id such as
 * "chests/simple_dungeon" of the chest parameter set.
 */
public final class DumpLootRuns {
    private static final long[] SEEDS = {1L, 2L, 3L, 12345L, 987654321L};

    public static void main(String[] args) throws Exception {
        Path data = Path.of(args[0]);
        Path out = Path.of(args[1]);
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        HolderLookup.Provider lookup = loadRegistriesWithTags();
        BuiltInRegistries.DATA_COMPONENT_INITIALIZERS
            .build(lookup)
            .forEach(DataComponentInitializers.PendingComponents::apply);
        ServerLevel level = fakeLevel();

        JsonObject result = new JsonObject();
        for (int i = 2; i < args.length; i++) {
            String id = args[i];
            // "trade_rebalance:chests/x" names a table of the bundled trade_rebalance datapack.
            String root = id.startsWith("trade_rebalance:")
                ? "data/minecraft/datapacks/trade_rebalance/data/minecraft/loot_table/" + id.substring(16)
                : "data/minecraft/loot_table/" + id;
            JsonElement json = JsonParser.parseString(Files.readString(data.resolve(root + ".json")));
            LootTable table = LootTable.DIRECT_CODEC
                .parse(lookup.createSerializationContext(JsonOps.INSTANCE), json)
                .getOrThrow(message -> new IllegalStateException(id + ": " + message));
            JsonObject bySeed = new JsonObject();
            for (long seed : SEEDS) {
                LootParams params = new LootParams.Builder(level)
                    .withParameter(LootContextParams.ORIGIN, Vec3.ZERO)
                    .create(LootContextParamSets.CHEST);
                LootContext context = context(params, seed);
                JsonArray stacks = new JsonArray();
                List<ItemStack> raw = new ArrayList<>();
                table.getRandomItemsRaw(context, raw::add);
                for (ItemStack stack : raw) {
                    // LootTable.createStackSplitter without the feature-flag check.
                    if (stack.getCount() < stack.getMaxStackSize()) {
                        stacks.add(describe(stack));
                    } else {
                        int count = stack.getCount();
                        while (count > 0) {
                            ItemStack copy = stack.copyWithCount(Math.min(stack.getMaxStackSize(), count));
                            count -= copy.getCount();
                            stacks.add(describe(copy));
                        }
                    }
                }
                bySeed.add(Long.toString(seed), stacks);
            }
            result.add(id, bySeed);
        }
        Files.writeString(out, new GsonBuilder().setPrettyPrinting().disableHtmlEscaping().create().toJson(result) + "\n");
    }

    private static Unsafe unsafe() throws Exception {
        Field theUnsafe = Unsafe.class.getDeclaredField("theUnsafe");
        theUnsafe.setAccessible(true);
        return (Unsafe) theUnsafe.get(null);
    }

    private static void set(Unsafe unsafe, Object target, String field, Object value) throws Exception {
        Field f = target.getClass().getDeclaredField(field);
        unsafe.putObject(target, unsafe.objectFieldOffset(f), value);
    }

    /**
     * {@code LootContext.Builder.create} needs a server for the reference resolver; the
     * chosen tables never resolve references, so the context is assembled directly with
     * {@code RandomSource.create(seed)} (what {@code withOptionalRandomSeed} installs).
     */
    private static LootContext context(LootParams params, long seed) throws Exception {
        Unsafe unsafe = unsafe();
        LootContext context = (LootContext) unsafe.allocateInstance(LootContext.class);
        set(unsafe, context, "params", params);
        set(unsafe, context, "random", net.minecraft.util.RandomSource.create(seed));
        set(unsafe, context, "visitedElements", new java.util.LinkedHashSet<>());
        return context;
    }

    /** A {@code ServerLevel} that only knows what loot evaluation asks it for. */
    private static ServerLevel fakeLevel() throws Exception {
        Field theUnsafe = Unsafe.class.getDeclaredField("theUnsafe");
        theUnsafe.setAccessible(true);
        Unsafe unsafe = (Unsafe) theUnsafe.get(null);
        ServerLevel level = (ServerLevel) unsafe.allocateInstance(ServerLevel.class);
        return level;
    }

    /**
     * The server's own load path: static registries plus the data-driven registries
     * decoded from the bundled vanilla data pack, with every registry's tags bound.
     */
    private static HolderLookup.Provider loadRegistriesWithTags() throws Exception {
        ResourceManager resources = new MultiPackResourceManager(
            PackType.SERVER_DATA, List.of(ServerPacksSource.createVanillaPackSource()));
        RegistryAccess.Frozen staticAccess = RegistryAccess.fromRegistryOfRegistries(BuiltInRegistries.REGISTRY);
        List<Registry.PendingTags<?>> staticTags = TagLoader.loadTagsForExistingRegistries(resources, staticAccess);
        staticTags.forEach(Registry.PendingTags::apply);
        List<HolderLookup.RegistryLookup<?>> context = TagLoader.buildUpdatedLookups(staticAccess, staticTags);
        RegistryAccess.Frozen dynamic = RegistryDataLoader
            .load(resources, context, RegistryDataLoader.WORLDGEN_REGISTRIES, Runnable::run)
            .join();
        List<Registry.PendingTags<?>> dynamicTags = TagLoader.loadTagsForExistingRegistries(resources, dynamic);
        dynamicTags.forEach(Registry.PendingTags::apply);
        List<HolderLookup.RegistryLookup<?>> all = new ArrayList<>(context);
        all.addAll(TagLoader.buildUpdatedLookups(dynamic, dynamicTags));
        return HolderLookup.Provider.create(all.stream());
    }

    /** Replaces {@code "options": "#tag"} by the tag's resolved id list. */
    private static void expandTags(JsonElement element, Path data) throws Exception {
        if (element.isJsonObject()) {
            JsonObject object = element.getAsJsonObject();
            for (Map.Entry<String, JsonElement> entry : new ArrayList<>(object.entrySet())) {
                if (entry.getKey().equals("options")
                    && entry.getValue().isJsonPrimitive()
                    && entry.getValue().getAsString().startsWith("#")) {
                    String function = object.get("function").getAsString();
                    String registry = function.equals("minecraft:set_instrument") ? "instrument" : "enchantment";
                    List<String> ids = new ArrayList<>();
                    resolveTag(registry, entry.getValue().getAsString().substring(1), data, ids);
                    JsonArray array = new JsonArray();
                    ids.forEach(array::add);
                    object.add("options", array);
                } else {
                    expandTags(entry.getValue(), data);
                }
            }
        } else if (element.isJsonArray()) {
            for (JsonElement child : element.getAsJsonArray()) {
                expandTags(child, data);
            }
        }
    }

    private static void resolveTag(String registry, String tag, Path data, List<String> out) throws Exception {
        String path = tag.substring(tag.indexOf(':') + 1);
        JsonObject json = JsonParser.parseString(
            Files.readString(data.resolve("data/minecraft/tags/" + registry + "/" + path + ".json")))
            .getAsJsonObject();
        for (JsonElement value : json.getAsJsonArray("values")) {
            String id = value.isJsonObject() ? value.getAsJsonObject().get("id").getAsString() : value.getAsString();
            if (id.startsWith("#")) {
                resolveTag(registry, id.substring(1), data, out);
            } else {
                out.add(id);
            }
        }
    }

    private static String enchantments(ItemEnchantments enchantments) {
        TreeMap<String, Integer> sorted = new TreeMap<>();
        for (Object2IntMap.Entry<Holder<net.minecraft.world.item.enchantment.Enchantment>> entry
            : enchantments.entrySet()) {
            sorted.put(entry.getKey().unwrapKey().orElseThrow().identifier().toString(), entry.getIntValue());
        }
        List<String> parts = new ArrayList<>();
        sorted.forEach((id, level) -> parts.add(id + ":" + level));
        return String.join(",", parts);
    }

    private static String describe(ItemStack stack) {
        StringBuilder text = new StringBuilder();
        text.append(BuiltInRegistries.ITEM.getKey(stack.getItem())).append(" x").append(stack.getCount());
        ItemEnchantments held = stack.get(DataComponents.ENCHANTMENTS);
        if (held != null && !held.isEmpty()) {
            text.append("|enchantments=").append(enchantments(held));
        }
        ItemEnchantments stored = stack.get(DataComponents.STORED_ENCHANTMENTS);
        if (stored != null && !stored.isEmpty()) {
            text.append("|stored_enchantments=").append(enchantments(stored));
        }
        if (stack.isDamageableItem()) {
            text.append("|damage=").append(stack.getDamageValue());
        }
        var amplifier = stack.get(DataComponents.OMINOUS_BOTTLE_AMPLIFIER);
        if (amplifier != null) {
            text.append("|ominous_bottle_amplifier=").append(amplifier.value());
        }
        SuspiciousStewEffects stew = stack.get(DataComponents.SUSPICIOUS_STEW_EFFECTS);
        if (stew != null && !stew.effects().isEmpty()) {
            List<String> parts = new ArrayList<>();
            for (SuspiciousStewEffects.Entry entry : stew.effects()) {
                parts.add(entry.effect().unwrapKey().orElseThrow().identifier() + ":" + entry.duration());
            }
            text.append("|suspicious_stew_effects=").append(String.join(",", parts));
        }
        var instrument = stack.get(DataComponents.INSTRUMENT);
        if (instrument != null) {
            text.append("|instrument=").append(instrument.instrument().unwrapKey().orElseThrow().identifier());
        }
        Integer cost = stack.get(DataComponents.ADDITIONAL_TRADE_COST);
        if (cost != null) {
            text.append("|additional_trade_cost=").append(cost);
        }
        return text.toString();
    }
}
