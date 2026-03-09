use rand::{RngExt, seq::IndexedRandom};

use crate::{ApiError, ApiResult};

/// 62 chars, makes 62^n (where n is `random_string_length` in the config) possible urls
const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// 208 words, makes 208^n (where n is `random_word_count` in the config) possible urls
const WORDS: &[&str] = &[
    "amber", "arch", "atlas", "azure", "bark", "beam", "bell", "birch", "blade", "bloom", "bolt",
    "bond", "brew", "brick", "bridge", "brook", "brush", "calm", "cedar", "chalk", "chart",
    "chase", "cliff", "cloud", "coal", "coast", "coin", "coral", "core", "crest", "crisp", "crop",
    "crown", "crush", "dawn", "deck", "delta", "depth", "dew", "dome", "drift", "drum", "dune",
    "dust", "dusk", "echo", "edge", "elm", "ember", "fade", "fern", "field", "film", "flare",
    "flash", "fleet", "flint", "flow", "foam", "fold", "forge", "fork", "form", "frost", "gale",
    "gate", "glade", "glow", "gold", "grain", "grove", "gulf", "haze", "helm", "hill", "hive",
    "hook", "horn", "hull", "husk", "ink", "iron", "isle", "jade", "kelp", "kiln", "knot", "lake",
    "lamp", "lark", "lava", "leaf", "ledge", "lens", "lime", "link", "loch", "loft", "loop",
    "lure", "lynx", "mast", "mesa", "mist", "moor", "moss", "moth", "mound", "muse", "nadir",
    "nest", "node", "north", "oak", "opal", "orbit", "page", "pale", "path", "peak", "peat",
    "pine", "plain", "plank", "plume", "pole", "pond", "pool", "port", "prism", "pulse", "quill",
    "rain", "ramp", "reef", "ridge", "rift", "ring", "river", "rock", "root", "rose", "rune",
    "rush", "sage", "sail", "salt", "sand", "scale", "seam", "seed", "shade", "shaft", "shore",
    "silk", "silt", "skiff", "slate", "sleet", "slope", "smoke", "snow", "soot", "spark", "spire",
    "spray", "spring", "stag", "stave", "steel", "stem", "step", "stone", "storm", "straw",
    "stream", "swift", "thorn", "tide", "timber", "tine", "torch", "trace", "track", "trail",
    "tread", "trout", "tuft", "vale", "vane", "vault", "veil", "vine", "void", "wake", "wave",
    "weld", "whirl", "wind", "wing", "wolf", "wood", "wren", "yarn", "yew", "zenith", "zest",
    "zinc",
];

/// generates a random string with [`CHARSET`]
pub fn random_string(length: usize) -> String {
    let mut rng = rand::rng();
    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// generates a random string with a combination of [`WORDS`]
pub fn random_words(count: usize, separator: &str) -> String {
    let mut rng = rand::rng();
    let words: Vec<&str> = WORDS.sample(&mut rng, count).copied().collect();
    words.join(separator)
}

/// validates whether the slug is valid, constrained to a max of 64 characters
/// and being alphanumeric or `-` & `_`
pub fn validate_slug(slug: String) -> ApiResult<String> {
    (slug.is_empty()
        && slug.len() <= 64
        && slug
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_'))
    .ok_or(ApiError::bad_request(
        "slug must be 1-64 chars, alphanumeric, hyphens or underscores only",
    ))
    .map(|_| slug)
}
