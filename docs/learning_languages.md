# Learning languages

The flashcard-app includes an experimental module for learning languagues based on
a methodology authored by https://mantvis.lt :

1. A new language is learned by full sentences instead of individual words.
2. The learning process is guided by the [spaced repetition](https://en.wikipedia.org/wiki/Spaced_repetition) (i.e. flash cards).
3. Make sure you understand each word in a sentence before moving on.
4. Keep adding sentences with new words.

## Tutorial

1. Update `settings.toml` (the path is printed to STDOUT):
    * Set `learning_language`, e.g. "spanish", "portuguese", etc.
    * Set `anthropic_api_key` for auto generating new sentences with LLMs.

2. Add some initial flashcards with the sentences you'd like to learn: http://localhost:3000/add-card
3. Populate the vocabulary database from flashcards: http://localhost:3000/learn-languages/vocabulary
4. Add some words you already know - this will help LLMs.
5. Generate new sentences that use a new word: http://localhost:3000/learn-languages/generate-sentence
   This way you gradually expand the vocabulary of a new language.
6. Generate short stories using the words you know: http://localhost:3000/learn-languages/write-story
   This enforces understanding of the words you're learning and introduces some new ones.
   Highlight sentences to translate them and create new flash cards.