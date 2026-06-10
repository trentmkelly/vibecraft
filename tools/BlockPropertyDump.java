// Dumps every vanilla block state's physical properties straight from the official
// 26.1.2 server registries, as ground truth for VibeCraft's generated block tables.
//
// Run with the bundled server jar's nested jars on the classpath (single-file source
// launcher; javac is not required):
//
//   cd datagen
//   java -cp "bundle/META-INF/versions/26.1.2/server-26.1.2.jar:$(find bundle/META-INF/libraries -name '*.jar' | tr '\n' ':')" \
//        BlockPropertyDump.java > block_properties_26_1_2.json
//
// The output is consumed by VibeCraft/tools/generate_block_properties.py.

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.lang.reflect.Field;
import java.lang.reflect.Modifier;
import java.util.LinkedHashMap;
import java.util.Map;
import net.minecraft.SharedConstants;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.level.EmptyBlockGetter;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.SoundType;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.material.MapColor;
import net.minecraft.world.level.pathfinder.PathComputationType;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.shapes.Shapes;
import net.minecraft.world.phys.shapes.VoxelShape;

public class BlockPropertyDump {
   private static final Map<String, Integer> SHAPE_INDICES = new LinkedHashMap<>();
   private static final JsonArray SHAPES = new JsonArray();

   /// Representative VoxelShape instance per interned occlusion-shape index, used
   /// to compute the exact light-occlusion matrices with the real Shapes functions.
   private static final Map<Integer, VoxelShape> OCCLUSION_SHAPE_INSTANCES = new LinkedHashMap<>();

   public static void main(String[] args) throws java.io.IOException {
      java.nio.file.Path outputDir = java.nio.file.Path.of(args.length > 0 ? args[0] : ".");
      SharedConstants.tryDetectVersion();
      Bootstrap.bootStrap();

      JsonObject root = new JsonObject();
      root.addProperty("format", "vibecraft-block-properties-v1");
      root.add("map_colors", mapColorNames());
      root.add("sound_types", soundTypeTable());

      JsonObject blocks = new JsonObject();
      for (Block block : BuiltInRegistries.BLOCK) {
         JsonObject blockJson = new JsonObject();
         blockJson.addProperty("explosion_resistance", block.getExplosionResistance());
         blockJson.addProperty("friction", block.getFriction());
         blockJson.addProperty("speed_factor", block.getSpeedFactor());
         blockJson.addProperty("jump_factor", block.getJumpFactor());
         JsonArray states = new JsonArray();
         for (BlockState state : block.getStateDefinition().getPossibleStates()) {
            states.add(dumpState(state));
         }
         blockJson.add("states", states);
         blocks.add(BuiltInRegistries.BLOCK.getKey(block).toString(), blockJson);
      }
      root.add("blocks", blocks);
      root.add("shapes", SHAPES);

      Gson gson = new GsonBuilder().create();
      java.nio.file.Files.writeString(
         outputDir.resolve("block_properties_26_1_2.json"), gson.toJson(root)
      );
      java.nio.file.Files.writeString(
         outputDir.resolve("light_occlusion_26_1_2.json"), gson.toJson(lightOcclusionTables())
      );
   }

   /// Exact light-occlusion relationship tables, evaluated by the real Java shape
   /// code so VibeCraft never has to reimplement VoxelShape boolean joins:
   ///
   /// - LightEngine.getLightBlockInto -> Shapes.mergedFaceOccludes(from, to, dir)
   ///   over full occlusion shapes. Direction folds onto its axis: for the negative
   ///   direction of an axis, mergedFaceOccludes(a, b, neg) == matrix_axis[b][a].
   /// - LightEngine.shapeOccludes / ChunkSkyLightSources.isEdgeOccluded ->
   ///   Shapes.faceShapeOccludes(faceShape(from, dir), faceShape(to, dir.opposite))
   ///   over per-direction face shapes (BlockStateBase.getFaceOcclusionShape).
   ///
   /// All indices into `occlusion_shapes` reference the block-properties report's
   /// global `shapes` interning; `face_shapes` are interned locally.
   private static JsonObject lightOcclusionTables() {
      JsonObject root = new JsonObject();
      root.addProperty("format", "vibecraft-light-occlusion-v1");

      java.util.List<Integer> occlusionIndices = new java.util.ArrayList<>(
         OCCLUSION_SHAPE_INSTANCES.keySet()
      );
      JsonArray occlusionShapeIndices = new JsonArray();
      occlusionIndices.forEach(occlusionShapeIndices::add);
      root.add("occlusion_shapes", occlusionShapeIndices);

      // Face shapes per occlusion shape, in Java Direction ordinal order
      // (DOWN, UP, NORTH, SOUTH, WEST, EAST). Interned locally.
      Map<String, Integer> faceShapeIndices = new LinkedHashMap<>();
      java.util.List<VoxelShape> faceShapes = new java.util.ArrayList<>();
      JsonArray faceShapeBoxes = new JsonArray();
      JsonArray facesPerOcclusionShape = new JsonArray();
      for (Integer occlusionIndex : occlusionIndices) {
         VoxelShape shape = OCCLUSION_SHAPE_INSTANCES.get(occlusionIndex);
         JsonArray faces = new JsonArray();
         for (Direction direction : Direction.values()) {
            VoxelShape faceShape = shape.getFaceShape(direction);
            String key = shapeKey(faceShape);
            Integer index = faceShapeIndices.get(key);
            if (index == null) {
               index = faceShapes.size();
               faceShapeIndices.put(key, index);
               faceShapes.add(faceShape);
               faceShapeBoxes.add(shapeBoxes(faceShape));
            }
            faces.add(index);
         }
         facesPerOcclusionShape.add(faces);
      }
      root.add("face_shapes", faceShapeBoxes);
      root.add("faces_per_occlusion_shape", facesPerOcclusionShape);

      // faceShapeOccludes over every face-shape pair.
      JsonArray faceOccludes = new JsonArray();
      for (VoxelShape first : faceShapes) {
         StringBuilder row = new StringBuilder(faceShapes.size());
         for (VoxelShape second : faceShapes) {
            row.append(Shapes.faceShapeOccludes(first, second) ? '1' : '0');
         }
         faceOccludes.add(row.toString());
      }
      root.add("face_shape_occludes", faceOccludes);

      // mergedFaceOccludes over every occlusion-shape pair, for the positive
      // direction of each axis (negative direction = transposed lookup).
      JsonObject merged = new JsonObject();
      for (Direction direction : new Direction[]{Direction.EAST, Direction.UP, Direction.SOUTH}) {
         JsonArray matrix = new JsonArray();
         for (Integer fromIndex : occlusionIndices) {
            VoxelShape from = OCCLUSION_SHAPE_INSTANCES.get(fromIndex);
            StringBuilder row = new StringBuilder(occlusionIndices.size());
            for (Integer toIndex : occlusionIndices) {
               VoxelShape to = OCCLUSION_SHAPE_INSTANCES.get(toIndex);
               row.append(Shapes.mergedFaceOccludes(from, to, direction) ? '1' : '0');
            }
            matrix.add(row.toString());
         }
         merged.add(direction.getAxis().getName(), matrix);
      }
      root.add("merged_face_occludes", merged);
      return root;
   }

   private static String shapeKey(VoxelShape shape) {
      StringBuilder key = new StringBuilder();
      for (AABB box : shape.toAabbs()) {
         for (double value : new double[]{box.minX, box.minY, box.minZ, box.maxX, box.maxY, box.maxZ}) {
            key.append(value).append(',');
         }
      }
      return key.toString();
   }

   private static JsonArray shapeBoxes(VoxelShape shape) {
      JsonArray boxes = new JsonArray();
      for (AABB box : shape.toAabbs()) {
         JsonArray coordinates = new JsonArray();
         for (double value : new double[]{box.minX, box.minY, box.minZ, box.maxX, box.maxY, box.maxZ}) {
            coordinates.add(value);
         }
         boxes.add(coordinates);
      }
      return boxes;
   }

   private static JsonObject dumpState(BlockState state) {
      JsonObject json = new JsonObject();
      json.addProperty("id", Block.getId(state));
      json.addProperty("destroy_speed", state.getDestroySpeed(null, null));
      json.addProperty("map_color", state.getMapColor(null, null).id);
      json.addProperty("sound_type", soundTypeName(state.getSoundType()));
      json.addProperty("light_emission", state.getLightEmission());
      json.addProperty("light_dampening", state.getLightDampening());
      json.addProperty("is_air", state.isAir());
      json.addProperty("liquid", state.liquid());
      json.addProperty("blocks_motion", state.blocksMotion());
      json.addProperty("is_solid", state.isSolid());
      json.addProperty("propagates_skylight_down", state.propagatesSkylightDown());
      json.addProperty("can_occlude", state.canOcclude());
      json.addProperty("is_solid_render", state.isSolidRender());
      json.addProperty("use_shape_for_light_occlusion", state.useShapeForLightOcclusion());
      json.addProperty("large_collision_shape", state.hasLargeCollisionShape());
      json.addProperty("requires_correct_tool_for_drops", state.requiresCorrectToolForDrops());
      json.addProperty("ignited_by_lava", state.ignitedByLava());
      json.addProperty("push_reaction", state.getPistonPushReaction().name());
      json.addProperty("is_signal_source", state.isSignalSource());
      json.addProperty("is_randomly_ticking", state.isRandomlyTicking());
      json.addProperty("has_block_entity", state.hasBlockEntity());
      json.addProperty("replaceable", state.canBeReplaced());
      json.addProperty("instrument", state.instrument().name());
      json.addProperty("render_shape", state.getRenderShape().name());
      json.addProperty("pathfind_land", state.isPathfindable(PathComputationType.LAND));
      json.addProperty("pathfind_air", state.isPathfindable(PathComputationType.AIR));
      json.addProperty("pathfind_water", state.isPathfindable(PathComputationType.WATER));

      JsonObject fluid = new JsonObject();
      fluid.addProperty(
         "type", BuiltInRegistries.FLUID.getKey(state.getFluidState().getType()).toString()
      );
      fluid.addProperty("amount", state.getFluidState().getAmount());
      fluid.addProperty("source", state.getFluidState().isSource());
      json.add("fluid", fluid);

      json.addProperty("shape", shapeIndex(() -> state.getShape(EmptyBlockGetter.INSTANCE, BlockPos.ZERO)));
      json.addProperty(
         "collision_shape", shapeIndex(() -> state.getCollisionShape(EmptyBlockGetter.INSTANCE, BlockPos.ZERO))
      );
      Integer occlusionShapeIndex = shapeIndex(state::getOcclusionShape);
      if (occlusionShapeIndex != null) {
         OCCLUSION_SHAPE_INSTANCES.putIfAbsent(occlusionShapeIndex, state.getOcclusionShape());
      }
      json.addProperty("occlusion_shape", occlusionShapeIndex);
      json.addProperty(
         "interaction_shape", shapeIndex(() -> state.getInteractionShape(EmptyBlockGetter.INSTANCE, BlockPos.ZERO))
      );
      json.addProperty(
         "support_shape", shapeIndex(() -> state.getBlockSupportShape(EmptyBlockGetter.INSTANCE, BlockPos.ZERO))
      );
      json.addProperty("suffocating", suppress(() -> state.isSuffocating(EmptyBlockGetter.INSTANCE, BlockPos.ZERO)));
      json.addProperty(
         "view_blocking", suppress(() -> state.isViewBlocking(EmptyBlockGetter.INSTANCE, BlockPos.ZERO))
      );
      return json;
   }

   private interface ShapeSupplier {
      VoxelShape get();
   }

   private interface BoolSupplier {
      boolean get();
   }

   private static Integer shapeIndex(ShapeSupplier supplier) {
      VoxelShape shape;
      try {
         shape = supplier.get();
      } catch (RuntimeException exception) {
         return null;
      }
      StringBuilder key = new StringBuilder();
      JsonArray boxes = new JsonArray();
      for (AABB box : shape.toAabbs()) {
         JsonArray coordinates = new JsonArray();
         for (double value : new double[]{box.minX, box.minY, box.minZ, box.maxX, box.maxY, box.maxZ}) {
            coordinates.add(value);
            key.append(value).append(',');
         }
         boxes.add(coordinates);
      }
      Integer existing = SHAPE_INDICES.get(key.toString());
      if (existing != null) {
         return existing;
      }
      int index = SHAPES.size();
      SHAPES.add(boxes);
      SHAPE_INDICES.put(key.toString(), index);
      return index;
   }

   private static Boolean suppress(BoolSupplier supplier) {
      try {
         return supplier.get();
      } catch (RuntimeException exception) {
         return null;
      }
   }

   private static JsonObject mapColorNames() {
      JsonObject names = new JsonObject();
      for (Field field : MapColor.class.getDeclaredFields()) {
         if (Modifier.isStatic(field.getModifiers()) && field.getType() == MapColor.class) {
            try {
               MapColor color = (MapColor) field.get(null);
               names.addProperty(Integer.toString(color.id), field.getName());
            } catch (IllegalAccessException ignored) {
            }
         }
      }
      return names;
   }

   private static String soundTypeName(SoundType soundType) {
      for (Field field : SoundType.class.getDeclaredFields()) {
         if (Modifier.isStatic(field.getModifiers()) && field.getType() == SoundType.class) {
            try {
               if (field.get(null) == soundType) {
                  return field.getName();
               }
            } catch (IllegalAccessException ignored) {
            }
         }
      }
      return "CUSTOM:" + soundType.getBreakSound().location();
   }

   private static JsonObject soundTypeTable() {
      JsonObject table = new JsonObject();
      for (Field field : SoundType.class.getDeclaredFields()) {
         if (Modifier.isStatic(field.getModifiers()) && field.getType() == SoundType.class) {
            try {
               SoundType soundType = (SoundType) field.get(null);
               JsonObject entry = new JsonObject();
               entry.addProperty("volume", soundType.getVolume());
               entry.addProperty("pitch", soundType.getPitch());
               entry.addProperty("break", soundType.getBreakSound().location().toString());
               entry.addProperty("step", soundType.getStepSound().location().toString());
               entry.addProperty("place", soundType.getPlaceSound().location().toString());
               entry.addProperty("hit", soundType.getHitSound().location().toString());
               entry.addProperty("fall", soundType.getFallSound().location().toString());
               table.add(field.getName(), entry);
            } catch (IllegalAccessException ignored) {
            }
         }
      }
      return table;
   }
}
