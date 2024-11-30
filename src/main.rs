use anyhow::Result;
use colored::*;
mod claude;
mod cortex;
use crate::cortex::Cortex;

const BANNER_TEXT: &str = r#"
   ▄████▄   ▒█████   ██▀███  ▄▄▄█████▓▓█████ ▒██   ██▒
  ▒██▀ ▀█  ▒██▒  ██▒▓██ ▒ ██▒▓  ██▒ ▓▒▓█   ▀ ▒▒ █ █ ▒░
  ▒▓█    ▄ ▒██░  ██▒▓██ ░▄█ ▒▒ ▓██░ ▒░▒███   ░░  █   ░
  ▒▓▓▄ ▄██▒▒██   ██░▒██▀▀█▄  ░ ▓██▓ ░ ▒▓█  ▄  ░ █ █ ▒
  ▒ ▓███▀ ░░ ████▓▒░░██▓ ▒██▒  ▒██▒ ░ ░▒████▒▒██▒ ▒██▒
  ░ ░▒ ▒  ░░ ▒░▒░▒░ ░ ▒▓ ░▒▓░  ▒ ░░   ░░ ▒░ ░▒▒ ░ ░▓ ░
    ░  ▒     ░ ▒ ▒░   ░▒ ░ ▒░    ░     ░ ░  ░░░   ░▒ ░
  ░        ░ ░ ░ ▒    ░░   ░   ░         ░    ░    ░
  ░ ░          ░ ░     ░                 ░  ░ ░    ░
  ░                                                    "#;

pub fn get_colored_banner() -> String {
    let mut colored_lines = Vec::new();
    
    // Neon cyberpunk colors with deep purple, electric blue, and bright orange
    for (i, line) in BANNER_TEXT.lines().enumerate() {
        let colored_line = match i % 3 {
            0 => line.truecolor(255, 128, 0).on_black(),     // Neon orange
            1 => line.truecolor(80, 200, 255).on_black(),    // Electric blue
            _ => line.truecolor(190, 0, 255).on_black(),     // Deep purple
        };
        colored_lines.push(colored_line.to_string());
    }
    
    format!("{}", colored_lines.join("\n").on_black())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let mut cortex = Cortex::new()?;
    cortex.run().await?;
    Ok(())
}
