#![cfg(feature = "ssr")]

use rig::client::CompletionClient;
use rig::{completion::Prompt, providers::anthropic, client::ProviderClient};

use crate::languages::{Database, NewSentence};

static GEN_NEW_WORDS_PROMPT: &str = "
You are bilingual {lang} and English speaker.
Help me to gradually learn the {lang} language. Here is the words I already know in {lang}:

<dictionary>
{dict}
</dictionarty>

Generate a sentence that uses the words from my dictionary but introduces one new word. 
This new word cannot be an article like 'el', 'la', 'los', 'las', 'un', 'una', etc.
Keep the sentence short. 
Output in this format:

<new_sentence>
...
</new_sentence>
<new_word>
manana
</new_word>
<translation>
tomorrow
</translation>

Use only ASCII characters.
Don't start with 'Hola'.
";


pub async fn gen_new_sentence(lang: &str) -> Result<NewSentence, anyhow::Error> {
    let words = {
        let words_db = Database::get_instance(lang)?.lock().unwrap();
        words_db.all_words()?
    };
    let prompt = GEN_NEW_WORDS_PROMPT.replace("{lang}", lang)
        .replace("{dict}", &words.iter().map(|word| format!("{}", word.word)).collect::<Vec<String>>().join("\n"));

    let anthropic = anthropic::Client::from_env();
    let agent = anthropic.agent(anthropic::CLAUDE_3_7_SONNET).max_tokens(1000).build();
    let response = agent.prompt(&prompt).await?;

    let text = parse_xml_tag(&response, "new_sentence").unwrap();
    let new_word = parse_xml_tag(&response, "new_word").unwrap();
    let translation = parse_xml_tag(&response, "translation").unwrap();
    Ok(NewSentence { text, new_word, translation })
}

// Minimal XML parser for non-perfect LLM output.
fn parse_xml_tag(text: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{}>", tag);
    let end_tag = format!("</{}>", tag);
    
    if let Some(start_idx) = text.find(&start_tag) {
        if let Some(end_idx) = text.find(&end_tag) {
            let start_content = start_idx + start_tag.len();
            let content = &text[start_content..end_idx];
            return Some(content.trim().to_string());
        }
    }
    None
}
