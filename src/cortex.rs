use anyhow::Result;
use crate::claude::ClaudeClient;
use rustyline::DefaultEditor;
use colored::*;
use serde_json::Value;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

pub struct Cortex {
    claude_client: ClaudeClient,
    conversation_history: Vec<Value>,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl Cortex {
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY must be set");

        Ok(Self {
            claude_client: ClaudeClient::new(api_key)?,
            conversation_history: Vec::new(),
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        })
    }

    async fn send_message(&mut self, input: &str) -> Result<String> {
        // Add user message to history
        self.conversation_history.push(serde_json::json!({
            "role": "user",
            "content": input
        }));

        // Send message with history
        let response = self.claude_client.send_message_with_history(input, &self.conversation_history).await?;

        // Add assistant response to history
        self.conversation_history.push(serde_json::json!({
            "role": "assistant",
            "content": response.clone()
        }));

        Ok(response)
    }

    fn process_code_blocks(&self, text: &str) -> String {
        let mut processed = String::new();
        let mut in_code_block = false;
        let mut current_lang = String::new();
        let mut current_block = String::new();

        for line in text.lines() {
            if line.starts_with("```") {
                if in_code_block {
                    // End of code block - syntax highlight it
                    let syntax = self.syntax_set.find_syntax_by_token(&current_lang)
                        .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
                    let mut h = HighlightLines::new(syntax, &self.theme_set.themes["base16-ocean.dark"]);

                    processed.push('\n');

                    for line in LinesWithEndings::from(&current_block) {
                        let ranges: Vec<(_, &str)> = h.highlight_line(line, &self.syntax_set).unwrap();
                        let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                        processed.push_str(&escaped);
                    }

                    processed.push('\n');

                    in_code_block = false;
                    current_block.clear();
                } else {
                    // Start of code block
                    current_lang = line.trim_start_matches('`').to_string();
                    in_code_block = true;
                }
            } else if in_code_block {
                current_block.push_str(line);
                current_block.push('\n');
            } else {
                processed.push_str(line);
                processed.push('\n');
            }
        }

        processed
    }

    fn process_text(&self, text: &str) -> String {
        let mut result = String::new();
        let mut lines = text.lines().peekable();

        while let Some(line) = lines.next() {
            let processed_line = if line.starts_with('#') {
                // Handle headers
                let hash_count = line.chars().take_while(|&c| c == '#').count();
                let header_text = line[hash_count..].trim();
                format!("{}\n", header_text.cyan().bold())
            } else if line.starts_with('*') && line.ends_with('*') {
                // Handle emphasis
                let text = line.trim_matches('*');
                format!("{}\n", text.bright_white().italic())
            } else if line.starts_with("- ") || line.starts_with("* ") {
                // Handle list items
                format!("  • {}\n", &line[2..])
            } else if line.contains('`') {
                // Handle inline code
                let parts: Vec<&str> = line.split('`').collect();
                let mut formatted = String::new();
                for (i, part) in parts.iter().enumerate() {
                    if i % 2 == 1 {
                        formatted.push_str(&part.yellow().to_string());
                    } else {
                        formatted.push_str(part);
                    }
                }
                format!("{}\n", formatted)
            } else {
                format!("{}\n", line)
            };

            result.push_str(&processed_line);
        }

        result
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("{}", crate::BANNER_TEXT.truecolor(80, 200, 255));
        println!("\n{}", "[ CORTEX NEURAL INTERFACE INITIALIZED ]".truecolor(80, 200, 255).on_black());

        let mut rl = DefaultEditor::new()?;

        loop {
            let readline = rl.readline(&format!("\n{}", "CORTEX://> ".truecolor(255, 128, 0).on_black()));

            match readline {
                Ok(line) => {
                    let input = line.trim();
                    if input.eq_ignore_ascii_case("exit") {
                        println!("{}", "[ NEURAL LINK TERMINATED ]".truecolor(190, 0, 255).on_black());
                        break;
                    }

                    rl.add_history_entry(input)?;

                    match self.send_message(input).await {
                        Ok(response) => {
                            let with_code_blocks = self.process_code_blocks(&response);
                            let formatted = self.process_text(&with_code_blocks);
                            println!("\n{}", formatted.bright_white().on_black());
                        },
                        Err(e) => eprintln!("{} {}",
                            "[ NEURAL LINK ERROR ]".truecolor(255, 128, 0).on_black(),
                            e.to_string().bright_red().on_black()
                        ),
                    }
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    println!("{}", "[ NEURAL LINK INTERRUPTED ]".truecolor(190, 0, 255).on_black());
                    break;
                }
                Err(rustyline::error::ReadlineError::Eof) => {
                    println!("{}", "[ NEURAL LINK TERMINATED ]".truecolor(190, 0, 255).on_black());
                    break;
                }
                Err(err) => {
                    eprintln!("{} {}",
                        "[ FATAL ERROR ]".truecolor(255, 128, 0).on_black(),
                        err.to_string().bright_red().on_black()
                    );
                    break;
                }
            }
        }

        Ok(())
    }
}
