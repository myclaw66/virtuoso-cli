use crate::client::bridge::escape_skill_string;

#[derive(Default)]
pub struct SchematicOps;

impl SchematicOps {
    pub fn new() -> Self {
        Self
    }

    pub fn create_instance(
        &self,
        lib: &str,
        cell: &str,
        view: &str,
        name: &str,
        origin: (i64, i64),
    ) -> String {
        let lib = escape_skill_string(lib);
        let cell = escape_skill_string(cell);
        let view = escape_skill_string(view);
        let name = escape_skill_string(name);
        let (x, y) = origin;
        format!(
            r#"let((cv master inst) cv = RB_SCH_CV master = dbOpenCellViewByType("{lib}" "{cell}" "{view}" nil "r") inst = dbCreateInst(cv master "{name}" list({x} {y}) "R0" 1) inst)"#
        )
    }

    pub fn create_wire(&self, points: &[(i64, i64)], layer: &str, net_name: &str) -> String {
        let layer = escape_skill_string(layer);
        let net_name = escape_skill_string(net_name);
        let pts: String = points
            .iter()
            .map(|(x, y)| format!("list({x} {y})"))
            .collect::<Vec<_>>()
            .join(" ");
        format!(
            r#"let((cv) cv = RB_SCH_CV dbCreateWire(cv dbMakeNet(cv "{net_name}") dbFindLayerByName(cv "{layer}") list({pts})))"#
        )
    }

    pub fn create_wire_between_terms(
        &self,
        inst1: &str,
        _term1: &str,
        inst2: &str,
        _term2: &str,
        net_name: &str,
    ) -> String {
        let inst1 = escape_skill_string(inst1);
        let inst2 = escape_skill_string(inst2);
        let net_name = escape_skill_string(net_name);
        format!(
            r#"let((cv net) cv = RB_SCH_CV net = dbMakeNet(cv "{net_name}") dbCreateWire(net dbFindTermByName(cv "{inst1}") dbFindTermByName(cv "{inst2}")))"#
        )
    }

    pub fn create_wire_label(&self, net_name: &str, origin: (i64, i64)) -> String {
        let net_name = escape_skill_string(net_name);
        let (x, y) = origin;
        format!(
            r#"let((cv net) cv = RB_SCH_CV net = dbFindNetByName(cv "{net_name}") when(net dbCreateLabel(cv net "{net_name}" list({x} {y}) "centerCenter" "R0" "stick" 0.0625)))"#
        )
    }

    pub fn create_pin(&self, net_name: &str, _pin_type: &str, origin: (i64, i64)) -> String {
        let net_name = escape_skill_string(net_name);
        let (x, y) = origin;
        format!(
            r#"let((cv net pinInst) cv = RB_SCH_CV net = dbMakeNet(cv "{net_name}") pinInst = dbCreateInst(cv dbOpenCellViewByType("basic" "ipin" "symbol" nil "r") "PIN_{net_name}" list({x} {y}) "R0" 1) dbCreatePin(net pinInst))"#
        )
    }

    pub fn check(&self) -> String {
        r#"let((cv) cv = RB_SCH_CV schCheck(cv))"#.into()
    }

    pub fn open_cellview(&self, lib: &str, cell: &str, view: &str) -> String {
        let lib = escape_skill_string(lib);
        let cell = escape_skill_string(cell);
        let view = escape_skill_string(view);
        // dbOpenCellViewByType with viewType="schematic" mode="a":
        //   creates cellview if absent, opens for editing (non-interactive)
        // Store in RB_SCH_CV global for use by subsequent commands
        format!(r#"RB_SCH_CV = dbOpenCellViewByType("{lib}" "{cell}" "{view}" "schematic" "a")"#)
    }

    pub fn save(&self) -> String {
        r#"let((cv) cv = RB_SCH_CV dbSave(cv))"#.into()
    }

    pub fn set_instance_param(&self, inst_name: &str, param: &str, value: &str) -> String {
        let inst_name = escape_skill_string(inst_name);
        let param = escape_skill_string(param);
        let value = escape_skill_string(value);
        format!(
            r#"let((cv inst) cv = RB_SCH_CV inst = car(setof(i cv~>instances i~>name == "{inst_name}")) when(inst dbReplaceProp(inst "{param}" "string" "{value}")))"#
        )
    }

    // ── Read operations ──────────────────────────────────────────────

    /// List all instances in the open cellview. Returns JSON array via sprintf.
    pub fn list_instances(&self) -> String {
        r#"let((cv out sep lib cell) cv = RB_SCH_CV out = "[" sep = "" foreach(inst cv~>instances lib = if(inst~>master inst~>master~>libName "?") cell = if(inst~>master inst~>master~>cellName "?") out = strcat(out sep sprintf(nil "{\"name\":\"%s\",\"master\":\"%s/%s\",\"x\":%g,\"y\":%g}" inst~>name lib cell car(inst~>xy) cadr(inst~>xy))) sep = ",") strcat(out "]"))"#.into()
    }

    /// List all nets in the open cellview. Returns JSON array.
    pub fn list_nets(&self) -> String {
        r#"let((cv out sep) cv = RB_SCH_CV out = "[" sep = "" foreach(net cv~>nets out = strcat(out sep sprintf(nil "\"%s\"" net~>name)) sep = ",") strcat(out "]"))"#.into()
    }

    /// List all pins (terminals) in the open cellview. Returns JSON array.
    pub fn list_pins(&self) -> String {
        r#"let((cv out sep) cv = RB_SCH_CV out = "[" sep = "" foreach(term cv~>terminals out = strcat(out sep sprintf(nil "{\"name\":\"%s\",\"direction\":\"%s\"}" term~>name term~>direction)) sep = ",") strcat(out "]"))"#.into()
    }

    /// Get parameters of a specific instance. Returns JSON object.
    pub fn get_instance_params(&self, inst_name: &str) -> String {
        let inst_name = escape_skill_string(inst_name);
        format!(
            r#"let((cv inst out sep v) cv = RB_SCH_CV inst = car(setof(i cv~>instances strcmp(i~>name "{inst_name}")==0)) if(inst then out = "{{" sep = "" foreach(prop inst~>prop when(prop~>name != nil v = prop~>value when(v out = strcat(out sep sprintf(nil "\"%s\":\"%s\"" prop~>name if(stringp(v) v sprintf(nil "%L" v)))) sep = ","))) strcat(out "}}") else "null"))"#
        )
    }

    /// Assign net name to instance terminal.
    /// Creates a named net and connects it to the instTerm via let-scoped locals.
    pub fn assign_net(&self, inst_name: &str, term_name: &str, net_name: &str) -> String {
        let inst_name = escape_skill_string(inst_name);
        let term_name = escape_skill_string(term_name);
        let net_name = escape_skill_string(net_name);
        format!(
            r#"let((inst iterm net) inst = car(setof(i RB_SCH_CV~>instances strcmp(i~>name "{inst_name}")==0)) iterm = car(setof(x inst~>instTerms strcmp(x~>name "{term_name}")==0)) net = dbMakeNet(RB_SCH_CV "{net_name}") schCreateWire(RB_SCH_CV net "draw" "full" list(list(0 0) list(0 0))))"#
        )
    }
}
