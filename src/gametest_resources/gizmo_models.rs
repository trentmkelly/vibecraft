use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3Model {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3Model {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn add(self, x: f64, y: f64, z: f64) -> Self {
        Self::new(self.x + x, self.y + y, self.z + z)
    }

    pub fn subtract(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    pub fn distance_to(self, other: Self) -> f64 {
        let delta = self.subtract(other);
        (delta.x * delta.x + delta.y * delta.y + delta.z * delta.z).sqrt()
    }

    pub fn normalize(self) -> Self {
        let length = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if length == 0.0 {
            Self::new(0.0, 0.0, 0.0)
        } else {
            Self::new(self.x / length, self.y / length, self.z / length)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AabbModel {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl AabbModel {
    pub const fn new(
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
    ) -> Self {
        Self {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }

    pub fn from_block_pos(pos: (i32, i32, i32)) -> Self {
        Self::new(
            f64::from(pos.0),
            f64::from(pos.1),
            f64::from(pos.2),
            f64::from(pos.0 + 1),
            f64::from(pos.1 + 1),
            f64::from(pos.2 + 1),
        )
    }

    pub fn inflate(self, padding: f64) -> Self {
        Self::new(
            self.min_x - padding,
            self.min_y - padding,
            self.min_z - padding,
            self.max_x + padding,
            self.max_y + padding,
            self.max_z + padding,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionModel {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GizmoStyleModel {
    pub stroke: i32,
    pub stroke_width: f32,
    pub fill: i32,
}

impl GizmoStyleModel {
    pub const DEFAULT_WIDTH: f32 = 2.5;

    pub const fn stroke(argb: i32) -> Self {
        Self::stroke_with_width(argb, Self::DEFAULT_WIDTH)
    }

    pub const fn stroke_with_width(argb: i32, width: f32) -> Self {
        Self {
            stroke: argb,
            stroke_width: width,
            fill: 0,
        }
    }

    pub const fn fill(argb: i32) -> Self {
        Self {
            stroke: 0,
            stroke_width: 0.0,
            fill: argb,
        }
    }

    pub const fn stroke_and_fill(stroke: i32, stroke_width: f32, fill: i32) -> Self {
        Self {
            stroke,
            stroke_width,
            fill,
        }
    }

    pub fn has_fill(self) -> bool {
        self.fill != 0
    }

    pub fn has_stroke(self) -> bool {
        self.stroke != 0 && self.stroke_width > 0.0
    }

    pub fn multiplied_stroke(self, alpha_multiplier: f32) -> i32 {
        multiply_alpha(self.stroke, alpha_multiplier)
    }

    pub fn multiplied_fill(self, alpha_multiplier: f32) -> i32 {
        multiply_alpha(self.fill, alpha_multiplier)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextStyleModel {
    pub color: i32,
    pub scale: f32,
    pub adjust_left: Option<f64>,
}

impl TextStyleModel {
    pub const DEFAULT_SCALE: f32 = 0.32;

    pub fn white_and_centered() -> Self {
        Self {
            color: -1,
            scale: Self::DEFAULT_SCALE,
            adjust_left: None,
        }
    }

    pub fn for_color_and_centered(argb: i32) -> Self {
        Self {
            color: argb,
            scale: Self::DEFAULT_SCALE,
            adjust_left: None,
        }
    }

    pub fn for_color(argb: i32) -> Self {
        Self {
            color: argb,
            scale: Self::DEFAULT_SCALE,
            adjust_left: Some(0.0),
        }
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_left_alignment(mut self, adjust_left: f32) -> Self {
        self.adjust_left = Some(f64::from(adjust_left));
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GizmoModel {
    Arrow {
        start: Vec3Model,
        end: Vec3Model,
        color: i32,
        width: f32,
    },
    Circle {
        pos: Vec3Model,
        radius: f32,
        style: GizmoStyleModel,
    },
    Cuboid {
        aabb: AabbModel,
        style: GizmoStyleModel,
        colored_corner_stroke: bool,
    },
    Line {
        start: Vec3Model,
        end: Vec3Model,
        color: i32,
        width: f32,
    },
    Point {
        pos: Vec3Model,
        color: i32,
        size: f32,
    },
    Rect {
        a: Vec3Model,
        b: Vec3Model,
        c: Vec3Model,
        d: Vec3Model,
        style: GizmoStyleModel,
    },
    Text {
        pos: Vec3Model,
        text: String,
        style: TextStyleModel,
    },
}

impl GizmoModel {
    pub const LINE_DEFAULT_WIDTH: f32 = 3.0;
    pub const ARROW_DEFAULT_WIDTH: f32 = 2.5;

    pub fn emit(&self, primitives: &mut Vec<GizmoPrimitiveModel>, alpha_multiplier: f32) {
        match self {
            Self::Arrow {
                start,
                end,
                color,
                width,
            } => emit_arrow(*start, *end, *color, *width, alpha_multiplier, primitives),
            Self::Circle { pos, radius, style } => {
                emit_circle(*pos, *radius, *style, alpha_multiplier, primitives);
            }
            Self::Cuboid {
                aabb,
                style,
                colored_corner_stroke,
            } => emit_cuboid(
                *aabb,
                *style,
                *colored_corner_stroke,
                alpha_multiplier,
                primitives,
            ),
            Self::Line {
                start,
                end,
                color,
                width,
            } => primitives.push(GizmoPrimitiveModel::Line {
                start: *start,
                end: *end,
                color: multiply_alpha(*color, alpha_multiplier),
                width: *width,
            }),
            Self::Point { pos, color, size } => primitives.push(GizmoPrimitiveModel::Point {
                pos: *pos,
                color: multiply_alpha(*color, alpha_multiplier),
                size: *size,
            }),
            Self::Rect { a, b, c, d, style } => {
                emit_rect(*a, *b, *c, *d, *style, alpha_multiplier, primitives);
            }
            Self::Text { pos, text, style } => {
                let style = if alpha_multiplier < 1.0 {
                    TextStyleModel {
                        color: multiply_alpha(style.color, alpha_multiplier),
                        scale: style.scale,
                        adjust_left: style.adjust_left,
                    }
                } else {
                    style.clone()
                };
                primitives.push(GizmoPrimitiveModel::Text {
                    pos: *pos,
                    text: text.clone(),
                    style,
                });
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GizmoPrimitiveModel {
    Point {
        pos: Vec3Model,
        color: i32,
        size: f32,
    },
    Line {
        start: Vec3Model,
        end: Vec3Model,
        color: i32,
        width: f32,
    },
    TriangleFan {
        points: Vec<Vec3Model>,
        color: i32,
    },
    Quad {
        a: Vec3Model,
        b: Vec3Model,
        c: Vec3Model,
        d: Vec3Model,
        color: i32,
    },
    Text {
        pos: Vec3Model,
        text: String,
        style: TextStyleModel,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct GizmoInstanceModel {
    pub gizmo: GizmoModel,
    pub is_always_on_top: bool,
    pub start_time_millis: i64,
    pub expire_time_millis: i64,
    pub should_fade_out: bool,
}

impl GizmoInstanceModel {
    pub fn new(gizmo: GizmoModel) -> Self {
        Self {
            gizmo,
            is_always_on_top: false,
            start_time_millis: 0,
            expire_time_millis: 0,
            should_fade_out: false,
        }
    }

    pub fn alpha_multiplier(&self, current_millis: i64) -> f32 {
        if self.should_fade_out {
            let duration = self.expire_time_millis - self.start_time_millis;
            let time_since_start = current_millis - self.start_time_millis;
            1.0 - ((time_since_start as f32) / (duration as f32)).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct SimpleGizmoCollectorModel {
    pub gizmos: Vec<GizmoInstanceModel>,
    pub temporary_gizmos: Vec<GizmoInstanceModel>,
}

impl SimpleGizmoCollectorModel {
    pub fn add(&mut self, gizmo: GizmoModel) -> usize {
        let index = self.gizmos.len();
        self.gizmos.push(GizmoInstanceModel::new(gizmo));
        index
    }

    pub fn add_temporary_gizmos(&mut self, gizmos: impl IntoIterator<Item = GizmoInstanceModel>) {
        self.temporary_gizmos.extend(gizmos);
    }

    pub fn drain_gizmos(&mut self, current_millis: i64) -> Vec<GizmoInstanceModel> {
        let mut result = self.gizmos.clone();
        result.extend(self.temporary_gizmos.clone());
        self.gizmos
            .retain(|gizmo| gizmo.expire_time_millis >= current_millis);
        self.temporary_gizmos.clear();
        result
    }
}

#[derive(Clone)]
pub enum GizmoPropertiesModel {
    Ignored,
    Instance {
        collector: Rc<RefCell<SimpleGizmoCollectorModel>>,
        index: usize,
    },
}

impl GizmoPropertiesModel {
    pub fn set_always_on_top(&self) -> Self {
        if let Self::Instance { collector, index } = self {
            if let Some(instance) = collector.borrow_mut().gizmos.get_mut(*index) {
                instance.is_always_on_top = true;
            }
        }
        self.clone()
    }

    pub fn persist_for_millis(&self, start_millis: i64, milliseconds: i32) -> Self {
        if let Self::Instance { collector, index } = self {
            if let Some(instance) = collector.borrow_mut().gizmos.get_mut(*index) {
                instance.start_time_millis = start_millis;
                instance.expire_time_millis = start_millis + i64::from(milliseconds);
            }
        }
        self.clone()
    }

    pub fn fade_out(&self) -> Self {
        if let Self::Instance { collector, index } = self {
            if let Some(instance) = collector.borrow_mut().gizmos.get_mut(*index) {
                instance.should_fade_out = true;
            }
        }
        self.clone()
    }
}

thread_local! {
    static ACTIVE_COLLECTOR: RefCell<Option<Rc<RefCell<SimpleGizmoCollectorModel>>>> = const { RefCell::new(None) };
}

pub struct TemporaryCollectionModel {
    old: Option<Rc<RefCell<SimpleGizmoCollectorModel>>>,
    closed: bool,
}

impl TemporaryCollectionModel {
    pub fn close(&mut self) {
        if !self.closed {
            self.closed = true;
            let old = self.old.clone();
            ACTIVE_COLLECTOR.with(|active| *active.borrow_mut() = old);
        }
    }
}

impl Drop for TemporaryCollectionModel {
    fn drop(&mut self) {
        self.close();
    }
}

pub struct GizmosModel;

impl GizmosModel {
    pub fn with_collector(
        collector: Rc<RefCell<SimpleGizmoCollectorModel>>,
    ) -> TemporaryCollectionModel {
        let old = ACTIVE_COLLECTOR.with(|active| active.borrow_mut().replace(collector));
        TemporaryCollectionModel { old, closed: false }
    }

    pub fn add_gizmo(gizmo: GizmoModel) -> Result<GizmoPropertiesModel, String> {
        ACTIVE_COLLECTOR.with(|active| {
            let Some(collector) = active.borrow().clone() else {
                return Err(
                    "Gizmos cannot be created here! No GizmoCollector has been registered."
                        .to_string(),
                );
            };
            let index = collector.borrow_mut().add(gizmo);
            Ok(GizmoPropertiesModel::Instance { collector, index })
        })
    }

    pub fn cuboid(aabb: AabbModel, style: GizmoStyleModel) -> Result<GizmoPropertiesModel, String> {
        Self::cuboid_colored(aabb, style, false)
    }

    pub fn cuboid_colored(
        aabb: AabbModel,
        style: GizmoStyleModel,
        colored_corner: bool,
    ) -> Result<GizmoPropertiesModel, String> {
        Self::add_gizmo(GizmoModel::Cuboid {
            aabb,
            style,
            colored_corner_stroke: colored_corner,
        })
    }

    pub fn cuboid_block(
        block_pos: (i32, i32, i32),
        style: GizmoStyleModel,
    ) -> Result<GizmoPropertiesModel, String> {
        Self::cuboid(AabbModel::from_block_pos(block_pos), style)
    }

    pub fn cuboid_block_padded(
        block_pos: (i32, i32, i32),
        padding: f32,
        style: GizmoStyleModel,
    ) -> Result<GizmoPropertiesModel, String> {
        Self::cuboid(
            AabbModel::from_block_pos(block_pos).inflate(f64::from(padding)),
            style,
        )
    }

    pub fn circle(
        pos: Vec3Model,
        radius: f32,
        style: GizmoStyleModel,
    ) -> Result<GizmoPropertiesModel, String> {
        Self::add_gizmo(GizmoModel::Circle { pos, radius, style })
    }

    pub fn line(
        start: Vec3Model,
        end: Vec3Model,
        argb: i32,
    ) -> Result<GizmoPropertiesModel, String> {
        Self::line_with_width(start, end, argb, GizmoModel::LINE_DEFAULT_WIDTH)
    }

    pub fn line_with_width(
        start: Vec3Model,
        end: Vec3Model,
        argb: i32,
        width: f32,
    ) -> Result<GizmoPropertiesModel, String> {
        Self::add_gizmo(GizmoModel::Line {
            start,
            end,
            color: argb,
            width,
        })
    }

    pub fn arrow(
        start: Vec3Model,
        end: Vec3Model,
        argb: i32,
    ) -> Result<GizmoPropertiesModel, String> {
        Self::arrow_with_width(start, end, argb, GizmoModel::ARROW_DEFAULT_WIDTH)
    }

    pub fn arrow_with_width(
        start: Vec3Model,
        end: Vec3Model,
        argb: i32,
        width: f32,
    ) -> Result<GizmoPropertiesModel, String> {
        Self::add_gizmo(GizmoModel::Arrow {
            start,
            end,
            color: argb,
            width,
        })
    }

    pub fn rect_from_face(
        cuboid_corner_a: Vec3Model,
        cuboid_corner_b: Vec3Model,
        face: DirectionModel,
        style: GizmoStyleModel,
    ) -> Result<GizmoPropertiesModel, String> {
        Self::add_gizmo(rect_from_cuboid_face(
            cuboid_corner_a,
            cuboid_corner_b,
            face,
            style,
        ))
    }

    pub fn point(
        position: Vec3Model,
        argb: i32,
        size: f32,
    ) -> Result<GizmoPropertiesModel, String> {
        Self::add_gizmo(GizmoModel::Point {
            pos: position,
            color: argb,
            size,
        })
    }

    pub fn billboard_text(
        name: impl Into<String>,
        pos: Vec3Model,
        style: TextStyleModel,
    ) -> Result<GizmoPropertiesModel, String> {
        Self::add_gizmo(GizmoModel::Text {
            pos,
            text: name.into(),
            style,
        })
    }

    pub fn billboard_text_over_block(
        text: impl Into<String>,
        pos: (i32, i32, i32),
        row: i32,
        color: i32,
        scale: f32,
    ) -> Result<GizmoPropertiesModel, String> {
        let properties = Self::billboard_text(
            text,
            Vec3Model::new(
                f64::from(pos.0) + 0.5,
                f64::from(pos.1) + 1.3 + f64::from(row) * 0.2,
                f64::from(pos.2) + 0.5,
            ),
            TextStyleModel::for_color_and_centered(color).with_scale(scale),
        )?;
        Ok(properties.set_always_on_top())
    }
}

pub fn rect_from_cuboid_face(
    cuboid_corner_a: Vec3Model,
    cuboid_corner_b: Vec3Model,
    face: DirectionModel,
    style: GizmoStyleModel,
) -> GizmoModel {
    let a = cuboid_corner_a;
    let b = cuboid_corner_b;
    let (ra, rb, rc, rd) = match face {
        DirectionModel::Down => (
            Vec3Model::new(a.x, a.y, a.z),
            Vec3Model::new(b.x, a.y, a.z),
            Vec3Model::new(b.x, a.y, b.z),
            Vec3Model::new(a.x, a.y, b.z),
        ),
        DirectionModel::Up => (
            Vec3Model::new(a.x, b.y, a.z),
            Vec3Model::new(a.x, b.y, b.z),
            Vec3Model::new(b.x, b.y, b.z),
            Vec3Model::new(b.x, b.y, a.z),
        ),
        DirectionModel::North => (
            Vec3Model::new(a.x, a.y, a.z),
            Vec3Model::new(a.x, b.y, a.z),
            Vec3Model::new(b.x, b.y, a.z),
            Vec3Model::new(b.x, a.y, a.z),
        ),
        DirectionModel::South => (
            Vec3Model::new(a.x, a.y, b.z),
            Vec3Model::new(b.x, a.y, b.z),
            Vec3Model::new(b.x, b.y, b.z),
            Vec3Model::new(a.x, b.y, b.z),
        ),
        DirectionModel::West => (
            Vec3Model::new(a.x, a.y, a.z),
            Vec3Model::new(a.x, a.y, b.z),
            Vec3Model::new(a.x, b.y, b.z),
            Vec3Model::new(a.x, b.y, a.z),
        ),
        DirectionModel::East => (
            Vec3Model::new(b.x, a.y, a.z),
            Vec3Model::new(b.x, b.y, a.z),
            Vec3Model::new(b.x, b.y, b.z),
            Vec3Model::new(b.x, a.y, b.z),
        ),
    };
    GizmoModel::Rect {
        a: ra,
        b: rb,
        c: rc,
        d: rd,
        style,
    }
}

fn emit_rect(
    a: Vec3Model,
    b: Vec3Model,
    c: Vec3Model,
    d: Vec3Model,
    style: GizmoStyleModel,
    alpha_multiplier: f32,
    primitives: &mut Vec<GizmoPrimitiveModel>,
) {
    if style.has_fill() {
        primitives.push(GizmoPrimitiveModel::Quad {
            a,
            b,
            c,
            d,
            color: style.multiplied_fill(alpha_multiplier),
        });
    }
    if style.has_stroke() {
        let color = style.multiplied_stroke(alpha_multiplier);
        for (start, end) in [(a, b), (b, c), (c, d), (d, a)] {
            primitives.push(GizmoPrimitiveModel::Line {
                start,
                end,
                color,
                width: style.stroke_width,
            });
        }
    }
}

fn emit_circle(
    pos: Vec3Model,
    radius: f32,
    style: GizmoStyleModel,
    alpha_multiplier: f32,
    primitives: &mut Vec<GizmoPrimitiveModel>,
) {
    if !style.has_stroke() && !style.has_fill() {
        return;
    }
    let mut points = Vec::with_capacity(21);
    for i in 0..20 {
        let theta = (i as f32) * std::f32::consts::PI / 10.0;
        points.push(pos.add(
            f64::from(radius * theta.cos()),
            0.0,
            f64::from(radius * theta.sin()),
        ));
    }
    points.push(points[0]);

    if style.has_fill() {
        primitives.push(GizmoPrimitiveModel::TriangleFan {
            points: points.clone(),
            color: style.multiplied_fill(alpha_multiplier),
        });
    }
    if style.has_stroke() {
        let color = style.multiplied_stroke(alpha_multiplier);
        for i in 0..20 {
            primitives.push(GizmoPrimitiveModel::Line {
                start: points[i],
                end: points[i + 1],
                color,
                width: style.stroke_width,
            });
        }
    }
}

fn emit_cuboid(
    aabb: AabbModel,
    style: GizmoStyleModel,
    colored_corner_stroke: bool,
    alpha_multiplier: f32,
    primitives: &mut Vec<GizmoPrimitiveModel>,
) {
    let (x0, y0, z0) = (aabb.min_x, aabb.min_y, aabb.min_z);
    let (x1, y1, z1) = (aabb.max_x, aabb.max_y, aabb.max_z);
    if style.has_fill() {
        let color = style.multiplied_fill(alpha_multiplier);
        for (a, b, c, d) in [
            ((x1, y0, z0), (x1, y1, z0), (x1, y1, z1), (x1, y0, z1)),
            ((x0, y0, z0), (x0, y0, z1), (x0, y1, z1), (x0, y1, z0)),
            ((x0, y0, z0), (x0, y1, z0), (x1, y1, z0), (x1, y0, z0)),
            ((x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1)),
            ((x0, y1, z0), (x0, y1, z1), (x1, y1, z1), (x1, y1, z0)),
            ((x0, y0, z0), (x1, y0, z0), (x1, y0, z1), (x0, y0, z1)),
        ] {
            primitives.push(GizmoPrimitiveModel::Quad {
                a: tuple_vec(a),
                b: tuple_vec(b),
                c: tuple_vec(c),
                d: tuple_vec(d),
                color,
            });
        }
    }
    if style.has_stroke() {
        let color = style.multiplied_stroke(alpha_multiplier);
        let corner_colors = [
            if colored_corner_stroke {
                multiply_color(color, -34953)
            } else {
                color
            },
            if colored_corner_stroke {
                multiply_color(color, -8913033)
            } else {
                color
            },
            if colored_corner_stroke {
                multiply_color(color, -8947713)
            } else {
                color
            },
        ];
        for (index, (start, end)) in [
            ((x0, y0, z0), (x1, y0, z0)),
            ((x0, y0, z0), (x0, y1, z0)),
            ((x0, y0, z0), (x0, y0, z1)),
            ((x1, y0, z0), (x1, y1, z0)),
            ((x1, y1, z0), (x0, y1, z0)),
            ((x0, y1, z0), (x0, y1, z1)),
            ((x0, y1, z1), (x0, y0, z1)),
            ((x0, y0, z1), (x1, y0, z1)),
            ((x1, y0, z1), (x1, y0, z0)),
            ((x0, y1, z1), (x1, y1, z1)),
            ((x1, y0, z1), (x1, y1, z1)),
            ((x1, y1, z0), (x1, y1, z1)),
        ]
        .into_iter()
        .enumerate()
        {
            primitives.push(GizmoPrimitiveModel::Line {
                start: tuple_vec(start),
                end: tuple_vec(end),
                color: corner_colors.get(index).copied().unwrap_or(color),
                width: style.stroke_width,
            });
        }
    }
}

fn emit_arrow(
    start: Vec3Model,
    end: Vec3Model,
    color: i32,
    width: f32,
    alpha_multiplier: f32,
    primitives: &mut Vec<GizmoPrimitiveModel>,
) {
    let color = multiply_alpha(color, alpha_multiplier);
    primitives.push(GizmoPrimitiveModel::Line {
        start,
        end,
        color,
        width,
    });
    let direction = end.subtract(start).normalize();
    let len = (end.distance_to(start) * 0.1).clamp(0.1, 1.0);
    for tip in [
        Vec3Model::new(-len, len, 0.0),
        Vec3Model::new(-len, 0.0, len),
        Vec3Model::new(-len, -len, 0.0),
        Vec3Model::new(-len, 0.0, -len),
    ] {
        let transformed = rotate_from_positive_x(tip, direction);
        primitives.push(GizmoPrimitiveModel::Line {
            start: end.add(transformed.x, transformed.y, transformed.z),
            end,
            color,
            width,
        });
    }
}

fn rotate_from_positive_x(point: Vec3Model, direction: Vec3Model) -> Vec3Model {
    let from = Vec3Model::new(1.0, 0.0, 0.0);
    let dot = (from.x * direction.x + from.y * direction.y + from.z * direction.z).clamp(-1.0, 1.0);
    if dot > 0.999_999 {
        return point;
    }
    if dot < -0.999_999 {
        return Vec3Model::new(-point.x, point.y, -point.z);
    }
    let axis = Vec3Model::new(0.0, -direction.z, direction.y).normalize();
    let cos = dot;
    let sin = (1.0 - dot * dot).sqrt();
    let axis_dot_point = axis.x * point.x + axis.y * point.y + axis.z * point.z;
    Vec3Model::new(
        point.x * cos
            + (axis.y * point.z - axis.z * point.y) * sin
            + axis.x * axis_dot_point * (1.0 - cos),
        point.y * cos
            + (axis.z * point.x - axis.x * point.z) * sin
            + axis.y * axis_dot_point * (1.0 - cos),
        point.z * cos
            + (axis.x * point.y - axis.y * point.x) * sin
            + axis.z * axis_dot_point * (1.0 - cos),
    )
}

fn tuple_vec(value: (f64, f64, f64)) -> Vec3Model {
    Vec3Model::new(value.0, value.1, value.2)
}

fn multiply_alpha(color: i32, alpha_multiplier: f32) -> i32 {
    if color == 0 || alpha_multiplier <= 0.0 {
        0
    } else if alpha_multiplier >= 1.0 {
        color
    } else {
        color_with_alpha(
            (alpha_float(color) * alpha_multiplier * 255.0).floor() as i32,
            color,
        )
    }
}

fn multiply_color(lhs: i32, rhs: i32) -> i32 {
    if lhs == -1 {
        rhs
    } else if rhs == -1 {
        lhs
    } else {
        color(
            alpha(lhs) * alpha(rhs) / 255,
            red(lhs) * red(rhs) / 255,
            green(lhs) * green(rhs) / 255,
            blue(lhs) * blue(rhs) / 255,
        )
    }
}

fn alpha_float(color: i32) -> f32 {
    alpha(color) as f32 / 255.0
}

fn color_with_alpha(alpha: i32, rgb: i32) -> i32 {
    (((alpha & 0xFF) as u32) << 24 | ((rgb as u32) & 0x00FF_FFFF)) as i32
}

fn color(alpha: i32, red: i32, green: i32, blue: i32) -> i32 {
    (((alpha & 0xFF) as u32) << 24
        | ((red & 0xFF) as u32) << 16
        | ((green & 0xFF) as u32) << 8
        | ((blue & 0xFF) as u32)) as i32
}

fn alpha(color: i32) -> i32 {
    ((color as u32) >> 24) as i32
}

fn red(color: i32) -> i32 {
    ((color as u32) >> 16 & 0xFF) as i32
}

fn green(color: i32) -> i32 {
    ((color as u32) >> 8 & 0xFF) as i32
}

fn blue(color: i32) -> i32 {
    ((color as u32) & 0xFF) as i32
}
