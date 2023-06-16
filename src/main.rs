use atlas::Sprite;
use ggez::nalgebra::{Point2, Vector2};
use ggez::{
    event::EventHandler,
    graphics::{spritebatch::SpriteBatch, Text},
    Context,
    GameResult,
    graphics,
    event
};
mod entity;         //referencing all our other files
mod atlas;
mod pipe;
mod game_state;
mod bird;
mod audio;
mod window;
mod tile;
use entity::PlayState;
use std::time::Duration;
use crate::bird::PlayerEntity;
use crate::game_state::GameState;

pub const NUMBER_OF_TILES: u8 = 14;
pub const RESTART_AFTER: Duration = std::time::Duration::from_secs(1);


impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let state = self.play_state.clone();
        self.handle_after_losing(ctx, state);
        let state = self.player.update(ctx, &self.play_state);
        if !self.play_state.is_playing() && state == PlayState::Play {
            self.play_state = PlayState::Play;
        }
        for i in 0..self.pipes.len() {
            self.pipes[i].update(&mut self.pipe_tracker, &self.play_state);
        }
        update_it(self, ctx);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::Color::from_rgb(46, 68, 130));       //background sky

        self.player.draw(&mut self.sprite_batch)?;
        if !self.tiles_drawn {
            for i in 0..self.tiles.len() {  //draw all tiles related to self (in this case player)
                self.tiles[i].draw(&mut self.sprite_batch);
            }
            self.tiles_drawn = false;
        }
        for i in 0..self.pipes.len() {      //draw pipes
            self.pipes[i].draw(ctx, &mut self.sprite_batch)?;

        }

        let p = graphics::DrawParam::new().scale(Vector2::new(4.0, 4.0));   //scaling all sprites to 4x the size
        {
            graphics::draw(ctx, &mut self.sprite_batch, p)?;
            self.sprite_batch.clear();
        }

        draw_scores(self.score, self.best_score, ctx);      //draw score counter

        graphics::present(ctx)?;
        std::thread::yield_now();

        Ok(())
    }
}

fn update_it(game: &mut GameState, ctx: &mut Context) {
    let player = &game.player;
    let pipes = &mut game.pipes;
    for i in 0..pipes.len() {
        if pipes[i].set_scored(&game.play_state) {      //score increment
            game.score += 1;
            println!("{}", i);
            game.sound_player.score();
        }
        // if bird hits a pipe or hits ground
        if (player.overlaps(&pipes[i]) || hits_ground(player)) && game.play_state.is_playing() {
            game.sound_player.ouch();
            game.play_state.set_dead(ggez::timer::time_since_start(ctx));
        }
    }
}

fn hits_ground(player: &Box<PlayerEntity>) -> bool {
    player.position.y > 135.0
}

fn main() {
    let resource_dir = std::path::PathBuf::from("./resources");

    let cb = window::build_window(resource_dir);    //create window

    let (ctx, event_loop) = &mut cb.build().expect("Failed to build ggez!");

    let batch = create_batch_sprite(ctx);           //create sprites

    let mut state = GameState::new(ctx, batch);     //initialize gamestate

    state.sound_player.begin();
    event::run(ctx, event_loop, &mut state).unwrap();
}

fn create_batch_sprite(ctx: &mut Context) -> SpriteBatch {
    let image = graphics::Image::new(ctx, "/texture_atlas.png").unwrap();
    let mut batch = graphics::spritebatch::SpriteBatch::new(image);
    batch.set_filter(graphics::FilterMode::Nearest);
    batch
}

impl PlayState {    
    fn is_playing(&self) -> bool {
        *self == PlayState::Play
    }

    fn set_dead (&mut self, time : std::time::Duration) {
        *self = PlayState::Dead {
            time
        }
    }

    fn is_not_dead (&self) -> bool {
        *self == PlayState::Play || *self == PlayState::StartScreen
    }
}

fn draw_scores(score : i128, best_score: i128, ctx: &mut Context) {
    let fps_display = Text::new(format!(
        "Best Score: {}   Current Score: {}",
        best_score, score
    ));

    graphics::draw(
        ctx,
        &fps_display,
        (Point2::new(10.0, 10.0), graphics::WHITE),
    );
}
