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
    cui-sys/probe/gen_probe.c  SZ() lines for the generated structs

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
    out, comments, i, n = [], [], 0, len(text)
    while i < n:
        if text.startswith("/*", i):
            j = text.find("*/", i + 2)
            j = n if j < 0 else j
            comments.append((i, j, text[i + 2:j].strip()))
            i = j + 2
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


def doc_before(comments, pos, limit=600):
    best, best_at = "", -1
    for start, end, c in comments:
        if end <= pos and pos - end < limit and not c.startswith("==") and not c.startswith("Generated"):
            if end > best_at and len(c) > 3:
                best, best_at = c, end
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


def parse_param(p):
    p = " ".join(p.split())
    # array declarators: "char key[64]" / "bool on[CUI_FLAG_ENVS]" — the array
    # suffix travels with the type so the struct emitter can size it
    m = re.match(r"(.+?)\b(\w+)\s*((?:\[\s*\w+\s*\])+)$", p)
    if m:
        ptype, name, arr = m.group(1).strip(), m.group(2), re.sub(r"\s", "", m.group(3))
        core = re.sub(r"\bconst\b", "", ptype).strip()
        return (f"{core}{arr}", name, "value")
    m = re.match(r"(.+?)\b(\w+)$", p)
    if not m:
        return None
    ptype, name = m.group(1).strip(), m.group(2)
    if ptype.endswith("*"):
        mut = not re.search(r"\bconst\b", ptype)
        inner = re.sub(r"\bconst\b", "", ptype).rstrip("*").strip()
        return (inner, name, "mut_ptr" if mut else "const_ptr")
    core = re.sub(r"\bconst\b", "", ptype).strip()
    return (core, name, "value")


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
        for m in re.finditer(r"#define[ \t]+(CUI_\w+)[ \t]+(\d+)[ \t]*(?:$|//|/\*)", t, re.M):
            self.defines[m.group(1)] = m.group(2)

    def _add(self, kind, body, name, pos):
        if kind == "enum":
            entries = []
            for em in re.finditer(r"(CUI_\w+)\s*(=\s*[^,\n}]+)?\s*(?=[,\n}]|$)", body):
                entries.append((em.group(1), em.group(2).strip() if em.group(2) else None))
            self.enums[name] = (entries, pos)
        else:
            fields = [" ".join(fm.group(1).split()) for fm in re.finditer(r"([^;{}]+);", body)]
            self.structs[name] = (fields, pos)


def snake_to_camel(snake):
    return snake  # methods stay snake_case


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
            core = parse_param(f)
            if core:
                add_type(core[0])

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
        if ctype.endswith("*"):
            m = re.match(r"(.*?)(const\s*)?\*$", ctype)
            inner = re.sub(r"\bconst\b", "", ctype[:-1]).strip()
            it = ffi_type(inner)
            if it.startswith("*"):
                return it
            const = (m and m.group(2)) or "const" in ctype.split("*")[0]
            return f"*const {it}" if const else f"*mut {it}"
        if ctype in PRIM:
            return PRIM[ctype]
        if ctype == "char":
            return "c_char"
        if ctype in ("int64_t", "long long"):
            return "i64"
        if ctype == "void":
            return "c_void"
        if ctype in ("va_list", "..."):
            return "c_void"
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
        raise SystemExit(f"unmapped ffi type: {ctype!r} in struct {tname} field {fname}")


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

    def enum_value(name):
        # resolves an enum member to its integer value (implicit counters too)
        for entries, _pos in h.enums.values():
            v = 0
            for ename, expr in entries:
                if expr:
                    try:
                        v = int(expr, 0)
                    except ValueError:
                        v = v  # non-numeric expressions are beyond the probe
                if ename == name:
                    return v
                v += 1
        return None

    def dim_of(d):
        if d.isdigit():
            return d
        if d in h.defines:
            return h.defines[d]
        v = enum_value(d)
        return str(v) if v is not None else d

    for tname, (cfields, pos) in sorted(needed.items()):
        doc = doc_before(h.comments, pos)
        ffi.append("// " + (doc.splitlines()[0].strip()[:110] if doc else tname))
        ffi.append("#[repr(C)]")
        has_array = any(re.search(r"\[\s*\w+\s*\]", f) for f in cfields)
        ffi.append("#[derive(Clone, Copy, Debug" + ("" if has_array else ", Default") + ")]")
        ffi.append(f"pub struct {tname} {{")
        for cf in cfields:
            for f in expand_field(cf):
                p = parse_param(f)
                if p is None:
                    ffi.append(f"    // unparsed: {f}")
                    continue
                ctype, fname, ptr = p
                if fname == "_":
                    fname = "_pad"
                    ctype = "int"
                if fname in ("in", "loop", "type", "fn", "match", "ref", "box", "move", "impl", "as", "use", "mod", "true", "false"):
                    fname = fname + "_"
                # array dims, innermost first: "char[10][24]" -> [[c_char; 24]; 10]
                dims = re.findall(r"\[\s*(\w+)\s*\]", ctype)
                arr = []
                if dims:
                    ctype = re.sub(r"(?:\[\s*\w+\s*\])+$", "", ctype).strip()
                    for d in reversed(dims):
                        arr.append(dim_of(d))
                rt = ffi_type(ctype)
                if ptr == "const_ptr":
                    rt = f"*const {rt}" if not rt.startswith("*") else rt
                elif ptr == "mut_ptr":
                    rt = f"*mut {rt}" if not rt.startswith("*") else rt
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
    for ename, (entries, _pos) in sorted(h.enums.items()):
        ffi.append(f"// {ename}")
        v = 0
        for mem, expr in entries:
            if expr:
                try:
                    v = int(expr.strip().lstrip("=").strip(), 0)
                except ValueError:
                    pass
            ffi.append(f"pub const {mem}: i32 = {v};")
            v += 1
    ffi.append("")

    ffi.append('extern "C" {')
    for fn in new_fns:
        ret, params, pos = h.functions[fn]
        doc = doc_before(h.comments, pos)
        if doc:
            ffi.append("    // " + doc.splitlines()[0].strip()[:110])
        rust_params = []
        has_ctx = False
        for p in split_params(params):
            parsed = parse_param(p)
            if parsed is None:
                continue
            ctype, pname, ptr = parsed
            if pname == "ui" and ctype == "cui_ctx":
                has_ctx = True
                continue  # the context is the receiver
            if pname in ("in", "loop", "type", "fn", "match", "ref", "box", "move", "impl"):
                pname = pname + "_"
            rt = ffi_type(ctype)
            if ptr == "mut_ptr" and not rt.startswith("*"):
                rt = f"*mut {rt}"
            elif ptr == "const_ptr" and not rt.startswith("*"):
                rt = f"*const {rt}"
            rust_params.append(f"{pname}: {rt}")
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
            mname = re.sub(r"^cui_", "", begin)
            mname = re.sub(r"_begin$", "", mname)
            if mname in hand_methods or mname in emitted_methods:
                continue
            emitted_methods.add(mname)
            used_fns.update([begin, end])
            ret, params, pos = h.functions[begin]
            doc = doc_before(h.comments, pos)
            safe.append("    /// " + (doc.splitlines()[0].strip()[:130] if doc else entry))
            sig = []
            begin_args = []
            skip_next = False
            dropped_len = ""
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
                    begin_args.append(f"{dropped_len}.len() as c_int")
                    continue
                pair = array_pair_at(plist_all, i)
                if pair == "drop":
                    skip_next = True
                    dropped_len = pname
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
            doc = doc_before(h.comments, pos)
            safe.append("    /// " + (doc.splitlines()[0].strip()[:130] if doc else fn))
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
                    call.append(f"{pname}.as_mut_ptr() as *mut c_char")
                    continue
                if (buffer_at is not None and i == buffer_at + 1
                        and i < len(plist) and plist[i][0] == "size_t"):
                    continue  # the capacity comes from the String
                if skip_next:
                    skip_next = False
                    call.append(f"{dropped_len}.len() as c_int")
                    continue
                pair = array_pair_at(plist, i)
                if pair == "drop":
                    skip_next = True
                    dropped_len = pname
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
            body_call = f"ffi::{fn}(self.ctx{', ' if call else ''}{', '.join(call)}){ret_call}"
            if buffer_at is not None:
                # C wrote into the String's buffer: re-derive its contents
                bufname = split_params(params)[buffer_at].split("*")[-1].strip()
                if ret.strip() == "void":
                    safe.append(f"        unsafe {{ {body_call} }};")
                    safe.append(f"        crate::ext::sync_buffer({bufname});")
                else:
                    safe.append(f"        let r = unsafe {{ {body_call} }};")
                    safe.append(f"        crate::ext::sync_buffer({bufname});")
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
    OUT_SAFE.write_text(body)

    probe = ["// Generated by scripts/gen_ffi.py: appended by vendor.sh runs.",
             "void gen_probe_all(void) {"]
    for tname in sorted(needed):
        probe.append(f"    SZ({tname});")
    probe.append("}")
    OUT_PROBE.write_text("\n".join(probe) + "\n")
    (ROOT / "scripts/ffi-generated.txt").write_text("\n".join(todo) + "\n")
    print(f"gen.rs: {len(new_fns)} functions, {len(needed)} types; gen_safe.rs methods: {len(emitted_methods)}")


if __name__ == "__main__":
    main()
