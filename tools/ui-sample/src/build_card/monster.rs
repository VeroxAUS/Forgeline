// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Element 9 of docs/ux/design-plan.md: the monster selector. A small panel under the Monster
//! alternative: a search field, one list on a fixed 28 point row grid (recent picks first, then
//! monsters, Guardian and Arch-Tempered) and a Tempered switch. Picking one makes the build for it.
//!
//! The roster is the research list in docs/research.md (names from public community pages, not
//! yet checked against the game files). Weak elements are unknown (`None`) until the extractor
//! reads them; the row has a column reserved for them so the food plan's elemental matchup can
//! fill it without moving anything.

use super::*;

/// The five elements a monster can be weak to. Placeholder letters stand in until real icons exist.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Element {
    Fire,
    Water,
    Thunder,
    Ice,
    Dragon,
}

impl Element {
    fn letter(self) -> &'static str {
        match self {
            Element::Fire => "F",
            Element::Water => "W",
            Element::Thunder => "T",
            Element::Ice => "I",
            Element::Dragon => "D",
        }
    }

    fn name(self) -> &'static str {
        match self {
            Element::Fire => "Fire",
            Element::Water => "Water",
            Element::Thunder => "Thunder",
            Element::Ice => "Ice",
            Element::Dragon => "Dragon",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Group {
    Base,
    Guardian,
    ArchTempered,
}

impl Group {
    const ALL: [Group; 3] = [Group::Base, Group::Guardian, Group::ArchTempered];

    fn label(self) -> &'static str {
        match self {
            Group::Base => "MONSTERS",
            Group::Guardian => "GUARDIAN",
            Group::ArchTempered => "ARCH-TEMPERED",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Monster {
    pub name: &'static str,
    pub group: Group,
    /// Weak elements, best first. `None` is unknown (never filled from memory).
    pub weak: Option<&'static [Element]>,
}

impl Monster {
    const fn new(name: &'static str, group: Group) -> Self {
        Monster {
            name,
            group,
            weak: None,
        }
    }

    /// Per the community notes in docs/research.md, every large monster can be Tempered except Zoh
    /// Shia, Guardian Arkveld and Gogmazios; Arch-Tempered ones are already the top version.
    pub fn can_temper(&self) -> bool {
        self.group != Group::ArchTempered
            && !matches!(self.name, "Zoh Shia" | "Guardian Arkveld" | "Gogmazios")
    }
}

const fn base(name: &'static str) -> Monster {
    Monster::new(name, Group::Base)
}
const fn guardian(name: &'static str) -> Monster {
    Monster::new(name, Group::Guardian)
}
const fn arch(name: &'static str) -> Monster {
    Monster::new(name, Group::ArchTempered)
}

/// The roster from docs/research.md: 28 monsters, 5 Guardian and 5 Arch-Tempered, each group
/// A to Z. Sources disagree on a few late additions; see the research note.
pub const ROSTER: &[Monster] = &[
    base("Ajarakan"),
    base("Arkveld"),
    base("Balahara"),
    base("Blangonga"),
    base("Chatacabra"),
    base("Congalala"),
    base("Doshaguma"),
    base("Gogmazios"),
    base("Gore Magala"),
    base("Gravios"),
    base("Gypceros"),
    base("Hirabami"),
    base("Jin Dahaad"),
    base("Lagiacrus"),
    base("Lala Barina"),
    base("Mizutsune"),
    base("Nerscylla"),
    base("Nu Udra"),
    base("Quematrice"),
    base("Rathalos"),
    base("Rathian"),
    base("Rey Dau"),
    base("Rompopolo"),
    base("Seregios"),
    base("Uth Duna"),
    base("Xu Wu"),
    base("Yian Kut-Ku"),
    base("Zoh Shia"),
    guardian("Guardian Arkveld"),
    guardian("Guardian Doshaguma"),
    guardian("Guardian Ebony Odogaron"),
    guardian("Guardian Fulgur Anjanath"),
    guardian("Guardian Rathalos"),
    arch("Arch-Tempered Arkveld"),
    arch("Arch-Tempered Jin Dahaad"),
    arch("Arch-Tempered Nu Udra"),
    arch("Arch-Tempered Rey Dau"),
    arch("Arch-Tempered Uth Duna"),
];

/// A chosen monster: its place in the roster and whether it is the Tempered version. This is what
/// the optimizer (and later the food plan) reads.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pick {
    pub index: usize,
    pub tempered: bool,
}

impl Pick {
    pub fn new(index: usize, tempered: bool) -> Self {
        let tempered = tempered && ROSTER[index].can_temper();
        Pick { index, tempered }
    }

    pub fn monster(&self) -> &'static Monster {
        &ROSTER[self.index]
    }

    /// The name as the game words it, `Tempered Rathalos` for the Tempered version.
    pub fn label(&self) -> String {
        if self.tempered {
            format!("Tempered {}", self.monster().name)
        } else {
            self.monster().name.to_string()
        }
    }
}

/// How many recent picks the list keeps.
pub const RECENT_MAX: usize = 3;

/// Puts a pick at the front of the recent list, once, at most `RECENT_MAX` long.
pub fn remember(recent: &mut Vec<Pick>, pick: Pick) {
    recent.retain(|p| p.index != pick.index);
    recent.insert(0, pick);
    recent.truncate(RECENT_MAX);
}

/// The popup's own state: the search text, the keyboard cursor and the Tempered switch.
#[derive(Clone, Default, Debug)]
pub struct Selector {
    pub query: String,
    /// Index into the current list of monster rows (headers do not count).
    pub cursor: usize,
    pub tempered: bool,
    /// Set when the popup opens, so the search field takes focus once.
    pub fresh: bool,
    /// Review only: draws invented weak elements on every row, to show the reserved column
    /// filled. Real weaknesses come from the game files (unknown for now).
    pub demo_weak: bool,
    /// Where the list is scrolled to, in points.
    scroll: f32,
    /// Scroll the list to the cursor on the next frame (set by the arrow keys and by `reveal`).
    follow: bool,
}

impl Selector {
    /// Scrolls the list to the cursor row on the next frame.
    pub fn reveal(&mut self) {
        self.follow = true;
    }

    /// Opens with an empty search, the switch matching the current pick.
    pub fn open(&mut self, current: Option<Pick>) {
        self.query.clear();
        self.cursor = 0;
        self.tempered = current.is_some_and(|p| p.tempered);
        self.fresh = true;
        self.scroll = 0.0;
    }
}

/// One row of the list: a section label or a monster, all the same height.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Item {
    Header(&'static str),
    Monster(usize),
}

/// The rows for a search text: recent picks first when the search is empty, then each group that has
/// a match (case-insensitive, anywhere in the name). With `tempered_only` (the Tempered switch on)
/// only monsters that have a Tempered version are listed.
pub fn items(query: &str, recent: &[Pick], tempered_only: bool) -> Vec<Item> {
    let q = query.trim().to_lowercase();
    let keep = |i: usize| !tempered_only || ROSTER[i].can_temper();
    let mut v = Vec::new();
    let recent: Vec<&Pick> = recent.iter().filter(|p| keep(p.index)).collect();
    if q.is_empty() && !recent.is_empty() {
        v.push(Item::Header("RECENT"));
        v.extend(recent.iter().map(|p| Item::Monster(p.index)));
    }
    for g in Group::ALL {
        let start = v.len();
        v.push(Item::Header(g.label()));
        for (i, m) in ROSTER.iter().enumerate() {
            if m.group == g && keep(i) && (q.is_empty() || m.name.to_lowercase().contains(&q)) {
                v.push(Item::Monster(i));
            }
        }
        if v.len() == start + 1 {
            v.pop();
        }
    }
    v
}

/// What the popup asks for.
pub enum Outcome {
    /// A monster was chosen: close and build for it.
    Pick(Pick),
    /// Only the Tempered switch moved: rebuild for the current monster, keep the panel open.
    Switch(Pick),
}

pub const LIST_W: f32 = 300.0;
const ROW_H: f32 = 28.0;
const VISIBLE_ROWS: usize = 8;
const FIELD_H: f32 = 28.0;
const FOOT_H: f32 = 32.0;
const TILE_S: f32 = 18.0;
/// The reserved weak-element column: five cells of this width, right aligned.
const WEAK_W: f32 = 14.0;

/// The panel's rows, search field and Tempered switch. `current` is the monster the build is for.
pub fn panel(
    ui: &mut Ui,
    st: &mut Selector,
    current: Option<Pick>,
    recent: &[Pick],
    out: &mut Option<Outcome>,
) {
    let ctx = ui.ctx().clone();
    let ppp = ctx.pixels_per_point();
    let snap = |v: f32| (v * ppp).round() / ppp;
    // Rows and gaps are set by this panel alone, so the 28 point grid is exact.
    ui.spacing_mut().item_spacing = Vec2::ZERO;
    let list = items(&st.query, recent, st.tempered);
    let count = list
        .iter()
        .filter(|i| matches!(i, Item::Monster(_)))
        .count();

    // Search field: a hairline underneath, no box.
    let (field, _) = ui.allocate_exact_size(Vec2::new(LIST_W, FIELD_H), egui::Sense::hover());
    let before = st.query.clone();
    let mut edit_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(field.shrink2(Vec2::new(2.0, 0.0)))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    let edit = egui::TextEdit::singleline(&mut st.query)
        .frame(egui::Frame::NONE)
        .font(font("inter", 12.5))
        .text_color(FG)
        .hint_text(
            egui::RichText::new("Search monsters")
                .font(font("inter", 12.5))
                .color(FAINT),
        )
        .desired_width(field.width() - 4.0);
    let resp = edit_ui.add(edit);
    if st.fresh {
        resp.request_focus();
        st.fresh = false;
    }
    if st.query != before {
        st.cursor = 0;
    }
    hair(ui.painter(), field.left(), field.right(), field.bottom());
    ui.add_space(4.0);

    // Keyboard: arrows move the cursor, Enter picks it.
    let (down, up, enter) = ctx.input(|i| {
        (
            i.key_pressed(egui::Key::ArrowDown),
            i.key_pressed(egui::Key::ArrowUp),
            i.key_pressed(egui::Key::Enter),
        )
    });
    if count > 0 {
        if down {
            st.cursor = (st.cursor + 1).min(count - 1);
            st.follow = true;
        }
        if up {
            st.cursor = st.cursor.saturating_sub(1);
            st.follow = true;
        }
    }
    st.cursor = st.cursor.min(count.saturating_sub(1));

    if count == 0 {
        let (r, _) = ui.allocate_exact_size(Vec2::new(LIST_W, ROW_H), egui::Sense::hover());
        ui.painter().text(
            Pos2::new(r.left() + 4.0, r.center().y),
            Align2::LEFT_CENTER,
            "No monster matches that.",
            font("inter", 12.0),
            DIM,
        );
    } else {
        let height = ROW_H * list.len().min(VISIBLE_ROWS) as f32;
        let mut area = egui::ScrollArea::vertical()
            .min_scrolled_height(ROW_H * list.len().min(VISIBLE_ROWS) as f32)
            .max_height(ROW_H * list.len().min(VISIBLE_ROWS) as f32)
            .auto_shrink([false, false]);
        if std::mem::take(&mut st.follow) {
            // Keep the cursor row inside the visible window (rows are 28 points, headers count).
            let at = list
                .iter()
                .enumerate()
                .filter(|(_, it)| matches!(it, Item::Monster(_)))
                .nth(st.cursor)
                .map_or(0, |(row, _)| row);
            let top = at as f32 * ROW_H;
            let offset = st.scroll.clamp(top + ROW_H - height, top);
            st.scroll = offset;
            area = area.vertical_scroll_offset(offset);
        }
        let shown = area.show_rows(ui, ROW_H, list.len(), |ui, range| {
            let mut ordinal = list[..range.start]
                .iter()
                .filter(|i| matches!(i, Item::Monster(_)))
                .count();
            for item in &list[range] {
                let (r, resp) =
                    ui.allocate_exact_size(Vec2::new(LIST_W, ROW_H), egui::Sense::click());
                // Whole pixels, so hairlines and tiles stay crisp while scrolling.
                let r = Rect::from_min_size(Pos2::new(snap(r.left()), snap(r.top())), r.size());
                match *item {
                    Item::Header(label) => {
                        ui.painter().text(
                            Pos2::new(r.left() + 4.0, r.center().y + 3.0),
                            Align2::LEFT_CENTER,
                            label,
                            mono(10.5),
                            FAINT,
                        );
                    }
                    Item::Monster(i) => {
                        let m = &ROSTER[i];
                        let hot = resp.hovered();
                        if hot {
                            st.cursor = ordinal;
                        }
                        let on = ordinal == st.cursor;
                        let is_current = current.is_some_and(|c| c.index == i);
                        monster_row(
                            ui.painter(),
                            r,
                            m,
                            on,
                            is_current,
                            st.demo_weak.then_some(i),
                        );
                        if resp.clicked() || (on && enter) {
                            *out = Some(Outcome::Pick(Pick::new(i, st.tempered)));
                        }
                        let tip = format!("{}. Weak to: {}.", m.name, weak_text(m));
                        resp.on_hover_cursor(egui::CursorIcon::PointingHand)
                            .on_hover_text(tip);
                        ordinal += 1;
                    }
                }
            }
        });
        st.scroll = shown.state.clamped_offset().y;
    }
    ui.add_space(4.0);

    // Footer: the Tempered switch. On, the list shows only monsters that have a Tempered version.
    let (foot, resp) = ui.allocate_exact_size(Vec2::new(LIST_W, FOOT_H), egui::Sense::click());
    hair(ui.painter(), foot.left(), foot.right(), foot.top());
    tempered_switch(ui.painter(), foot, st.tempered, resp.hovered());
    if resp.clicked() {
        st.tempered = !st.tempered;
        st.cursor = 0;
        st.scroll = 0.0;
        // The build follows the switch only when its monster has a Tempered version.
        if let Some(c) = current.filter(|c| c.monster().can_temper()) {
            *out = Some(Outcome::Switch(Pick::new(c.index, st.tempered)));
        }
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text("Tempered: list only monsters that have a Tempered version.");
}

/// One monster row: plain tile, name (ellipsis when it would not fit), the reserved weak-element
/// column, and an accent bar on the left when it is the monster the build is for.
fn monster_row(p: &Painter, r: Rect, m: &Monster, lit: bool, current: bool, demo: Option<usize>) {
    if lit {
        p.rect_filled(r, 2.0, LINE);
    }
    if current {
        p.rect_filled(
            Rect::from_min_size(r.left_top(), Vec2::new(2.0, r.height())),
            0.0,
            ACCENT,
        );
    }
    let tile = Rect::from_center_size(
        Pos2::new(r.left() + 8.0 + TILE_S / 2.0, r.center().y),
        Vec2::splat(TILE_S),
    );
    p.rect_filled(tile, 2.0, Color32::from_rgb(40, 29, 22));
    p.circle_stroke(tile.center(), 4.0, Stroke::new(1.4, ACCENT));
    let weak_w = WEAK_W * 5.0;
    let name_x = tile.right() + 8.0;
    let name_w = r.right() - weak_w - 8.0 - name_x;
    p.text(
        Pos2::new(name_x, r.center().y),
        Align2::LEFT_CENTER,
        fit(p, m.name, name_w, "inter", 12.5),
        font("inter", 12.5),
        if current { ACCENT } else { FG },
    );
    // Weak elements, best first, in the same five cells on every row. Unknown draws nothing.
    const DEMO: [&[Element]; 4] = [
        &[Element::Fire],
        &[Element::Water, Element::Ice],
        &[Element::Thunder, Element::Dragon, Element::Fire],
        &[
            Element::Ice,
            Element::Water,
            Element::Thunder,
            Element::Dragon,
            Element::Fire,
        ],
    ];
    let shown = demo.map(|i| DEMO[i % DEMO.len()]).or(m.weak);
    if let Some(list) = shown {
        for (k, e) in list.iter().take(5).enumerate() {
            p.text(
                Pos2::new(
                    r.right() - 8.0 - weak_w + (k as f32 + 0.5) * WEAK_W,
                    r.center().y,
                ),
                Align2::CENTER_CENTER,
                e.letter(),
                mono(10.5),
                DIM,
            );
        }
    }
}

/// `TEMPERED` label and a small switch at the right end of the footer.
fn tempered_switch(p: &Painter, foot: Rect, on: bool, hot: bool) {
    p.text(
        Pos2::new(foot.left() + 4.0, foot.center().y),
        Align2::LEFT_CENTER,
        "TEMPERED",
        mono(10.5),
        if on || hot { FG } else { DIM },
    );
    let w = 30.0;
    let h = 16.0;
    let track = Rect::from_center_size(
        Pos2::new(foot.right() - 4.0 - w / 2.0, foot.center().y),
        Vec2::new(w, h),
    );
    p.rect_filled(track, h / 2.0, if on { ACCENT } else { LINE });
    let knob = Pos2::new(
        if on {
            track.right() - h / 2.0
        } else {
            track.left() + h / 2.0
        },
        track.center().y,
    );
    p.circle_filled(knob, h / 2.0 - 3.0, if on { ACCENT_INK } else { DIM });
}

/// Names the weak elements for a tooltip, or says they are unknown.
pub fn weak_text(m: &Monster) -> String {
    match m.weak {
        None => "unknown".to_string(),
        Some(list) => list.iter().map(|e| e.name()).collect::<Vec<_>>().join(", "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn idx(name: &str) -> usize {
        ROSTER.iter().position(|m| m.name == name).unwrap()
    }

    fn names(list: &[Item]) -> Vec<&'static str> {
        list.iter()
            .filter_map(|i| match i {
                Item::Monster(i) => Some(ROSTER[*i].name),
                Item::Header(_) => None,
            })
            .collect()
    }

    #[test]
    fn roster_is_sorted_within_groups_and_has_no_repeats() {
        for g in Group::ALL {
            let v: Vec<_> = ROSTER
                .iter()
                .filter(|m| m.group == g)
                .map(|m| m.name)
                .collect();
            let mut sorted = v.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(v, sorted, "{g:?}");
        }
        assert_eq!(ROSTER.len(), 38);
    }

    #[test]
    fn tempered_follows_the_research_notes() {
        assert!(ROSTER[idx("Rathalos")].can_temper());
        assert!(!ROSTER[idx("Zoh Shia")].can_temper());
        assert!(!ROSTER[idx("Gogmazios")].can_temper());
        assert!(!ROSTER[idx("Guardian Arkveld")].can_temper());
        assert!(ROSTER[idx("Guardian Rathalos")].can_temper());
        assert!(!ROSTER[idx("Arch-Tempered Rey Dau")].can_temper());
        // Asking for Tempered on one that has none gives the plain monster.
        let p = Pick::new(idx("Zoh Shia"), true);
        assert!(!p.tempered);
        assert_eq!(p.label(), "Zoh Shia");
        assert_eq!(
            Pick::new(idx("Rathalos"), true).label(),
            "Tempered Rathalos"
        );
    }

    #[test]
    fn search_matches_anywhere_and_drops_empty_groups() {
        let all = items("", &[], false);
        assert_eq!(names(&all).len(), ROSTER.len());
        let q = items("DAH", &[], false);
        assert_eq!(names(&q), ["Jin Dahaad", "Arch-Tempered Jin Dahaad"]);
        assert_eq!(
            q.iter().filter(|i| matches!(i, Item::Header(_))).count(),
            2,
            "Guardian has no match so it has no header"
        );
        assert!(items("zzz", &[], false).is_empty());
    }

    #[test]
    fn tempered_switch_lists_only_monsters_with_a_tempered_version() {
        let v = items("", &[], true);
        let n = names(&v);
        assert!(n.contains(&"Rathalos") && n.contains(&"Guardian Rathalos"));
        for gone in [
            "Zoh Shia",
            "Gogmazios",
            "Guardian Arkveld",
            "Arch-Tempered Rey Dau",
        ] {
            assert!(!n.contains(&gone), "{gone}");
        }
        assert!(
            !v.contains(&Item::Header("ARCH-TEMPERED")),
            "a group with nothing left loses its label"
        );
        // Recent picks without a Tempered version drop out of the list too.
        let recent = [
            Pick::new(idx("Zoh Shia"), false),
            Pick::new(idx("Rathalos"), false),
        ];
        assert_eq!(names(&items("", &recent, true))[0], "Rathalos");
    }

    #[test]
    fn recent_picks_lead_only_when_not_searching() {
        let mut recent = Vec::new();
        for n in ["Rathalos", "Arkveld", "Rathalos", "Gravios", "Xu Wu"] {
            remember(&mut recent, Pick::new(idx(n), false));
        }
        assert_eq!(recent.len(), RECENT_MAX);
        assert_eq!(recent[0].index, idx("Xu Wu"));
        let v = items("", &recent, false);
        assert_eq!(v[0], Item::Header("RECENT"));
        assert_eq!(v[1], Item::Monster(idx("Xu Wu")));
        assert_eq!(items("rath", &recent, false)[0], Item::Header("MONSTERS"));
    }
}
