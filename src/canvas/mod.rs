mod imp;
mod renderer;

use std::path::PathBuf;

use gtk::gdk;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use uuid::Uuid;

use crate::annotations::{
    ActiveTool, Annotation, AnnotationCommand, AnnotationEngine, AnnotationKind, AnnotationStyle,
    ArrowData, CalloutData, DrawingState, FreehandData, HandleIndex, NumberMarkerData, Point,
    Rect, TextData,
};
use crate::models::ImageData;

glib::wrapper! {
    pub struct Canvas(ObjectSubclass<imp::Canvas>)
        @extends gtk::Widget, gtk::DrawingArea;
}

impl Canvas {
    const ZOOM_MIN: f64 = 0.1;
    const ZOOM_MAX: f64 = 8.0;
    const ZOOM_STEP: f64 = 1.25;
    const SCROLL_STEP: f64 = 1.1;
    const MIN_DRAG_DISTANCE: f64 = 4.0;
    const HANDLE_HIT_RADIUS: f64 = 8.0;
    const MIN_BOUNDS_SIZE: f64 = 4.0;
    const NUMBER_MARKER_SIZE: f64 = 24.0;

    pub fn new() -> Self {
        let canvas = glib::Object::new::<Self>();
        canvas.set_focusable(true);

        canvas.set_draw_func(move |widget, cr, width, height| {
            let Some(canvas) = widget.downcast_ref::<Canvas>() else {
                return;
            };
            let imp = canvas.imp();
            let zoom = imp.zoom.get();
            let (pan_x, pan_y) = imp.pan_offset.get();

            if let Some(image) = imp.image.borrow().as_ref() {
                let pixbuf = image.pixbuf();
                cr.save().expect("cairo save");
                cr.translate(pan_x, pan_y);
                cr.scale(zoom, zoom);
                cr.set_source_pixbuf(pixbuf, 0.0, 0.0);
                cr.source().set_filter(gtk::cairo::Filter::Bilinear);
                let _ = cr.paint();
                cr.restore().expect("cairo restore");

                let annotations = imp.annotations.borrow();
                renderer::draw_all(
                    cr,
                    annotations.all(),
                    annotations.selected_id(),
                    Some(pixbuf),
                    zoom,
                    pan_x,
                    pan_y,
                );

                if let Some(preview) = canvas.build_preview_annotation() {
                    renderer::draw_preview(cr, &preview, Some(pixbuf), zoom, pan_x, pan_y);
                }
            } else {
                cr.set_source_rgb(0.12, 0.12, 0.12);
                cr.rectangle(0.0, 0.0, width as f64, height as f64);
                let _ = cr.fill();
            }
        });

        let motion = gtk::EventControllerMotion::new();
        let canvas_weak = canvas.downgrade();
        motion.connect_motion(move |_, x, y| {
            if let Some(c) = canvas_weak.upgrade() {
                c.imp().pointer_pos.set((x, y));
            }
        });
        canvas.add_controller(motion);

        let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        let canvas_weak = canvas.downgrade();
        scroll.connect_scroll(move |_, _dx, dy| {
            if let Some(c) = canvas_weak.upgrade() {
                let factor = if dy < 0.0 {
                    Canvas::SCROLL_STEP
                } else {
                    1.0 / Canvas::SCROLL_STEP
                };
                let anchor = c.imp().pointer_pos.get();
                c.apply_zoom(c.imp().zoom.get() * factor, Some(anchor));
            }
            glib::Propagation::Stop
        });
        canvas.add_controller(scroll);

        let pan_drag = gtk::GestureDrag::new();
        pan_drag.set_button(2);
        let canvas_weak_begin = canvas.downgrade();
        pan_drag.connect_drag_begin(move |_, _x, _y| {
            if let Some(c) = canvas_weak_begin.upgrade() {
                c.imp().pan_base.set(c.imp().pan_offset.get());
                c.set_cursor_from_name(Some("grabbing"));
            }
        });
        let canvas_weak_update = canvas.downgrade();
        pan_drag.connect_drag_update(move |_, offset_x, offset_y| {
            if let Some(c) = canvas_weak_update.upgrade() {
                let (base_x, base_y) = c.imp().pan_base.get();
                c.imp()
                    .pan_offset
                    .set((base_x + offset_x, base_y + offset_y));
                c.queue_draw();
            }
        });
        let canvas_weak_end = canvas.downgrade();
        pan_drag.connect_drag_end(move |_, _, _| {
            if let Some(c) = canvas_weak_end.upgrade() {
                c.set_cursor_from_name(None);
            }
        });
        canvas.add_controller(pan_drag);

        let draw_drag = gtk::GestureDrag::new();
        draw_drag.set_button(1);
        let canvas_weak = canvas.downgrade();
        draw_drag.connect_drag_begin(move |gesture, start_x, start_y| {
            let Some(c) = canvas_weak.upgrade() else {
                return;
            };
            if gesture.current_button() != 1 {
                return;
            }
            let p = c.screen_to_image(start_x, start_y);
            let tool = c.imp().active_tool.get();

            if tool == ActiveTool::Select {
                let mut engine = c.imp().annotations.borrow_mut();
                if let Some(id) = engine.hit_test(p) {
                    if let Some(ann) = engine.annotation_at(id) {
                        let bounds = ann.bounds;
                        drop(engine);
                        if let Some(handle) = c.handle_at(&bounds, p) {
                            c.imp().drawing_state.replace(DrawingState::ResizingHandle {
                                id,
                                handle,
                                original_bounds: bounds,
                                drag_start: p,
                            });
                        } else {
                            c.imp().annotations.borrow_mut().select(id);
                            c.imp().drawing_state.replace(DrawingState::Moving {
                                id,
                                drag_start: p,
                                original_bounds: bounds,
                            });
                        }
                        c.queue_draw();
                    }
                } else {
                    engine.deselect();
                    drop(engine);
                    c.imp().drawing_state.replace(DrawingState::Idle);
                    c.queue_draw();
                }
            } else {
                c.imp().freehand_points.replace(vec![p]);
                c.imp().drawing_state.replace(DrawingState::Drawing {
                    start: p,
                    current: p,
                });
            }
        });

        let canvas_weak = canvas.downgrade();
        draw_drag.connect_drag_update(move |_, offset_x, offset_y| {
            let Some(c) = canvas_weak.upgrade() else {
                return;
            };
            let (start_x, start_y) = {
                let state = c.imp().drawing_state.borrow();
                match &*state {
                    DrawingState::Drawing { start, .. } => {
                        c.image_to_screen(*start)
                    }
                    DrawingState::Moving { drag_start, .. } => c.image_to_screen(*drag_start),
                    DrawingState::ResizingHandle { drag_start, .. } => {
                        c.image_to_screen(*drag_start)
                    }
                    _ => return,
                }
            };

            let current = c.screen_to_image(start_x + offset_x, start_y + offset_y);
            let mut state = c.imp().drawing_state.borrow_mut();

            match &mut *state {
                DrawingState::Drawing { current: cur, .. } => {
                    *cur = current;
                    if c.imp().active_tool.get() == ActiveTool::Freehand {
                        let mut points = c.imp().freehand_points.borrow_mut();
                        if points
                            .last()
                            .map(|last| distance(*last, current) >= 1.0)
                            .unwrap_or(true)
                        {
                            points.push(current);
                        }
                    }
                    drop(state);
                    c.queue_draw();
                }
                DrawingState::Moving {
                    id,
                    drag_start,
                    original_bounds,
                } => {
                    let dx = current.x - drag_start.x;
                    let dy = current.y - drag_start.y;
                    let new_bounds = Rect {
                        x: original_bounds.x + dx,
                        y: original_bounds.y + dy,
                        width: original_bounds.width,
                        height: original_bounds.height,
                    };
                    let id = *id;
                    let original = *original_bounds;
                    drop(state);
                    c.imp()
                        .annotations
                        .borrow_mut()
                        .move_to_bounds(id, original, new_bounds);
                    c.queue_draw();
                }
                DrawingState::ResizingHandle {
                    id,
                    handle,
                    original_bounds,
                    drag_start,
                } => {
                    let dx = current.x - drag_start.x;
                    let dy = current.y - drag_start.y;
                    let new_bounds = resize_bounds(*original_bounds, *handle, dx, dy);
                    let id = *id;
                    let original = *original_bounds;
                    drop(state);
                    c.imp()
                        .annotations
                        .borrow_mut()
                        .resize_to_bounds(id, original, new_bounds);
                    c.queue_draw();
                }
                _ => {}
            }
        });

        let canvas_weak = canvas.downgrade();
        draw_drag.connect_drag_end(move |_, offset_x, offset_y| {
            let Some(c) = canvas_weak.upgrade() else {
                return;
            };
            c.finish_draw_drag(offset_x, offset_y);
        });
        canvas.add_controller(draw_drag);

        let click = gtk::GestureClick::new();
        click.set_button(1);
        click.set_propagation_phase(gtk::PropagationPhase::Capture);
        let canvas_weak = canvas.downgrade();
        click.connect_pressed(move |gesture, n_press, x, y| {
            if gesture.current_button() != 1 || n_press != 2 {
                return;
            }
            let Some(c) = canvas_weak.upgrade() else {
                return;
            };
            let p = c.screen_to_image(x, y);
            let engine = c.imp().annotations.borrow();
            if let Some(id) = engine.hit_test(p) {
                if let Some(ann) = engine.annotation_at(id) {
                    let is_text = matches!(
                        ann.kind,
                        AnnotationKind::Text(_) | AnnotationKind::Callout(_)
                    );
                    if is_text {
                        drop(engine);
                        c.open_text_editor(p, Some(id));
                    }
                }
            }
        });
        canvas.add_controller(click);

        let key_ctrl = gtk::EventControllerKey::new();
        let canvas_weak = canvas.downgrade();
        key_ctrl.connect_key_pressed(move |_, key, _keycode, _state| {
            let Some(c) = canvas_weak.upgrade() else {
                return glib::Propagation::Proceed;
            };
            match key {
                gdk::Key::Delete => {
                    let mut engine = c.imp().annotations.borrow_mut();
                    if let Some(id) = engine.selected_id() {
                        if let Some(ann) = engine.remove(id) {
                            drop(engine);
                            c.imp()
                                .history
                                .borrow_mut()
                                .push(AnnotationCommand::Remove(ann));
                            c.notify_annotation_changed();
                            c.queue_draw();
                        }
                    }
                    glib::Propagation::Stop
                }
                gdk::Key::Escape => {
                    c.imp().annotations.borrow_mut().deselect();
                    c.imp().drawing_state.replace(DrawingState::Idle);
                    c.queue_draw();
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        canvas.add_controller(key_ctrl);

        canvas
    }

    pub fn set_image(&self, image: ImageData) {
        self.imp().image.replace(Some(image));
        self.queue_draw();
    }

    pub fn clear(&self) {
        self.imp().image.replace(None);
        self.queue_draw();
    }

    pub fn set_active_tool(&self, tool: ActiveTool) {
        self.imp().active_tool.set(tool);
        self.set_cursor_from_name(None);
        self.queue_draw();
    }

    pub fn current_style(&self) -> AnnotationStyle {
        self.imp().current_style.borrow().clone()
    }

    pub fn set_current_style(&self, style: AnnotationStyle) {
        let mut engine = self.imp().annotations.borrow_mut();
        if let Some(id) = engine.selected_id() {
            if let Some(ann) = engine.annotation_at(id) {
                let old_style = ann.style.clone();
                if old_style != style {
                    engine.update_style(id, style.clone());
                    drop(engine);
                    self.imp().history.borrow_mut().push(AnnotationCommand::UpdateStyle {
                        id,
                        old_style,
                        new_style: style.clone(),
                    });
                    *self.imp().current_style.borrow_mut() = style;
                    self.notify_annotation_changed();
                    self.queue_draw();
                    return;
                }
            }
        }
        drop(engine);
        *self.imp().current_style.borrow_mut() = style;
    }

    pub fn undo(&self) -> bool {
        let mut history = self.imp().history.borrow_mut();
        let mut engine = self.imp().annotations.borrow_mut();
        let result = history.undo(&mut engine);
        drop(history);
        drop(engine);
        if result {
            self.notify_annotation_changed();
            self.queue_draw();
        }
        result
    }

    pub fn redo(&self) -> bool {
        let mut history = self.imp().history.borrow_mut();
        let mut engine = self.imp().annotations.borrow_mut();
        let result = history.redo(&mut engine);
        drop(history);
        drop(engine);
        if result {
            self.notify_annotation_changed();
            self.queue_draw();
        }
        result
    }

    pub fn can_undo(&self) -> bool {
        self.imp().history.borrow().can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.imp().history.borrow().can_redo()
    }

    pub fn on_annotation_changed(&self, cb: impl Fn() + 'static) {
        self.imp()
            .annotation_changed_cb
            .replace(Some(Box::new(cb)));
    }

    pub fn open_text_editor(&self, position: Point, existing_id: Option<Uuid>) {
        let existing_text = existing_id.and_then(|id| {
            self.imp()
                .annotations
                .borrow()
                .annotation_at(id)
                .and_then(|ann| match &ann.kind {
                    AnnotationKind::Text(data) => Some(data.text.clone()),
                    AnnotationKind::Callout(data) => Some(data.text.clone()),
                    _ => None,
                })
        });

        let tool = if existing_id.is_some() {
            self.imp()
                .annotations
                .borrow()
                .annotation_at(existing_id.unwrap())
                .map(|ann| match ann.kind {
                    AnnotationKind::Callout(_) => ActiveTool::Callout,
                    _ => ActiveTool::Text,
                })
                .unwrap_or(ActiveTool::Text)
        } else {
            self.imp().active_tool.get()
        };

        let dialog = gtk::Dialog::builder()
            .title(if existing_id.is_some() {
                "Edit Text"
            } else {
                "Add Text"
            })
            .modal(true)
            .build();

        if let Some(root) = self.root() {
            dialog.set_transient_for(Some(
                root.downcast_ref::<gtk::Window>().expect("root is Window"),
            ));
        }

        let content = dialog.content_area();
        let entry = gtk::Entry::builder()
            .hexpand(true)
            .activates_default(true)
            .build();
        if let Some(text) = existing_text {
            entry.set_text(&text);
        }
        content.append(&entry);

        dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        dialog.add_button("OK", gtk::ResponseType::Accept);
        dialog.set_default_response(gtk::ResponseType::Accept);

        let canvas_weak = self.downgrade();
        let style = self.imp().current_style.borrow().clone();
        let entry_for_response = entry.clone();
        dialog.connect_response(move |dialog, response| {
            if response == gtk::ResponseType::Accept {
                let text = entry_for_response.text().to_string();
                if !text.is_empty() {
                    if let Some(c) = canvas_weak.upgrade() {
                        c.confirm_text_editor(text, position, existing_id, tool, style.clone());
                    }
                }
            }
            dialog.close();
        });

        dialog.present();
        entry.grab_focus_without_selecting();
    }

    pub fn zoom_in(&self) {
        self.apply_zoom(self.imp().zoom.get() * Self::ZOOM_STEP, None);
    }

    pub fn zoom_out(&self) {
        self.apply_zoom(self.imp().zoom.get() / Self::ZOOM_STEP, None);
    }

    pub fn zoom_100(&self) {
        let zoom = 1.0;
        let cx = self.width() as f64 / 2.0;
        let cy = self.height() as f64 / 2.0;
        let (iw, ih) = self.image_size();
        let pan_x = cx - iw / 2.0;
        let pan_y = cy - ih / 2.0;
        self.imp().zoom.set(zoom);
        self.imp().pan_offset.set((pan_x, pan_y));
        self.notify_zoom_changed(zoom);
        self.queue_draw();
    }

    pub fn fit_to_window(&self) {
        let cw = self.width() as f64;
        let ch = self.height() as f64;
        if cw <= 0.0 || ch <= 0.0 {
            return;
        }
        let (iw, ih) = self.image_size();
        if iw <= 0.0 || ih <= 0.0 {
            return;
        }
        let zoom = (cw / iw).min(ch / ih).clamp(Self::ZOOM_MIN, Self::ZOOM_MAX);
        let pan_x = (cw - iw * zoom) / 2.0;
        let pan_y = (ch - ih * zoom) / 2.0;
        self.imp().zoom.set(zoom);
        self.imp().pan_offset.set((pan_x, pan_y));
        self.notify_zoom_changed(zoom);
        self.queue_draw();
    }

    pub fn zoom_level(&self) -> f64 {
        self.imp().zoom.get()
    }

    pub fn pan_offset(&self) -> (f64, f64) {
        self.imp().pan_offset.get()
    }

    pub fn all_annotations(&self) -> Vec<Annotation> {
        self.imp().annotations.borrow().all().to_vec()
    }

    pub fn source_image_path(&self) -> Option<PathBuf> {
        self.imp()
            .image
            .borrow()
            .as_ref()
            .map(|img| img.source().path.clone())
    }

    pub fn source_image_dimensions(&self) -> Option<(u32, u32)> {
        self.imp().image.borrow().as_ref().map(|img| {
            (
                img.width() as u32,
                img.height() as u32,
            )
        })
    }

    pub fn restore_annotations(&self, annotations: Vec<Annotation>) {
        let mut engine = AnnotationEngine::new();
        for ann in annotations {
            engine.add(ann);
        }
        *self.imp().annotations.borrow_mut() = engine;
        self.queue_draw();
    }

    pub fn restore_zoom_pan(&self, zoom: f64, pan_x: f64, pan_y: f64) {
        self.imp().zoom.set(zoom);
        self.imp().pan_offset.set((pan_x, pan_y));
        self.notify_zoom_changed(zoom);
        self.queue_draw();
    }

    pub fn on_zoom_changed(&self, cb: impl Fn(f64) + 'static) {
        self.imp().zoom_changed_cb.replace(Some(Box::new(cb)));
    }

    fn screen_to_image(&self, x: f64, y: f64) -> Point {
        let (pan_x, pan_y) = self.imp().pan_offset.get();
        let zoom = self.imp().zoom.get();
        Point {
            x: (x - pan_x) / zoom,
            y: (y - pan_y) / zoom,
        }
    }

    fn image_to_screen(&self, p: Point) -> (f64, f64) {
        let (pan_x, pan_y) = self.imp().pan_offset.get();
        let zoom = self.imp().zoom.get();
        (p.x * zoom + pan_x, p.y * zoom + pan_y)
    }

    fn handle_at(&self, bounds: &Rect, p: Point) -> Option<HandleIndex> {
        let r = Self::HANDLE_HIT_RADIUS;
        let corners = [
            (HandleIndex::TopLeft, bounds.x, bounds.y),
            (
                HandleIndex::TopRight,
                bounds.x + bounds.width,
                bounds.y,
            ),
            (
                HandleIndex::BottomLeft,
                bounds.x,
                bounds.y + bounds.height,
            ),
            (
                HandleIndex::BottomRight,
                bounds.x + bounds.width,
                bounds.y + bounds.height,
            ),
        ];

        for (handle, hx, hy) in corners {
            let dx = p.x - hx;
            let dy = p.y - hy;
            if (dx * dx + dy * dy).sqrt() <= r {
                return Some(handle);
            }
        }
        None
    }

    fn finish_draw_drag(&self, offset_x: f64, offset_y: f64) {
        let state = self.imp().drawing_state.replace(DrawingState::Idle);
        self.imp().freehand_points.borrow_mut().clear();

        match state {
            DrawingState::Drawing { start, current } => {
                if distance(start, current) < Self::MIN_DRAG_DISTANCE {
                    if matches!(
                        self.imp().active_tool.get(),
                        ActiveTool::Text | ActiveTool::Callout | ActiveTool::Timestamp
                            | ActiveTool::NumberMarker
                    ) {
                        self.handle_click_tool(start);
                    }
                    return;
                }
                self.finalize_new_annotation(start, current);
            }
            DrawingState::Moving {
                id,
                original_bounds,
                ..
            } => {
                let engine = self.imp().annotations.borrow();
                if let Some(ann) = engine.annotation_at(id) {
                    let new_bounds = ann.bounds;
                    drop(engine);
                    if new_bounds != original_bounds {
                        self.imp().history.borrow_mut().push(
                            AnnotationCommand::UpdateBounds {
                                id,
                                old_bounds: original_bounds,
                                new_bounds,
                            },
                        );
                        self.notify_annotation_changed();
                    }
                }
            }
            DrawingState::ResizingHandle {
                id,
                original_bounds,
                ..
            } => {
                let engine = self.imp().annotations.borrow();
                if let Some(ann) = engine.annotation_at(id) {
                    let new_bounds = ann.bounds;
                    drop(engine);
                    if new_bounds != original_bounds {
                        self.imp().history.borrow_mut().push(
                            AnnotationCommand::UpdateBounds {
                                id,
                                old_bounds: original_bounds,
                                new_bounds,
                            },
                        );
                        self.notify_annotation_changed();
                    }
                }
            }
            _ => {
                let _ = (offset_x, offset_y);
            }
        }
    }

    fn handle_click_tool(&self, position: Point) {
        match self.imp().active_tool.get() {
            ActiveTool::Text | ActiveTool::Callout => {
                self.open_text_editor(position, None);
            }
            ActiveTool::Timestamp => {
                let text = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                self.create_timestamp_annotation(position, text);
            }
            ActiveTool::NumberMarker => {
                self.create_number_marker(position);
            }
            _ => {}
        }
    }

    fn finalize_new_annotation(&self, start: Point, current: Point) {
        let tool = self.imp().active_tool.get();
        let style = self.imp().current_style.borrow().clone();

        match tool {
            ActiveTool::Text | ActiveTool::Callout => {
                self.open_text_editor(start, None);
            }
            ActiveTool::Timestamp => {
                let text = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                self.create_timestamp_annotation(start, text);
            }
            ActiveTool::NumberMarker => {
                self.create_number_marker(start);
            }
            _ => {
                if let Some(ann) = self.build_annotation_from_drag(tool, start, current, style) {
                    self.imp().annotations.borrow_mut().add(ann.clone());
                    self.imp()
                        .history
                        .borrow_mut()
                        .push(AnnotationCommand::Add(ann));
                    self.notify_annotation_changed();
                    self.queue_draw();
                }
            }
        }
    }

    fn build_annotation_from_drag(
        &self,
        tool: ActiveTool,
        start: Point,
        current: Point,
        style: AnnotationStyle,
    ) -> Option<Annotation> {
        let id = Uuid::new_v4();
        match tool {
            ActiveTool::Rectangle | ActiveTool::Blur | ActiveTool::Pixelate | ActiveTool::Redaction => {
                let bounds = rect_from_points(start, current);
                let kind = match tool {
                    ActiveTool::Blur => AnnotationKind::Blur,
                    ActiveTool::Pixelate => AnnotationKind::Pixelate,
                    ActiveTool::Redaction => AnnotationKind::Redaction,
                    _ => AnnotationKind::Rectangle,
                };
                Some(Annotation {
                    id,
                    kind,
                    bounds,
                    style,
                })
            }
            ActiveTool::Ellipse => Some(Annotation {
                id,
                kind: AnnotationKind::Ellipse,
                bounds: rect_from_points(start, current),
                style,
            }),
            ActiveTool::Arrow => {
                let data = ArrowData { start, end: current };
                Some(Annotation {
                    id,
                    kind: AnnotationKind::Arrow(data),
                    bounds: bounds_for_arrow(&data),
                    style,
                })
            }
            ActiveTool::Line => {
                let data = ArrowData { start, end: current };
                Some(Annotation {
                    id,
                    kind: AnnotationKind::Line(data),
                    bounds: bounds_for_arrow(&data),
                    style,
                })
            }
            ActiveTool::Freehand => {
                let mut points = self.imp().freehand_points.borrow().clone();
                if points.len() < 2 {
                    points = vec![start, current];
                }
                let data = FreehandData { points };
                Some(Annotation {
                    id,
                    kind: AnnotationKind::Freehand(data.clone()),
                    bounds: bounds_for_freehand(&data),
                    style,
                })
            }
            _ => None,
        }
    }

    fn build_preview_annotation(&self) -> Option<Annotation> {
        let state = self.imp().drawing_state.borrow();
        let tool = self.imp().active_tool.get();
        let style = self.imp().current_style.borrow().clone();

        match &*state {
            DrawingState::Drawing { start, current } => {
                self.build_annotation_from_drag(tool, *start, *current, style)
            }
            _ => None,
        }
    }

    fn confirm_text_editor(
        &self,
        text: String,
        position: Point,
        existing_id: Option<Uuid>,
        tool: ActiveTool,
        style: AnnotationStyle,
    ) {
        if let Some(id) = existing_id {
            let old_text = self
                .imp()
                .annotations
                .borrow()
                .annotation_at(id)
                .and_then(|ann| match &ann.kind {
                    AnnotationKind::Text(data) => Some(data.text.clone()),
                    AnnotationKind::Callout(data) => Some(data.text.clone()),
                    _ => None,
                });

            if let Some(old) = old_text {
                if old != text {
                    let new_text = text.clone();
                    self.imp().annotations.borrow_mut().update_text(id, text);
                    self.imp().history.borrow_mut().push(AnnotationCommand::UpdateText {
                        id,
                        old_text: old,
                        new_text,
                    });
                    self.notify_annotation_changed();
                    self.queue_draw();
                }
            }
            return;
        }

        let bounds = Rect {
            x: position.x,
            y: position.y,
            width: 120.0,
            height: 32.0,
        };

        let kind = if tool == ActiveTool::Callout {
            AnnotationKind::Callout(CalloutData {
                text: text.clone(),
                anchor: Point {
                    x: position.x,
                    y: position.y + bounds.height + 20.0,
                },
            })
        } else {
            AnnotationKind::Text(TextData {
                text: text.clone(),
                font_size: 16.0,
            })
        };

        let ann = Annotation {
            id: Uuid::new_v4(),
            kind,
            bounds,
            style,
        };
        self.imp().annotations.borrow_mut().add(ann.clone());
        self.imp()
            .history
            .borrow_mut()
            .push(AnnotationCommand::Add(ann));
        self.notify_annotation_changed();
        self.queue_draw();
    }

    fn create_timestamp_annotation(&self, position: Point, text: String) {
        let style = self.imp().current_style.borrow().clone();
        let ann = Annotation {
            id: Uuid::new_v4(),
            kind: AnnotationKind::Timestamp(TextData {
                text,
                font_size: 16.0,
            }),
            bounds: Rect {
                x: position.x,
                y: position.y,
                width: 160.0,
                height: 24.0,
            },
            style,
        };
        self.imp().annotations.borrow_mut().add(ann.clone());
        self.imp()
            .history
            .borrow_mut()
            .push(AnnotationCommand::Add(ann));
        self.notify_annotation_changed();
        self.queue_draw();
    }

    fn create_number_marker(&self, position: Point) {
        let style = self.imp().current_style.borrow().clone();
        let number = self.imp().annotations.borrow_mut().next_number();
        let size = Self::NUMBER_MARKER_SIZE;
        let ann = Annotation {
            id: Uuid::new_v4(),
            kind: AnnotationKind::NumberMarker(NumberMarkerData { number }),
            bounds: Rect {
                x: position.x - size / 2.0,
                y: position.y - size / 2.0,
                width: size,
                height: size,
            },
            style,
        };
        self.imp().annotations.borrow_mut().add(ann.clone());
        self.imp()
            .history
            .borrow_mut()
            .push(AnnotationCommand::Add(ann));
        self.notify_annotation_changed();
        self.queue_draw();
    }

    fn apply_zoom(&self, raw_zoom: f64, anchor: Option<(f64, f64)>) {
        let old_zoom = self.imp().zoom.get();
        let new_zoom = raw_zoom.clamp(Self::ZOOM_MIN, Self::ZOOM_MAX);
        if (new_zoom - old_zoom).abs() < f64::EPSILON {
            return;
        }

        let (old_pan_x, old_pan_y) = self.imp().pan_offset.get();
        let (ax, ay) = anchor.unwrap_or((self.width() as f64 / 2.0, self.height() as f64 / 2.0));

        let new_pan_x = ax - (ax - old_pan_x) * (new_zoom / old_zoom);
        let new_pan_y = ay - (ay - old_pan_y) * (new_zoom / old_zoom);

        self.imp().zoom.set(new_zoom);
        self.imp().pan_offset.set((new_pan_x, new_pan_y));
        self.notify_zoom_changed(new_zoom);
        self.queue_draw();
    }

    fn image_size(&self) -> (f64, f64) {
        self.imp()
            .image
            .borrow()
            .as_ref()
            .map(|img| (img.width() as f64, img.height() as f64))
            .unwrap_or((0.0, 0.0))
    }

    fn notify_zoom_changed(&self, zoom: f64) {
        if let Some(cb) = self.imp().zoom_changed_cb.borrow().as_ref() {
            cb(zoom);
        }
    }

    fn notify_annotation_changed(&self) {
        if let Some(cb) = self.imp().annotation_changed_cb.borrow().as_ref() {
            cb();
        }
    }
}

fn distance(a: Point, b: Point) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

fn rect_from_points(start: Point, current: Point) -> Rect {
    let x = start.x.min(current.x);
    let y = start.y.min(current.y);
    Rect {
        x,
        y,
        width: (start.x - current.x).abs(),
        height: (start.y - current.y).abs(),
    }
}

fn bounds_for_arrow(data: &ArrowData) -> Rect {
    let x = data.start.x.min(data.end.x);
    let y = data.start.y.min(data.end.y);
    Rect {
        x,
        y,
        width: (data.start.x - data.end.x).abs().max(1.0),
        height: (data.start.y - data.end.y).abs().max(1.0),
    }
}

fn bounds_for_freehand(data: &FreehandData) -> Rect {
    if data.points.is_empty() {
        return Rect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        };
    }
    let mut min_x = data.points[0].x;
    let mut min_y = data.points[0].y;
    let mut max_x = data.points[0].x;
    let mut max_y = data.points[0].y;
    for p in &data.points[1..] {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }
    Rect {
        x: min_x,
        y: min_y,
        width: (max_x - min_x).max(1.0),
        height: (max_y - min_y).max(1.0),
    }
}

fn resize_bounds(original: Rect, handle: HandleIndex, dx: f64, dy: f64) -> Rect {
    let mut x = original.x;
    let mut y = original.y;
    let mut w = original.width;
    let mut h = original.height;

    match handle {
        HandleIndex::TopLeft => {
            x += dx;
            y += dy;
            w -= dx;
            h -= dy;
        }
        HandleIndex::TopRight => {
            y += dy;
            w += dx;
            h -= dy;
        }
        HandleIndex::BottomLeft => {
            x += dx;
            w -= dx;
            h += dy;
        }
        HandleIndex::BottomRight => {
            w += dx;
            h += dy;
        }
    }

    if w < Canvas::MIN_BOUNDS_SIZE {
        if matches!(handle, HandleIndex::TopLeft | HandleIndex::BottomLeft) {
            x = original.x + original.width - Canvas::MIN_BOUNDS_SIZE;
        }
        w = Canvas::MIN_BOUNDS_SIZE;
    }
    if h < Canvas::MIN_BOUNDS_SIZE {
        if matches!(handle, HandleIndex::TopLeft | HandleIndex::TopRight) {
            y = original.y + original.height - Canvas::MIN_BOUNDS_SIZE;
        }
        h = Canvas::MIN_BOUNDS_SIZE;
    }

    Rect {
        x,
        y,
        width: w,
        height: h,
    }
}
