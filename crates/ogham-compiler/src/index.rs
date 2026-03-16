//! Pass 2: Index collection.
//!
//! Walks the typed AST and registers all top-level declarations
//! (types, enums, shapes, services, annotation defs) into arenas
//! and the symbol table. No references are resolved here — that
//! happens in subsequent passes.

use crate::ast::{self, AstNode};
use crate::diagnostics::Diagnostics;
use crate::hir::{self, Arenas, Interner, Loc, Sym, SymbolTable};
use crate::syntax_kind::SyntaxNode;

/// A parsed file ready for indexing.
pub struct ParsedFile {
    pub file_name: String,
    pub root: SyntaxNode,
    pub package: String,
}

fn make_loc(file: Sym, node: &SyntaxNode) -> Loc {
    let range = node.text_range();
    Loc {
        file: Some(file),
        span: usize::from(range.start())..usize::from(range.end()),
    }
}

/// Collect all declarations from a parsed file into arenas and symbol table.
pub fn collect(
    file: &ParsedFile,
    interner: &mut Interner,
    arenas: &mut Arenas,
    symbols: &mut SymbolTable,
    diag: &mut Diagnostics,
) {
    let root = match ast::Root::cast(file.root.clone()) {
        Some(r) => r,
        None => return,
    };

    let file_sym = interner.intern(&file.file_name);
    let pkg = &file.package;

    // Index types
    for type_decl in root.type_decls() {
        let name_text = match type_decl.name() {
            Some(t) => t.text().to_string(),
            None => continue,
        };
        let full = format!("{}.{}", pkg, name_text);
        let name_sym = interner.intern(&name_text);
        let full_sym = interner.intern(&full);

        let type_def = hir::TypeDef {
            name: name_sym,
            full_name: full_sym,
            fields: Vec::new(),
            oneofs: Vec::new(),
            nested_types: Vec::new(),
            nested_enums: Vec::new(),
            annotations: Vec::new(),
            back_references: Vec::new(),
            trace: None,
            loc: make_loc(file_sym, type_decl.syntax()),
        };

        let id = arenas.types.alloc(type_def);
        if symbols.types.insert(full_sym, id).is_some() {
            let range = type_decl.syntax().text_range();
            diag.error(
                &file.file_name,
                usize::from(range.start())..usize::from(range.end()),
                format!("duplicate type: {}", full),
            );
        }
    }

    // Index enums
    for enum_decl in root.enum_decls() {
        let name_text = match enum_decl.name() {
            Some(t) => t.text().to_string(),
            None => continue,
        };
        let full = format!("{}.{}", pkg, name_text);
        let name_sym = interner.intern(&name_text);
        let full_sym = interner.intern(&full);

        let loc = make_loc(file_sym, enum_decl.syntax());

        let mut values = vec![hir::EnumValueDef {
            name: interner.intern("Unspecified"),
            number: 0,
            is_removed: false,
            fallback: None,
            annotations: Vec::new(),
            loc: loc.clone(),
        }];

        for val in enum_decl.values() {
            let val_name = match val.name() {
                Some(t) => t.text().to_string(),
                None => continue,
            };
            let val_number = val.value().unwrap_or(0) as i32;
            values.push(hir::EnumValueDef {
                name: interner.intern(&val_name),
                number: val_number,
                is_removed: false,
                fallback: None,
                annotations: Vec::new(),
                loc: make_loc(file_sym, val.syntax()),
            });
        }

        let enum_def = hir::EnumDef {
            name: name_sym,
            full_name: full_sym,
            values,
            annotations: Vec::new(),
            loc,
        };

        let id = arenas.enums.alloc(enum_def);
        if symbols.enums.insert(full_sym, id).is_some() {
            let range = enum_decl.syntax().text_range();
            diag.error(
                &file.file_name,
                usize::from(range.start())..usize::from(range.end()),
                format!("duplicate enum: {}", full),
            );
        }
    }

    // Index shapes
    for shape_decl in root.shape_decls() {
        let name_text = match shape_decl.name() {
            Some(t) => t.text().to_string(),
            None => continue,
        };
        let full = format!("{}.{}", pkg, name_text);
        let name_sym = interner.intern(&name_text);
        let full_sym = interner.intern(&full);

        let type_params: Vec<Sym> = shape_decl
            .type_params()
            .map(|tp| {
                tp.params()
                    .iter()
                    .map(|t| interner.intern(t.text()))
                    .collect()
            })
            .unwrap_or_default();

        let includes: Vec<Sym> = shape_decl
            .includes()
            .iter()
            .flat_map(|inc| {
                inc.names()
                    .iter()
                    .map(|t| interner.intern(t.text()))
                    .collect::<Vec<_>>()
            })
            .collect();

        let shape_def = hir::ShapeDef {
            name: name_sym,
            full_name: full_sym,
            fields: Vec::new(),
            includes,
            type_params,
            annotations: Vec::new(),
            loc: make_loc(file_sym, shape_decl.syntax()),
        };

        let id = arenas.shapes.alloc(shape_def);
        if symbols.shapes.insert(full_sym, id).is_some() {
            let range = shape_decl.syntax().text_range();
            diag.error(
                &file.file_name,
                usize::from(range.start())..usize::from(range.end()),
                format!("duplicate shape: {}", full),
            );
        }
    }

    // Index services
    for svc_decl in root.service_decls() {
        let name_text = match svc_decl.name() {
            Some(t) => t.text().to_string(),
            None => continue,
        };
        let full = format!("{}.{}", pkg, name_text);
        let name_sym = interner.intern(&name_text);
        let full_sym = interner.intern(&full);

        let svc_def = hir::ServiceDef {
            name: name_sym,
            full_name: full_sym,
            rpcs: Vec::new(),
            annotations: Vec::new(),
            loc: make_loc(file_sym, svc_decl.syntax()),
        };

        let id = arenas.services.alloc(svc_def);
        if symbols.services.insert(full_sym, id).is_some() {
            let range = svc_decl.syntax().text_range();
            diag.error(
                &file.file_name,
                usize::from(range.start())..usize::from(range.end()),
                format!("duplicate service: {}", full),
            );
        }
    }

    // Index annotation definitions
    for ann_decl in root.annotation_decls() {
        let name_text = match ann_decl.name() {
            Some(t) => t.text().to_string(),
            None => continue,
        };
        let lib_sym = interner.intern(pkg);
        let name_sym = interner.intern(&name_text);
        let full = format!("{}::{}", pkg, name_text);
        let full_sym = interner.intern(&full);

        let targets: Vec<Sym> = ann_decl
            .targets()
            .map(|t| {
                t.targets()
                    .iter()
                    .map(|s| interner.intern(s))
                    .collect()
            })
            .unwrap_or_default();

        let ann_def = hir::AnnotationDef {
            library: lib_sym,
            name: name_sym,
            full_name: full_sym,
            targets,
            params: Vec::new(),
            compositions: Vec::new(),
            loc: make_loc(file_sym, ann_decl.syntax()),
        };

        let id = arenas.annotation_defs.alloc(ann_def);
        if symbols.annotations.insert((lib_sym, name_sym), id).is_some() {
            let range = ann_decl.syntax().text_range();
            diag.error(
                &file.file_name,
                usize::from(range.start())..usize::from(range.end()),
                format!("duplicate annotation: {}", full),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    fn index_source(source: &str) -> (Interner, Arenas, SymbolTable, Diagnostics) {
        let parse = parser::parse(source);
        assert!(parse.errors.is_empty(), "parse errors: {:?}", parse.errors);

        let root = parse.syntax();
        let pkg = {
            let r = ast::Root::cast(root.clone()).unwrap();
            r.package_decl()
                .and_then(|p| p.name().map(|t| t.text().to_string()))
                .unwrap_or_else(|| "default".to_string())
        };

        let file = ParsedFile {
            file_name: "test.ogham".to_string(),
            root,
            package: pkg,
        };

        let mut interner = Interner::default();
        let mut arenas = Arenas::default();
        let mut symbols = SymbolTable::default();
        let mut diag = Diagnostics::new();

        collect(&file, &mut interner, &mut arenas, &mut symbols, &mut diag);

        (interner, arenas, symbols, diag)
    }

    #[test]
    fn index_type() {
        let (int, arenas, symbols, diag) =
            index_source("package example;\ntype User { string email = 1; }");
        assert!(!diag.has_errors());
        let key = int.inner.get("example.User").unwrap();
        assert!(symbols.types.contains_key(&key));
        assert_eq!(arenas.types.len(), 1);
    }

    #[test]
    fn index_enum() {
        let (int, arenas, symbols, diag) =
            index_source("package example;\nenum Status { Active = 1; }");
        assert!(!diag.has_errors());
        let key = int.inner.get("example.Status").unwrap();
        assert!(symbols.enums.contains_key(&key));
        // Implicit Unspecified=0 + Active=1
        let id = symbols.enums[&key];
        assert_eq!(arenas.enums[id].values.len(), 2);
    }

    #[test]
    fn index_shape() {
        let (int, _arenas, symbols, diag) =
            index_source("package example;\nshape Timestamps { uint64 created_at; }");
        assert!(!diag.has_errors());
        let key = int.inner.get("example.Timestamps").unwrap();
        assert!(symbols.shapes.contains_key(&key));
    }

    #[test]
    fn index_service() {
        let (int, _arenas, symbols, diag) =
            index_source("package example;\nservice UserAPI { rpc Get(void) -> void; }");
        assert!(!diag.has_errors());
        let key = int.inner.get("example.UserAPI").unwrap();
        assert!(symbols.services.contains_key(&key));
    }

    #[test]
    fn index_annotation_def() {
        let (int, _arenas, symbols, diag) =
            index_source("package example;\nannotation Table for type { string table_name; }");
        assert!(!diag.has_errors());
        let lib = int.inner.get("example").unwrap();
        let name = int.inner.get("Table").unwrap();
        assert!(symbols.annotations.contains_key(&(lib, name)));
    }

    #[test]
    fn duplicate_type_error() {
        let (_, _, _, diag) = index_source(
            "package example;\ntype User { string a = 1; }\ntype User { string b = 1; }",
        );
        assert!(diag.has_errors());
    }

    #[test]
    fn multiple_declarations() {
        let (_, arenas, _, diag) = index_source(
            r#"package example;
type User { string email = 1; }
type Order { string id = 1; }
enum Status { Active = 1; }
shape Timestamps { uint64 created_at; }
service UserAPI { rpc Get(void) -> void; }
annotation Table for type { string name; }
"#,
        );
        assert!(!diag.has_errors());
        assert_eq!(arenas.types.len(), 2);
        assert_eq!(arenas.enums.len(), 1);
        assert_eq!(arenas.shapes.len(), 1);
        assert_eq!(arenas.services.len(), 1);
        assert_eq!(arenas.annotation_defs.len(), 1);
    }
}
