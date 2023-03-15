static SYNTAX_DIR: include_dir::Dir<'_> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/syntax");
pub const DEFAULT_THEME: &str = "fastn-theme.dark";

pub static SS: once_cell::sync::Lazy<syntect::parsing::SyntaxSet> =
    once_cell::sync::Lazy::new(|| {
        let mut builder = syntect::parsing::SyntaxSet::load_defaults_newlines().into_builder();
        for f in SYNTAX_DIR.files() {
            builder.add(
                syntect::parsing::syntax_definition::SyntaxDefinition::load_from_str(
                    f.contents_utf8().unwrap(),
                    true,
                    f.path().file_stem().and_then(|x| x.to_str()),
                )
                .unwrap(),
            );
        }
        builder.build()
    });

/*pub static KNOWN_EXTENSIONS: once_cell::sync::Lazy<std::collections::HashSet<String>> =
once_cell::sync::Lazy::new(|| {
    SS.syntaxes()
        .iter()
        .flat_map(|v| v.file_extensions.to_vec())
        .collect()
});*/

pub static TS: once_cell::sync::Lazy<syntect::highlighting::ThemeSet> =
    once_cell::sync::Lazy::new(syntect::highlighting::ThemeSet::load_defaults);

pub static TS1: syntect::highlighting::ThemeSet =
    syntect::highlighting::ThemeSet::load_from_folder(
        std::path::PathBuf::from(
            option_env!("OUT_DIR").unwrap_or(option_env!("CARGO_MANIFEST_DIR").unwrap_or(".")),
        )
        .join("theme"),
    )
    .unwrap();

/*fn ts1() -> syntect::highlighting::ThemeSet {
    let mut theme_set = syntect::highlighting::ThemeSet::new();

    let mut dark_theme = include_str!("../../theme/fastn-theme.dark.tmTheme").as_bytes();
    theme_set.themes.insert(
        "fastn-theme.dark".to_owned(),
        syntect::highlighting::ThemeSet::load_from_reader(&mut dark_theme).unwrap(),
    );

    let mut light_theme = include_str!("../../theme/fastn-theme.light.tmTheme").as_bytes();
    theme_set.themes.insert(
        "fastn-theme.light".to_owned(),
        syntect::highlighting::ThemeSet::load_from_reader(&mut light_theme).unwrap(),
    );
    theme_set
}*/

pub fn code(code: &str, ext: &str, theme: &str, doc_id: &str) -> ftd::executor::Result<String> {
    let syntax = SS
        .find_syntax_by_extension(ext)
        .unwrap_or_else(|| SS.find_syntax_plain_text());

    let theme = if let Some(theme) = TS.themes.get(theme).or(TS1.themes.get(theme)) {
        theme
    } else {
        return Err(ftd::executor::Error::ParseError {
            message: format!("'{}' is not a valid theme", theme),
            doc_id: doc_id.to_string(),
            line_number: 0,
        });
    };

    let code = code
        .lines()
        .skip_while(|l| l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
        + "\n";

    // TODO: handle various params
    Ok(highlighted_html_for_string(code.as_str(), ext, &SS, syntax, theme)?.replacen('\n', "", 1))
}

fn highlighted_html_for_string(
    s: &str,
    ext: &str,
    ss: &syntect::parsing::SyntaxSet,
    syntax: &syntect::parsing::SyntaxReference,
    theme: &syntect::highlighting::Theme,
) -> Result<String, syntect::Error> {
    let mut highlighter = syntect::easy::HighlightLines::new(syntax, theme);
    let mut output = start_highlighted_html_snippet(theme);

    for line in syntect::util::LinesWithEndings::from(s) {
        let mut regions = highlighter.highlight_line(line, ss)?;
        if ext.eq("ftd") && line.starts_with('.') {
            regions.remove(0);
        }
        syntect::html::append_highlighted_html_for_styled_line(
            &regions[..],
            syntect::html::IncludeBackground::IfDifferent(syntect::highlighting::Color::WHITE),
            &mut output,
        )?;
    }
    output.push_str("</pre>\n");
    Ok(output)
}

fn start_highlighted_html_snippet(t: &syntect::highlighting::Theme) -> String {
    let c = t
        .settings
        .background
        .map(|c| {
            let a = if c.a != 255 {
                format!("{:02x}", c.a)
            } else {
                Default::default()
            };
            format!(
                " style=\"background-color:#{:02x}{:02x}{:02x}{};\"",
                c.r, c.g, c.b, a
            )
        })
        .unwrap_or_default();

    format!("<pre{}>\n", c)
}
