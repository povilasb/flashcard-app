//! Usage:
//! cargo run --bin=lang --features=ssr
 
use rig::client::CompletionClient;
use rig::{completion::Prompt, providers::anthropic, client::ProviderClient};

use flashcard_app::languages::Database;
use flashcard_app::db::Database as FlashcardsDb;
use anyhow;
use dotenv::dotenv;


static EXTRACT_WORDS_PROMPT: &str = "
Here is the {lang} sentences I already know.

<sentences>
{sentences}
</sentences>

List all the words in there. Omit articles like 'a', 'the', etc.
Omit places, names, etc.
Use such format:

<words>
gustaria
cocina
</words>

Place each word on a new line.
";

static GEN_NEW_WORDS_PROMPT: &str = "
You are bilingual {lang} and English speaker.
Help me to gradually learn the {lang} language. Here is the words I already know in {lang}:

<dictionary>
{dict}
</dictionarty>

Generate a sentence that uses the words from my dictionary but introduces one new word. Keep the sentence short. 
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


#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Load environment variables from .env file
    dotenv().ok();
    //populate_words_db("spanish").await?;
    let sentence = gen_new_words("spanish").await?;
    println!("{}", sentence);
    Ok(())
}

async fn gen_new_words(lang: &str) -> Result<String, anyhow::Error> {
    let words_db = Database::get_instance(lang)?.lock().unwrap();
    let words = words_db.all_words()?;
    let prompt = GEN_NEW_WORDS_PROMPT.replace("{lang}", lang)
        .replace("{dict}", &words.iter().map(|word| format!("{}", word.word)).collect::<Vec<String>>().join("\n"));

    let anthropic = anthropic::Client::from_env();
    let agent = anthropic.agent(anthropic::CLAUDE_3_7_SONNET).max_tokens(1000).build();
    let response = agent.prompt(&prompt).await?;
    Ok(response)
}

// From flashcards...
async fn populate_words_db(lang: &str) -> Result<(), anyhow::Error> {
    let sentences = get_all_sentences(lang)?;
    let prompt = EXTRACT_WORDS_PROMPT.replace("{lang}", lang).replace("{sentences}", &sentences);

    let anthropic = anthropic::Client::from_env();
    let agent = anthropic.agent(anthropic::CLAUDE_3_7_SONNET).max_tokens(1000).build();
    let response = agent.prompt(&prompt).await?;
    let words = llm_resp_parse_words(&response);

    let words_db = Database::get_instance(lang)?.lock().unwrap();
    for word in words {
        words_db.add_word(&word)?;
    }

    Ok(())
}

fn get_all_sentences(lang: &str) -> Result<String, anyhow::Error> {
    let flashcards_db = FlashcardsDb::get_instance()?.lock().unwrap();
    let cards = flashcards_db.all_cards(Some(lang.to_string()))?;
    Ok(cards.iter().map(|card| card.answer.clone()).collect::<Vec<String>>().join("\n"))
}

// Parse words from LLM response.
// e.g.
// <words>
// gustaria
// cocina
// </words>
//
// Returns:
// ["gustaria", "cocina"]
fn llm_resp_parse_words(response: &str) -> Vec<String> {
    let start_tag = "<words>";
    let end_tag = "</words>";
    
    if let Some(start_idx) = response.find(start_tag) {
        if let Some(end_idx) = response.find(end_tag) {
            let start_content = start_idx + start_tag.len();
            let words_content = &response[start_content..end_idx];
            
            // Split by newlines and filter out empty lines
            let words = words_content
                .split('\n')
                .map(|word| word.trim())
                .filter(|word| !word.is_empty())
                .map(|word| word.to_string())
                .collect();
            
            return words;
        }
    }
    
    // Return empty vector if tags are not found
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_resp_parse_words() {
        // Test case 1: Normal response with words
        let response1 = "Here is the response:\n<words>\ngustaria\ncocina\n</words>\nThank you!";
        let result1 = llm_resp_parse_words(response1);
        assert_eq!(result1, vec!["gustaria", "cocina"]);

        // Test case 2: Response with extra whitespace
        let response2 = "  <words>  \n  gustaria  \n  cocina  \n  </words>  ";
        let result2 = llm_resp_parse_words(response2);
        assert_eq!(result2, vec!["gustaria", "cocina"]);

        // Test case 3: Response with empty lines
        let response3 = "<words>\ngustaria\n\ncocina\n\n</words>";
        let result3 = llm_resp_parse_words(response3);
        assert_eq!(result3, vec!["gustaria", "cocina"]);

        // Test case 4: Response without tags
        let response4 = "This response has no tags";
        let result4 = llm_resp_parse_words(response4);
        assert_eq!(result4, Vec::<String>::new());

        // Test case 5: Response with only opening tag
        let response5 = "<words>\ngustaria\ncocina";
        let result5 = llm_resp_parse_words(response5);
        assert_eq!(result5, Vec::<String>::new());

        // Test case 6: Empty response
        let response6 = "";
        let result6 = llm_resp_parse_words(response6);
        assert_eq!(result6, Vec::<String>::new());

        // Test case 7: Response with empty words section
        let response7 = "<words>\n</words>";
        let result7 = llm_resp_parse_words(response7);
        assert_eq!(result7, Vec::<String>::new());
    }
}