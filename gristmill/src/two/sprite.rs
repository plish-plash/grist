use serde::Deserialize;
use std::{collections::HashMap, time::Duration};

use crate::{asset, math::Vec2, two::Rect, RenderingContext, Texture};

#[derive(Deserialize)]
struct SpriteSheetDefinition {
    fps: f32,
    frames: HashMap<String, Vec<Rect>>,
}

impl Default for SpriteSheetDefinition {
    fn default() -> Self {
        Self {
            fps: 24.,
            frames: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct SpriteSheet {
    texture: Texture,
    texture_size: Vec2,
    frames: HashMap<String, Vec<Rect>>,
    current_frame: Rect,
    current_animation: String,
    current_animation_frame: usize,
    frame_duration: f32,
    frame_time: f32,
}

impl SpriteSheet {
    pub fn load(context: &mut RenderingContext, file: &str) -> asset::Result<Self> {
        let image_file = format!("{}.png", file);
        let definition_file = format!("{}.yaml", file);
        let texture = asset::load_png_file(context, "images", &image_file)?;
        let texture_size = context.texture_size(texture.id());
        let frames: SpriteSheetDefinition = asset::load_yaml_file("images", &definition_file)?;
        Ok(SpriteSheet {
            texture,
            texture_size: Vec2::new(texture_size.0 as f32, texture_size.1 as f32),
            frames: frames.frames,
            current_frame: Rect::ZERO,
            current_animation: String::new(),
            current_animation_frame: 0,
            frame_duration: 1. / frames.fps,
            frame_time: 0.,
        })
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }
    pub fn current_frame(&self) -> Rect {
        self.current_frame
    }
    pub fn uv_rect(&self) -> Rect {
        let mut rect = self.current_frame;
        rect.position /= self.texture_size;
        rect.size /= self.texture_size;
        rect
    }
    pub fn set_animation(&mut self, animation: &str) {
        self.current_animation = animation.to_string();
        self.frame_time = 0.0;
        self.set_animation_frame(0);
    }
    pub fn set_animation_frame(&mut self, frame: usize) {
        if let Some(frames) = self.frames.get(&self.current_animation) {
            self.current_animation_frame = frame % frames.len();
            self.current_frame = frames[self.current_animation_frame];
        } else {
            if self.current_animation.is_empty() {
                eprintln!("Animation not set");
            } else {
                eprintln!("No animation called {}", self.current_animation);
            }
        }
    }
    pub fn animate(&mut self, frame_time: Duration) {
        self.frame_time += frame_time.as_secs_f32();
        if self.frame_time >= self.frame_duration {
            self.frame_time -= self.frame_duration;
            self.set_animation_frame(self.current_animation_frame + 1);
        }
    }
}
