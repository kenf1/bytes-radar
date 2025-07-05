use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LanguageType {
    Programming,
    Markup,
    Data,
    Configuration,
    Documentation,
    Other,
}

impl Display for LanguageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LanguageType::Programming => write!(f, "Programming"),
            LanguageType::Markup => write!(f, "Markup"),
            LanguageType::Data => write!(f, "Data"),
            LanguageType::Configuration => write!(f, "Configuration"),
            LanguageType::Documentation => write!(f, "Documentation"),
            LanguageType::Other => write!(f, "Other"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LineCommentPosition {
    Any,
    Start,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageDefinition {
    pub name: String,
    pub display_name: Option<String>,
    pub extensions: Vec<String>,
    pub filenames: Vec<String>,
    pub shebangs: Vec<String>,
    pub env: Vec<String>,
    pub mime_types: Vec<String>,
    pub line_comments: Vec<String>,
    pub multi_line_comments: Vec<(String, String)>,
    pub nested_comments: Vec<(String, String)>,
    pub doc_quotes: Vec<(String, String)>,
    pub quotes: Vec<(String, String)>,
    pub verbatim_quotes: Vec<(String, String)>,
    pub important_syntax: Vec<String>,
    pub language_type: LanguageType,
    pub is_literate: bool,
    pub is_nested: bool,
    pub is_blank: bool,
    pub case_sensitive: bool,
    pub line_comment_position: LineCommentPosition,
}

impl LanguageDefinition {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            display_name: None,
            extensions: Vec::new(),
            filenames: Vec::new(),
            shebangs: Vec::new(),
            env: Vec::new(),
            mime_types: Vec::new(),
            line_comments: Vec::new(),
            multi_line_comments: Vec::new(),
            nested_comments: Vec::new(),
            doc_quotes: Vec::new(),
            quotes: Vec::new(),
            verbatim_quotes: Vec::new(),
            important_syntax: Vec::new(),
            language_type: LanguageType::Programming,
            is_literate: false,
            is_nested: false,
            is_blank: false,
            case_sensitive: true,
            line_comment_position: LineCommentPosition::Any,
        }
    }

    pub fn with_display_name(mut self, name: &str) -> Self {
        self.display_name = Some(name.to_string());
        self
    }

    pub fn with_extensions(mut self, extensions: &[&str]) -> Self {
        self.extensions = extensions.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_filenames(mut self, filenames: &[&str]) -> Self {
        self.filenames = filenames.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_line_comments(mut self, comments: &[&str]) -> Self {
        self.line_comments = comments.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn with_multi_line_comments(mut self, comments: &[(&str, &str)]) -> Self {
        self.multi_line_comments = comments
            .iter()
            .map(|(start, end)| (start.to_string(), end.to_string()))
            .collect();
        self
    }

    pub fn with_quotes(mut self, quotes: &[(&str, &str)]) -> Self {
        self.quotes = quotes
            .iter()
            .map(|(start, end)| (start.to_string(), end.to_string()))
            .collect();
        self
    }

    pub fn with_type(mut self, lang_type: LanguageType) -> Self {
        self.language_type = lang_type;
        self
    }

    pub fn with_nested(mut self, nested: bool) -> Self {
        self.is_nested = nested;
        self
    }

    pub fn with_blank(mut self, blank: bool) -> Self {
        self.is_blank = blank;
        self
    }
}

pub struct LanguageRegistry;

impl LanguageRegistry {
    pub fn get_language(name: &str) -> Option<&'static LanguageDefinition> {
        LANGUAGE_MAP.get(name)
    }

    pub fn detect_by_extension(extension: &str) -> Option<&'static LanguageDefinition> {
        let ext = extension.to_lowercase();
        EXTENSION_MAP
            .get(&ext)
            .and_then(|name| LANGUAGE_MAP.get(name))
    }

    pub fn detect_by_filename(filename: &str) -> Option<&'static LanguageDefinition> {
        let lower_filename = filename.to_lowercase();
        for (_, lang) in LANGUAGE_MAP.iter() {
            if lang
                .filenames
                .iter()
                .any(|f| f.to_lowercase() == lower_filename)
            {
                return Some(lang);
            }
        }
        None
    }

    pub fn detect_by_path<P: AsRef<Path>>(path: P) -> Option<&'static LanguageDefinition> {
        let path = path.as_ref();

        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if let Some(lang) = Self::detect_by_filename(filename) {
                return Some(lang);
            }
        }

        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            return Self::detect_by_extension(extension);
        }

        None
    }

    pub fn all_languages() -> impl Iterator<Item = &'static LanguageDefinition> {
        LANGUAGE_MAP.values()
    }

    pub fn languages_by_type(
        lang_type: LanguageType,
    ) -> impl Iterator<Item = &'static LanguageDefinition> {
        LANGUAGE_MAP
            .values()
            .filter(move |lang| lang.language_type == lang_type)
    }
}

// Create comprehensive language definitions based on tokei
fn create_languages() -> HashMap<String, LanguageDefinition> {
    let mut languages = HashMap::new();

    // System languages
    languages.insert(
        "C".to_string(),
        LanguageDefinition::new("C")
            .with_extensions(&["c", "ec", "pgc"])
            .with_line_comments(&["//"])
            .with_multi_line_comments(&[("/*", "*/")])
            .with_quotes(&[("\"", "\"")]),
    );

    languages.insert(
        "Cpp".to_string(),
        LanguageDefinition::new("Cpp")
            .with_display_name("C++")
            .with_extensions(&["cc", "cpp", "cxx", "c++", "pcc", "tpp"])
            .with_line_comments(&["//"])
            .with_multi_line_comments(&[("/*", "*/")])
            .with_quotes(&[("\"", "\"")]),
    );

    languages.insert(
        "CHeader".to_string(),
        LanguageDefinition::new("CHeader")
            .with_display_name("C Header")
            .with_extensions(&["h"])
            .with_line_comments(&["//"])
            .with_multi_line_comments(&[("/*", "*/")])
            .with_quotes(&[("\"", "\"")]),
    );

    languages.insert(
        "CppHeader".to_string(),
        LanguageDefinition::new("CppHeader")
            .with_display_name("C++ Header")
            .with_extensions(&["hh", "hpp", "hxx", "inl", "ipp"])
            .with_line_comments(&["//"])
            .with_multi_line_comments(&[("/*", "*/")])
            .with_quotes(&[("\"", "\"")]),
    );

    languages.insert(
        "Rust".to_string(),
        LanguageDefinition::new("Rust")
            .with_extensions(&["rs"])
            .with_line_comments(&["//"])
            .with_multi_line_comments(&[("/*", "*/")])
            .with_quotes(&[("\"", "\"")])
            .with_nested(true),
    );

    // Popular web languages
    languages.insert(
        "JavaScript".to_string(),
        LanguageDefinition::new("JavaScript")
            .with_extensions(&["js", "mjs", "cjs"])
            .with_line_comments(&["//"])
            .with_multi_line_comments(&[("/*", "*/")])
            .with_quotes(&[("\"", "\""), ("'", "'"), ("`", "`")]),
    );

    languages.insert(
        "TypeScript".to_string(),
        LanguageDefinition::new("TypeScript")
            .with_extensions(&["ts", "mts", "cts"])
            .with_line_comments(&["//"])
            .with_multi_line_comments(&[("/*", "*/")])
            .with_quotes(&[("\"", "\""), ("'", "'"), ("`", "`")]),
    );

    languages.insert(
        "Jsx".to_string(),
        LanguageDefinition::new("Jsx")
            .with_display_name("JSX")
            .with_extensions(&["jsx"])
            .with_line_comments(&["//"])
            .with_multi_line_comments(&[("/*", "*/")])
            .with_quotes(&[("\"", "\""), ("'", "'"), ("`", "`")]),
    );

    languages.insert(
        "Tsx".to_string(),
        LanguageDefinition::new("Tsx")
            .with_display_name("TSX")
            .with_extensions(&["tsx"])
            .with_line_comments(&["//"])
            .with_multi_line_comments(&[("/*", "*/")])
            .with_quotes(&[("\"", "\""), ("'", "'"), ("`", "`")]),
    );

    languages.insert(
        "Python".to_string(),
        LanguageDefinition::new("Python")
            .with_extensions(&["py", "pyw", "pyi"])
            .with_line_comments(&["#"])
            .with_quotes(&[("\"", "\""), ("'", "'")]),
    );

    languages.insert(
        "Java".to_string(),
        LanguageDefinition::new("Java")
            .with_extensions(&["java"])
            .with_line_comments(&["//"])
            .with_multi_line_comments(&[("/*", "*/")])
            .with_quotes(&[("\"", "\"")]),
    );

    languages.insert(
        "Go".to_string(),
        LanguageDefinition::new("Go")
            .with_extensions(&["go"])
            .with_line_comments(&["//"])
            .with_multi_line_comments(&[("/*", "*/")])
            .with_quotes(&[("\"", "\"")]),
    );

    languages.insert(
        "CSharp".to_string(),
        LanguageDefinition::new("CSharp")
            .with_display_name("C#")
            .with_extensions(&["cs", "csx"])
            .with_line_comments(&["//"])
            .with_multi_line_comments(&[("/*", "*/")])
            .with_quotes(&[("\"", "\"")]),
    );

    // Functional languages
    languages.insert(
        "Haskell".to_string(),
        LanguageDefinition::new("Haskell")
            .with_extensions(&["hs"])
            .with_line_comments(&["--"])
            .with_multi_line_comments(&[("{-", "-}")])
            .with_nested(true),
    );

    languages.insert(
        "Scala".to_string(),
        LanguageDefinition::new("Scala")
            .with_extensions(&["sc", "scala"])
            .with_line_comments(&["//"])
            .with_multi_line_comments(&[("/*", "*/")])
            .with_quotes(&[("\"", "\"")]),
    );

    languages.insert(
        "Kotlin".to_string(),
        LanguageDefinition::new("Kotlin")
            .with_extensions(&["kt", "kts"])
            .with_line_comments(&["//"])
            .with_multi_line_comments(&[("/*", "*/")])
            .with_quotes(&[("\"", "\""), ("\"\"\"", "\"\"\"")])
            .with_nested(true),
    );

    // Shell scripts
    languages.insert(
        "Bash".to_string(),
        LanguageDefinition::new("Bash")
            .with_display_name("BASH")
            .with_extensions(&["bash"])
            .with_line_comments(&["#"])
            .with_quotes(&[("\"", "\""), ("'", "'")]),
    );

    languages.insert(
        "Sh".to_string(),
        LanguageDefinition::new("Sh")
            .with_display_name("Shell")
            .with_extensions(&["sh"])
            .with_line_comments(&["#"])
            .with_quotes(&[("\"", "\""), ("'", "'")]),
    );

    // Web markup and styling
    languages.insert(
        "Html".to_string(),
        LanguageDefinition::new("Html")
            .with_display_name("HTML")
            .with_extensions(&["html", "htm"])
            .with_multi_line_comments(&[("<!--", "-->")])
            .with_quotes(&[("\"", "\""), ("'", "'")])
            .with_type(LanguageType::Markup),
    );

    languages.insert(
        "Css".to_string(),
        LanguageDefinition::new("Css")
            .with_display_name("CSS")
            .with_extensions(&["css"])
            .with_line_comments(&["//"])
            .with_multi_line_comments(&[("/*", "*/")])
            .with_quotes(&[("\"", "\""), ("'", "'")])
            .with_type(LanguageType::Markup),
    );

    // Data formats
    languages.insert(
        "Json".to_string(),
        LanguageDefinition::new("Json")
            .with_display_name("JSON")
            .with_extensions(&["json"])
            .with_type(LanguageType::Data)
            .with_blank(true),
    );

    languages.insert(
        "Yaml".to_string(),
        LanguageDefinition::new("Yaml")
            .with_display_name("YAML")
            .with_extensions(&["yaml", "yml"])
            .with_line_comments(&["#"])
            .with_quotes(&[("\"", "\""), ("'", "'")])
            .with_type(LanguageType::Data),
    );

    languages.insert(
        "Toml".to_string(),
        LanguageDefinition::new("Toml")
            .with_display_name("TOML")
            .with_extensions(&["toml"])
            .with_line_comments(&["#"])
            .with_quotes(&[
                ("\"", "\""),
                ("'", "'"),
                ("\"\"\"", "\"\"\""),
                ("'''", "'''"),
            ])
            .with_type(LanguageType::Configuration),
    );

    // Documentation
    languages.insert(
        "Markdown".to_string(),
        LanguageDefinition::new("Markdown")
            .with_extensions(&["md", "markdown"])
            .with_type(LanguageType::Documentation),
    );

    languages.insert(
        "Text".to_string(),
        LanguageDefinition::new("Text")
            .with_display_name("Plain Text")
            .with_extensions(&["text", "txt"])
            .with_type(LanguageType::Documentation),
    );

    languages
}

fn create_extension_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (name, lang) in LANGUAGE_MAP.iter() {
        for ext in &lang.extensions {
            map.insert(ext.clone(), name.clone());
        }
    }
    map
}

static LANGUAGE_MAP: Lazy<HashMap<String, LanguageDefinition>> = Lazy::new(create_languages);
static EXTENSION_MAP: Lazy<HashMap<String, String>> = Lazy::new(create_extension_map);
