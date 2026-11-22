//! NATIVE (this fork, decisions/0009-texture-workflow.md): texture_dump's
//! index.html, a page of the saved textures for finding the ones to
//! replace. Grouped by where the game loaded them (sources.tsv: the
//! characters, levels, shared packages and HUD sets), then by label, with
//! a texture's animation frames (labels that differ only in a trailing
//! number) together. Each shows its file name (the name its replacement in
//! texture_pack takes), its size and whether it repeats (record flags
//! 0x4/0x8 clamp U/V), and is marked when the pack replaces it. Static
//! HTML beside the PNG files: it opens from the folder with a double click.
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::path::Path;

struct Entry<'a> {
    label: &'a str,
    width: u32,
    height: u32,
    flags: u16,
}

/// Write `dump`/index.html from its index.tsv and sources.tsv; `pack` is
/// the replacement folder in use (its PNG files are marked).
pub fn write(dump: &Path, pack: Option<&Path>) -> std::io::Result<()> {
    let index = std::fs::read_to_string(dump.join("index.tsv")).unwrap_or_default();
    let sources = std::fs::read_to_string(dump.join("sources.tsv")).unwrap_or_default();
    let html = page(&index, &sources, &|stem| pack.is_some_and(|p| p.join(format!("{stem}.png")).is_file()));
    let tmp = dump.join("index.html.tmp");
    std::fs::write(&tmp, html)?;
    std::fs::rename(tmp, dump.join("index.html"))
}

/// The kinds of source in page order, with their headings.
const KINDS: [(&str, &str); 5] = [
    ("character", "Characters"),
    ("level", "Levels"),
    ("package", "Shared packages (items, monsters, effects)"),
    ("hud", "HUD and menus"),
    ("", "Source not recorded"),
];

/// A label without its trailing frame number ("FIRE_POTION18" →
/// "FIRE_POTION").
fn family(label: &str) -> &str {
    let t = label.trim_end_matches(|c: char| c.is_ascii_digit());
    let t = t.trim_end_matches(['_', '-']);
    if t.is_empty() { label } else { t }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// The page for an index.tsv and a sources.tsv (`replaced`: whether the
/// pack has a texture's file).
fn page(index: &str, sources: &str, replaced: &dyn Fn(&str) -> bool) -> String {
    let mut entries: HashMap<&str, Entry> = HashMap::new();
    for line in index.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        let [stem, label, size, flags] = f[..] else { continue };
        let Some((w, h)) = size.split_once('x').and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?))) else { continue };
        let flags = u16::from_str_radix(flags.trim_start_matches("0x"), 16).unwrap_or(0);
        entries.entry(stem).or_insert(Entry { label, width: w, height: h, flags });
    }
    // (kind order, source) → its textures.
    let mut groups: BTreeMap<(usize, &str), Vec<&str>> = BTreeMap::new();
    let mut placed = HashSet::new();
    for line in sources.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        let [stem, kind, source] = f[..] else { continue };
        if entries.contains_key(stem) {
            let k = KINDS.iter().position(|(n, _)| *n == kind).unwrap_or(KINDS.len() - 1);
            groups.entry((k, source)).or_default().push(stem);
            placed.insert(stem);
        }
    }
    let mut unplaced: Vec<&str> = entries.keys().copied().filter(|s| !placed.contains(s)).collect();
    if !unplaced.is_empty() {
        unplaced.sort_unstable();
        groups.insert((KINDS.len() - 1, ""), unplaced);
    }
    let replaced_count = entries.keys().filter(|s| replaced(s)).count();
    let mut body = String::new();
    let mut last_kind = usize::MAX;
    for ((kind, source), mut stems) in groups {
        stems.sort_unstable_by_key(|s| (family(entries[s].label), entries[s].label, *s));
        stems.dedup();
        if kind != last_kind {
            if last_kind != usize::MAX {
                body.push_str("</section>\n");
            }
            let _ = writeln!(body, "<section class=\"kind\"><h2>{}</h2>", KINDS[kind].1);
            last_kind = kind;
        }
        let _ = writeln!(body, "<div class=\"source\"><h3>{} <small>{} textures</small></h3>", if source.is_empty() { "(unknown)".to_string() } else { esc(source) }, stems.len());
        let mut i = 0;
        while i < stems.len() {
            let fam = family(entries[stems[i]].label);
            let n = stems[i..].iter().take_while(|s| family(entries[*s].label) == fam).count();
            if 1 < n {
                let _ = writeln!(body, "<div class=\"frames\"><h4>{} <small>{n} frames</small></h4><div class=\"row\">", esc(fam));
            }
            for stem in &stems[i..i + n] {
                card(&mut body, stem, &entries[stem], replaced(stem));
            }
            if 1 < n {
                body.push_str("</div></div>\n");
            }
            i += n;
        }
        body.push_str("</div>\n");
    }
    if last_kind != usize::MAX {
        body.push_str("</section>\n");
    }
    TEMPLATE.replace("{COUNT}", &entries.len().to_string()).replace("{REPLACED}", &replaced_count.to_string()).replace("{BODY}", &body)
}

fn card(out: &mut String, stem: &str, e: &Entry, replaced: bool) {
    // Integer zoom up to a 128-pixel box (pixels stay square), else fit.
    let side = e.width.max(e.height).max(1) as f32;
    let zoom = if side <= 128.0 { (128.0 / side).floor().min(8.0) } else { 128.0 / side };
    let (w, h) = ((e.width as f32 * zoom).round().max(1.0), (e.height as f32 * zoom).round().max(1.0));
    let wrap = match (e.flags & 0x4 != 0, e.flags & 0x8 != 0) {
        (false, false) => "repeats",
        (true, true) => "edges clamped",
        (false, true) => "repeats across",
        (true, false) => "repeats down",
    };
    let mark = if replaced { "<span class=\"rep\">replaced</span>" } else { "" };
    let _ = writeln!(
        out,
        "<figure data-k=\"{} {stem}\"><div class=\"img\"><img loading=\"lazy\" src=\"{stem}.png\" width=\"{w}\" height=\"{h}\" alt=\"\"></div><figcaption><b>{}</b>{mark}<br><code>{stem}.png</code><br>{}&times;{} &middot; {wrap}</figcaption></figure>",
        esc(&e.label.to_lowercase()),
        esc(e.label),
        e.width,
        e.height
    );
}

const TEMPLATE: &str = r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Saved textures</title>
<style>
:root { --bg: #f6f5f2; --fg: #1d1d1b; --muted: #6b6a66; --card: #ffffff; --line: #dddad3; --accent: #2f6db5; --rep: #2e7d32; --chk1: #e3e1dc; --chk2: #ffffff; }
@media (prefers-color-scheme: dark) { :root { --bg: #1b1c1e; --fg: #e8e6e1; --muted: #9c9a94; --card: #26282b; --line: #3a3c40; --accent: #7fb0ec; --rep: #81c784; --chk1: #3a3c40; --chk2: #2c2e31; } }
body { margin: 0; background: var(--bg); color: var(--fg); font: 14px/1.4 system-ui, "Segoe UI", sans-serif; }
header { position: sticky; top: 0; z-index: 1; background: var(--bg); border-bottom: 1px solid var(--line); padding: 12px 16px; }
header h1 { margin: 0 0 4px; font-size: 20px; }
header p { margin: 4px 0; color: var(--muted); max-width: 70em; }
input { font: inherit; padding: 6px 10px; width: min(28em, 100%); box-sizing: border-box; border: 1px solid var(--line); border-radius: 6px; background: var(--card); color: var(--fg); }
main { padding: 8px 16px 48px; }
h2 { margin: 24px 0 8px; font-size: 18px; }
h3 { margin: 16px 0 8px; font-size: 15px; border-bottom: 1px solid var(--line); padding-bottom: 4px; }
h4 { margin: 4px 0; font-size: 13px; }
small { color: var(--muted); font-weight: normal; }
.source, .frames .row { display: flex; flex-wrap: wrap; gap: 8px; align-items: flex-start; }
.source > h3 { flex-basis: 100%; }
.frames { flex-basis: 100%; background: var(--card); border: 1px solid var(--line); border-radius: 8px; padding: 6px 8px; }
.frames figure { border: 0; padding: 0; background: none; }
figure { margin: 0; width: 140px; background: var(--card); border: 1px solid var(--line); border-radius: 8px; padding: 6px; font-size: 12px; overflow-wrap: anywhere; }
.img { height: 128px; display: flex; align-items: center; justify-content: center; background: repeating-conic-gradient(var(--chk1) 0 25%, var(--chk2) 0 50%) 0 0 / 16px 16px; border-radius: 4px; }
img { image-rendering: pixelated; max-width: 128px; max-height: 128px; }
code { font-size: 11px; color: var(--accent); user-select: all; cursor: copy; }
.rep { margin-left: 4px; color: var(--rep); font-weight: 600; }
.hidden { display: none !important; }
</style>
</head>
<body>
<header>
<h1>Saved textures <small>{COUNT} textures, {REPLACED} replaced</small></h1>
<p>To replace a texture, make your own version of its picture and save it as a PNG with the same file name (click a name to select it, then copy) in the <b>texture_pack</b> folder beside the game. Any size up to 4096&times;4096 works; keep the proportions. Textures that repeat should join up seamlessly at their edges. The game reads the folder when it starts; press F5 in play to reload it. To make one with ChatGPT, see <b>texture_work\README.txt</b> beside the game.</p>
<input id="q" type="search" placeholder="Search labels or file names (for example WIZ or FLOOR)" autofocus>
</header>
<main>
{BODY}</main>
<script>
const q = document.getElementById('q');
q.addEventListener('input', () => {
  const t = q.value.trim().toLowerCase();
  for (const f of document.querySelectorAll('figure')) f.classList.toggle('hidden', t !== '' && !f.dataset.k.includes(t));
  for (const g of document.querySelectorAll('.frames, .source, .kind')) g.classList.toggle('hidden', !g.querySelector('figure:not(.hidden)'));
});
</script>
</body>
</html>
"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn families_drop_frame_numbers() {
        assert_eq!(family("FIRE_POTION18"), "FIRE_POTION");
        assert_eq!(family("MASK_01"), "MASK");
        assert_eq!(family("WIZ_HEAD_Y00"), "WIZ_HEAD_Y");
        assert_eq!(family("BUTTON_R"), "BUTTON_R");
        assert_eq!(family("0042"), "0042");
    }

    /// Sources in page order, frames together, a shared texture in each of
    /// its sources, unknown sources last, replacements marked, labels
    /// escaped.
    #[test]
    fn page_groups_by_source_and_family() {
        let index = "aaaa\tFIRE_POTION01\t16x32\t0x81\nbbbb\tFIRE_POTION02\t16x32\t0x81\ncccc\tWIZ_HEAD\t16x16\t0x80\ndddd\tA<B\t256x64\t0xc\neeee\tbad\tline\n";
        let sources = "cccc\tcharacter\tWIZ_Y\naaaa\tlevel\tL1 The Tower\nbbbb\tlevel\tL1 The Tower\ncccc\tlevel\tL1 The Tower\n";
        let html = page(index, sources, &|s| s == "cccc");
        let at = |s: &str| html.find(s).unwrap_or_else(|| panic!("{s} missing"));
        assert!(at("<h2>Characters") < at("<h2>Levels") && at("<h2>Levels") < at("<h2>Source not recorded"));
        assert!(html.contains("FIRE_POTION <small>2 frames</small>"));
        assert_eq!(html.matches("src=\"cccc.png\"").count(), 2);
        assert!(html.contains("A&lt;B") && !html.contains("A<B"));
        assert!(html.contains("4 textures, 1 replaced"));
        assert!(html.contains("<b>WIZ_HEAD</b><span class=\"rep\">replaced</span>"));
        // 16×32 zoomed ×4 into the 128 box; 256×64 fitted; flags 0xc clamp both.
        assert!(html.contains("src=\"aaaa.png\" width=\"64\" height=\"128\""));
        assert!(html.contains("src=\"dddd.png\" width=\"128\" height=\"32\""));
        assert!(html.contains("256&times;64 &middot; edges clamped"));
        assert!(!html.contains("eeee"));
    }
}
