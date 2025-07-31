use leptos::prelude::*;

use crate::components::markdown::Markdown;

/// Rich text is composed of a series of text blocks.
/// Each can be rendered using a different technique.
#[derive(Debug, Clone, PartialEq)]
pub enum TextBlock {
    /// Just a string.
    Raw(String),
    AsciiMath(String),
}

impl TextBlock {
    pub fn is_empty(&self) -> bool {
        match self {
            TextBlock::Raw(text) | TextBlock::AsciiMath(text) => text.is_empty(),
        }
    }
}

/// Render an [ASCIIMath](https://asciimath.org/) expression as MathML.
#[component]
pub fn AsciiMath(input: String) -> impl IntoView {
    let ascii_math = mathemascii::parse(&input);
    let math_ml = mathemascii::render_mathml(ascii_math);
    view! { <span inner_html=math_ml></span> }
}

/// Renders AsciiMath and Markdown.
#[component]
pub fn RichText(#[prop(into)] text: String) -> impl IntoView {
    parse_rich_text(&text)
        .into_iter()
        .map(|block| match block {
            TextBlock::Raw(text) => {
                view! { <Markdown text=Memo::new(move |_| text.clone()) /> }.into_any()
            }
            TextBlock::AsciiMath(text) => view! { <AsciiMath input=text /> }.into_any(),
        })
        .collect_view()
}

#[derive(Debug)]
enum ParseState {
    RawStr(usize, usize),
    InAsciiMath(usize, usize),
}

impl ParseState {
    fn text_block(&self, text: &str) -> TextBlock {
        match self {
            ParseState::RawStr(start, end) => {
                TextBlock::Raw(text.chars().skip(*start).take(*end - *start).collect())
            }
            ParseState::InAsciiMath(start, end) => {
                TextBlock::AsciiMath(text.chars().skip(*start).take(*end - *start).collect())
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum Parsed {
    Yes(usize),
    No(usize),
}

/// A minimalistic parser for rich text:
/// ```
/// * Markdown
/// * AsciiMath: `math 1/3`
/// ```
fn parse_rich_text(text: &str) -> Vec<TextBlock> {
    let mut text_blocks = Vec::new();
    let mut state = ParseState::RawStr(0, 0);

    let mut char_iter = text.chars().enumerate().peekable();
    while let Some((i, c)) = char_iter.peek().cloned() {
        match state {
            ParseState::RawStr(start, end) => match match_str("`math ", &mut char_iter) {
                Parsed::Yes(parsed_chars) => {
                    text_blocks.push(state.text_block(text));
                    state = ParseState::InAsciiMath(end + parsed_chars, end + parsed_chars);
                }
                Parsed::No(parsed_char) => {
                    state = ParseState::RawStr(start, end + parsed_char);
                }
            },
            ParseState::InAsciiMath(start, _) => {
                // Consume the peeked character.
                char_iter.next();
                if c == '`' {
                    text_blocks.push(state.text_block(text));
                    state = ParseState::RawStr(i + 1, i + 1);
                } else {
                    state = ParseState::InAsciiMath(start, i + 1);
                }
            }
        }
    }

    let last_block = state.text_block(text);
    if !last_block.is_empty() {
        text_blocks.push(last_block);
    }

    text_blocks
}

fn match_str(s: &str, char_iter: &mut impl Iterator<Item = (usize, char)>) -> Parsed {
    let mut match_chars = s.chars().enumerate();
    while let Some((i, c)) = match_chars.next() {
        if let Some((_, next_char)) = char_iter.next() {
            if c != next_char {
                return Parsed::No(i + 1);
            }
        } else {
            return Parsed::No(i + 1);
        }
    }
    Parsed::Yes(s.len())
}

#[cfg(test)]
mod tests {
    use super::{match_str, parse_rich_text, Parsed, TextBlock};

    #[test]
    fn test_match_str() {
        assert_eq!(
            match_str("`math", &mut "What is".chars().enumerate()),
            Parsed::No(1)
        );
        assert_eq!(
            match_str("`math", &mut "`math 0.1`".chars().enumerate()),
            Parsed::Yes(5)
        );
    }

    #[test]
    fn test_parse_rich_text() {
        let blocks = parse_rich_text("What is this notation: `math 0.bar(3)` ?");
        assert_eq!(blocks.len(), 3);
        assert_eq!(
            blocks[0],
            TextBlock::Raw("What is this notation: ".to_string())
        );
        assert_eq!(blocks[1], TextBlock::AsciiMath("0.bar(3)".to_string()));
        assert_eq!(blocks[2], TextBlock::Raw(" ?".to_string()));
    }
}
