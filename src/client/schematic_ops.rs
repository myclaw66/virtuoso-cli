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
            r#"let((cv master inst) cv = geGetEditCellView() master = dbOpenCellViewByType("{lib}" "{cell}" "{view}" nil "r") inst = dbCreateInst(cv master "{name}" nil {x}:{y} "R0" 1) inst)"#
        )
    }

    pub fn create_wire(&self, points: &[(i64, i64)], layer: &str, net_name: &str) -> String {
        let layer = escape_skill_string(layer);
        let net_name = escape_skill_string(net_name);
        let pts: String = points
            .iter()
            .map(|(x, y)| format!("{x}:{y}"))
            .collect::<Vec<_>>()
            .join(" ");
        format!(
            r#"let((cv) cv = geGetEditCellView() dbCreateWire(cv dbMakeNet(cv "{net_name}") dbFindLayerByName(cv "{layer}") list({pts})))"#
        )
    }

    pub fn create_wire_between_terms(
        &self,
        inst1: &str,
        term1: &str,
        inst2: &str,
        term2: &str,
        net_name: &str,
    ) -> String {
        let inst1 = escape_skill_string(inst1);
        let inst2 = escape_skill_string(inst2);
        let net_name = escape_skill_string(net_name);
        format!(
            r#"let((cv net) cv = geGetEditCellView() net = dbMakeNet(cv "{net_name}") dbCreateWire(net dbFindTermByName(cv "{inst1}") dbFindTermByName(cv "{inst2}")))"#
        )
    }

    pub fn create_wire_label(&self, net_name: &str, origin: (i64, i64)) -> String {
        let net_name = escape_skill_string(net_name);
        let (x, y) = origin;
        format!(
            r#"let((cv net) cv = geGetEditCellView() net = dbFindNetByName(cv "{net_name}") when(net dbCreateLabel(cv net "{net_name}" {x}:{y} "centerCenter" "R0" "stick" 0.0625)))"#
        )
    }

    pub fn create_pin(&self, net_name: &str, pin_type: &str, origin: (i64, i64)) -> String {
        let net_name = escape_skill_string(net_name);
        let (x, y) = origin;
        format!(
            r#"let((cv) cv = geGetEditCellView() dbCreatePin(dbFindNetByName(cv "{net_name}") dbCreateInst(cv dbOpenCellViewByType("basic" "pin" "symbol" nil "r") nil nil {x}:{y} "R0" 1)))"#
        )
    }

    pub fn check(&self) -> String {
        r#"let((cv) cv = geGetEditCellView() schCheck(cv))"#.into()
    }
}
