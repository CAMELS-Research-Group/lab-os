# V1 Articulation Reference Table

**Status:** draft v0.1 — engineering-authored, **requires review by a phonetician or ESL specialist before student-facing use**.
**Scope:** the 13 V1 target phonemes selected by the IAS office for Mandarin and Hindi L1 learners of English.

---

## Purpose

This document is the V1 authoritative reference for:

- Feature data (place, manner, voicing / height, backness, tenseness) used by the articulatory feedback generator.
- Natural-language mouth-shape descriptions used in corrective feedback text.
- Typical L1 interference patterns the feedback should be prepared to address.

It is tech-independent — no assumptions about the encoder, rendering approach, or platform — and doubles as a starting artifact if the engineering proposal for a written pronunciation-teaching deliverable (see `specifications/PRD.md` §12, §14 item 6) is accepted at the 2026-04-22 meeting.

## Caveats

1. **Descriptions use General American English articulation** as the reference accent. Regional variation exists; V1 does not attempt to handle it.
2. **L1 interference notes are typical tendencies**, not individual-learner predictions. Do not render them as "because you are a Mandarin speaker, you said X" — that's bad pedagogy and bad UX.
3. **This is engineering-drafted phonetics content.** Before any of it reaches a student screen, it must be reviewed by Martin, Nicole, or another qualified ESL/phonetics reviewer. Errors in articulatory instructions can teach incorrect motor patterns.
4. **Example words are drawn from the V1 passage** (`passages/visiting_nyc.txt`) where possible, because familiar context helps learners. When the passage does not contain a clear example, a common everyday word is substituted and marked with an asterisk.
5. **Minimal pairs** are drawn from within the 13-phoneme inventory. Full English minimal-pair lists are intentionally out of scope for V1.

## Notation key

- **Place of articulation** — where in the vocal tract the constriction happens (bilabial, labiodental, dental, alveolar, post-alveolar, palatal, velar, glottal, labial-velar).
- **Manner of articulation** — how the airstream is modified (stop/plosive, fricative, affricate, nasal, lateral, approximant).
- **Voicing** — whether the vocal cords vibrate (voiced) or not (voiceless).
- **Vowel features** — tongue height (close/mid/open), backness (front/central/back), lip rounding, tenseness (tense/lax).

---

## Consonants (9)

### /r/ — "river", "tourists", "Brooklyn"

- **Features:** voiced · alveolar (American English: retroflex or bunched) · approximant
- **Mouth shape:** Curl the tip of your tongue up and slightly back, toward the roof of your mouth just behind your upper teeth, **without touching it**. Lips are slightly rounded. Your throat vibrates. The tongue does not touch anywhere — air flows freely.
- **L1 interference:**
  - *Mandarin* — English /r/ is often substituted with a /ʒ/-like sound (the Mandarin "r" in pinyin is a retroflex fricative /ʐ/, different from English /r/). Syllable-final /r/ may be deleted entirely.
  - *Hindi* — Hindi /r/ is usually a tap /ɾ/, produced by a single quick touch of the tongue on the alveolar ridge. English /r/ requires sustained approximation, not contact.
- **Minimal pair in inventory:** /r/ vs. /l/ — "right" / "light".

### /l/ — "learn", "Liberty", "walking"

- **Features:** voiced · alveolar · lateral approximant
- **Mouth shape:** Touch the **tip** of your tongue to the ridge just behind your upper front teeth. Keep the sides of your tongue lowered so air can flow around them on both sides. Your throat vibrates.
- **L1 interference:**
  - *Mandarin* — syllable-final /l/ does not exist; tendency to substitute with /n/ or delete.
  - *Hindi* — Hindi /l/ is dental (tongue touches the teeth themselves, not the ridge behind them). Close but not identical; can sound slightly "softer" than English /l/.
- **Minimal pair in inventory:** /l/ vs. /r/.

### /v/ — "visit", "visitors", "view"

- **Features:** voiced · labiodental · fricative
- **Mouth shape:** Bring your **lower lip lightly against your upper front teeth**. Push air through the gap — you should feel a buzz on your lip. Your throat vibrates.
- **L1 interference:**
  - *Mandarin* — /v/ does not exist in Mandarin. Commonly substituted with /w/ ("visit" → "wisit").
  - *Hindi* — Hindi does not distinguish /v/ from /w/; both are typically realized as /ʋ/, a labiodental approximant that lies between the two.
- **Minimal pair in inventory:** /v/ vs. /w/ — "vine"* / "wine"*.

### /w/ — "walking", "when", "wide"

- **Features:** voiced · labial-velar · approximant
- **Mouth shape:** Round your lips into a small, tight "o" shape. At the same time, raise the back of your tongue toward the soft roof of your mouth (no contact). Your throat vibrates. The lips quickly open as the /w/ transitions into the following vowel.
- **L1 interference:**
  - *Mandarin* — has a similar approximant; usually produced correctly.
  - *Hindi* — often merged with /v/ and realized as /ʋ/. For Hindi speakers the distinction between /v/ and /w/ is the key issue, not either sound individually.
- **Minimal pair in inventory:** /w/ vs. /v/.

### /θ/ — "through"*, "thanks"*, "think"*

- **Features:** voiceless · dental · fricative
- **Mouth shape:** Place the **tip of your tongue lightly between your upper and lower front teeth** (or just behind the upper teeth). Blow air gently through the gap. Your throat does **not** vibrate — this is a voiceless sound; you should only hear and feel air, not a buzz.
- **L1 interference:**
  - *Mandarin* — /θ/ does not exist. Commonly substituted with /s/ ("three" → "sree") or /f/.
  - *Hindi* — /θ/ does not exist. Commonly substituted with a dental /t̪/ ("three" → "tree") — the tongue touches the teeth instead of allowing air to pass through.
- **Minimal pair in inventory:** /θ/ vs. /ð/ (voicing only).
- **Note:** /θ/ appears in "through" in the passage (paragraph 3: "stroll through Ellis Island").

### /ð/ — "the", "they", "there"

- **Features:** voiced · dental · fricative
- **Mouth shape:** Identical tongue and teeth position to /θ/ — tip of the tongue lightly between or behind the upper front teeth, air flowing through the gap — **but this time your throat vibrates**. You should feel both the buzz in your throat *and* the air on your tongue.
- **L1 interference:**
  - *Mandarin* — no dental fricatives; commonly substituted with /z/ or /d/.
  - *Hindi* — commonly substituted with dental /d̪/ ("the" → close to "day").
- **Minimal pair in inventory:** /ð/ vs. /θ/ (voicing); /ð/ vs. /z/ (place).

### /ʒ/ — "usually", "treasures"

- **Features:** voiced · post-alveolar (palato-alveolar) · fricative
- **Mouth shape:** Raise the body of your tongue toward the hard roof of your mouth, with the tip just behind (not touching) the ridge behind your upper teeth. Round and slightly push your lips forward. Force air through the narrow channel — it creates a soft hiss. Your throat vibrates.
- **L1 interference:**
  - *Mandarin* — /ʒ/ does not exist. The closest sound is the Mandarin retroflex fricative /ʐ/ (the "r" in pinyin), which uses a different tongue posture. Often substituted with /ʃ/ (voiceless) or /z/.
  - *Hindi* — rare in Hindi; tends toward /z/ or /ʃ/.
- **Minimal pair in inventory:** /ʒ/ vs. /dʒ/ (fricative vs. affricate).
- **Note:** /ʒ/ is comparatively rare in English and appears in the passage in "usually" (paragraph 1) and "treasures" (paragraph 2).

### /dʒ/ — "jazz", "bridge", "enjoy", "huge"

- **Features:** voiced · post-alveolar · affricate
- **Mouth shape:** An affricate is **a stop followed immediately by a fricative, as one sound**. Start by touching the front of your tongue to the ridge behind your upper teeth, briefly blocking the airflow. Then release the tongue into the /ʒ/ posture above — tongue raised toward the palate, lips slightly rounded, air hissing through. Your throat vibrates throughout.
- **L1 interference:**
  - *Mandarin* — the native affricate /tʂ/ is retroflex; substitution is common.
  - *Hindi* — Hindi has /dʒ/ natively and this sound is usually easy for Hindi speakers.
- **Minimal pair in inventory:** /dʒ/ vs. /ʒ/.

### /z/ — "zoo", "visit", "visitors", "museums"*

- **Features:** voiced · alveolar · fricative
- **Mouth shape:** Bring the tip of your tongue close to — but **not touching** — the ridge behind your upper front teeth. Leave a very narrow channel. Push air through; it should produce a buzzing hiss. Your throat vibrates.
- **L1 interference:**
  - *Mandarin* — /z/ does not exist as an independent phoneme. Commonly devoiced to /s/ ("zoo" → "soo").
  - *Hindi* — /z/ exists in Hindi (primarily in loanwords) and is usually produced correctly, though some speakers substitute /dʒ/ in specific contexts.
- **Minimal pair in inventory:** /z/ vs. /ʒ/ (place of articulation).
- **Note:** /z/ appears in the passage in "zoo" (paragraph 2).

---

## Vowels (4)

### /i/ — "see", "street"*, "feel"

- **Features:** close · front · unrounded · **tense**
- **Mouth shape:** Raise the front of your tongue **high and forward**, close to the roof of your mouth without touching it. Spread your lips wide, as if smiling. Keep the muscles in your tongue and lips **tense** — this tension is the main thing that separates /i/ from its lax partner /ɪ/. The sound is slightly longer and clearer than /ɪ/.
- **L1 interference:**
  - *Mandarin* — has a similar /i/ vowel, but the contrast with /ɪ/ is usually absent. Tendency is to produce every English "i"-like vowel as /i/.
  - *Hindi* — same pattern: /i/ vs. /ɪ/ contrast is weak or absent.
- **Minimal pair in inventory:** /i/ vs. /ɪ/ — "sheep"* / "ship"*, "feet"* / "fit"*.

### /ɪ/ — "visit", "bridge", "big"*

- **Features:** near-close · near-front · unrounded · **lax**
- **Mouth shape:** Similar to /i/ but with the tongue slightly **lower and more relaxed**, pulled a bit back toward the center. Lips are slightly less spread than /i/. The muscles are **relaxed** rather than tense. The sound is shorter than /i/.
- **L1 interference:**
  - *Mandarin, Hindi* — both commonly merge /ɪ/ with /i/. This is one of the hardest contrasts in English for many L2 learners and is worth practicing explicitly.
- **Minimal pair in inventory:** /ɪ/ vs. /i/.

### /ɛ/ — "end", "Ellis"*, "ferry"

- **Features:** open-mid · front · unrounded · (neutral tense/lax)
- **Mouth shape:** Tongue is at **mid-height**, toward the front of the mouth. Open your jaw a little more than for /ɪ/. Lips are neutral — not spread, not rounded. Relaxed muscles.
- **L1 interference:**
  - *Mandarin* — often merged with /æ/ (lower) or /eɪ/ (the diphthong). Pure /ɛ/ is not a separate phoneme.
  - *Hindi* — often merged with /e/ or /æ/.
- **Minimal pair in inventory:** /ɛ/ vs. /æ/ — "bet"* / "bat"*, "head"* / "had"*.

### /æ/ — "have", "that", "attract", "back"*, "jazz"

- **Features:** near-open · front · unrounded
- **Mouth shape:** **Drop your jaw wide open** and push the front of your tongue low and forward — lower than /ɛ/. Lips are slightly spread and open. This is one of the most "open-mouthed" vowels in English; learners often under-open their jaw, producing /ɛ/ instead.
- **L1 interference:**
  - *Mandarin* — /æ/ does not exist; commonly merged with /ɛ/ or /ɑ/ ("cat" → "ket" or "cot").
  - *Hindi* — commonly merged with /ɛ/ or /e/.
- **Minimal pair in inventory:** /æ/ vs. /ɛ/.

---

## Minimal pair matrix (within-inventory)

Pairs where the two phonemes differ by exactly one feature and are confusable for the target L1 populations. V1 feedback should weight pairs on this matrix heavily.

| Pair | Contrast | Example | Notes |
| --- | --- | --- | --- |
| /r/ ↔ /l/ | manner (approximant vs. lateral) | right / light | Classic Mandarin difficulty |
| /v/ ↔ /w/ | manner + place | vine / wine | Classic Hindi difficulty |
| /θ/ ↔ /ð/ | voicing only | thin / then | Voicing drill |
| /z/ ↔ /ʒ/ | place only | buzz / beige*-final | Rare but distinctive |
| /ʒ/ ↔ /dʒ/ | manner (fricative vs. affricate) | beige / badge* | Easy to confuse |
| /i/ ↔ /ɪ/ | tenseness + slight height | sheep / ship | The single hardest English vowel contrast for many L1s |
| /ɛ/ ↔ /æ/ | height | bet / bat | Classic Mandarin/Hindi vowel difficulty |

Pairs **not** included on the matrix but relevant as substitution destinations (the phoneme the learner produced instead):

- /θ/ → /s/, /f/, /t̪/ — common substitutions by L1 Mandarin and Hindi speakers
- /ð/ → /z/, /d/, /d̪/ — same
- /v/ → /w/, /ʋ/ — Hindi/Mandarin
- /r/ → /ʐ/, /ɾ/ — Mandarin/Hindi
- /z/ → /s/ — Mandarin (devoicing)

These cross-inventory substitutions are important for **error detection** (the encoder reports what the learner actually produced) but are **out of scope for V1 feedback text** — V1 only teaches the 13 target phonemes themselves, not a full substitution diagnosis.

## Sources and further reading

Phonetic descriptions in this document are drawn from standard references and engineering knowledge of English phonology; nothing is copied from a specific source.

For review and expansion, the following are recommended but not incorporated here:

- The International Phonetic Association's *Handbook of the IPA* (2005) — canonical articulatory descriptions.
- Peter Ladefoged and Keith Johnson, *A Course in Phonetics* — standard pedagogical reference for English phonology.
- Seeing Speech (seeingspeech.ac.uk) — IPA articulation videos from six Scottish universities. **License:** CC BY-NC-ND 4.0 — useful for **reviewer reference only**, not for redistribution inside a Pace-owned app.
- L1-specific references on Mandarin and Hindi phonology for deeper L1-interference analysis.

## Revision history

- **v0.1** — initial draft, engineering-authored, pending review by a phonetician / ESL specialist. Accompanies PRD v0.1.
- **v0.2** — added the `## Structured table (machine-readable companion)` section below. Derived from the prose above; parsed by `app/src-tauri/build.rs` (CL-18) into a compile-time static articulation table. Same "subject to IAS phonetician review before student-facing use" caveat applies.
- **v0.3** — IAS review (Nicole Gunn / Martin Molden, 2026-06). Structured table reshaped to five columns (`phoneme | example_word | mouth_shape | minimal_pair | l1_notes`): the tongue_placement / lip_shape / voicing / airflow split collapsed into a single plain-language `mouth_shape` paragraph, articulatory jargon removed, and L1 notes excluded from learner-facing copy (the `l1_notes` column is retained only to carry the SPIKE-16 §3 hedges, never displayed). Example words for /l/ ("light") and /ʒ/ ("treasure") changed per IAS request; these two are not passage-resident. The `minimal_pair` column was added so this table is the **single source of truth** for all learner-facing copy — the former `app/src/data/articulation.ts` (which had duplicated this copy) was deleted, and the Results screen now reads the pair from `FeedbackEntry`. `build.rs` and `evaluation::feedback` updated to the new shape. Prose sections above pending reconciliation.
- **v0.4** — IAS follow-up review (Nicole Gunn / Martin Molden, 2026-06-09). Added learner-facing minimal pairs for the three phonemes previously left empty: /z/ → `sip / zip`, /ʒ/ → `version / virgin`, /dʒ/ → `version / virgin` (`version` carries /ʒ/, `virgin` carries /dʒ/, so the one pair serves both post-alveolar rows). These were supplied by IAS as pedagogy owner; `sip / zip` intentionally reaches outside the 13-phoneme inventory (/s/ is not a target) per IAS preference, relaxing the v0.1 "within-inventory only" caveat for this case.

---

## Structured table (machine-readable companion)

This section is **derived from the prose above** and is parsed by the IAS client's build pipeline (`app/src-tauri/build.rs`, task CL-18) into a compile-time static articulation table. The prose remains the human-authored source; this table is a structured projection for the runtime feedback generator.

**IAS review status:** the `mouth_shape`, `example_word`, and `minimal_pair` columns below were reviewed and revised by IAS (Nicole Gunn / Martin Molden, 2026-06). Per that review the former four-column articulation split (tongue_placement / lip_shape / voicing / airflow) was collapsed into a single plain-language `mouth_shape` paragraph, articulatory jargon (e.g. "alveolar", "approximant") was removed, and L1-specific guidance was dropped from learner-facing copy.

**This table is the single source of truth for all learner-facing articulation copy.** Everything the learner sees — the example word, the "how to make this sound" paragraph, and the "How to Practice" pair — is generated from these columns at build time (`build.rs` → `evaluation::feedback::FeedbackEntry` → the Results screen). To change learner copy, edit a cell here and rebuild; nothing else holds a copy. Conventions:

- `minimal_pair` is the two contrasting words shown on the "Say this pair" practice line (e.g. `light / right`). Leave the cell **empty** only where the phoneme has no usable word pair — the UI omits the pair line rather than inventing copy. (/z/, /ʒ/, /dʒ/ were previously left empty as "conceptual contrasts"; IAS supplied learner-facing pairs for them in the 2026-06-09 review — see revision history.)
- `l1_notes` is **retained but never shown to learners** — `FeedbackEntry` drops it — solely to carry the SPIKE-16 §3 tool-reliability hedges described below, which the `feedback.rs` rule checks enforce at build and selection time.

The prose sections above this table predate the 2026-06 review and are pending reconciliation; this structured table is authoritative.

Three rows encode SPIKE-16 §3 guardrails directly in the copy, because the spike showed the model's per-phoneme certainty signal is unreliable for these phonemes in opposite ways:

- **/ð/** — the model cannot certify a correct /ð/ (distortion-blind). Copy must use a hedged tone and must not claim correct production; the `l1_notes` column carries an explicit "the tool may not reliably detect" caveat.
- **/θ/** — many correct /θ/ productions also score low. Copy uses hedged framing ("may sound unclear if…"), never confident correction.
- **/ʒ/** — provisional signal, low sample count (n=66 in the spike). Copy carries the exact substring "harder for the tool to score" so downstream UI can render the caveat verbatim.

| phoneme | example_word | mouth_shape | minimal_pair | l1_notes |
| --- | --- | --- | --- | --- |
| w | walking | Round your lips into a small, tight 'o' shape. At the same time, raise the back of your tongue toward the soft roof of your mouth — no contact. Your throat vibrates. The lips quickly open as the /w/ moves into the next vowel. | wine / vine | Mandarin speakers usually produce this correctly; Hindi speakers often merge it with /v/ as /ʋ/ |
| i | see | Raise the front of your tongue high and forward, close to the roof of your mouth without touching it. Spread your lips wide, as if smiling. Keep the muscles in your tongue and lips tense — this tension is what separates /i/ from /ɪ/. The sound is slightly longer and clearer than /ɪ/. Your throat vibrates. | sheep / ship | Mandarin and Hindi speakers often merge /i/ with /ɪ/; practice the length and tension contrast |
| l | light | Touch the tip of your tongue to the ridge just behind your upper front teeth. Keep the sides of your tongue lowered so air can flow around them on both sides. Your throat vibrates. | light / right | Mandarin lacks syllable-final /l/ and may substitute /n/ or delete; Hindi /l/ is dental and may sound softer |
| ʒ | treasure | Put the tip of your tongue at the front of the top of your mouth, behind where the /s/ is pronounced. Vibrate your throat and push air between the top of your mouth and the tip of your tongue. | version / virgin | this phoneme is harder for the tool to score; results are provisional and Mandarin or Hindi speakers may also substitute /ʃ/ or /z/ |
| v | visit | Bring your lower lip lightly against your upper front teeth. Push air through the gap — you should feel a buzz on your lip. Your throat vibrates. | vine / wine | Mandarin lacks /v/ and may substitute /w/; Hindi merges /v/ and /w/ as /ʋ/ |
| z | zoo | Bring the tip of your tongue close to — but not touching — the ridge behind your upper front teeth. Leave a very narrow channel. Push air through; it should produce a buzzing hiss. Your throat vibrates. | sip / zip | Mandarin lacks voiced /z/ and may devoice to /s/; Hindi speakers usually produce it correctly |
| θ | through | Place the tip of your tongue lightly between your upper and lower front teeth (or just behind the upper teeth). Blow air gently through the gap. Both “th” sounds (/θ/ and /ð/) are pronounced the same except for voicing. For this voiceless /θ/, your throat does not vibrate — this is a voiceless sound; you should only hear and feel air, not a buzz. | thin / then | Mandarin and Hindi lack dental fricatives; common substitutions are /s/, /f/, or a dental /t/; the tool was uncertain on /θ/, so apparent flags may reflect tool limits rather than learner errors |
| æ | have | Drop your jaw wide open and push the front of your tongue low and forward — lower than /ɛ/. Lips are slightly spread and open. Your throat vibrates. This is one of the most open-mouthed vowels in English; learners often under-open the jaw, producing /ɛ/ instead. | bat / bet | Mandarin and Hindi lack a low front /æ/ and often merge it with /ɛ/ or /ɑ/ |
| ɛ | end | Place your tongue at mid-height, toward the front of the mouth. Open your jaw a little more than for /ɪ/. Your mouth and lips are relaxed — not spread, not rounded. Your throat vibrates. | bet / bat | Mandarin and Hindi speakers often merge /ɛ/ with /æ/ or the /eɪ/ diphthong |
| dʒ | bridge | Start by touching the front of your tongue to the ridge behind your upper teeth, briefly blocking the airflow. Then release into the /ʒ/ position above — tongue raised toward the top of your mouth, lips slightly rounded, air hissing through. Your throat vibrates throughout. | version / virgin | Mandarin's native affricate is retroflex /tʂ/; Hindi has /dʒ/ natively and this is usually easy |
| ɪ | visit | Raise your tongue high and forward. Keep the muscles in your lips relaxed — this lack of tension is what separates /ɪ/ from /i/. The sound is slightly shorter than /i/. Your throat vibrates. | ship / sheep | Mandarin and Hindi commonly merge /ɪ/ with /i/; this is one of the hardest contrasts in English |
| ɹ | river | Curl the tip of your tongue up and slightly back, toward the roof of your mouth just behind your upper teeth, without touching it. Lips are slightly rounded. Your throat vibrates. The tongue does not touch anywhere — air flows freely. | right / light | Mandarin substitutes the retroflex fricative /ʐ/; Hindi uses a tap /ɾ/ instead of a sustained approximant |
| ð | the | Place the tip of your tongue lightly between your upper and lower front teeth (or just behind the upper teeth). Blow air gently through the gap. Both “th” sounds (/θ/ and /ð/) are pronounced the same except for voicing. For this voiced /ð/, your throat vibrates — this is a voiced sound; you should feel both the buzz in your throat and the air on your tongue. | then / thin | Mandarin lacks dental fricatives and may substitute /z/ or /d/; Hindi may substitute dental /d/; the tool may not reliably detect distortion on /ð/, so this guidance is offered as a reminder rather than a correction |

### Inventory invariants enforced at build time

The build script (`app/src-tauri/build.rs`) asserts that this table has exactly 13 rows, that every phoneme in `V1_TARGET_PHONEMES` is present, and that there are no duplicate phonemes. Drift between this table and `app/src-tauri/src/evaluation/thresholds.rs::V1_TARGET_PHONEMES` is a compile-time error, not a runtime one.
