// src/claude/text_formatter.rs

use colored::*;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

#[derive(Debug)]
pub struct TextFormatter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl TextFormatter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    pub fn format_text(&self, text: &str) -> String {
        let mut result = String::new();
        let mut in_code_block = false;
        let mut in_table = false;
        let mut code_buffer = String::new();
        let mut language = String::new();

        for line in text.lines() {
            if line.starts_with("```") {
                if in_code_block {
                    if !code_buffer.is_empty() {
                        result.push_str(&self.highlight_code(&code_buffer, &language));
                    }
                    code_buffer.clear();
                    in_code_block = false;
                } else {
                    language = line[3..].trim().to_string();
                    in_code_block = true;
                }
                continue;
            }

            if in_code_block {
                code_buffer.push_str(line);
                code_buffer.push('\n');
                continue;
            }

            let processed_line = self.process_line(line, &mut in_table);
            result.push_str(&processed_line);
        }

        if in_table {
            result.push_str(&("└".to_string() + &"─".repeat(40) + "┘\n"));
        }

        result
    }

    fn highlight_code(&self, code: &str, language: &str) -> String {
        let syntax = self.syntax_set.find_syntax_by_token(language)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let mut h = HighlightLines::new(syntax, &self.theme_set.themes["base16-ocean.dark"]);
        
        let mut result = String::from("\n");
        for line in LinesWithEndings::from(code) {
            let ranges: Vec<(_, &str)> = h.highlight_line(line, &self.syntax_set).unwrap();
            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
            result.push_str(&escaped);
        }
        result.push('\n');
        result
    }

    fn process_line(&self, line: &str, in_table: &mut bool) -> String {
        if line.starts_with('#') {
            self.format_header(line)
        } else if line.starts_with("> ") {
            self.format_blockquote(line)
        } else if line.starts_with("| ") && line.ends_with(" |") {
            self.format_table_row(line, in_table)
        } else if line.is_empty() {
            self.handle_empty_line(in_table)
        } else {
            self.format_regular_line(line)
        }
    }

    fn format_header(&self, line: &str) -> String {
        let level = line.chars().take_while(|&c| c == '#').count();
        let text = line[level..].trim();
        format!("{}\n", text.cyan().bold())
    }

    fn format_blockquote(&self, line: &str) -> String {
        format!("│ {}\n", line[2..].bright_blue().italic())
    }

    fn format_table_row(&self, line: &str, in_table: &mut bool) -> String {
        let cells: Vec<&str> = line.split('|')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        
        let formatted = if line.contains("---") {
            "├".to_string() + &"─".repeat(40) + "┤\n"
        } else {
            let row = cells.iter()
                .map(|&cell| format!("{:^15}", cell))
                .collect::<Vec<_>>()
                .join(" │ ");
            format!("│ {} │\n", row)
        };
        
        if !*in_table {
            *in_table = true;
            "┌".to_string() + &"─".repeat(40) + "┐\n" + &formatted
        } else {
            formatted
        }
    }

    fn handle_empty_line(&self, in_table: &mut bool) -> String {
        if *in_table {
            *in_table = false;
            "└".to_string() + &"─".repeat(40) + "┘\n"
        } else {
            "\n".to_string()
        }
    }

    fn format_regular_line(&self, line: &str) -> String {
        let mut formatted = line.to_string();
        
        // Handle bold
        while let Some(start) = formatted.find("**") {
            if let Some(end) = formatted[start + 2..].find("**") {
                let bold_text = formatted[start + 2..start + 2 + end].bright_white().bold();
                formatted = formatted[..start].to_string() + &bold_text.to_string() + &formatted[start + 2 + end + 2..];
            } else {
                break;
            }
        }
        
        // Handle italic
        while let Some(start) = formatted.find('*') {
            if let Some(end) = formatted[start + 1..].find('*') {
                let italic_text = formatted[start + 1..start + 1 + end].italic();
                formatted = formatted[..start].to_string() + &italic_text.to_string() + &formatted[start + 1 + end + 1..];
            } else {
                break;
            }
        }

        // Handle inline code
        while let Some(start) = formatted.find('`') {
            if let Some(end) = formatted[start + 1..].find('`') {
                let code_text = formatted[start + 1..start + 1 + end].yellow();
                formatted = formatted[..start].to_string() + &code_text.to_string() + &formatted[start + 1 + end + 1..];
            } else {
                break;
            }
        }

        format!("{}\n", formatted)
    }
}
