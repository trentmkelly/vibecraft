use std::cell::RefCell;
use std::rc::Rc;

use super::gizmo_models::*;

const GIZMO_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/gizmos/Gizmo.java");
const GIZMO_COLLECTOR_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/gizmos/GizmoCollector.java");
const GIZMO_PRIMITIVES_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/gizmos/GizmoPrimitives.java");
const GIZMO_STYLE_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/gizmos/GizmoStyle.java");
const GIZMOS_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/gizmos/Gizmos.java");
const SIMPLE_COLLECTOR_JAVA: &str = vibecraft_java_source!("/net/minecraft/gizmos/SimpleGizmoCollector.java");
const PACKAGE_INFO_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/gizmos/package-info.java");

#[test]
fn gizmo_java_source_shapes_are_tracked() {
    for sentinel in [
        "public interface Gizmo",
        "void emit(GizmoPrimitives primitives, float alphaMultiplier);",
    ] {
        assert!(
            GIZMO_JAVA.contains(sentinel),
            "missing Gizmo sentinel {sentinel}"
        );
    }
    for sentinel in [
        "GizmoCollector NOOP = gizmo -> IGNORED;",
        "GizmoProperties add(final Gizmo gizmo);",
    ] {
        assert!(
            GIZMO_COLLECTOR_JAVA.contains(sentinel),
            "missing collector sentinel {sentinel}"
        );
    }
    for sentinel in [
        "void addPoint(Vec3 pos, int color, float size);",
        "void addLine(Vec3 start, Vec3 end, int color, float width);",
        "void addTriangleFan(Vec3[] points, int color);",
        "void addQuad(Vec3 a, Vec3 b, Vec3 c, Vec3 d, int color);",
        "void addText(Vec3 pos, String text, TextGizmo.Style style);",
    ] {
        assert!(
            GIZMO_PRIMITIVES_JAVA.contains(sentinel),
            "missing primitive sentinel {sentinel}"
        );
    }
    assert!(PACKAGE_INFO_JAVA.contains("@NullMarked"));
}

#[test]
fn gizmo_style_factories_and_alpha_match_argb_java() {
    let stroke = GizmoStyleModel::stroke(0x4012_3456);
    assert_eq!(stroke.stroke_width, 2.5);
    assert!(stroke.has_stroke());
    assert!(!stroke.has_fill());
    assert_eq!(stroke.multiplied_stroke(0.5), 0x2012_3456);

    let fill = GizmoStyleModel::fill(0x7F00_FF00);
    assert!(fill.has_fill());
    assert!(!fill.has_stroke());
    assert_eq!(fill.multiplied_fill(0.25), 0x1F00_FF00);

    assert!(GIZMO_STYLE_JAVA.contains("private static final float DEFAULT_WIDTH = 2.5F;"));
    assert!(GIZMO_STYLE_JAVA.contains("ARGB.multiplyAlpha(this.stroke, alphaMultiplier)"));
}

#[test]
fn rect_from_cuboid_face_uses_java_corner_order() {
    let a = Vec3Model::new(1.0, 2.0, 3.0);
    let b = Vec3Model::new(4.0, 6.0, 8.0);
    let style = GizmoStyleModel::stroke(-1);
    let GizmoModel::Rect {
        a: ra, b: rb, c, d, ..
    } = rect_from_cuboid_face(a, b, DirectionModel::East, style)
    else {
        panic!("expected rect");
    };
    assert_eq!(ra, Vec3Model::new(4.0, 2.0, 3.0));
    assert_eq!(rb, Vec3Model::new(4.0, 6.0, 3.0));
    assert_eq!(c, Vec3Model::new(4.0, 6.0, 8.0));
    assert_eq!(d, Vec3Model::new(4.0, 2.0, 8.0));
}

#[test]
fn point_line_rect_circle_cuboid_and_text_emit_java_primitives() {
    let mut primitives = Vec::new();
    GizmoModel::Point {
        pos: Vec3Model::new(1.0, 2.0, 3.0),
        color: 0x4000_FF00,
        size: 4.0,
    }
    .emit(&mut primitives, 0.5);
    assert_eq!(
        primitives[0],
        GizmoPrimitiveModel::Point {
            pos: Vec3Model::new(1.0, 2.0, 3.0),
            color: 0x2000_FF00,
            size: 4.0
        }
    );

    primitives.clear();
    let style = GizmoStyleModel::stroke_and_fill(0x7F12_3456, 2.0, 0x4077_8899);
    GizmoModel::Rect {
        a: Vec3Model::new(0.0, 0.0, 0.0),
        b: Vec3Model::new(1.0, 0.0, 0.0),
        c: Vec3Model::new(1.0, 1.0, 0.0),
        d: Vec3Model::new(0.0, 1.0, 0.0),
        style,
    }
    .emit(&mut primitives, 0.5);
    assert!(matches!(
        primitives[0],
        GizmoPrimitiveModel::Quad {
            color: 0x2077_8899,
            ..
        }
    ));
    assert_eq!(primitives.len(), 5);

    primitives.clear();
    GizmoModel::Circle {
        pos: Vec3Model::new(0.0, 0.0, 0.0),
        radius: 2.0,
        style,
    }
    .emit(&mut primitives, 1.0);
    assert!(matches!(
        &primitives[0],
        GizmoPrimitiveModel::TriangleFan { points, .. } if points.len() == 21 && points[20] == points[0]
    ));
    assert_eq!(primitives.len(), 21);

    primitives.clear();
    GizmoModel::Cuboid {
        aabb: AabbModel::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        style,
        colored_corner_stroke: true,
    }
    .emit(&mut primitives, 1.0);
    assert_eq!(primitives.len(), 18);

    primitives.clear();
    GizmoModel::Text {
        pos: Vec3Model::new(0.5, 1.0, 0.5),
        text: "label".to_string(),
        style: TextStyleModel::for_color(-1).with_left_alignment(0.5),
    }
    .emit(&mut primitives, 0.5);
    assert!(matches!(
        &primitives[0],
        GizmoPrimitiveModel::Text { style, .. } if style.color == 0x7FFF_FFFF
    ));
}

#[test]
fn arrow_emits_shaft_and_four_tips_with_java_default_width() {
    let mut primitives = Vec::new();
    GizmoModel::Arrow {
        start: Vec3Model::new(0.0, 0.0, 0.0),
        end: Vec3Model::new(10.0, 0.0, 0.0),
        color: -1,
        width: GizmoModel::ARROW_DEFAULT_WIDTH,
    }
    .emit(&mut primitives, 1.0);
    assert_eq!(primitives.len(), 5);
    assert_eq!(
        primitives[1],
        GizmoPrimitiveModel::Line {
            start: Vec3Model::new(9.0, 1.0, 0.0),
            end: Vec3Model::new(10.0, 0.0, 0.0),
            color: -1,
            width: 2.5
        }
    );
}

#[test]
fn simple_collector_properties_drain_and_fade_like_java() {
    assert!(SIMPLE_COLLECTOR_JAVA
        .contains("this.gizmos.removeIf(gizmo -> gizmo.getExpireTimeMillis() < currentMillis);"));
    let mut collector = SimpleGizmoCollectorModel::default();
    let index = collector.add(GizmoModel::Line {
        start: Vec3Model::new(0.0, 0.0, 0.0),
        end: Vec3Model::new(1.0, 0.0, 0.0),
        color: -1,
        width: 3.0,
    });
    collector.gizmos[index].is_always_on_top = true;
    collector.gizmos[index].start_time_millis = 100;
    collector.gizmos[index].expire_time_millis = 300;
    collector.gizmos[index].should_fade_out = true;
    assert_eq!(collector.gizmos[index].alpha_multiplier(200), 0.5);

    let drained = collector.drain_gizmos(301);
    assert_eq!(drained.len(), 1);
    assert!(collector.gizmos.is_empty());
}

#[test]
fn collector_temporary_and_ignored_properties_match_java_noop_shape() {
    let ignored = GizmoPropertiesModel::Ignored;
    ignored
        .set_always_on_top()
        .persist_for_millis(10, 20)
        .fade_out();

    let mut collector = SimpleGizmoCollectorModel::default();
    collector.add_temporary_gizmos([GizmoInstanceModel::new(GizmoModel::Point {
        pos: Vec3Model::new(1.0, 2.0, 3.0),
        color: -1,
        size: 1.0,
    })]);
    let drained = collector.drain_gizmos(0);
    assert_eq!(drained.len(), 1);
    assert!(collector.temporary_gizmos.is_empty());
}

#[test]
fn gizmos_global_scope_restores_previous_collector() {
    assert!(GIZMOS_JAVA
        .contains("private static final ThreadLocal<@Nullable GizmoCollector> collector"));
    assert!(GizmosModel::add_gizmo(GizmoModel::Point {
        pos: Vec3Model::new(0.0, 0.0, 0.0),
        color: -1,
        size: 1.0,
    })
    .is_err());

    let outer = Rc::new(RefCell::new(SimpleGizmoCollectorModel::default()));
    let inner = Rc::new(RefCell::new(SimpleGizmoCollectorModel::default()));
    let mut outer_scope = GizmosModel::with_collector(outer.clone());
    GizmosModel::line(
        Vec3Model::new(0.0, 0.0, 0.0),
        Vec3Model::new(1.0, 0.0, 0.0),
        -1,
    )
    .unwrap();
    {
        let _inner_scope = GizmosModel::with_collector(inner.clone());
        GizmosModel::point(Vec3Model::new(2.0, 0.0, 0.0), -1, 1.0).unwrap();
    }
    GizmosModel::arrow(
        Vec3Model::new(0.0, 0.0, 0.0),
        Vec3Model::new(3.0, 0.0, 0.0),
        -1,
    )
    .unwrap();
    outer_scope.close();

    assert_eq!(outer.borrow().gizmos.len(), 2);
    assert_eq!(inner.borrow().gizmos.len(), 1);
}

#[test]
fn gizmos_helper_constructors_match_java_defaults_and_property_fluency() {
    let collector = Rc::new(RefCell::new(SimpleGizmoCollectorModel::default()));
    let _scope = GizmosModel::with_collector(collector.clone());
    let style = GizmoStyleModel::stroke_and_fill(-1, 1.0, 0x4000_0000);

    GizmosModel::cuboid(AabbModel::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0), style)
        .unwrap()
        .set_always_on_top();
    GizmosModel::cuboid_colored(
        AabbModel::from_block_pos((1, 2, 3)).inflate(0.25),
        style,
        true,
    )
    .unwrap();
    GizmosModel::cuboid_block((2, 3, 4), style).unwrap();
    GizmosModel::cuboid_block_padded((2, 3, 4), 0.5, style).unwrap();
    GizmosModel::circle(Vec3Model::new(0.0, 0.0, 0.0), 2.0, style).unwrap();
    for face in [
        DirectionModel::Down,
        DirectionModel::Up,
        DirectionModel::North,
        DirectionModel::South,
        DirectionModel::West,
        DirectionModel::East,
    ] {
        GizmosModel::rect_from_face(
            Vec3Model::new(0.0, 0.0, 0.0),
            Vec3Model::new(1.0, 1.0, 1.0),
            face,
            style,
        )
        .unwrap();
    }
    GizmosModel::billboard_text(
        "plain",
        Vec3Model::new(0.0, 1.0, 0.0),
        TextStyleModel::white_and_centered(),
    )
    .unwrap();
    GizmosModel::billboard_text_over_block("block", (10, 64, 20), 2, -1, 0.5)
        .unwrap()
        .persist_for_millis(100, 400)
        .fade_out();

    let gizmos = &collector.borrow().gizmos;
    assert_eq!(gizmos.len(), 13);
    assert!(gizmos[0].is_always_on_top);
    assert!(matches!(
        &gizmos[12].gizmo,
        GizmoModel::Text { pos, style, .. }
            if *pos == Vec3Model::new(10.5, 65.7, 20.5)
                && style.scale == 0.5
                && style.adjust_left.is_none()
    ));
    assert!(gizmos[12].is_always_on_top);
    assert_eq!(gizmos[12].start_time_millis, 100);
    assert_eq!(gizmos[12].expire_time_millis, 500);
    assert!(gizmos[12].should_fade_out);
}
