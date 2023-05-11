#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Header {
    KV(ftd::p1::header::KV),
    Section(ftd::p1::header::Section),
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct KV {
    pub line_number: usize,
    pub key: String,
    pub kind: Option<String>,
    pub value: Option<String>,
    pub condition: Option<String>,
    pub access_modifier: AccessModifier,
}

impl KV {
    pub fn new(
        key: &str,
        mut kind: Option<String>,
        value: Option<String>,
        line_number: usize,
        condition: Option<String>,
    ) -> KV {
        let mut access_modifier = AccessModifier::PUBLIC;
        if let Some(k) = kind.as_ref() {
            let (rest_kind, access) = AccessModifier::get_kind_and_modifier(k.as_str());
            kind = Some(rest_kind);
            access_modifier = access.unwrap_or(AccessModifier::PUBLIC);
        }

        KV {
            line_number,
            key: key.to_string(),
            kind,
            value,
            condition,
            access_modifier,
        }
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum AccessModifier {
    PUBLIC,
    PRIVATE,
}

impl Default for AccessModifier {
    fn default() -> Self {
        AccessModifier::PUBLIC
    }
}

impl AccessModifier {
    pub fn is_public(&self) -> bool {
        matches!(self, AccessModifier::PUBLIC)
    }

    pub fn remove_modifiers(name: &str) -> String {
        let mut result = vec![];
        for part in name.split(' ') {
            if !AccessModifier::is_modifier(part) {
                result.push(part)
            }
        }
        result.join(" ").to_string()
    }

    pub fn is_modifier(s: &str) -> bool {
        matches!(s, "public" | "private")
    }

    pub fn get_modifier_from_string(modifier: &str) -> Option<AccessModifier> {
        match modifier {
            "public" => Some(AccessModifier::PUBLIC),
            "private" => Some(AccessModifier::PRIVATE),
            _ => None,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            AccessModifier::PUBLIC => "public".to_string(),
            AccessModifier::PRIVATE => "private".to_string(),
        }
    }

    pub fn get_kind_and_modifier(kind: &str) -> (String, Option<AccessModifier>) {
        let mut access_modifier: Option<AccessModifier> = None;

        let mut rest_kind = vec![];
        for part in kind.split(' ') {
            if !AccessModifier::is_modifier(part) {
                rest_kind.push(part);
                continue;
            }
            access_modifier = AccessModifier::get_modifier_from_string(part)
        }
        (rest_kind.join(" ").to_string(), access_modifier)
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct Section {
    pub line_number: usize,
    pub key: String,
    pub kind: Option<String>,
    pub section: Vec<ftd::p1::Section>,
    pub condition: Option<String>,
}

impl Header {
    pub(crate) fn from_caption(value: &str, line_number: usize) -> Header {
        Header::kv(
            line_number,
            ftd::p1::utils::CAPTION,
            None,
            Some(value.to_string()),
            None,
        )
    }

    pub(crate) fn kv(
        line_number: usize,
        key: &str,
        kind: Option<String>,
        value: Option<String>,
        condition: Option<String>,
    ) -> Header {
        Header::KV(KV::new(key, kind, value, line_number, condition))
    }

    pub(crate) fn section(
        line_number: usize,
        key: &str,
        kind: Option<String>,
        section: Vec<ftd::p1::Section>,
        condition: Option<String>,
    ) -> Header {
        Header::Section(Section {
            line_number,
            key: key.to_string(),
            kind,
            section,
            condition,
        })
    }

    pub fn without_line_number(&self) -> Self {
        use itertools::Itertools;

        match self {
            Header::KV(kv) => {
                let mut kv = (*kv).clone();
                kv.line_number = 0;
                Header::KV(kv)
            }
            Header::Section(s) => {
                let mut s = (*s).clone();
                s.line_number = 0;
                s.section = s
                    .section
                    .iter()
                    .map(|v| v.without_line_number())
                    .collect_vec();
                Header::Section(s)
            }
        }
    }

    pub(crate) fn get_key(&self) -> String {
        match self {
            Header::KV(ftd::p1::header::KV { key, .. })
            | Header::Section(ftd::p1::header::Section { key, .. }) => key.to_string(),
        }
    }

    pub(crate) fn get_access_modifier(&self) -> AccessModifier {
        match self {
            Header::KV(ftd::p1::header::KV {
                access_modifier, ..
            }) => access_modifier.clone(),
            Header::Section(ftd::p1::header::Section { .. }) => AccessModifier::PUBLIC,
        }
    }

    pub(crate) fn set_key(&mut self, value: &str) {
        match self {
            Header::KV(ftd::p1::header::KV { key, .. })
            | Header::Section(ftd::p1::header::Section { key, .. }) => {
                *key = value.to_string();
            }
        }
    }

    pub(crate) fn set_kind(&mut self, value: &str) {
        match self {
            Header::KV(ftd::p1::header::KV {
                kind: Some(kind), ..
            })
            | Header::Section(ftd::p1::header::Section {
                kind: Some(kind), ..
            }) => {
                *kind = value.to_string();
            }
            _ => {}
        }
    }

    pub(crate) fn get_value(&self, doc_id: &str) -> ftd::p1::Result<Option<String>> {
        match self {
            Header::KV(ftd::p1::header::KV { value, .. }) => Ok(value.to_owned()),
            Header::Section(_) => Err(ftd::p1::Error::ParseError {
                message: format!(
                    "Expected Header of type: KV, found: Section {}",
                    self.get_key()
                ),
                doc_id: doc_id.to_string(),
                line_number: self.get_line_number(),
            }),
        }
    }

    pub(crate) fn get_sections(&self, doc_id: &str) -> ftd::p1::Result<&Vec<ftd::p1::Section>> {
        match self {
            Header::KV(_) => Err(ftd::p1::Error::ParseError {
                message: format!(
                    "Expected Header of type: Sections, found: KV {}",
                    self.get_key()
                ),
                doc_id: doc_id.to_string(),
                line_number: self.get_line_number(),
            }),
            Header::Section(ftd::p1::header::Section { section, .. }) => Ok(section),
        }
    }

    pub(crate) fn get_line_number(&self) -> usize {
        match self {
            Header::KV(ftd::p1::header::KV { line_number, .. })
            | Header::Section(ftd::p1::header::Section { line_number, .. }) => *line_number,
        }
    }

    pub(crate) fn get_kind(&self) -> Option<String> {
        match self {
            Header::KV(ftd::p1::header::KV { kind, .. })
            | Header::Section(ftd::p1::header::Section { kind, .. }) => kind.to_owned(),
        }
    }

    pub(crate) fn get_condition(&self) -> Option<String> {
        match self {
            Header::KV(ftd::p1::header::KV { condition, .. })
            | Header::Section(ftd::p1::header::Section { condition, .. }) => condition.to_owned(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        match self {
            Header::KV(ftd::p1::header::KV { value, .. }) => value.is_none(),
            Header::Section(ftd::p1::header::Section { section, .. }) => section.is_empty(),
        }
    }

    pub fn remove_comments(&self) -> Option<Header> {
        let mut header = self.clone();
        let key = header.get_key().trim().to_string();
        if let Some(kind) = header.get_kind() {
            if kind.starts_with('/') {
                return None;
            }
            if key.starts_with(r"\/") {
                header.set_kind(kind.trim_start_matches('\\'));
            }
        } else {
            if key.starts_with('/') {
                return None;
            }

            if key.starts_with(r"\/") {
                header.set_key(key.trim_start_matches('\\'));
            }
        }

        match &mut header {
            Header::KV(ftd::p1::header::KV { .. }) => {}
            Header::Section(ftd::p1::header::Section { section, .. }) => {
                *section = section
                    .iter_mut()
                    .filter_map(|s| s.remove_comments())
                    .collect();
            }
        }
        Some(header)
    }
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Headers(pub Vec<Header>);

impl Headers {
    pub fn find(&self, key: &str) -> Vec<&ftd::p1::Header> {
        use itertools::Itertools;

        self.0.iter().filter(|v| v.get_key().eq(key)).collect_vec()
    }

    pub fn find_once(
        &self,
        key: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::p1::Result<&ftd::p1::Header> {
        let headers = self.find(key);
        let header = headers.first().ok_or(ftd::p1::Error::HeaderNotFound {
            key: key.to_string(),
            doc_id: doc_id.to_string(),
            line_number,
        })?;
        if headers.len() > 1 {
            return Err(ftd::p1::Error::MoreThanOneHeader {
                key: key.to_string(),
                doc_id: doc_id.to_string(),
                line_number: header.get_line_number(),
            });
        }
        Ok(header)
    }

    pub fn push(&mut self, item: ftd::p1::Header) {
        self.0.push(item)
    }

    /// returns a copy of Header after processing comments "/" and escape "\\/" (if any)
    ///
    /// only used by [`Section::remove_comments()`] and [`SubSection::remove_comments()`]
    ///
    /// [`SubSection::remove_comments()`]: ftd::p1::sub_section::SubSection::remove_comments
    /// [`Section::remove_comments()`]: ftd::p1::section::Section::remove_comments
    pub fn remove_comments(self) -> Headers {
        use itertools::Itertools;

        Headers(
            self.0
                .into_iter()
                .filter_map(|h| h.remove_comments())
                .collect_vec(),
        )
    }
}
