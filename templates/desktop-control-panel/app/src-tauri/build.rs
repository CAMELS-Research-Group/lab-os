use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// V1 target phoneme inventory — must mirror
/// `app/src-tauri/src/evaluation/thresholds.rs::V1_TARGET_PHONEMES`.
///
/// Duplicated here because `build.rs` is compiled before the crate itself, so
/// we cannot import from the crate's modules. The runtime `feedback`
/// module's tests verify the baked data matches `V1_TARGET_PHONEMES` exactly
/// (belt-and-suspenders: build-time + test-time).
const V1_TARGET_PHONEMES: &[&str] = &[
    "w", "i", "l", "ʒ", "v", "z", "θ", "æ", "ɛ", "dʒ", "ɪ", "ɹ", "ð",
];

/// Section header that marks the start of the machine-readable table in
/// `documentation/docs/pedagogy/articulation_table.md`.
const STRUCTURED_SECTION_HEADER: &str = "## Structured table (machine-readable companion)";

/// Expected column order in the structured table. Validated at build time so
/// reordering the markdown without updating this build script (or vice versa)
/// fails the build rather than silently mis-mapping fields.
const EXPECTED_COLUMNS: &[&str] =
    &["phoneme", "example_word", "mouth_shape", "minimal_pair", "l1_notes"];

fn main() {
    // Rebuild whenever any IAS_* build-time env var changes. These feed the
    // `BuildConfig` constants in `src/shared/config.rs` (see CL-3).
    println!("cargo:rerun-if-env-changed=IAS_BACKEND_URL");
    println!("cargo:rerun-if-env-changed=IAS_MODEL_URL");
    println!("cargo:rerun-if-env-changed=IAS_MODEL_SHA256");
    println!("cargo:rerun-if-env-changed=IAS_MODEL_VERSION");
    println!("cargo:rerun-if-env-changed=IAS_APP_VERSION");
    println!("cargo:rerun-if-env-changed=IAS_UPDATER_PUBKEY");
    println!("cargo:rerun-if-env-changed=IAS_UPDATER_MANIFEST_URL");

    // CL-18: bake the IAS-facing articulation table's structured section into
    // a static Rust array consumed by the runtime feedback module. Parsing
    // markdown at runtime is unnecessary cost and ships parser bugs as
    // production bugs; doing it here turns "stale or malformed table" into a
    // compile error.
    generate_articulation_table();

    tauri_build::build()
}

/// Parse `documentation/docs/pedagogy/articulation_table.md`'s structured
/// section and emit a Rust source file at `$OUT_DIR/articulation_table_generated.rs`
/// containing a `static ARTICULATION_TABLE: &[ArticulationEntry] = &[...]`.
fn generate_articulation_table() {
    // `CARGO_MANIFEST_DIR` points at `app/src-tauri/`; the docs tree is two
    // levels up under `documentation/docs/pedagogy/`.
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let md_path = manifest_dir
        .join("..")
        .join("..")
        .join("documentation")
        .join("docs")
        .join("pedagogy")
        .join("articulation_table.md");

    println!("cargo:rerun-if-changed={}", md_path.display());

    let md = fs::read_to_string(&md_path).unwrap_or_else(|e| {
        panic!(
            "CL-18 build.rs: could not read articulation table at {}: {}",
            md_path.display(),
            e
        )
    });

    let rows = parse_structured_table(&md, &md_path);
    validate_inventory(&rows, &md_path);

    let generated = render_rust(&rows);

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_path: PathBuf = out_dir.join("articulation_table_generated.rs");
    fs::write(&out_path, generated).unwrap_or_else(|e| {
        panic!(
            "CL-18 build.rs: could not write generated articulation table to {}: {}",
            out_path.display(),
            e
        )
    });
}

/// One parsed row of the structured table. Field order matches `EXPECTED_COLUMNS`.
struct ParsedRow {
    phoneme: String,
    example_word: String,
    mouth_shape: String,
    minimal_pair: String,
    l1_notes: String,
}

/// Locate the structured table section, find the first pipe-delimited
/// markdown table inside it, and return the parsed data rows.
fn parse_structured_table(md: &str, md_path: &Path) -> Vec<ParsedRow> {
    let section_start = md.find(STRUCTURED_SECTION_HEADER).unwrap_or_else(|| {
        panic!(
            "CL-18 build.rs: section header {:?} not found in {}. The articulation table is \
             out of sync with the build script's parser.",
            STRUCTURED_SECTION_HEADER,
            md_path.display()
        )
    });

    let after_header = &md[section_start + STRUCTURED_SECTION_HEADER.len()..];

    // Scan line-by-line for the first table header row (starts with `|`).
    // The line immediately after must be the separator row (also starts with
    // `|` and contains only `|`, `-`, `:`, and whitespace). Subsequent
    // `|`-starting lines are data; the table ends at the first non-`|` line.
    let mut lines = after_header.lines();
    let mut header_row: Option<&str> = None;
    let mut saw_separator = false;
    let mut data_rows: Vec<&str> = Vec::new();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            // Blank line ends the table once we've started; before the
            // header it's just spacing inside the section.
            if header_row.is_some() && saw_separator && !data_rows.is_empty() {
                break;
            }
            continue;
        }
        if !trimmed.starts_with('|') {
            if header_row.is_some() && saw_separator && !data_rows.is_empty() {
                break;
            }
            // Lines like prose between section start and table.
            continue;
        }

        if header_row.is_none() {
            header_row = Some(trimmed);
            continue;
        }

        if !saw_separator {
            // Markdown separator row: only `|`, `-`, `:`, spaces.
            let chars_ok = trimmed
                .chars()
                .all(|c| matches!(c, '|' | '-' | ':' | ' ' | '\t'));
            if !chars_ok {
                panic!(
                    "CL-18 build.rs: expected separator row after table header in {}, got {:?}",
                    md_path.display(),
                    trimmed
                );
            }
            saw_separator = true;
            continue;
        }

        data_rows.push(trimmed);
    }

    let header_row = header_row.unwrap_or_else(|| {
        panic!(
            "CL-18 build.rs: no markdown table found inside structured section in {}",
            md_path.display()
        )
    });

    // Validate column order matches EXPECTED_COLUMNS.
    let header_cells = split_pipe_row(header_row);
    if header_cells.len() != EXPECTED_COLUMNS.len() {
        panic!(
            "CL-18 build.rs: structured table header has {} columns, expected {} in {}",
            header_cells.len(),
            EXPECTED_COLUMNS.len(),
            md_path.display()
        );
    }
    for (idx, (got, expected)) in header_cells.iter().zip(EXPECTED_COLUMNS.iter()).enumerate() {
        if got != expected {
            panic!(
                "CL-18 build.rs: column {} of structured table is {:?}, expected {:?} in {}",
                idx,
                got,
                expected,
                md_path.display()
            );
        }
    }

    // Parse each data row.
    let mut out = Vec::with_capacity(data_rows.len());
    for row in data_rows {
        let cells = split_pipe_row(row);
        if cells.len() != EXPECTED_COLUMNS.len() {
            panic!(
                "CL-18 build.rs: data row has {} columns, expected {} in {}; row was {:?}",
                cells.len(),
                EXPECTED_COLUMNS.len(),
                md_path.display(),
                row
            );
        }
        out.push(ParsedRow {
            phoneme: cells[0].clone(),
            example_word: cells[1].clone(),
            mouth_shape: cells[2].clone(),
            minimal_pair: cells[3].clone(),
            l1_notes: cells[4].clone(),
        });
    }
    out
}

/// Split a markdown table row on `|`, dropping the empty leading/trailing
/// segments and trimming each cell.
fn split_pipe_row(row: &str) -> Vec<String> {
    let mut parts: Vec<&str> = row.split('|').collect();
    // A well-formed pipe row starts and ends with `|`, so the split has empty
    // strings at both ends — drop them.
    if parts.first().map_or(false, |s| s.trim().is_empty()) {
        parts.remove(0);
    }
    if parts.last().map_or(false, |s| s.trim().is_empty()) {
        parts.pop();
    }
    parts.into_iter().map(|s| s.trim().to_string()).collect()
}

/// Assert exactly 13 rows, no duplicate phonemes, and every
/// `V1_TARGET_PHONEMES` symbol present. Drift between the spike inventory
/// and the structured table fails the build with a clear list.
fn validate_inventory(rows: &[ParsedRow], md_path: &Path) {
    if rows.len() != V1_TARGET_PHONEMES.len() {
        panic!(
            "CL-18 build.rs: structured table has {} rows, expected {} (V1 target inventory) in {}",
            rows.len(),
            V1_TARGET_PHONEMES.len(),
            md_path.display()
        );
    }

    let mut seen: HashSet<&str> = HashSet::new();
    for row in rows {
        if !seen.insert(row.phoneme.as_str()) {
            panic!(
                "CL-18 build.rs: duplicate phoneme {:?} in structured table at {}",
                row.phoneme,
                md_path.display()
            );
        }
    }

    let expected: HashSet<&str> = V1_TARGET_PHONEMES.iter().copied().collect();
    let missing: Vec<&&str> = expected.iter().filter(|p| !seen.contains(**p)).collect();
    if !missing.is_empty() {
        panic!(
            "CL-18 build.rs: structured table missing required phonemes {:?} in {}",
            missing,
            md_path.display()
        );
    }
    let extras: Vec<&str> = seen
        .iter()
        .filter(|p| !expected.contains(*p))
        .copied()
        .collect();
    if !extras.is_empty() {
        panic!(
            "CL-18 build.rs: structured table contains phonemes {:?} not in V1 inventory in {}",
            extras,
            md_path.display()
        );
    }
}

/// Render the parsed rows into a Rust source file. The generated file
/// references `ArticulationEntry` from the parent module via `super::`
/// because it is `include!`-d inside `evaluation::feedback`.
fn render_rust(rows: &[ParsedRow]) -> String {
    let mut s = String::new();
    s.push_str("// AUTO-GENERATED by build.rs from\n");
    s.push_str("// documentation/docs/pedagogy/articulation_table.md\n");
    s.push_str("// (CL-18). Do not edit by hand; regenerate via cargo build.\n\n");
    s.push_str("pub static ARTICULATION_TABLE: &[ArticulationEntry] = &[\n");
    for row in rows {
        s.push_str("    ArticulationEntry {\n");
        s.push_str(&format!("        phoneme: {},\n", rust_str(&row.phoneme)));
        s.push_str(&format!(
            "        example_word: {},\n",
            rust_str(&row.example_word)
        ));
        s.push_str(&format!(
            "        mouth_shape: {},\n",
            rust_str(&row.mouth_shape)
        ));
        s.push_str(&format!(
            "        minimal_pair: {},\n",
            rust_str(&row.minimal_pair)
        ));
        s.push_str(&format!(
            "        l1_notes: {},\n",
            rust_str(&row.l1_notes)
        ));
        s.push_str("    },\n");
    }
    s.push_str("];\n");
    s
}

/// Render a Rust string literal that escapes backslashes and double quotes.
/// The cells we ingest are short prose; we do not need raw-string handling.
fn rust_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}
