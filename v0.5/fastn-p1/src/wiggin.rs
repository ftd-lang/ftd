#![allow(dead_code)]

/// calls `inner_ender` for all the embedded section inside section in the
/// list and then calls `ender` for the list itself
pub fn ender(
    source: &str,
    o: &mut fastn_p1::ParseOutput,
    sections: Vec<fastn_p1::Section>,
) -> Vec<fastn_p1::Section> {
    // recursive part
    let sections = sections
        .into_iter()
        .map(|s| section_ender(source, o, s))
        .collect();

    // non recursive part
    inner_ender(source, o, sections)
}

fn section_ender(
    source: &str,
    o: &mut fastn_p1::ParseOutput,
    mut section: fastn_p1::Section,
) -> fastn_p1::Section {
    if let Some(caption) = section.caption {
        section.caption = Some(header_value_ender(source, o, caption));
    }
    section.headers = section
        .headers
        .into_iter()
        .map(|mut h| {
            h.value = header_value_ender(source, o, h.value);
            h
        })
        .collect();
    if let Some(body) = section.body {
        section.body = Some(header_value_ender(source, o, body));
    }
    section.children = ender(source, o, section.children);
    section
}

fn header_value_ender(
    source: &str,
    o: &mut fastn_p1::ParseOutput,
    header: fastn_p1::HeaderValue,
) -> fastn_p1::HeaderValue {
    header
        .into_iter()
        .map(|ses| match ses {
            fastn_p1::SES::String(span) => fastn_p1::SES::String(span),
            fastn_p1::SES::Expression {
                start,
                end,
                content,
            } => fastn_p1::SES::Expression {
                start,
                end,
                content: header_value_ender(source, o, content),
            },
            fastn_p1::SES::Section(sections) => fastn_p1::SES::Section(ender(source, o, sections)),
        })
        .collect()
}

/// converts a section list, with interleaved `-- end: <section-name>`, into a nested section list
///
/// example:
/// [{section: "foo"}, {section: "bar"}, "-- end: foo"] -> [{section: "foo", children: [{section: "bar"}]}]
fn inner_ender<T: SectionProxy>(
    source: &str,
    o: &mut fastn_p1::ParseOutput,
    sections: Vec<T>,
) -> Vec<T> {
    let mut stack = Vec::new();
    'outer: for mut section in sections {
        match section.name(source).unwrap() {
            Mark::Start(_name) => {
                stack.push(section);
            }
            Mark::End(e_name) => {
                let mut children = Vec::new();
                while let Some(candidate) = stack.pop() {
                    match candidate.name(source).unwrap() {
                        Mark::Start(name) => {
                            if name == e_name {
                                section.add_children(children);
                                stack.push(section);
                                continue 'outer;
                            } else {
                                children.push(candidate);
                            }
                        }
                        Mark::End(_name) => unreachable!("we never put section end on the stack"),
                    }
                }
                // we have run out of sections, and we have not found the section end, return
                // error, put the children back on the stack
                o.items.push(fastn_p1::Spanned {
                    span: section.span(),
                    value: fastn_p1::Item::Error(fastn_p1::SingleError::EndWithoutStart),
                });
                stack.extend(children.into_iter().rev());
            }
        }
    }
    stack
}

pub enum Mark<'input> {
    Start(&'input str),
    End(&'input str),
}

/// we are using a proxy trait so we can write tests against a fake type, and then implement the
/// trait for the real Section type
pub trait SectionProxy: Sized {
    /// returns the name of the section, and if it starts or ends the section
    fn name<'input>(
        &'input self,
        source: &'input str,
    ) -> Result<Mark<'input>, fastn_p1::SingleError>;
    fn add_children(&mut self, children: Vec<Self>);
    fn span(&self) -> fastn_p1::Span;
}

impl SectionProxy for fastn_p1::Section {
    fn name<'input>(
        &'input self,
        source: &'input str,
    ) -> Result<Mark<'input>, fastn_p1::SingleError> {
        let span = &self.init.name.name.name;
        let name = &source[span.start..span.end];
        if name != "end" {
            return Ok(Mark::Start(name));
        }

        let caption = match self.caption.as_ref() {
            Some(caption) => caption,
            None => return Err(fastn_p1::SingleError::SectionNameNotFoundForEnd),
        };

        let v = match (caption.get(0), caption.len()) {
            (Some(fastn_p1::SES::String(span)), 1) => &source[span.start..span.end].trim(),
            (Some(_), _) => return Err(fastn_p1::SingleError::EndContainsData),
            (None, _) => return Err(fastn_p1::SingleError::SectionNameNotFoundForEnd),
        };

        // if v is not a single word, we have a problem
        if v.contains(' ') || v.contains('\t') {
            // SES::String cannot contain new lines.
            return Err(fastn_p1::SingleError::EndContainsData);
        }

        Ok(Mark::End(v))
    }

    fn add_children(&mut self, children: Vec<Self>) {
        self.children = children;
    }

    fn span(&self) -> fastn_p1::Span {
        self.init.dashdash.clone()
    }
}

#[allow(dead_code)] // #[expect(dead_code)] is not working
struct DummySection {
    name: String,
    is_end: bool,
    children: Vec<DummySection>,
}

impl SectionProxy for DummySection {
    fn name<'input>(
        &'input self,
        _source: &'input str,
    ) -> Result<Mark<'input>, fastn_p1::SingleError> {
        if self.is_end {
            Ok(Mark::End(&self.name))
        } else {
            Ok(Mark::Start(&self.name))
        }
    }

    fn add_children(&mut self, children: Vec<Self>) {
        self.children = children;
    }

    fn span(&self) -> fastn_p1::Span {
        Default::default()
    }
}
