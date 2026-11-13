//! A game hook never waits on a lock a plugin thread can hold (playbook WORKFLOW, "Hooks never wait on a plugin
//! thread"). This test reads the shipped source under `src/` (no `cfg(test)`, `diag` or `coverage` code), finds every
//! entry the game or another plugin calls into, follows the call graph from each and collects every blocking lock it
//! reaches: `.lock()` on any mutex, `get_or_init` on a `OnceLock`, `.read()`/`.write()` on a static `RwLock`, or a
//! Horizon wait. `try_lock` is the allowed form. It also walks every plugin thread (the functions that spawn one, and
//! the exports) and marks a hook-reached lock `shared` when a plugin thread takes it too.
//!
//! `HOOK_LOCKS` lists the locks hooks reached when the test was added (2026-11-13, f33cfef), with `shared`: they are
//! known defects, not permissions. The test fails when a hook reaches a lock that is not listed, when a listed lock is
//! no longer reached (delete the row), and when `shared` disagrees with the scan.
//!
//! - **Entries:** every `#[skyline::hook]` function, and every `extern "C" fn`, classified by hand in `FOREIGN`
//!   (NVN procs and manual hooks the game calls, ImGui callbacks run inside the NVN hooks, exports other plugins call
//!   from their own hooks). A new `extern "C" fn` fails `every_foreign_entry_is_classified` until it is listed, and
//!   every function handed to `A64HookFunction` / `A64InlineHook` must be classified as a hook.
//! - **Edges:** path calls (`f(..)`, `module::f(..)`, `Type::f(..)`, `Self::f(..)`), functions handed over by name
//!   (`f as *const _`, `g(f)`), and method calls `.m(..)` to every scanned method named `m` (on `self`, only the
//!   enclosing type's). Closure bodies belong to the function that contains them. `#[cfg(test)]` and `#[test]` items
//!   are not read.
//! - **Limits:** calls through `dyn` objects and through function pointers kept in data are not seen; `MANUAL_EDGES`
//!   lists the ones the source has.
//! - **Exemptions:** `NOT_HOOK_PATHS` names functions that are reachable in the graph but run only on a plugin thread
//!   or at install, each with the reason; the walk does not enter them.
//!
//! `a_lock_planted_in_a_hook_path_is_found` runs the same scan on `tests/fixtures/hook_locks/`, where a hook reaches a
//! mutex through a helper and through a method call and an export initialises a `OnceLock`, and requires all three.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Tok {
    Ident(String),
    Punct(char),
    PathSep,
    Str(String),
    Lit,
}

fn lex(src: &str) -> Vec<Tok> {
    let b: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c.is_whitespace() {
            i += 1;
        } else if c == '/' && b.get(i + 1) == Some(&'/') {
            while i < b.len() && b[i] != '\n' {
                i += 1;
            }
        } else if c == '/' && b.get(i + 1) == Some(&'*') {
            let mut depth = 0;
            while i < b.len() {
                if b[i] == '/' && b.get(i + 1) == Some(&'*') {
                    depth += 1;
                    i += 2;
                } else if b[i] == '*' && b.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    i += 1;
                }
            }
        } else if (c == 'r' || c == 'b') && raw_string_start(&b, i).is_some() {
            let (hashes, start) = raw_string_start(&b, i).unwrap();
            let mut j = start;
            loop {
                if j >= b.len() {
                    break;
                }
                if b[j] == '"' && (0..hashes).all(|k| b.get(j + 1 + k) == Some(&'#')) {
                    break;
                }
                j += 1;
            }
            out.push(Tok::Str(b[start..j.min(b.len())].iter().collect()));
            i = j + 1 + hashes;
        } else if c == '"' || (c == 'b' && b.get(i + 1) == Some(&'"')) {
            let mut j = if c == 'b' { i + 2 } else { i + 1 };
            let start = j;
            while j < b.len() && b[j] != '"' {
                if b[j] == '\\' {
                    j += 1;
                }
                j += 1;
            }
            out.push(Tok::Str(b[start..j.min(b.len())].iter().collect()));
            i = j + 1;
        } else if c == '\'' || (c == 'b' && b.get(i + 1) == Some(&'\'')) {
            let q = if c == 'b' { i + 1 } else { i };
            // A char literal closes within a few characters; a lifetime does not.
            if b.get(q + 1) == Some(&'\\') {
                let mut j = q + 2;
                while j < b.len() && b[j] != '\'' {
                    j += 1;
                }
                out.push(Tok::Lit);
                i = j + 1;
            } else if b.get(q + 2) == Some(&'\'') {
                out.push(Tok::Lit);
                i = q + 3;
            } else {
                out.push(Tok::Punct('\''));
                i = q + 1;
            }
        } else if c.is_ascii_digit() {
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == '_' || (b[i] == '.' && b.get(i + 1).is_some_and(|d| d.is_ascii_digit()))) {
                i += 1;
            }
            out.push(Tok::Lit);
        } else if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < b.len() && (b[i].is_alphanumeric() || b[i] == '_') {
                i += 1;
            }
            out.push(Tok::Ident(b[start..i].iter().collect()));
        } else if c == ':' && b.get(i + 1) == Some(&':') {
            out.push(Tok::PathSep);
            i += 2;
        } else {
            out.push(Tok::Punct(c));
            i += 1;
        }
    }
    out
}

/// `(hash count, index of the first content char)` when a raw (byte) string starts at `i`.
fn raw_string_start(b: &[char], i: usize) -> Option<(usize, usize)> {
    let mut j = i;
    if b[j] == 'b' {
        j += 1;
    }
    if b.get(j) != Some(&'r') {
        return None;
    }
    if j > 0 && j != i + 1 && i > 0 && (b[i - 1].is_alphanumeric() || b[i - 1] == '_') {
        return None;
    }
    if i > 0 && (b[i - 1].is_alphanumeric() || b[i - 1] == '_') {
        return None;
    }
    j += 1;
    let mut hashes = 0;
    while b.get(j) == Some(&'#') {
        hashes += 1;
        j += 1;
    }
    (b.get(j) == Some(&'"')).then_some((hashes, j + 1))
}

/// Features the shipped NRO is built without (Cargo.toml: default = []); code under them never reaches the console.
const UNSHIPPED_FEATURES: &[&str] = &["diag", "coverage"];

/// Whether a `cfg(...)` predicate (the tokens inside the parentheses) is false in the shipped build: `test`, an
/// unshipped feature, or `any(...)` of only such terms.
fn cfg_unshipped(pred: &[Tok]) -> bool {
    match pred {
        [Tok::Ident(t)] if t == "test" => true,
        [Tok::Ident(f), Tok::Punct('='), Tok::Str(name)] if f == "feature" => UNSHIPPED_FEATURES.contains(&name.as_str()),
        [Tok::Ident(any), Tok::Punct('('), inner @ .., Tok::Punct(')')] if any == "any" => {
            let mut parts = Vec::new();
            let mut start = 0;
            let mut depth = 0;
            for (k, t) in inner.iter().enumerate() {
                match t {
                    Tok::Punct('(') => depth += 1,
                    Tok::Punct(')') => depth -= 1,
                    Tok::Punct(',') if depth == 0 => {
                        parts.push(&inner[start..k]);
                        start = k + 1;
                    }
                    _ => {}
                }
            }
            parts.push(&inner[start..]);
            parts.iter().filter(|p| !p.is_empty()).all(|p| cfg_unshipped(p))
        }
        _ => false,
    }
}

/// Drops items the shipped NRO does not contain: `#[test]`, `#[cfg(test)]` and `#[cfg(<unshipped>)]` items and
/// statements (attribute, item, body).
fn strip_tests(toks: Vec<Tok>) -> Vec<Tok> {
    let ident = |t: &Tok, s: &str| matches!(t, Tok::Ident(x) if x == s);
    let mut out = Vec::with_capacity(toks.len());
    let mut i = 0;
    while i < toks.len() {
        let is_test_attr = toks[i] == Tok::Punct('#')
            && toks.get(i + 1) == Some(&Tok::Punct('['))
            && ((toks.get(i + 2).is_some_and(|t| ident(t, "cfg"))
                && toks.get(i + 3) == Some(&Tok::Punct('('))
                && cfg_unshipped(&toks[i + 4..matching(&toks, i + 3, '(', ')')]))
                || (toks.get(i + 2).is_some_and(|t| ident(t, "test")) && toks.get(i + 3) == Some(&Tok::Punct(']'))));
        if !is_test_attr {
            out.push(toks[i].clone());
            i += 1;
            continue;
        }
        // Skip to the end of the item: a `;` or a balanced `{ }` at depth 0.
        let mut depth = 0i32;
        let mut j = i;
        // Past the attribute itself.
        while j < toks.len() && toks[j] != Tok::Punct(']') {
            j += 1;
        }
        j += 1;
        while j < toks.len() {
            match toks[j] {
                Tok::Punct('(') | Tok::Punct('[') => depth += 1,
                Tok::Punct(')') | Tok::Punct(']') => depth -= 1,
                Tok::Punct(';') if depth == 0 => break,
                Tok::Punct('{') if depth == 0 => {
                    let mut d = 0;
                    while j < toks.len() {
                        if toks[j] == Tok::Punct('{') {
                            d += 1;
                        } else if toks[j] == Tok::Punct('}') {
                            d -= 1;
                            if d == 0 {
                                break;
                            }
                        }
                        j += 1;
                    }
                    break;
                }
                _ => {}
            }
            j += 1;
        }
        i = j + 1;
    }
    out
}

#[derive(Debug)]
struct Func {
    file: String,
    module: Vec<String>,
    name: String,
    impl_type: Option<String>,
    receiver: bool,
    hook_attr: bool,
    extern_c: bool,
    /// Parameter names (they shadow functions of the same name).
    params: Vec<String>,
    body: Vec<Tok>,
}

impl Func {
    fn key(&self) -> String {
        let mut parts = self.module.clone();
        if let Some(t) = &self.impl_type {
            parts.push(t.clone());
        }
        parts.push(self.name.clone());
        parts.join("::")
    }
}

fn module_of(rel: &Path) -> Vec<String> {
    let mut parts: Vec<String> = rel.iter().map(|p| p.to_string_lossy().into_owned()).collect();
    let last = parts.pop().unwrap_or_default();
    let stem = last.trim_end_matches(".rs");
    if stem != "mod" && stem != "lib" {
        parts.push(stem.to_owned());
    }
    parts
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

fn matching(toks: &[Tok], open: usize, o: char, c: char) -> usize {
    let mut d = 0;
    for (k, t) in toks.iter().enumerate().skip(open) {
        if *t == Tok::Punct(o) {
            d += 1;
        } else if *t == Tok::Punct(c) {
            d -= 1;
            if d == 0 {
                return k;
            }
        }
    }
    toks.len() - 1
}

struct Graph {
    funcs: Vec<Func>,
    /// `use` imports per file: imported name -> full path segments.
    uses: BTreeMap<String, BTreeMap<String, Vec<String>>>,
    /// Static `RwLock`s by name.
    rwlocks: BTreeSet<String>,
    /// `file::STATIC` -> its type's last path segment, for `STATIC.method()` calls.
    static_types: BTreeMap<String, String>,
}

fn parse_file(rel: &str, module: &[String], toks: &[Tok], funcs: &mut Vec<Func>) {
    let ident = |k: usize| match toks.get(k) {
        Some(Tok::Ident(s)) => Some(s.as_str()),
        _ => None,
    };
    // Impl blocks: token ranges and their type.
    let mut impls: Vec<(usize, usize, String)> = Vec::new();
    for i in 0..toks.len() {
        if ident(i) != Some("impl") {
            continue;
        }
        let mut j = i + 1;
        let mut last_type = None;
        let mut angle = 0i32;
        while j < toks.len() && toks[j] != Tok::Punct('{') && toks[j] != Tok::Punct(';') {
            match &toks[j] {
                Tok::Punct('<') => angle += 1,
                Tok::Punct('>') => angle -= 1,
                Tok::Ident(s) if angle == 0 && s != "for" && s != "where" && s != "dyn" && s != "unsafe" => last_type = Some(s.clone()),
                Tok::Ident(s) if s == "where" => break,
                _ => {}
            }
            j += 1;
        }
        while j < toks.len() && toks[j] != Tok::Punct('{') && toks[j] != Tok::Punct(';') {
            j += 1;
        }
        if j < toks.len() && toks[j] == Tok::Punct('{') {
            let end = matching(toks, j, '{', '}');
            impls.push((j, end, last_type.unwrap_or_default()));
        }
    }
    for i in 0..toks.len() {
        if ident(i) != Some("fn") {
            continue;
        }
        let Some(name) = ident(i + 1) else { continue };
        // Qualifiers and attributes before `fn`.
        let mut k = i;
        let mut extern_c = false;
        while k > 0 {
            match &toks[k - 1] {
                Tok::Ident(s) if matches!(s.as_str(), "pub" | "unsafe" | "const" | "async" | "extern" | "crate" | "in" | "super" | "self") => k -= 1,
                Tok::Str(s) if s == "C" => {
                    extern_c = true;
                    k -= 1
                }
                Tok::Punct(')') => {
                    // `pub(crate)`
                    let mut m = k - 1;
                    while m > 0 && toks[m] != Tok::Punct('(') {
                        m -= 1;
                    }
                    k = m;
                }
                _ => break,
            }
        }
        let mut hook_attr = false;
        while k >= 2 && toks[k - 1] == Tok::Punct(']') {
            let mut m = k - 1;
            let mut d = 0;
            loop {
                if toks[m] == Tok::Punct(']') {
                    d += 1;
                } else if toks[m] == Tok::Punct('[') {
                    d -= 1;
                    if d == 0 {
                        break;
                    }
                }
                if m == 0 {
                    break;
                }
                m -= 1;
            }
            let attr = &toks[m..k];
            if attr.windows(3).any(|w| w[0] == Tok::Ident("skyline".into()) && w[1] == Tok::PathSep && w[2] == Tok::Ident("hook".into())) {
                hook_attr = true;
            }
            if m == 0 || toks[m - 1] != Tok::Punct('#') {
                break;
            }
            k = m - 1;
        }
        // Parameters and body.
        let Some(open) = (i + 2..toks.len()).find(|&j| toks[j] == Tok::Punct('(')) else { continue };
        let close = matching(toks, open, '(', ')');
        let params = &toks[open + 1..close];
        let first_param: Vec<&Tok> = params.iter().take_while(|t| **t != Tok::Punct(',')).collect();
        let receiver = first_param.iter().any(|t| **t == Tok::Ident("self".into())) && !first_param.iter().any(|t| **t == Tok::Punct(':'));
        let param_names: Vec<String> = (0..params.len())
            .filter(|&p| params.get(p + 1) == Some(&Tok::Punct(':')) && (p == 0 || params[p - 1] == Tok::Punct(',') || params[p - 1] == Tok::Ident("mut".into())))
            .filter_map(|p| match &params[p] {
                Tok::Ident(s) => Some(s.clone()),
                _ => None,
            })
            .collect();
        let mut j = close + 1;
        let mut depth = 0i32;
        while j < toks.len() {
            match toks[j] {
                Tok::Punct('(') | Tok::Punct('[') => depth += 1,
                Tok::Punct(')') | Tok::Punct(']') => depth -= 1,
                Tok::Punct('{') | Tok::Punct(';') if depth == 0 => break,
                _ => {}
            }
            j += 1;
        }
        if j >= toks.len() || toks[j] != Tok::Punct('{') {
            continue;
        }
        let end = matching(toks, j, '{', '}');
        let impl_type = impls.iter().filter(|(s, e, _)| *s < i && i < *e).max_by_key(|(s, _, _)| *s).map(|(_, _, t)| t.clone());
        funcs.push(Func {
            file: rel.to_owned(),
            module: module.to_vec(),
            name: name.to_owned(),
            impl_type,
            receiver,
            hook_attr,
            extern_c,
            params: param_names,
            body: toks[j..=end].to_vec(),
        });
    }
}

/// `use a::b::{c, d as e};` -> name -> path.
fn parse_uses(toks: &[Tok]) -> BTreeMap<String, Vec<String>> {
    let mut out = BTreeMap::new();
    let mut i = 0;
    while i < toks.len() {
        if toks[i] != Tok::Ident("use".into()) {
            i += 1;
            continue;
        }
        let end = (i..toks.len()).find(|&j| toks[j] == Tok::Punct(';')).unwrap_or(toks.len());
        collect_use(&toks[i + 1..end], &[], &mut out);
        i = end + 1;
    }
    out
}

fn collect_use(toks: &[Tok], prefix: &[String], out: &mut BTreeMap<String, Vec<String>>) {
    let mut path = prefix.to_vec();
    let mut i = 0;
    while i < toks.len() {
        match &toks[i] {
            Tok::Ident(s) if s == "as" => {
                if let Some(Tok::Ident(alias)) = toks.get(i + 1) {
                    out.insert(alias.clone(), path.clone());
                }
                return;
            }
            Tok::Ident(s) => path.push(s.clone()),
            Tok::PathSep => {}
            Tok::Punct('{') => {
                let close = matching(toks, i, '{', '}');
                let inner = &toks[i + 1..close];
                let mut start = 0;
                let mut depth = 0;
                for k in 0..=inner.len() {
                    let at_end = k == inner.len();
                    if !at_end {
                        match inner[k] {
                            Tok::Punct('{') => depth += 1,
                            Tok::Punct('}') => depth -= 1,
                            _ => {}
                        }
                    }
                    if at_end || (inner[k] == Tok::Punct(',') && depth == 0) {
                        collect_use(&inner[start..k], &path, out);
                        start = k + 1;
                    }
                }
                return;
            }
            _ => {}
        }
        i += 1;
    }
    if let Some(last) = path.last() {
        if last != "self" && last != "*" {
            out.insert(last.clone(), path.clone());
        } else if last == "self" && path.len() >= 2 {
            out.insert(path[path.len() - 2].clone(), path[..path.len() - 1].to_vec());
        }
    }
}

impl Graph {
    fn scan(src: &Path) -> Graph {
        let mut files = Vec::new();
        rust_files(src, &mut files);
        let mut funcs = Vec::new();
        let mut uses = BTreeMap::new();
        let mut rwlocks = BTreeSet::new();
        let mut static_types = BTreeMap::new();
        let excluded = unshipped_modules(src, &files);
        for path in files {
            let rel = path.strip_prefix(src).unwrap().to_string_lossy().replace('\\', "/");
            let module = module_of(Path::new(&rel));
            if excluded.iter().any(|m| module.starts_with(m)) {
                continue;
            }
            let toks = strip_tests(lex(&std::fs::read_to_string(&path).unwrap()));
            for w in toks.windows(4) {
                if let [Tok::Ident(kw), Tok::Ident(name), Tok::Punct(':'), _] = w {
                    if kw == "static" && toks_contains_rwlock_after(&toks, name) {
                        rwlocks.insert(name.clone());
                    }
                }
            }
            for k in 0..toks.len().saturating_sub(2) {
                if let (Tok::Ident(kw), Tok::Ident(name), Tok::Punct(':')) = (&toks[k], &toks[k + 1], &toks[k + 2]) {
                    if kw != "static" {
                        continue;
                    }
                    // `static NAME: a::b::Type<..> = ..`: the type's last segment before its generics.
                    let ty = toks[k + 3..]
                        .iter()
                        .take_while(|t| !matches!(t, Tok::Punct('<') | Tok::Punct('=') | Tok::Punct(';')))
                        .filter_map(|t| if let Tok::Ident(s) = t { Some(s.clone()) } else { None })
                        .last();
                    if let Some(ty) = ty {
                        static_types.insert(format!("{rel}::{name}"), ty);
                    }
                }
            }
            uses.insert(rel.clone(), parse_uses(&toks));
            parse_file(&rel, &module_of(Path::new(&rel)), &toks, &mut funcs);
        }
        Graph { funcs, uses, rwlocks, static_types }
    }

    fn find(&self, key: &str) -> Vec<usize> {
        (0..self.funcs.len()).filter(|&i| self.funcs[i].key() == key || self.funcs[i].key().ends_with(&format!("::{key}"))).collect()
    }

    /// Candidates for a call `segments(..)` made from `from`.
    fn resolve_path(&self, from: usize, segments: &[String]) -> Vec<usize> {
        let f = &self.funcs[from];
        let name = segments.last().unwrap();
        let named: Vec<usize> = (0..self.funcs.len()).filter(|&i| &self.funcs[i].name == name).collect();
        if named.is_empty() {
            return named;
        }
        let mut qual: Vec<String> = segments[..segments.len() - 1].to_vec();
        if qual.is_empty() {
            // Local function first (an associated fn is not callable unqualified), then a `use` import.
            let local: Vec<usize> = named.iter().copied().filter(|&i| self.funcs[i].file == f.file && self.funcs[i].impl_type.is_none()).collect();
            if !local.is_empty() {
                return local;
            }
            if let Some(path) = self.uses.get(&f.file).and_then(|u| u.get(name)) {
                qual = path[..path.len() - 1].to_vec();
            } else {
                // Nested fns and closures named like a function elsewhere: keep only free functions.
                return named.into_iter().filter(|&i| self.funcs[i].impl_type.is_none()).collect();
            }
        } else if let Some(path) = self.uses.get(&f.file).and_then(|u| u.get(&qual[0])) {
            let mut full = path.clone();
            full.extend(qual[1..].iter().cloned());
            qual = full;
        }
        // Type::f / Self::f
        let last_q = qual.last().cloned().unwrap_or_default();
        if last_q == "Self" {
            return named.into_iter().filter(|&i| self.funcs[i].file == f.file && self.funcs[i].impl_type == f.impl_type).collect();
        }
        if last_q.chars().next().is_some_and(|c| c.is_uppercase()) {
            return named.into_iter().filter(|&i| self.funcs[i].impl_type.as_deref() == Some(last_q.as_str())).collect();
        }
        // Module path: absolute after `crate`, relative after `self`/`super`.
        let mut module: Vec<String> = Vec::new();
        let mut rest = qual.as_slice();
        match rest.first().map(String::as_str) {
            Some("crate") => rest = &rest[1..],
            Some("self") => {
                module = f.module.clone();
                rest = &rest[1..];
            }
            Some("super") => {
                module = f.module.clone();
                while rest.first().map(String::as_str) == Some("super") {
                    module.pop();
                    rest = &rest[1..];
                }
            }
            _ => {}
        }
        module.extend(rest.iter().cloned());
        let exact: Vec<usize> = named.iter().copied().filter(|&i| self.funcs[i].module == module && self.funcs[i].impl_type.is_none()).collect();
        if !exact.is_empty() {
            return exact;
        }
        let suffix: Vec<usize> = named.iter().copied().filter(|&i| self.funcs[i].module.ends_with(&module) && self.funcs[i].impl_type.is_none()).collect();
        if !suffix.is_empty() {
            return suffix;
        }
        // An external crate (`core::ptr::read`, `skyline::nn::...`): no edge.
        Vec::new()
    }

    fn edges(&self, from: usize) -> BTreeSet<usize> {
        let f = &self.funcs[from];
        let toks = &f.body;
        let mut out = BTreeSet::new();
        let ident = |k: usize| match toks.get(k) {
            Some(Tok::Ident(s)) => Some(s.as_str()),
            _ => None,
        };
        let mut shadowed: BTreeSet<&str> = (0..toks.len())
            .filter(|&k| ident(k) == Some("let") || toks[k] == Tok::Punct('|'))
            .filter_map(|k| ident(k + 1).filter(|s| *s != "mut").or_else(|| ident(k + 2)))
            .collect();
        // Tuple patterns: every name in `let (a, mut b) = ..`.
        for k in (0..toks.len()).filter(|&k| ident(k) == Some("let") && toks.get(k + 1) == Some(&Tok::Punct('('))) {
            let close = matching(toks, k + 1, '(', ')');
            shadowed.extend((k + 2..close).filter_map(|j| ident(j).filter(|s| *s != "mut" && *s != "ref")));
        }
        shadowed.extend(f.params.iter().map(String::as_str));
        for k in 0..toks.len() {
            let Some(name) = ident(k) else { continue };
            let path_head = matches!(name, "self" | "Self" | "super" | "crate") && toks.get(k + 1) == Some(&Tok::PathSep);
            if KEYWORDS.contains(&name) && !path_head {
                continue;
            }
            let prev_dot = k > 0 && toks[k - 1] == Tok::Punct('.');
            let next = toks.get(k + 1);
            let call = next == Some(&Tok::Punct('('))
                || (next == Some(&Tok::PathSep) && toks.get(k + 2) == Some(&Tok::Punct('<')));
            if prev_dot {
                if !call || LOCK_METHODS.contains(&name) {
                    continue;
                }
                let on_self = k >= 2 && toks[k - 2] == Tok::Ident("self".into());
                // A static (`GLOBAL.clear()`) is one of ours: follow it even under a std name.
                let on_static = matches!(&toks[k - 2], Tok::Ident(r) if r.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'));
                if !on_self && !on_static && STD_METHODS.contains(&name) {
                    continue;
                }
                // A static of this file whose type is known: only that type's method.
                let static_type = match &toks[k - 2] {
                    Tok::Ident(r) if on_static => self.static_types.get(&format!("{}::{r}", f.file)),
                    _ => None,
                };
                for i in 0..self.funcs.len() {
                    let g = &self.funcs[i];
                    if g.name == name
                        && g.receiver
                        && (!on_self || (g.impl_type == f.impl_type && g.file == f.file))
                        && static_type.is_none_or(|t| g.impl_type.as_ref() == Some(t))
                    {
                        out.insert(i);
                    }
                }
                continue;
            }
            if k > 0 && (toks[k - 1] == Tok::PathSep) {
                continue; // handled from the path's first segment
            }
            if k > 0 && toks[k - 1] == Tok::Ident("fn".into()) {
                continue;
            }
            // A path `a::b::name`.
            let mut segs = vec![name.to_owned()];
            let mut m = k + 1;
            while toks.get(m) == Some(&Tok::PathSep) {
                match toks.get(m + 1) {
                    Some(Tok::Ident(s)) => {
                        segs.push(s.clone());
                        m += 2;
                    }
                    _ => break,
                }
            }
            if toks.get(m) == Some(&Tok::Punct('!')) {
                continue; // a macro; its arguments are scanned as tokens
            }
            let is_call = toks.get(m) == Some(&Tok::Punct('('));
            // A function handed over as a pointer: `f as *const _`, `module::f as _`, or a path argument `g(a::f)`.
            let cast = toks.get(m) == Some(&Tok::Ident("as".into())) && !shadowed.contains(name);
            let path_ref = !is_call && segs.len() > 1 && matches!(toks.get(m), Some(Tok::Punct(')')) | Some(Tok::Punct(',')));
            let reference = cast || path_ref;
            if is_call && segs.len() == 1 && shadowed.contains(name) {
                continue; // a local closure
            }
            if is_call || reference {
                out.extend(self.resolve_path(from, &segs));
            }
        }
        out.remove(&from);
        out
    }

    /// The blocking lock operations in `index`'s own body, by lock: `file::STATIC` for a static, `Type.field` for a
    /// field of `self`, `key:receiver` otherwise, with the operation (`lock`, `get_or_init`, ...).
    fn locks(&self, index: usize) -> Vec<(String, String)> {
        let f = &self.funcs[index];
        let toks = &f.body;
        let mut out = Vec::new();
        for k in 1..toks.len() {
            let Tok::Ident(name) = &toks[k] else { continue };
            let call = toks.get(k + 1) == Some(&Tok::Punct('('));
            let after_dot = toks[k - 1] == Tok::Punct('.');
            let receiver = if k >= 2 {
                match &toks[k - 2] {
                    Tok::Ident(s) => s.clone(),
                    Tok::Punct(')') => "(..)".into(),
                    t => format!("{t:?}"),
                }
            } else {
                String::new()
            };
            let blocking = after_dot
                && call
                && (name == "lock"
                    || name == "get_or_init"
                    || name == "get_or_try_init"
                    || ((name == "read" || name == "write") && self.rwlocks.contains(&receiver)));
            if blocking {
                let is_static = !receiver.is_empty() && receiver.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
                let on_self_field = k >= 4 && toks[k - 3] == Tok::Punct('.') && toks[k - 4] == Tok::Ident("self".into());
                let id = if is_static {
                    format!("{}::{receiver}", f.file)
                } else if on_self_field {
                    format!("{}.{receiver}", f.impl_type.clone().unwrap_or_default())
                } else {
                    format!("{}:{receiver}", f.key())
                };
                out.push((id, name.clone()));
            } else if call && WAITS.contains(&name.as_str()) {
                out.push((format!("{}:{name}", f.key()), name.clone()));
            }
        }
        out
    }
}

/// Module paths declared `#[cfg(test)]` or `#[cfg(<unshipped feature>)] mod name;`: their files are not read.
fn unshipped_modules(src: &Path, files: &[PathBuf]) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    for path in files {
        let rel = path.strip_prefix(src).unwrap();
        let toks = lex(&std::fs::read_to_string(path).unwrap());
        for i in 0..toks.len() {
            if toks[i] != Tok::Punct('#') || toks.get(i + 1) != Some(&Tok::Punct('[')) || toks.get(i + 2) != Some(&Tok::Ident("cfg".into())) {
                continue;
            }
            let close = matching(&toks, i + 3, '(', ')');
            if !cfg_unshipped(&toks[i + 4..close]) {
                continue;
            }
            let mut k = close + 2;
            if toks.get(k) == Some(&Tok::Ident("pub".into())) {
                k += 1;
                if toks.get(k) == Some(&Tok::Punct('(')) {
                    k = matching(&toks, k, '(', ')') + 1;
                }
            }
            if let (Some(Tok::Ident(m)), Some(Tok::Ident(name))) = (toks.get(k), toks.get(k + 1)) {
                if m == "mod" {
                    let mut module = module_of(rel);
                    module.push(name.clone());
                    out.push(module);
                }
            }
        }
    }
    out
}

fn toks_contains_rwlock_after(toks: &[Tok], name: &str) -> bool {
    let Some(at) = toks.windows(2).position(|w| w[0] == Tok::Ident("static".into()) && w[1] == Tok::Ident(name.into())) else { return false };
    toks[at..].iter().take_while(|t| **t != Tok::Punct('=') && **t != Tok::Punct(';')).any(|t| *t == Tok::Ident("RwLock".into()))
}

const KEYWORDS: &[&str] = &[
    "if", "while", "for", "match", "loop", "return", "let", "mut", "fn", "as", "in", "move", "unsafe", "ref", "Some", "None", "Ok", "Err",
    "Box", "self", "Self", "super", "crate", "where", "impl", "dyn", "true", "false", "break", "continue", "else", "const", "static",
];
/// Method names std types use on hook paths (files, collections, atomics, options). A call `x.open(..)` on anything
/// but `self` is taken as std's; `project_methods_with_std_names` lists the scanned methods this hides, each checked
/// by hand in `STD_NAMED_METHODS`.
const STD_METHODS: &[&str] = &[
    "open", "write", "read", "flush", "push", "pop", "get", "get_mut", "insert", "remove", "clear", "len", "is_empty", "iter", "iter_mut",
    "next", "take", "map", "unwrap", "clone", "contains", "extend", "drain", "load", "store", "swap", "set", "replace", "sync_all",
    "sync_data", "write_all", "as_ref", "as_mut", "find", "position", "join", "split", "fetch_add", "fetch_sub", "compare_exchange",
    "entry", "retain", "sort", "last", "first", "copy_from_slice", "fill", "truncate", "reserve", "with_capacity", "min", "max", "rotate_left",
];
const LOCK_METHODS: &[&str] = &["lock", "try_lock", "read", "write", "get_or_init", "get_or_try_init"];
/// Horizon and std waits that block on another thread.
const WAITS: &[&str] = &["LockMutex", "WaitEvent", "WaitConditionVariable", "AcquireSemaphore", "WaitSystemEvent", "WaitSynchronization"];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Class {
    /// Called on a game thread: a hook, an NVN proc the game calls through `get_proc`, or an ImGui callback the NVN
    /// hooks run.
    Hook,
    /// An export another plugin calls; it may call it from its own hook, so it counts as a hook path.
    Export,
    /// Not called by the game or another plugin's hook (the reason says what calls it).
    NotHook,
}

/// Every `extern "C" fn` under `src/` by `file::name`, with what calls it.
const FOREIGN: &[(&str, &str, Class, &str)] = &[
    ("runtime/nvn.rs", "get_proc", Class::Hook, "nvnDeviceGetProcAddress replacement; the game's render thread"),
    ("runtime/nvn.rs", "window_initialize", Class::Hook, "NVN proc handed out by get_proc"),
    ("runtime/nvn.rs", "window_finalize", Class::Hook, "NVN proc handed out by get_proc"),
    ("runtime/nvn.rs", "window_set_crop", Class::Hook, "NVN proc handed out by get_proc"),
    ("runtime/nvn.rs", "queue_present", Class::Hook, "NVN proc handed out by get_proc"),
    ("runtime/nvn.rs", "window_acquire", Class::Hook, "NVN proc handed out by get_proc"),
    ("runtime/nvn.rs", "sync_wait", Class::Hook, "NVN proc handed out by get_proc"),
    ("runtime/render_sequence.rs", "model_hook", Class::Hook, "A64HookFunction replacement"),
    ("runtime/fighter_main.rs", "on_fighter_main", Class::Hook, "A64InlineHook callback"),
    ("runtime/fighter_main.rs", "on_cut_in", Class::Hook, "A64HookFunction replacement"),
    ("runtime/fighter_probe.rs", "on_load", Class::Hook, "A64HookFunction replacement (module load)"),
    ("runtime/fighter_probe.rs", "on_unload", Class::Hook, "A64HookFunction replacement (module unload)"),
    ("runtime/overlay_host.rs", "deluxe_rewrite_nvn_bootstrap_hook", Class::Hook, "A64HookFunction replacement of nvnBootstrapLoader"),
    ("runtime/overlay_host.rs", "bootstrap_original", Class::Hook, "the far-chain original; runs inside the bootstrap hook"),
    ("runtime/overlay_host.rs", "on_context_setup", Class::Hook, "ImGui context-setup callback, run inside the NVN hook"),
    ("runtime/overlay_host.rs", "on_backend_ready", Class::Hook, "ImGui backend callback, run inside the NVN hook"),
    ("runtime/overlay_host.rs", "accept_external_context", Class::Hook, "the standalone's context callback, run inside its NVN hook"),
    ("runtime/overlay_host.rs", "external_draw", Class::Hook, "the standalone's draw callback, run inside its present hook"),
    ("runtime/overlay_host.rs", "backend_logger", Class::Hook, "ImGui backend log callback, run inside the NVN hooks"),
    ("runtime/ui.rs", "draw", Class::Hook, "the overlay draw callback, run inside the present hook"),
    ("runtime/plugin_export.rs", "ssbu_online_deluxe_plugin_api_v1", Class::Export, "plugin API table lookup"),
    ("runtime/plugin_export.rs", "register", Class::Export, "plugin registration through the API table"),
    ("runtime/plugin_export.rs", "ssbu_online_deluxe_online_context_v1", Class::Export, "online context; ssbu-fx-control reads it from its hooks"),
    ("runtime/plugin_export.rs", "ssbu_online_deluxe_overlay_state_v1", Class::Export, "overlay state export"),
    ("runtime/plugin_export.rs", "host_status", Class::Export, "overlay host table: status"),
    ("runtime/plugin_export.rs", "host_build", Class::Export, "overlay host table: build string"),
];

/// Install helpers that hand a parameter to `A64HookFunction`: `(file, parameter, the functions callers pass)`.
const HOOK_PARAMETERS: &[(&str, &str, &str)] = &[
    ("runtime/fighter_probe.rs", "replacement", "on_load, on_unload (both Hook rows above)"),
    ("runtime/overlay_host.rs", "replacement", "deluxe_rewrite_nvn_bootstrap_hook (a Hook row above)"),
];

/// Calls the text cannot show (dyn objects, function pointers kept in data): `(from key, to key, why)`.
const MANUAL_EDGES: &[(&str, &str, &str)] = &[];

/// Functions reachable in the graph that never run on a hook path: `(key, why)`. The walk does not enter them.
const NOT_HOOK_PATHS: &[(&str, &str)] = &[];

/// Every lock a hook path takes today, by lock: `(lock, shared, note)`. `shared` is true when a plugin thread (the
/// diagnostics thread, the game-aware client thread, or another plugin's thread through an export) also takes it,
/// which is the case the playbook forbids: a holder that is never scheduled again leaves the hook waiting for good.
/// Each row is a known defect to remove, not a permission; the test fails on a lock that is not listed (a new one) and
/// on a row whose lock no hook reaches any more (delete it), and checks `shared` against the scan.
const HOOK_LOCKS: &[(&str, bool, &str)] = &[
    // The first run on f33cfef (2026-11-13) found 12 locks, 11 shared with a plugin thread. Fixed the same day: the
    // Registry's plugins are published lock-free and its bookkeeping, PLUGIN_PAGES and TICKS are only tried on hook
    // paths (`hook_lock::try_take`); BUILD is a static C string; a plugin thread reads POLICY's published status and
    // hands scene resets to the hooks (`fighter_main::request_reset`), so GATE, POLICY and the adaptive state are
    // taken on game threads only; Ring.state was a false edge (`row.begin()`, renamed `start_tick`).
    // Taken only on game threads (hooks) in the scan: a wait between game threads, not on a plugin thread.
    ("resolution.rs::POLICY", false, "NVN procs, fighter Main / cut-in and render hooks -> resolution::with_policy (state update, no I/O under the lock)"),
    ("runtime/adaptive_drs.rs::STATE", false, "queue_present and the render hooks -> adaptive_drs::on_present; resets under GATE from hooks"),
    ("runtime/fighter_main.rs::GATE", false, "fighter Main / cut-in / module load and unload / present hooks"),
    ("runtime/fighter_probe.rs::STATE", false, "module load and unload hooks -> fighter_probe::publish/snapshot"),
];

struct Reach {
    /// lock -> (operation, entries that reach it, a shortest chain to the taker).
    locks: BTreeMap<String, (String, BTreeSet<String>, Vec<String>)>,
}

fn walk(graph: &Graph, starts: &[(usize, String)], extra: &BTreeMap<usize, Vec<usize>>, stop: &BTreeSet<usize>) -> Reach {
    let mut locks: BTreeMap<String, (String, BTreeSet<String>, Vec<String>)> = BTreeMap::new();
    for (start, label) in starts {
        // Breadth first, so the chain kept is a shortest one.
        let mut seen = BTreeSet::new();
        let mut queue = std::collections::VecDeque::from([(*start, vec![graph.funcs[*start].key()])]);
        while let Some((i, chain)) = queue.pop_front() {
            if !seen.insert(i) || (i != *start && stop.contains(&i)) {
                continue;
            }
            for (lock, op) in graph.locks(i) {
                let entry = locks.entry(lock).or_insert_with(|| (op, BTreeSet::new(), chain.clone()));
                entry.1.insert(label.clone());
                if chain.len() < entry.2.len() {
                    entry.2 = chain.clone();
                }
            }
            let mut next = graph.edges(i);
            next.extend(extra.get(&i).into_iter().flatten().copied());
            for c in next {
                let mut ch = chain.clone();
                ch.push(graph.funcs[c].key());
                queue.push_back((c, ch));
            }
        }
    }
    Reach { locks }
}

struct Scan {
    hooks: Reach,
    threads: Reach,
    hook_entries: Vec<String>,
    thread_entries: Vec<String>,
}

/// Hook entries: `#[skyline::hook]` functions and the `Hook`/`Export` rows of `foreign`. Plugin-thread entries:
/// functions that spawn a thread (the closure is part of the function), the functions a spawn hands over by name,
/// and the `Export` rows (other plugins' threads call them too).
fn scan(src: &Path, foreign: &[(&str, &str, Class, &str)], manual: &[(&str, &str, &str)], exempt: &[(&str, &str)]) -> Scan {
    let graph = Graph::scan(src);
    let mut hooks: Vec<(usize, String)> = Vec::new();
    let mut threads: Vec<(usize, String)> = Vec::new();
    for (i, f) in graph.funcs.iter().enumerate() {
        if f.hook_attr {
            hooks.push((i, format!("hook {}", f.key())));
        } else if f.extern_c {
            if let Some((_, _, class, _)) = foreign.iter().find(|(file, name, _, _)| *file == f.file && *name == f.name) {
                match class {
                    Class::Hook => hooks.push((i, format!("Hook {}", f.key()))),
                    Class::Export => {
                        hooks.push((i, format!("Export {}", f.key())));
                        threads.push((i, format!("Export {}", f.key())));
                    }
                    Class::NotHook => {}
                }
            }
        }
        let spawns = f.body.windows(2).any(|w| w[0] == Tok::Ident("spawn".into()) && w[1] == Tok::Punct('('));
        if spawns {
            threads.push((i, format!("thread {}", f.key())));
        }
    }
    let mut extra: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (from, to, _) in manual {
        for a in graph.find(from) {
            extra.entry(a).or_default().extend(graph.find(to));
        }
    }
    let stop: BTreeSet<usize> = exempt.iter().flat_map(|(key, _)| graph.find(key)).collect();
    Scan {
        hooks: walk(&graph, &hooks, &extra, &stop),
        threads: walk(&graph, &threads, &extra, &BTreeSet::new()),
        hook_entries: hooks.into_iter().map(|(_, l)| l).collect(),
        thread_entries: threads.into_iter().map(|(_, l)| l).collect(),
    }
}

fn src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

#[test]
fn no_new_lock_is_reachable_from_a_hook() {
    let result = scan(&src(), FOREIGN, MANUAL_EDGES, NOT_HOOK_PATHS);
    println!("{} hook entries, {} plugin-thread entries: {:?}", result.hook_entries.len(), result.thread_entries.len(), result.thread_entries);
    assert!(result.hook_entries.len() >= 40, "too few hook entries: {:?}", result.hook_entries);
    assert!(result.thread_entries.iter().any(|e| e.contains("logging::start")), "the diagnostics thread is not an entry: {:?}", result.thread_entries);
    let mut problems = Vec::new();
    for (lock, (op, entries, chain)) in &result.hooks.locks {
        let shared = result.threads.locks.contains_key(lock);
        println!("HOOK-LOCK {lock} .{op}() shared={shared} from {} hook entries, e.g. {}", entries.len(), chain.join(" -> "));
        if let Some((_, threads, thread_chain)) = result.threads.locks.get(lock) {
            println!("  thread side: {} entries {:?}, e.g. {}", threads.len(), threads, thread_chain.join(" -> "));
        }
        match HOOK_LOCKS.iter().find(|(l, _, _)| l == lock) {
            None => problems.push(format!("new: {lock} .{op}() (shared={shared}) reached by {} hook entries, e.g. {}", entries.len(), chain.join(" -> "))),
            Some((_, listed, _)) if *listed != shared => problems.push(format!("{lock}: listed shared={listed}, the scan says {shared}")),
            Some(_) => {}
        }
    }
    for (lock, _, _) in HOOK_LOCKS {
        if !result.hooks.locks.contains_key(*lock) {
            problems.push(format!("{lock} is no longer reached from a hook: delete its HOOK_LOCKS row"));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn every_foreign_entry_is_classified() {
    let graph = Graph::scan(&src());
    let mut missing = Vec::new();
    for f in graph.funcs.iter().filter(|f| f.extern_c) {
        if !FOREIGN.iter().any(|(file, name, _, _)| *file == f.file && *name == f.name) {
            missing.push(format!("{}::{}", f.file, f.name));
        }
    }
    assert!(missing.is_empty(), "extern \"C\" fns not classified in FOREIGN: {missing:?}");
    let stale: Vec<&str> = FOREIGN
        .iter()
        .filter(|(file, name, _, _)| !graph.funcs.iter().any(|f| f.extern_c && f.file == *file && f.name == *name))
        .map(|(_, name, _, _)| *name)
        .collect();
    assert!(stale.is_empty(), "FOREIGN rows with no such extern \"C\" fn: {stale:?}");
    for (key, _) in NOT_HOOK_PATHS {
        assert!(!graph.find(key).is_empty(), "NOT_HOOK_PATHS row {key} names no function");
    }
    for (from, to, _) in MANUAL_EDGES {
        assert!(!graph.find(from).is_empty() && !graph.find(to).is_empty(), "MANUAL_EDGES row {from} -> {to} names no function");
    }
}

/// Every function handed to `A64HookFunction` / `A64InlineHook` is classified as a hook.
#[test]
fn every_manual_hook_is_classified_as_a_hook() {
    let mut files = Vec::new();
    rust_files(&src(), &mut files);
    let mut handed = Vec::new();
    for path in files {
        let rel = path.strip_prefix(src()).unwrap().to_string_lossy().replace('\\', "/");
        let toks = strip_tests(lex(&std::fs::read_to_string(&path).unwrap()));
        for k in 0..toks.len() {
            let Tok::Ident(name) = &toks[k] else { continue };
            if (name == "A64HookFunction" || name == "A64InlineHook") && toks.get(k + 1) == Some(&Tok::Punct('(')) {
                let close = matching(&toks, k + 1, '(', ')');
                // The second argument's first identifier.
                let args = &toks[k + 2..close];
                let comma = args.iter().position(|t| *t == Tok::Punct(',')).unwrap_or(args.len());
                let second: Vec<&Tok> = args[(comma + 1).min(args.len())..].iter().take_while(|t| **t != Tok::Punct(',')).collect();
                if let Some(Tok::Ident(f)) = second.iter().find(|t| matches!(t, Tok::Ident(s) if s != "as")) {
                    handed.push((rel.clone(), f.clone()));
                }
            }
        }
    }
    assert!(handed.len() >= 5, "manual hooks found: {handed:?}");
    for (file, f) in &handed {
        if HOOK_PARAMETERS.iter().any(|(ff, p, _)| ff == file && p == f) {
            continue;
        }
        let row = FOREIGN.iter().find(|(ff, n, _, _)| ff == file && n == f);
        assert!(
            row.is_some_and(|(_, _, c, _)| *c == Class::Hook),
            "{file}: {f} is handed to A64HookFunction/A64InlineHook but is not a Hook in FOREIGN (found {handed:?})"
        );
    }
}

/// Scanned methods named like a std method that reach a blocking lock themselves. A call `x.name(..)` on anything but
/// `self` is not followed to them (`STD_METHODS`), so each is listed with how its callers were checked. Methods with
/// std names that reach no lock lose nothing when the edge is missed.
const STD_NAMED_METHODS: &[(&str, &str)] = &[
    (
        "capture::Ring::drain",
        "shipped callers are statics, which the scan follows (runtime/capture.rs CALLBACKS.drain, TRACE.drain); \
         `ring.drain(..)` is in capture.rs's tests only (2026-11-13)",
    ),
    (
        "perf_scaler::Scaler::clear",
        "shipped caller is the static GLOBAL.clear() (followed); `scaler.clear()` is in perf_scaler.rs's tests only (2026-11-13)",
    ),
];

#[test]
fn project_methods_with_std_names() {
    let graph = Graph::scan(&src());
    let mut with_locks = BTreeMap::new();
    for (i, f) in graph.funcs.iter().enumerate() {
        if f.receiver && STD_METHODS.contains(&f.name.as_str()) {
            let reach = walk(&graph, &[(i, f.key())], &BTreeMap::new(), &BTreeSet::new());
            if !reach.locks.is_empty() {
                with_locks.insert(f.key(), reach.locks.keys().cloned().collect::<Vec<_>>());
            }
        }
    }
    let listed: BTreeSet<String> = STD_NAMED_METHODS.iter().map(|(k, _)| (*k).to_owned()).collect();
    let found: BTreeSet<String> = with_locks.keys().cloned().collect();
    assert_eq!(found, listed, "std-named methods that reach a lock changed ({with_locks:?}); check their callers and list each");
}

fn fixture_scan() -> Scan {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hook_locks");
    let foreign = [
        ("runtime/hooks.rs", "on_present", Class::Hook, "fixture"),
        ("runtime/hooks.rs", "thread_main", Class::NotHook, "fixture: a plugin thread"),
    ];
    scan(&fixture, &foreign, &[], &[("state::install_only", "fixture: install time only")])
}

#[test]
fn a_lock_planted_in_a_hook_path_is_found() {
    let result = fixture_scan();
    let found: BTreeSet<&str> = result.hooks.locks.keys().map(String::as_str).collect();
    // Through a helper, through a method call on another type, and a OnceLock initialiser in an extern "C" entry.
    assert!(found.contains("state.rs::STATE"), "helper path missed: {found:?}");
    assert!(found.contains("Ring.rows"), "method-call path missed: {found:?}");
    assert!(found.contains("runtime/hooks/shared.rs::CELL"), "OnceLock init missed: {found:?}");
    // `let (level, ..)` shadows the imported `state::level`: `level as u64` is a cast, not a handed-over function.
    assert!(!found.contains("state.rs::OTHER"), "tuple-pattern local taken for a function: {found:?}");
    assert_eq!(found.len(), 3, "only the planted locks: {found:?}");
    let chains: Vec<String> = result.hooks.locks.values().map(|(_, _, c)| c.join(" -> ")).collect();
    assert!(chains.iter().all(|c| !c.contains("try_peek") && !c.contains("install_only") && !c.contains("thread_only")), "{chains:?}");
    assert!(chains.iter().any(|c| c.ends_with("state::bump")), "{chains:?}");
    assert_eq!(result.hook_entries.len(), 2, "entries: {:?}", result.hook_entries);
}

#[test]
fn the_fixture_s_shared_lock_is_marked_shared() {
    // `thread_only` runs on the fixture's plugin thread (`spawn_worker` spawns it) and takes STATE: shared.
    let result = fixture_scan();
    assert!(result.threads.locks.contains_key("state.rs::STATE"), "{:?}", result.threads.locks.keys().collect::<Vec<_>>());
    assert!(!result.threads.locks.contains_key("Ring.rows"));
}

#[test]
fn the_lexer_skips_strings_comments_and_lifetimes() {
    let toks = lex("fn a<'b>(x: &'b str) { let s = \"STATE.lock()\"; // M.lock()\n /* N.lock() */ let r = r#\"Q.lock()\"#; c('x'); }");
    let idents: Vec<String> = toks.iter().filter_map(|t| if let Tok::Ident(s) = t { Some(s.clone()) } else { None }).collect();
    assert!(!idents.contains(&"STATE".to_owned()) && !idents.contains(&"M".to_owned()) && !idents.contains(&"N".to_owned()) && !idents.contains(&"Q".to_owned()), "{idents:?}");
    assert!(idents.contains(&"c".to_owned()));
}

#[test]
fn unshipped_cfg_items_are_dropped() {
    let toks = strip_tests(lex("fn a() { #[cfg(feature = \"diag\")] b(); c(); } #[cfg(any(test, feature = \"coverage\"))] fn d() { e(); } #[cfg(feature = \"other\")] fn f() {}"));
    let idents: Vec<String> = toks.iter().filter_map(|t| if let Tok::Ident(s) = t { Some(s.clone()) } else { None }).collect();
    assert!(!idents.contains(&"b".to_owned()) && !idents.contains(&"e".to_owned()), "{idents:?}");
    assert!(idents.contains(&"c".to_owned()) && idents.contains(&"f".to_owned()), "{idents:?}");
}
