#!/bin/bash

modules=("filebrowser" "todo" "note" "diary" "terminal" "git" "music" "project" "mail")

for module in "${modules[@]}"; do
  # Create Cargo.toml
  cat > "crates/modules/$module/Cargo.toml" << TOML
[package]
name = "$module"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
core = { path = "../../core" }
storage = { path = "../../storage" }
serde = { workspace = true }
anyhow = { workspace = true }
uuid = { workspace = true }
TOML

  # Create lib.rs
  cat > "crates/modules/$module/src/lib.rs" << RUST
// $module 模块
// TODO: 实现 Module trait

use crate::core::module::Module;
use ratatui::{layout::Rect, Frame};
use crossterm::event::Event as CrosstermEvent;
use crate::core::event::Action;

pub struct ${module^};

impl ${module^} {
    pub fn new() -> Self {
        Self
    }
}

impl Module for ${module^} {
    fn name(&self) -> &str {
        "$module"
    }

    fn title(&self) -> &str {
        "${module^}"
    }

    fn update(&mut self, _event: CrosstermEvent) -> Action {
        Action::None
    }

    fn draw(&mut self, _frame: &mut Frame, _area: Rect) {}

    fn save(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn load(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn shortcuts(&self) -> Vec<Shortcut> {
        vec![]
    }
}
RUST

  mkdir -p "crates/modules/$module/src"

done

echo "Module stubs created successfully!"
