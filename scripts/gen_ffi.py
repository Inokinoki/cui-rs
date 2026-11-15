#!/usr/bin/env python3
"""Generates the second-wave gallery FFI + thin safe methods from the
amalgamated CUI release header.

    python3 scripts/gen_ffi.py

The manifest (scripts/ffi-manifest.txt) lists the C entry points the binding
needs - CUI_API functions or macro forms (cui_button -> cui_button_ex;
cui_dock -> cui_dock_begin + cui_dock_end). Referenced typedefs are pulled in
automatically. Types the hand-written layers already define are reused.

Outputs:
    cui-sys/src/gen.rs         #[repr(C)] structs, enum constants, extern decls
    cui/src/gen_safe.rs        thin safe methods on Ui (strings via the frame
                               arena, buffers via &mut String, scopes via
                               closures), plus `Ui::cstr` for raw opts structs
    cui-sys/probe/gen_probe.c  SZ()/OFF() lines for every Rust FFI struct
    cui-sys/tests/link.rs      a typed fn-pointer reference per extern decl;
                               a symbol the C library does not export fails
                               the test link (catches rename typos)
    cui-sys/tests/layout.rs    size_of/offset_of checks against the probe table

After changing the manifest: re-run this script, then ./vendor.sh --fetch
(regenerates the probe table), then cargo build.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HEADER = ROOT / "cui-sys/c/cui.h"
MANIFEST = ROOT / "scripts/ffi-manifest.txt"
OUT_FFI = ROOT / "cui-sys/src/gen.rs"
OUT_SAFE = ROOT / "cui/src/gen_safe.rs"
OUT_PROBE = ROOT / "cui-sys/probe/gen_probe.c"

PRIM = {"bool": "bool", "float": "f32", "double": "f64", "int": "i32",
        "unsigned": "u32", "unsigned int": "u32", "size_t": "usize",
        "cui_font": "i32", "cui_id": "u32", "cui_texture": "usize"}


def strip_comments(text):
    """Blanks /* */ and // comments with same-length whitespace (newlines
    kept), so every offset in the result is also an offset in the original
    text. Returns (blanked text, [(start, end-of-body, body)]) for /* */."""
    out, comments, i, n = [], [], 0, len(text)

    def blank(a, b):
        return "".join("\n" if c == "\n" else " " for c in text[a:b])

    while i < n:
        if text.startswith("/*", i):
            j = text.find("*/", i + 2)
            j = n if j < 0 else j
            comments.append((i, j, text[i + 2:j].strip()))
            out.append(blank(i, min(j + 2, n)))
            i = j + 2
        elif text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            out.append(blank(i, j))
            i = j
        elif text[i] == '"':
            j = i + 1
            while j < n and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            out.append(text[i:j + 1])
            i = j + 1
        else:
            out.append(text[i])
            i += 1
    return "".join(out), comments


def doc_before(comments, pos, text, limit=600):
    """The comment directly above the declaration at `pos`: only whitespace
    (blanked comments included) may lie between its end and `pos`."""
    best = ""
    for start, end, c in comments:
        if end + 2 <= pos and pos - end < limit and not c.startswith("==") \
                and not c.startswith("Generated") and len(c) > 3 \
                and text[end + 2:pos].strip() == "":
            best = c
    return best


def split_params(params):
    params = params.strip()
    if not params:
        return []
    out, depth, cur = [], 0, ""
    for ch in params:
        if ch in "([":
            depth += 1
        elif ch in ")]":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur.strip())
    return out


def split_ctype(text):
    """'const char *const *' -> ('char', base_const, [const-after-star, ...]).
    The const that follows a star qualifies that pointer itself, so it is the
    pointee constness of the NEXT star outward."""
    toks = re.findall(r"\*|\w+", text)
    base, base_const, after = [], False, []
    for t in toks:
        if t == "*":
            after.append(False)
        elif t == "const":
            if after:
                after[-1] = True
            else:
                base_const = True
        elif after:
            raise SystemExit(f"unsupported C type: {text!r}")
        else:
            base.append(t)
    return " ".join(base), base_const, after


class P(tuple):
    """(core type, name, kind) with the pieces the emitters need on top.

    kind: "value" | "const_ptr" | "mut_ptr" (one level; core is the pointee
    with const stripped) | "multi" (two or more levels; `ctext` keeps every
    qualifier) | "fnptr" (`ret (*name)(args)`; `ret`/`args` hold the text)."""

    def __new__(cls, core, name, kind, ctext="", ret="", args=""):
        o = super().__new__(cls, (core, name, kind))
        o.ctext, o.ret, o.args = ctext, ret, args
        return o


def parse_param(p):
    """Parses one parameter or field declaration; None when it does not parse
    (callers that must not drop things go through `must_param`)."""
    p = " ".join(p.split())
    # function pointers: "void (*fn)(cui_ctx *ui, void *data)"
    m = re.match(r"^(.+?)\(\s*\*\s*(\w+)\s*\)\s*\((.*)\)$", p)
    if m:
        return P(p, m.group(2), "fnptr", ret=m.group(1).strip(), args=m.group(3))
    # array declarators: "char key[64]" / "bool on[CUI_FLAG_ENVS]" — the array
    # suffix travels with the type so the struct emitter can size it
    m = re.match(r"(.+?)\b(\w+)\s*((?:\[\s*\w+\s*\])+)$", p)
    if m:
        ptype, name, arr = m.group(1).strip(), m.group(2), re.sub(r"\s", "", m.group(3))
        core = re.sub(r"\bconst\b", "", ptype).strip()
        return P(f"{core}{arr}", name, "value")
    m = re.match(r"(.+?)\b(\w+)$", p)
    if not m:
        return None
    ptype, name = m.group(1).strip(), m.group(2)
    if ptype.endswith("*"):
        base, base_const, after = split_ctype(ptype)
        if len(after) >= 2:
            return P(base, name, "multi", ctext=ptype)
        return P(base, name, "const_ptr" if base_const else "mut_ptr", ctext=ptype)
    core = re.sub(r"\bconst\b", "", ptype).strip()
    return P(core, name, "value")


def must_param(p, where):
    r = parse_param(p)
    if r is None:
        raise SystemExit(f"cannot parse {where}: {p!r}")
    return r


def expand_field(f):
    """Expands C comma declarators: "int a, b" / "char *a, *b" -> two fields."""
    f = f.strip().rstrip(";").strip()
    if "," not in f:
        return [f]
    m = re.match(r"^(.*?\**)\s*(\w+)\s*(\[[^\]]*\])?,(.*)$", f)
    if not m:
        return [f]
    base = m.group(1).rstrip()
    first = m.group(2) + (m.group(3) or "")
    spec = re.sub(r"\*+\s*$", "", base).rstrip()
    out = [f"{base} {first}"]
    rest = m.group(4).strip()
    while rest:
        m2 = re.match(r"^(\**)\s*(\w+)\s*(\[[^\]]*\])?,?(.*)$", rest)
        if not m2:
            return [f]
        stars = m2.group(1)
        t = (spec + " " + stars).strip() if stars else spec
        out.append(f"{t} {m2.group(2)}{m2.group(3) or ''}")
        rest = m2.group(4).strip()
        if not rest:
            break
    return out


class Header:
    def __init__(self, text):
        self.text, self.comments = strip_comments(text)
        self.structs = {}
        self.enums = {}
        self.functions = {}
        self.macros = {}
        self.defines = {}
        self._parse()

    def _parse(self):
        t = self.text
        for m in re.finditer(r"typedef (struct|enum) (?:cui_\w+ )?\{([^{}]*?)\}\s*(\w+)\s*;", t):
            self._add(m.group(1), m.group(2), m.group(3), m.start())
        for m in re.finditer(r"^\s*enum\s*\{([^{}]*?)\}\s*;", t, re.M):
            self._add("enum", m.group(1), f"anon{m.start()}", m.start())
        for m in re.finditer(r"CUI_API\s+(.*?)\b(cui_\w+)\s*\(([^;]*?)\)\s*(?:CUI_PRINTF\([^)]*\))?\s*;", t):
            if m.group(2) not in self.functions:
                self.functions[m.group(2)] = (m.group(1).strip(), m.group(3), m.start())
        for m in re.finditer(r"#define (cui_\w+)\(([^)]*)\)\s+((?:[^\\\n]|\\\n)*)", t):
            self.macros[m.group(1)] = m.group(3)
        # integer #defines, e.g. CUI_FLAG_ENVS: array sizes in struct fields
        for m in re.finditer(r"#define[ \t]+(CUI_\w+)[ \t]+(\d+)[ \t]*$", t, re.M):
            self.defines[m.group(1)] = m.group(2)
        self._resolve_enums()

    def _resolve_enums(self):
        """enum_vals: {enum name: [(member, value)]}; const_env: member -> value.
        Initializers are evaluated (ints, shifts, |, +, -, parens, earlier
        members and integer #defines); anything else is a hard error."""
        self.const_env = {k: int(v) for k, v in self.defines.items()}
        self.enum_vals = {}
        for name, (entries, _pos) in sorted(self.enums.items(), key=lambda kv: kv[1][1]):
            v, out = 0, []
            for mem, expr in entries:
                if expr:
                    v = self.eval_const(expr.lstrip("=").strip(), mem)
                out.append((mem, v))
                self.const_env[mem] = v
                v += 1
            self.enum_vals[name] = out

    def eval_const(self, expr, where):
        e = re.sub(r"\b(0[xX][0-9a-fA-F]+|\d+)[uUlL]+\b", r"\1", expr)
        if not re.fullmatch(r"[\w\s()<>|&+\-*~^]+", e):
            raise SystemExit(f"enum {where}: unsupported initializer {expr!r}")

        def sub(m):
            w = m.group(0)
            if w[0].isdigit():
                return w
            if w not in self.const_env:
                raise SystemExit(f"enum {where}: unknown name {w!r} in {expr!r}")
            return str(self.const_env[w])

        try:
            return int(eval(re.sub(r"\b\w+\b", sub, e), {"__builtins__": {}}, {}))
        except SyntaxError:
            raise SystemExit(f"enum {where}: cannot evaluate {expr!r}")

    def _add(self, kind, body, name, pos):
        if kind == "enum":
            entries = []
            for em in re.finditer(r"(CUI_\w+)\s*(=\s*[^,\n}]+)?\s*(?=[,\n}]|$)", body):
                entries.append((em.group(1), em.group(2).strip() if em.group(2) else None))
            self.enums[name] = (entries, pos)
        else:
            fields = [" ".join(fm.group(1).split()) for fm in re.finditer(r"([^;{}]+);", body)]
            self.structs[name] = (fields, pos)


def doc_first_line(doc, limit=130):
    """First line of a header comment for a doc line, with `[` escaped so
    C prose like `values[i]` cannot be parsed as an intra-doc link."""
    line = doc.splitlines()[0].strip()[:limit] if doc else ""
    return line.replace("[", "\\[")


def snake_to_camel(snake):
    return snake  # methods stay snake_case


# Rust keywords the struct emitter suffixes with `_` (kept in sync with the
# emitters below), plus the `_` -> `_pad` special case.
RUST_KEYWORDS = ("in", "loop", "type", "fn", "match", "ref", "box", "move",
                 "impl", "as", "use", "mod", "true", "false")


def rust_field_name(cname):
    if cname == "_":
        return "_pad"
    return cname + "_" if cname in RUST_KEYWORDS else cname


def mask_rust_noise(text):
    """Blanks comments and string literals (keeping length and newlines) so
    delimiter matching and declaration search cannot be confused by prose
    with unbalanced brackets."""
    out = list(text)
    i, n = 0, len(text)
    while i < n:
        if text.startswith("//", i):
            j = text.find("\n", i)
            j = n if j < 0 else j
            for k in range(i, j):
                out[k] = " "
            i = j
        elif text.startswith("/*", i):
            j = text.find("*/", i + 2)
            j = n if j < 0 else j + 2
            for k in range(i, j):
                if out[k] != "\n":
                    out[k] = " "
            i = j
        elif text[i] == '"':
            j = i + 1
            while j < n and text[j] != '"':
                j += 2 if text[j] == "\\" else 1
            for k in range(i, min(j + 1, n)):
                if out[k] != "\n":
                    out[k] = " "
            i = j + 1
        else:
            i += 1
    return "".join(out)


def _match_delimiter(text, open_idx):
    """Index of the delimiter matching text[open_idx] ('{', '(' or '[')."""
    depth = 0
    for k in range(open_idx, len(text)):
        c = text[k]
        if c in "{([":
            depth += 1
        elif c in "})]":
            depth -= 1
            if depth == 0:
                return k
    raise SystemExit(f"unbalanced delimiters in the Rust source at offset {open_idx}")


def rust_extern_decls(src):
    """(fn name, [param types], return type or None) for every `pub fn` in an
    `extern "C"` block. Parameters are split at top-level commas (function-
    pointer types contain commas of their own) and a param's type is
    everything after the first `: ` (names cannot contain colons). The block
    headers are located in the raw source (masking blanks their `"C"`), the
    bodies are scanned on the masked text so prose cannot confuse it."""
    text = mask_rust_noise(src)
    decls = []
    i = 0
    while (j := src.find('extern "C"', i)) >= 0:
        k = j + len('extern "C"')
        while k < len(src) and src[k].isspace():
            k += 1
        if k >= len(src) or src[k] != "{":
            i = j + 1  # `extern "C" fn` in a type alias, not a block
            continue
        brace = k
        end = _match_delimiter(text, brace)
        block = text[brace:end]
        raw_block = src[brace:end]  # same offsets, strings intact
        m = 0
        while (mm := re.compile(r"\bpub fn (cui_\w+)\(").search(block, m)):
            p_close = _match_delimiter(block, mm.end() - 1)
            # param/return text comes from the raw source: masking blanks
            # string literals, and `extern "C" fn` params contain one
            params = raw_block[mm.end():p_close]
            semi = raw_block.find(";", p_close)
            rest = raw_block[p_close + 1:(semi if semi >= 0 else len(raw_block))]
            ret = rest.split("->", 1)[1].strip() if "->" in rest else None
            types = []
            for prm in split_params(params):
                if not prm:
                    continue
                types.append(prm.split(": ", 1)[1].strip() if ": " in prm else prm)
            decls.append((mm.group(1), types, ret))
            m = p_close
        i = end
    return decls


def rust_struct_fields(src):
    """{struct name: [Rust field names in declaration order]} for every
    `pub struct cui_* { ... }`."""
    text = mask_rust_noise(src)
    out = {}
    for m in re.finditer(r"pub struct (cui_\w+)\s*\{", text):
        end = _match_delimiter(text, m.end() - 1)
        out[m.group(1)] = re.findall(r"\bpub (\w+):", text[m.end():end])
    return out


def c_field_names(header, tname):
    """C field names in declaration order, or None when the struct is not in
    the header (opaque types) or a field does not parse. Function-pointer
    fields (`void (*set_cursor)(...)`) fall back to a name-only match."""
    if tname not in header.structs:
        return None
    names = []
    for f in header.structs[tname][0]:
        for one in expand_field(f):
            p = parse_param(one)
            if p is not None:
                names.append(p[1])
                continue
            m = re.search(r"\(\s*\*\s*(\w+)\s*\)", one)
            if not m:
                return None
            names.append(m.group(1))
    return names


def main():
    h = Header(HEADER.read_text())
    sys_src = (ROOT / "cui-sys/src/lib.rs").read_text()
    manifest = [l.strip().rstrip("(") for l in MANIFEST.read_text().splitlines()
                if l.strip() and not l.startswith("#") and not l.startswith("cui__")]

    todo = []
    for entry in manifest:
        if entry in h.functions:
            todo.append(entry)
            continue
        if entry in h.macros:
            exp = re.sub(r"\\\n", " ", h.macros[entry])
            found = False
            for fn in re.findall(r"(cui_\w+)\(", exp):
                if fn in h.functions:
                    todo.append(fn)
                    found = True
            if found:
                continue
        if f"{entry}_ex" in h.functions:
            todo.append(f"{entry}_ex")
            continue
        print(f"unresolved manifest entry: {entry}", file=sys.stderr)
    todo = list(dict.fromkeys(todo))
    hand_bound = set()
    hand_file = ROOT / "scripts/ffi-hand.txt"
    if hand_file.exists():
        hand_bound = {l.strip() for l in hand_file.read_text().splitlines() if l.strip()}
    new_fns = [f for f in todo if f"pub fn {f}(" not in sys_src]

    # ---- transitively collect needed typedefs ----
    needed = {}
    seen = set()

    def add_type(t):
        if t in needed or t in seen or t not in h.structs:
            return
        if re.search(rf"pub (struct|enum) {t}\b", sys_src):
            return  # already hand-declared in cui-sys
        seen.add(t)
        fields, pos = h.structs[t]
        needed[t] = (fields, pos)
        for f in fields:
            # every cui_* name in the field text, function-pointer
            # parameter types included
            for t2 in re.findall(r"\bcui_\w+\b", f):
                add_type(t2)

    # va_list has no Rust spelling (and no portable ABI): such functions
    # (cui_log_appendv) are not bound; the variadic sibling is.
    va = [f for f in new_fns if "va_list" in h.functions[f][1]]
    for f in va:
        print(f"skipping {f}: takes a va_list", file=sys.stderr)
    new_fns = [f for f in new_fns if f not in va]

    for fn in new_fns:
        ret, params, _ = h.functions[fn]
        for t in set(re.findall(r"\bcui_\w+\b", ret + " " + params)):
            if not t.startswith("cui__"):
                add_type(t)
        base = re.sub(r"_(ex|begin|end|next|item_begin|item_end|column_begin|column_end|card_begin|card_end|slide_begin|slide_end)$", "", fn)
        for cand in [base + "_opts", base + "s", base + "_item_opts", base + "_column_opts", base + "_range", base + "_entry", base + "_result"]:
            if cand in h.structs:
                add_type(cand)

    # ---- ffi type resolution ----
    def ffi_type(ctype):
        ctype = ctype.strip()
        if "*" in ctype:
            base, base_const, after = split_ctype(ctype)
            rt = ffi_type(base)
            for i in range(len(after)):
                const = base_const if i == 0 else after[i - 1]
                rt = f"*const {rt}" if const else f"*mut {rt}"
            return rt
        if ctype in PRIM:
            return PRIM[ctype]
        if ctype == "char":
            return "c_char"
        if ctype in ("int64_t", "long long"):
            return "i64"
        if ctype == "void":
            return "c_void"
        if ctype in ("va_list", "..."):
            raise SystemExit("va_list/variadic types have no Rust spelling; "
                             "skip the function (variadics are emitted as `...`)")
        if ctype in ("cui_gl_renderer", "cui_sw_renderer"):
            return "c_void"
        if ctype in ("uint8_t", "unsigned char"):
            return "u8"
        if ctype in ("uint64_t",):
            return "u64"
        if ctype in ("int8_t",):
            return "i8"
        if ctype in ("int16_t",):
            return "i16"
        if ctype in ("uint16_t",):
            return "u16"
        if ctype in ("int32_t",):
            return "i32"
        if ctype in ("uint32_t",):
            return "u32"
        if ctype in h.enums:
            return "c_int"
        if ctype.startswith("cui_") or ctype.startswith("cui__"):
            return ctype
        raise SystemExit(f"unmapped ffi type: {ctype!r}")

    def rust_ctype(ctype):
        """C type text (possibly a pointer) -> Rust type text."""
        return ffi_type(ctype)

    def param_rust_type(p):
        """Full Rust type of a parsed parameter/field (never drops a level)."""
        ctype, _, kind = p
        if kind == "fnptr":
            return fnptr_type(p.ret, p.args)
        if kind == "multi":
            return ffi_type(p.ctext)
        rt = ffi_type(ctype)
        if kind == "mut_ptr":
            return f"*mut {rt}"
        if kind == "const_ptr":
            return f"*const {rt}"
        return rt

    def fnptr_type(ret, args):
        """`Option<unsafe extern "C" fn(..) -> R>` for a C function pointer."""
        ftypes = []
        for prm in split_params(args):
            if prm.strip() == "void":
                continue
            if prm.strip() == "...":
                raise SystemExit(f"variadic function pointer: ({args})")
            pp = parse_param(prm) or parse_param(prm + " _a")  # unnamed param
            if pp is None:
                raise SystemExit(f"cannot parse function-pointer parameter {prm!r}")
            ftypes.append(param_rust_type(pp))
        ret_txt = "" if ret.strip() == "void" else " -> " + ffi_type(ret.strip())
        return f'Option<unsafe extern "C" fn({", ".join(ftypes)}){ret_txt}>'


    # A pointer param followed by an adjacent int is an array; if that int is
    # named "count"/"*_count" it is the length and is dropped (slices know it).
    def array_pair_at(plist, i):
        if i + 1 >= len(plist):
            return None
        ctype, pname, ptr = plist[i]
        ntype, nname, nptr = plist[i + 1]
        if ptr not in ("const_ptr", "mut_ptr") or ctype == "char":
            return None
        if nptr != "value" or ntype not in ("int", "uint", "unsigned", "size_t", "int32_t", "uint32_t"):
            return None
        if nname == "count" or nname.endswith("_count"):
            return "drop"
        # byte buffers sized by a size_t named size/len (cui_hex_view_ex);
        # gated on the element type so void* loaders stay pointer params
        if (nname in ("size", "len") or nname.endswith("_size") or nname.endswith("_len")) \
                and ntype == "size_t" and ctype in ("uint8_t", "unsigned char"):
            return "drop"
        # float tables whose dimensions follow (cui_heatmap, cui_level_meter):
        # slice them and keep the dimension params
        if ctype in ("float", "double"):
            return "keep"
        return None

    # ---- ffi structs ----
    ffi = []
    ffi.append("//! Generated by scripts/gen_ffi.py from the pinned release header.")
    ffi.append("//! Do not edit by hand: extend scripts/ffi-manifest.txt and re-run.")
    ffi.append("#![allow(non_camel_case_types, non_snake_case, missing_docs, dead_code)]")
    ffi.append("")
    ffi.append("use core::ffi::{c_char, c_int, c_void};")
    ffi.append("")
    ffi.append("use super::*;")
    ffi.append("")

    def dim_of(d):
        if d.isdigit():
            return d
        if d in h.const_env:
            return str(h.const_env[d])
        raise SystemExit(f"unknown array dimension {d!r}")

    for tname, (cfields, pos) in sorted(needed.items()):
        doc = doc_before(h.comments, pos, h.text)
        ffi.append("// " + (doc.splitlines()[0].strip()[:110] if doc else tname))
        ffi.append("#[repr(C)]")
        has_array = any(re.search(r"\[\s*\w+\s*\]", f) for f in cfields)
        ffi.append("#[derive(Clone, Copy, Debug" + ("" if has_array else ", Default") + ")]")
        ffi.append(f"pub struct {tname} {{")
        for cf in cfields:
            for f in expand_field(cf):
                p = must_param(f, f"field of {tname}")
                ctype, fname, ptr = p
                if ptr == "fnptr":
                    # `ret (*name)(params)` becomes a real Option<unsafe extern
                    # "C" fn(..)> so the struct keeps C's layout
                    ffi.append(f"    pub {rust_field_name(fname)}: {param_rust_type(p)},")
                    continue
                if fname == "_":
                    fname = "_pad"
                    ctype = "int"
                fname = rust_field_name(fname)
                # array dims, innermost first: "char[10][24]" -> [[c_char; 24]; 10]
                dims = re.findall(r"\[\s*(\w+)\s*\]", ctype)
                arr = []
                if dims:
                    ctype = re.sub(r"(?:\[\s*\w+\s*\])+$", "", ctype).strip()
                    for d in reversed(dims):
                        arr.append(dim_of(d))
                    rt = ffi_type(ctype)  # arrays of values or of pointers
                else:
                    rt = param_rust_type(P(ctype, fname, ptr, ctext=p.ctext))
                for d in arr:  # innermost dim first
                    rt = f"[{rt}; {d}]"
                ffi.append(f"    pub {fname}: {rt},")
        ffi.append("}")
        if has_array:
            # plain C data: every field is an integer, float, bool, char or a
            # raw pointer, so all-zero bits are a valid value
            ffi.append(f"impl Default for {tname} {{")
            ffi.append("    fn default() -> Self {")
            ffi.append("        // SAFETY: see above — a POD struct accepts the zeroed value")
            ffi.append("        unsafe { core::mem::zeroed() }")
            ffi.append("    }")
            ffi.append("}")
        ffi.append("")

    # ---- enum constants ----
    ffi.append("// ---- enum constants ----")
    for ename, members in sorted(h.enum_vals.items()):
        ffi.append(f"// {ename}")
        for mem, v in members:
            ffi.append(f"pub const {mem}: i32 = {v};")
    ffi.append("")

    ffi.append('extern "C" {')
    for fn in new_fns:
        ret, params, pos = h.functions[fn]
        doc = doc_before(h.comments, pos, h.text)
        if doc:
            ffi.append("    // " + doc.splitlines()[0].strip()[:110])
        rust_params = []
        has_ctx = False
        variadic = False
        for p in split_params(params):
            if p.strip() == "void":
                continue
            if p.strip() == "...":
                variadic = True
                continue
            parsed = must_param(p, f"parameter of {fn}")
            ctype, pname, ptr = parsed
            if pname == "ui" and ctype == "cui_ctx" and ptr == "mut_ptr":
                has_ctx = True
                continue  # the context is the receiver
            pname = rust_field_name(pname)
            rust_params.append(f"{pname}: {param_rust_type(parsed)}")
        if variadic:
            rust_params.append("...")
        ret_rust = "" if ret.strip() == "void" else " -> " + ffi_type(ret.strip())
        receiver = "ui: *mut cui_ctx, " if has_ctx else ""
        ffi.append(f"    pub fn {fn}({receiver}{', '.join(rust_params)}){ret_rust};")
    ffi.append("}")

    OUT_FFI.write_text("\n".join(ffi) + "\n")

    # ---- thin safe methods ----
    def method_name(fn):
        n = fn
        n = re.sub(r"_ex$", "", n)
        n = re.sub(r"^cui_", "", n)
        return snake_to_camel(n)

    # scope classification from the macros
    def scope_kind(entry):
        if entry not in h.macros:
            return None
        exp = re.sub(r"\\\n", " ", h.macros[entry])
        if "CUI__SCOPE_IF" in exp:
            return "if"
        if "CUI__SCOPE" in exp:
            return "scope"
        return None

    # map manifest entry -> method name (avoid collisions with hand-written methods)
    hand_methods = set(re.findall(r"pub fn (\w+)\(", (ROOT / "cui/src/lib.rs").read_text()))
    hand_methods |= set(re.findall(r"pub fn (\w+)\(", (ROOT / "cui/src/ext.rs").read_text()))
    hand_methods |= {"canvas", "state", "split", "split_next", "sheet", "menu", "context_menu",
                     "user_menu", "command_palette", "reasoning", "tool_call", "agent_plan",
                     "agent_status", "source_list", "citation", "response_actions",
                     "prompt_suggestions", "model_picker", "chat_composer", "context_meter",
                     "dot_chart", "event_track", "readout", "readout_with", "dot_display",
                     "dot_display_with", "breadcrumb", "pagination", "avatar", "avatar_with",
                     "avatar_group", "list_item", "list_item_with", "timeline_item",
                     "timeline_item_with", "copy_button", "progress_ring", "chat_message",
                     "chat_message_with", "number_roll", "set_font_scale", "set_reduced_motion"}

    safe = []
    safe_out = None
    safe.append("//! fields with `ui.cstr(\"...\")`).")
    safe.append("")
    safe.append("")

    emitted_methods = set()
    used_fns = set()
    not_auto_wrapped = []

    def autowrap_ok(fn):
        """A thin generated wrapper cannot make these sound: function
        pointers, pointer-to-pointer params (out-pointers, string arrays) and
        variadics need a hand-written method in ext.rs."""
        for prm in split_params(h.functions[fn][1]):
            if prm.strip() == "...":
                break
            if prm.strip() == "void":
                continue
            if must_param(prm, f"parameter of {fn}")[2] in ("fnptr", "multi"):
                break
        else:
            return True
        not_auto_wrapped.append(fn)
        return False

    for entry in manifest:
        # find the functions behind this entry
        fns = []
        if entry in h.functions:
            fns.append((entry, entry))
        elif entry in h.macros:
            exp = re.sub(r"\\\n", " ", h.macros[entry])
            for fn in re.findall(r"(cui_\w+)\(", exp):
                if fn in h.functions:
                    fns.append((entry, fn))
        elif f"{entry}_ex" in h.functions:
            fns.append((entry, f"{entry}_ex"))
        fns = [(e, f) for e, f in fns if f in new_fns]
        if not fns:
            continue
        kind = scope_kind(entry)
        if kind:
            begin = next((f for e, f in fns if f.endswith("_begin")), None)
            end = next((f for e, f in fns if f.endswith("_end")), None)
            if not begin or not end:
                continue
            if begin in hand_bound or end in hand_bound or entry in hand_bound:
                continue
            if not (autowrap_ok(begin) and autowrap_ok(end)):
                continue
            mname = re.sub(r"^cui_", "", begin)
            mname = re.sub(r"_begin$", "", mname)
            if mname in hand_methods or mname in emitted_methods:
                continue
            emitted_methods.add(mname)
            used_fns.update([begin, end])
            ret, params, pos = h.functions[begin]
            doc = doc_before(h.comments, pos, h.text)
            safe.append("    /// " + (doc_first_line(doc) or entry))
            sig = []
            begin_args = []
            skip_next = False
            dropped_len = ""
            dropped_ctype = "int"
            plist_all = [(parse_param(p) or (None, None, None)) for p in split_params(params)]
            for i, p in enumerate(split_params(params)):
                parsed = parse_param(p)
                if parsed is None:
                    continue
                ctype, pname, ptr = parsed
                if pname == "ui" and ctype == "cui_ctx":
                    continue
                if pname in ("in", "loop", "type", "fn", "match", "ref", "box", "move", "impl"):
                    pname = pname + "_"
                if skip_next:
                    skip_next = False
                    if dropped_ctype == "size_t":
                        begin_args.append(f"{dropped_len}.len()")
                    else:
                        begin_args.append(f"{dropped_len}.len() as c_int")
                    continue
                pair = array_pair_at(plist_all, i)
                if pair == "drop":
                    skip_next = True
                    dropped_len = pname
                    dropped_ctype = plist_all[i + 1][0]
                rt = ffi_type(ctype)
                if pair is not None:
                    elem = f"ffi::{rt}" if rt.startswith("cui_") or rt.startswith("cui__") else rt
                    sig.append(f"{pname}: &[{elem}]" if ptr == "const_ptr" else f"{pname}: &mut [{elem}]")
                    begin_args.append(f"{pname}.as_ptr()" if ptr == "const_ptr" else f"{pname}.as_mut_ptr()")
                    continue
                if ptr == "const_ptr" and rt == "c_char":
                    sig.append(f"{pname}: impl AsRef<str>")
                    begin_args.append(f"self.arena.cstr({pname}.as_ref())")
                elif ptr == "mut_ptr" and rt == "*mut bool":
                    sig.append(f"{pname}: &mut bool")
                    begin_args.append(pname)
                elif ptr == "mut_ptr" and rt == "*mut f32":
                    sig.append(f"{pname}: &mut f32")
                    begin_args.append(pname)
                elif ptr == "mut_ptr" and rt == "*mut i32":
                    sig.append(f"{pname}: &mut i32")
                    begin_args.append(pname)
                elif ptr == "mut_ptr" and rt.startswith("cui_"):
                    sig.append(f"{pname}: &mut ffi::{rt}")
                    begin_args.append(pname)
                elif ptr == "const_ptr" and rt.startswith("cui_"):
                    sig.append(f"{pname}: &ffi::{rt}")
                    begin_args.append(pname)
                elif ptr == "const_ptr":
                    sig.append(f"{pname}: &{rt}")
                    begin_args.append(pname)
                elif ptr == "mut_ptr":
                    sig.append(f"{pname}: &mut {rt}")
                    begin_args.append(pname)
                else:
                    if rt.startswith("cui_"):
                        sig.append(f"{pname}: ffi::{rt}")
                    else:
                        sig.append(f"{pname}: {rt}")
                    begin_args.append(pname)
            cond = "let shown = unsafe {" if kind == "if" else "unsafe {"
            safe.append(f"    pub fn {mname}(&self{', ' + ', '.join(sig) if sig else ''}, f: impl FnOnce(&mut Ui)) {{")
            safe.append(f"        {cond} ffi::{begin}(self.ctx{', ' if begin_args else ''}{', '.join(begin_args)}) }};")
            if kind == "if":
                safe.append("        if shown {")
            safe.append(f"            self.scope(ffi::{end}, f);")
            if kind == "if":
                safe.append("        }")
            safe.append("    }")
            safe.append("")
            continue
        # plain / _ex function -> one method
        for entry_name, fn in fns:
            if fn in hand_bound or entry in hand_bound:
                continue
            if not autowrap_ok(fn):
                continue
            mname = re.sub(r"_ex$", "", re.sub(r"^cui_", "", fn))
            if mname in hand_methods or mname in emitted_methods:
                continue
            emitted_methods.add(mname)
            used_fns.add(fn)
            ret, params, pos = h.functions[fn]
            first = split_params(params)[0] if split_params(params) else ""
            fp = parse_param(first)
            if not (fp and fp[0] == "cui_ctx" and fp[1] == "ui"):
                continue  # context-free function: hand-bound or free
            doc = doc_before(h.comments, pos, h.text)
            safe.append("    /// " + (doc_first_line(doc) or fn))
            plist = [(parse_param(p) or (None, None, None)) for p in split_params(params)]
            buffer_at = None
            for i in range(len(plist) - 1):
                if plist[i][2] == "mut_ptr" and plist[i][0] == "char" and \
                   plist[i + 1][2] == "value" and plist[i + 1][0] in ("size_t", "int"):
                    buffer_at = i
                    break
            sig = []
            call = []
            skip_next = False
            dropped_len = ""
            dropped_ctype = "int"
            buffer_names = []
            prelude = []
            # A lone mutable buffer whose capacity arrives as a later `cap`
            # or `capacity` int (cui_diff_view_rows' `rows, first, cap`;
            # cui_gradient_editor's `stops, count, capacity`): take a slice
            # and size the call from the slice itself.
            cap_rule = None
            scalar = {"int", "uint", "unsigned", "size_t", "int32_t", "uint32_t",
                      "int64_t", "uint64_t", "float", "double", "bool"}
            mut_bufs = [k for k, (t, n, p) in enumerate(plist)
                        if p == "mut_ptr" and t != "char" and t not in scalar
                        and not (t == "cui_ctx" and n == "ui")]
            if len(mut_bufs) == 1 and buffer_at is None:
                for k in range(mut_bufs[0] + 1, len(plist)):
                    t2, n2, p2 = plist[k]
                    if p2 == "value" and t2 in ("int", "uint", "unsigned", "int32_t", "uint32_t") \
                            and n2 in ("cap", "capacity"):
                        cap_rule = (mut_bufs[0], k)
                        break
            for i, p in enumerate(split_params(params)):
                parsed = parse_param(p)
                if parsed is None:
                    continue
                ctype, pname, ptr = parsed
                if pname == "ui" and ctype == "cui_ctx":
                    continue
                if pname in ("in", "loop", "type", "fn", "match", "ref", "box", "move", "impl"):
                    pname = pname + "_"
                if buffer_at is not None and i == buffer_at:
                    sig.append(f"{pname}: &mut String")
                    buffer_names.append(pname)
                    call.append(f"{pname}.as_mut_ptr() as *mut c_char")
                    continue
                if (buffer_at is not None and i == buffer_at + 1
                        and i < len(plist) and plist[i][0] == "size_t"):
                    continue  # the capacity comes from the String
                if skip_next:
                    skip_next = False
                    if dropped_ctype == "size_t":
                        call.append(f"{dropped_len}.len()")  # len() is already usize
                    else:
                        call.append(f"{dropped_len}.len() as c_int")
                    continue
                if cap_rule is not None and i == cap_rule[0]:
                    rt = ffi_type(ctype)
                    elem = f"ffi::{rt}" if rt.startswith("cui_") or rt.startswith("cui__") else rt
                    sig.append(f"{pname}: &mut [{elem}]")
                    call.append(f"{pname}.as_mut_ptr()")
                    continue
                if cap_rule is not None and i == cap_rule[1]:
                    call.append(f"{plist[cap_rule[0]][1]}.len() as c_int")
                    continue
                pair = array_pair_at(plist, i)
                if pair == "drop":
                    skip_next = True
                    dropped_len = pname
                    dropped_ctype = plist[i + 1][0]
                if pair == "keep":
                    # C indexes the slice by the adjacent int dimension(s);
                    # assert the read extent up front so a short slice panics
                    # instead of reading past its end
                    dims = []
                    j = i + 1
                    while j < len(plist) and plist[j][2] == "value" and \
                            plist[j][0] in ("int", "uint", "unsigned", "int32_t", "uint32_t"):
                        dims.append(plist[j][1])
                        j += 1
                    if dims:
                        if len(dims) == 1:
                            prod = f"{dims[0]} as i64"
                        else:
                            prod = " * ".join(f"({d} as i64)" for d in dims)
                        prelude.append(f"let need = ({prod}).max(0) as usize;")
                        prelude.append(f"assert!(need <= {pname}.len(), \"{fn}: `{pname}` holds fewer elements than its dimension parameters require\");")
                rt = ffi_type(ctype)
                if pair is not None:
                    elem = f"ffi::{rt}" if rt.startswith("cui_") or rt.startswith("cui__") else rt
                    sig.append(f"{pname}: &[{elem}]" if ptr == "const_ptr" else f"{pname}: &mut [{elem}]")
                    call.append(f"{pname}.as_ptr()" if ptr == "const_ptr" else f"{pname}.as_mut_ptr()")
                    continue
                if ptr == "const_ptr" and rt == "c_char":
                    sig.append(f"{pname}: impl AsRef<str>")
                    call.append(f"self.arena.cstr({pname}.as_ref())")
                elif ptr == "const_ptr" and rt.startswith("cui_"):
                    sig.append(f"{pname}: &ffi::{rt}")
                    call.append(pname)
                elif ptr == "mut_ptr" and rt.startswith("cui_"):
                    sig.append(f"{pname}: &mut ffi::{rt}")
                    call.append(pname)
                elif ptr == "const_ptr":
                    sig.append(f"{pname}: &{rt}")
                    call.append(pname)
                elif ptr == "mut_ptr":
                    sig.append(f"{pname}: &mut {rt}")
                    call.append(pname)
                else:
                    if rt.startswith("cui_") or rt.startswith("cui__"):
                        sig.append(f"{pname}: ffi::{rt}")
                        call.append(pname)
                    else:
                        sig.append(f"{pname}: {rt}")
                        call.append(pname)
            ret_rust = ""
            ret_call = ""
            if ret.strip() != "void":
                rrt = ffi_type(ret.strip())
                # pointer returns of struct types carry the ffi:: path too
                m = re.match(r"\*(const|mut) ((?:cui_)[\w_]+)$", rrt)
                if m:
                    ret_rust = f" -> *{m.group(1)} ffi::{m.group(2)}"
                elif rrt.startswith("cui_") or rrt.startswith("cui__"):
                    ret_rust = f" -> ffi::{rrt}"
                else:
                    ret_rust = f" -> {rrt}"
            joiner = ", " if sig else ""
            safe.append(f"    pub fn {mname}(&self{joiner}{', '.join(sig)}){ret_rust} {{")
            safe.extend(f"        {line}" for line in prelude)
            body_call = f"ffi::{fn}(self.ctx{', ' if call else ''}{', '.join(call)}){ret_call}"
            if buffer_at is not None:
                # C reads the buffer with strlen and writes it back bounded by
                # its capacity: NUL-terminate and zero the tail first, then
                # re-derive the String's contents after the call
                prep = "; ".join(f"crate::begin_c_string({b}, 63)" for b in buffer_names)
                if ret.strip() == "void":
                    safe.append(f"        {prep};")
                    safe.append(f"        unsafe {{ {body_call} }};")
                    safe.append("        crate::ext::sync_buffer(" + buffer_names[-1] + ");")
                else:
                    safe.append(f"        {prep};")
                    safe.append(f"        let r = unsafe {{ {body_call} }};")
                    safe.append(f"        crate::ext::sync_buffer({buffer_names[-1]});")
                    safe.append("        r")
            else:
                safe.append(f"        unsafe {{ {body_call} }}")
            safe.append("    }")
            safe.append("")

    body = "\n".join(safe)
    body = (
        "//! Generated by scripts/gen_ffi.py: thin safe methods over the raw\n"
        "//! generated opts structs. Hand-bound functions (scripts/ffi-hand.txt)\n"
        "//! are in ext.rs instead. Do not edit by hand.\n"
        "#![allow(missing_docs, unused_variables, clippy::too_many_arguments)]\n"
        "use crate::Ui;\n"
        "use cui_sys as ffi;\n"
        "use core::ffi::{c_char, c_int, c_void};\n\n"
        "impl<'a> Ui<'a> {\n"
        "    /// Converts a string into an arena-backed C string, valid for this\n"
        "    /// frame. For filling raw generated opts structs (their string fields\n"
        "    /// stay raw pointers).\n"
        "    pub fn cstr(&self, s: impl AsRef<str>) -> *const c_char {\n"
        "        self.arena.cstr(s.as_ref())\n"
        "    }\n\n"
        + re.sub(r"^(\s*//! .*\n)+", "", body.rstrip()) + "\n}\n"
    )
    OUT_SAFE.write_text(body)

    # ---- layout probe + link/layout tests (cover BOTH Rust FFI files) ----
    sys_src = (ROOT / "cui-sys/src/lib.rs").read_text()
    gen_src = OUT_FFI.read_text()
    structs = {**rust_struct_fields(sys_src), **rust_struct_fields(gen_src)}
    decls = rust_extern_decls(sys_src) + rust_extern_decls(gen_src)

    probe = ["// Generated by scripts/gen_ffi.py: appended by vendor.sh runs.",
             "// EN() for every enum member in the header, SZ/OFF for every",
             "// struct the Rust FFI declares, hand-written and generated alike.",
             "// layout_probe.c's base_probe prints only the version line.",
             "void gen_probe_all(void) {",
             '    puts("[enum]");']
    enum_checks = []
    for _ename, members in sorted(h.enum_vals.items()):
        for mem, _v in members:
            probe.append(f"    EN({mem});")
            enum_checks.append(f'    check(&t, "enum", "{mem}", {mem} as i64);')
    # the hand-written #[repr] enums in cui-sys/src/lib.rs carry their own
    # discriminants: pin every variant against the C value as well
    masked_sys = mask_rust_noise(sys_src)
    for em in re.finditer(r"pub enum (cui_\w+)\s*\{", masked_sys):
        body_end = _match_delimiter(masked_sys, em.end() - 1)
        for var in re.findall(r"\b(CUI_\w+)\b", masked_sys[em.end():body_end]):
            enum_checks.append(f'    check(&t, "enum", "{var}", {em.group(1)}::{var} as i32 as i64);')
    probe.append('    puts("[size]");')
    offsets = []
    checks = []
    for tname in sorted(structs):
        rfields = structs[tname]
        cfields = c_field_names(h, tname)
        if cfields is None:
            print(f"warning: {tname} not probed (opaque or unparsed in the header)", file=sys.stderr)
            continue
        probe.append(f"    SZ({tname});")
        checks.append(f'    check(&t, "size", "{tname}", size_of::<{tname}>() as i64);')
        if len(cfields) != len(rfields):
            raise SystemExit(f"{tname}: {len(rfields)} Rust fields vs {len(cfields)} C fields "
                             "(offsets would go unchecked)")
        for cf, rf in zip(cfields, rfields):
            offsets.append(f"    OFF({tname}, {cf});")
            checks.append(f'    check(&t, "offset", "{tname}.{cf}", offset_of!({tname}, {rf}) as i64);')
    probe.append('    puts("[offset]");')
    probe.extend(offsets)
    probe.append("}")
    OUT_PROBE.write_text("\n".join(probe) + "\n")

    c_imports = sorted({tok
                        for _, types, ret in decls
                        for tok in re.findall(r"\bc_\w+\b", " ".join(types + ([ret] if ret else [])))})
    link = [
        "//! Generated by scripts/gen_ffi.py. Every extern declaration in the",
        "//! crate is turned into a typed function pointer, so linking this",
        "//! test resolves all of them against the compiled C library: a",
        "//! declaration whose symbol the vendored CUI does not export (a",
        "//! rename typo, a drifted header) fails HERE, not in a downstream",
        "//! user's build.",
        "#![allow(non_snake_case, clippy::too_many_lines)]",
        "",
        f"use core::ffi::{{{', '.join(c_imports)}}};",
        "use cui_sys::*;",
        "",
        "#[test]",
        "fn every_extern_symbol_resolves() {",
    ]
    for name, types, ret in decls:
        arrow = f" -> {ret}" if ret else ""
        # cui_app_* is compiled out under CUI_NO_APP (the `app` feature off)
        if name.startswith("cui_app_"):
            link.append('    #[cfg(feature = "app")]')
        link.append(f"    let _: unsafe extern \"C\" fn({', '.join(types)}){arrow} = {name};")
    link.append("}")
    (ROOT / "cui-sys/tests").mkdir(exist_ok=True)
    (ROOT / "cui-sys/tests/link.rs").write_text("\n".join(link) + "\n")

    # ---- sigcheck.c: every Rust extern vs the header's own prototype ----
    # The Rust declaration is mapped back to C independently of the code that
    # produced it, then assigned to a function pointer of that type; the C
    # compiler rejects a dropped parameter, a lost pointer level, a missing
    # `...` or a changed qualifier (scripts/sigcheck.sh builds it with -Werror).
    # The header is consulted only to NAME an enum or opaque struct where
    # Rust spells it as an integer / `c_void`.
    rust2c = {"c_char": "char", "c_int": "int", "i32": "int", "u32": "uint32_t",
              "f32": "float", "f64": "double", "bool": "bool", "usize": "size_t",
              "u8": "uint8_t", "i8": "int8_t", "i16": "int16_t", "u16": "uint16_t",
              "i64": "int64_t", "u64": "uint64_t", "c_void": "void"}
    subst_ok = set(h.enums) | {"cui_gl_renderer", "cui_sw_renderer"}

    def hdr_info(prm):
        prm = prm.strip()
        if prm in ("", "void", "..."):
            return None
        p = parse_param(prm) or parse_param(prm + " _a")
        if p is None:
            raise SystemExit(f"sigcheck: cannot parse header parameter {prm!r}")
        if p[2] == "fnptr":
            return ("fn", p.ret, [a for a in split_params(p.args) if a.strip() != "void"])
        return ("t", split_ctype(p.ctext or p[0])[0])

    def match_paren(s, i):
        depth = 0
        for k in range(i, len(s)):
            if s[k] == "(":
                depth += 1
            elif s[k] == ")":
                depth -= 1
                if depth == 0:
                    return k
        raise SystemExit(f"sigcheck: unbalanced parentheses in {s!r}")

    def rust_to_c(rt, hi):
        rt = rt.strip()
        if rt.startswith("Option<"):
            inner = rt[len("Option<"):].rstrip()
            inner = inner[:-1].strip()  # the closing `>`
            pm = re.match(r'(?:unsafe\s+)?extern "C" fn\(', inner)
            if not pm:
                raise SystemExit(f"sigcheck: unsupported Option payload {rt!r}")
            op = pm.end() - 1
            cl = match_paren(inner, op)
            args = [re.sub(r"^\w+\s*:\s*", "", a.strip()) for a in split_params(inner[op + 1:cl]) if a.strip()]
            rest = inner[cl + 1:].strip()
            ret = rest[2:].strip() if rest.startswith("->") else None
            hargs = [None] * len(args)
            hret = None
            if hi and hi[0] == "fn":
                if len(hi[2]) == len(args):
                    hargs = [hdr_info(a) for a in hi[2]]
                if hi[1].strip() != "void":
                    hret = ("t", split_ctype(hi[1])[0])
            cret = "void" if ret is None else rust_to_c(ret, hret)
            cargs = ", ".join(rust_to_c(a, ha) for a, ha in zip(args, hargs)) or "void"
            return f"{cret} (*)({cargs})"
        m = re.match(r"^\*(const|mut)\s+(.*)$", rt)
        if m:
            inner = rust_to_c(m.group(2), hi)
            if m.group(1) == "const":
                return inner + " const *" if inner.endswith("*") else f"const {inner} *"
            return inner + " *"
        if hi and hi[0] == "t" and hi[1] in subst_ok and rt in ("c_int", "i32", "u32", "c_void"):
            return hi[1]
        if rt in rust2c:
            return rust2c[rt]
        if rt.startswith("cui_"):
            return rt
        raise SystemExit(f"sigcheck: no C spelling for Rust type {rt!r}")

    sig = ["/* Generated by scripts/gen_ffi.py - do not edit.",
           " * Every Rust extern declaration, mapped back to C, assigned to a function",
           " * pointer: scripts/sigcheck.sh compiles this with -Werror, so a parameter",
           " * count, pointer level, qualifier or variadic mismatch against cui.h fails",
           " * the build. */",
           "#include <stddef.h>", "#include <stdint.h>", "#include <stdbool.h>",
           '#include "cui.h"', ""]
    for name, types, ret in decls:
        if name not in h.functions:
            raise SystemExit(f"sigcheck: {name} is declared in Rust but is not a CUI_API function in the header")
        hret, hparams, _ = h.functions[name]
        hps = [x for x in split_params(hparams) if x.strip() not in ("void", "...")]
        variadic = any(t.strip() == "..." for t in types)
        rtypes = [t for t in types if t.strip() != "..."]
        his = [hdr_info(x) for x in hps] if len(hps) == len(rtypes) else [None] * len(rtypes)
        cargs = [rust_to_c(t, hi) for t, hi in zip(rtypes, his)]
        if variadic:
            cargs.append("...")
        hr = ("t", split_ctype(hret)[0]) if hret.strip() != "void" else None
        cret = "void" if ret is None else rust_to_c(ret, hr)
        sig.append(f"{cret} (*p_{name})({', '.join(cargs) or 'void'}) = {name};")
    (ROOT / "cui-sys/probe/sigcheck.c").write_text("\n".join(sig) + "\n")

    layout = [
        "//! Generated by scripts/gen_ffi.py: checks every FFI struct's size",
        "//! and every field offset against the C ABI facts produced by the",
        "//! layout probe (cui-sys/layout/<version>.toml). A missing entry",
        "//! means the probe table needs regenerating (see vendor.sh).",
        "#![allow(non_camel_case_types, non_snake_case, clippy::too_many_lines)]",
        "",
        "use core::mem::{offset_of, size_of};",
        "use cui_sys::*;",
        "",
        "fn layout_table() -> std::collections::BTreeMap<String, std::collections::BTreeMap<String, i64>> {",
        "    use std::collections::BTreeMap;",
        "    let path = format!(\"{}/layout/{}.toml\", env!(\"CARGO_MANIFEST_DIR\"), CUI_VERSION_STRING);",
        "    let src = std::fs::read_to_string(&path)",
        "        .unwrap_or_else(|e| panic!(\"no layout probe table for CUI {CUI_VERSION_STRING} ({e}); run ./vendor.sh\"));",
        "    let mut table: BTreeMap<String, BTreeMap<String, i64>> = BTreeMap::new();",
        "    let mut section = String::new();",
        "    for line in src.lines() {",
        "        let line = line.trim();",
        "        if line.is_empty() || line.starts_with('#') {",
        "            continue;",
        "        }",
        "        if line.starts_with('[') && line.ends_with(']') {",
        "            section = line[1..line.len() - 1].to_string();",
        "            continue;",
        "        }",
        "        let Some((key, value)) = line.split_once('=') else {",
        "            panic!(\"bad line in {path}: {line}\")",
        "        };",
        "        let key = key.trim().trim_matches('\"').to_string();",
        "        if key == \"version\" {",
        "            continue; // recorded in the filename; not a number",
        "        }",
        "        let value: i64 = value",
        "            .trim()",
        "            .parse()",
        "            .unwrap_or_else(|_| panic!(\"bad number in {path}: {line}\"));",
        "        table.entry(section.clone()).or_default().insert(key, value);",
        "    }",
        "    table",
        "}",
        "",
        "fn check(table: &std::collections::BTreeMap<String, std::collections::BTreeMap<String, i64>>,",
        "         section: &str, key: &str, actual: i64) {",
        "    let expected = table",
        "        .get(section)",
        "        .and_then(|s| s.get(key))",
        "        .unwrap_or_else(|| panic!(\"layout table for CUI {CUI_VERSION_STRING} lacks [{section}] {key}; regenerate with ./vendor.sh\"));",
        "    assert_eq!(actual, *expected, \"[{section}] {key}: Rust FFI vs C ABI (CUI {CUI_VERSION_STRING})\");",
        "}",
        "",
        "#[test]",
        "fn struct_sizes_and_offsets_match_c() {",
        "    let t = layout_table();",
    ]
    layout.extend(checks)
    layout.append("}")
    layout.extend(["", "#[test]", "fn enum_values_match_c() {", "    let t = layout_table();"])
    layout.extend(enum_checks)
    layout.append("}")
    (ROOT / "cui-sys/tests/layout.rs").write_text("\n".join(layout) + "\n")

    (ROOT / "scripts/ffi-generated.txt").write_text("\n".join(todo) + "\n")
    n_sizes = sum(1 for c in checks if '"size"' in c)
    n_offs = sum(1 for c in checks if '"offset"' in c)
    print(f"gen.rs: {len(new_fns)} functions, {len(needed)} types; gen_safe.rs methods: {len(emitted_methods)}; "
          f"link.rs: {len(decls)} decls; layout.rs: {n_sizes} sizes, {n_offs} offsets")


if __name__ == "__main__":
    main()
