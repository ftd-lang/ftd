#[derive(Debug, Default)]
pub struct Symbols {}

impl Symbols {
    fn lookup(
        &mut self,
        interner: &mut string_interner::DefaultStringInterner,
        module: &fastn_unresolved::ModuleName,
    ) -> Vec<fastn_compiler::LookupResult> {
        // we need to fetch the symbol from the store
        let source = match std::fs::File::open(format!("{}.ftd", module.name.0))
            .and_then(std::io::read_to_string)
        {
            Ok(v) => v,
            Err(_e) => {
                return vec![];
            }
        };

        let d = fastn_unresolved::parse(module, &source);
        let s = interner.get_or_intern(&source);

        d.definitions
            .into_iter()
            .map(|d| match d {
                fastn_unresolved::UR::UnResolved(v) => {
                    fastn_compiler::LookupResult::Unresolved(s, v)
                }
                fastn_unresolved::UR::Resolved(_) => {
                    unreachable!(
                        "resolved definitions should not be present in the unresolved document"
                    )
                }
            })
            .collect()
    }
}

impl fastn_compiler::SymbolStore for Symbols {
    fn lookup(
        &mut self,
        interner: &mut string_interner::DefaultStringInterner,
        symbols: &[fastn_unresolved::SymbolName],
    ) -> Vec<fastn_compiler::LookupResult> {
        symbols
            .iter()
            .map(|s| &s.module)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .flat_map(|m| self.lookup(interner, m))
            .collect()
    }
}
