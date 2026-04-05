use crate::client::bridge::VirtuosoClient;
use crate::error::{Result, VirtuosoError};
use serde_json::{json, Value};
use std::collections::HashMap;

pub fn char(
    lib: &str,
    cell: &str,
    view: &str,
    inst: &str,
    device_type: &str,
    l_values: &[f64],
    vgs_start: f64,
    vgs_stop: f64,
    vgs_step: f64,
    output: &str,
    timeout: u64,
) -> Result<Value> {
    let client = VirtuosoClient::from_env()?;

    // Setup simulator
    client.execute_skill("simulator('spectre)", None)?;
    let design_result = client.execute_skill(
        &format!("design(\"{lib}\" \"{cell}\" \"{view}\")"),
        None,
    )?;
    if !design_result.ok() {
        return Err(VirtuosoError::NotFound(format!(
            "design {lib}/{cell}/{view} not found"
        )));
    }

    let mut all_data: Vec<Value> = Vec::new();
    let mut total_points = 0;

    for &l in l_values {
        client.execute_skill(&format!("desVar(\"L\" {l:e})"), None)?;

        let mut points: Vec<Value> = Vec::new();
        let mut vgs = vgs_start;

        while vgs <= vgs_stop + vgs_step * 0.01 {
            client.execute_skill(&format!("desVar(\"VGS\" {vgs})"), None)?;
            let rdir = format!("/tmp/char_{device_type}_{l:e}_{vgs}");
            client.execute_skill(&format!("resultsDir(\"{rdir}\")"), None)?;
            client.execute_skill("analysis('dc ?saveOppoint t)", None)?;
            client.execute_skill("save('all)", None)?;
            client.execute_skill("run()", Some(timeout))?;

            // Extract oppoint
            let params = ["gm", "ids", "gds", "vth", "cgs"];
            let mut vals: HashMap<String, f64> = HashMap::new();
            let mut ok = true;

            for p in &params {
                let expr = format!("value(getData(\"{inst}:{p}\" ?result \"dcOpInfo\"))");
                let r = client.execute_skill(&expr, None)?;
                let v = r.output.trim().trim_matches('"');
                if v == "nil" || v.is_empty() {
                    ok = false;
                    break;
                }
                if let Ok(f) = v.parse::<f64>() {
                    vals.insert(p.to_string(), f);
                } else {
                    ok = false;
                    break;
                }
            }

            if ok {
                let gm = vals["gm"];
                let id = vals["ids"].abs();
                let gds = vals["gds"].abs();
                let vth = vals["vth"];
                let cgs = vals["cgs"];

                if id > 1e-15 && gm > 1e-15 {
                    let gmid = gm / id;
                    let gain = gm / gds;
                    let vov = vgs - vth;
                    let ft = gm / (2.0 * std::f64::consts::PI * cgs.abs());

                    points.push(json!({
                        "vgs": (vgs * 100.0).round() / 100.0,
                        "gmid": (gmid * 100.0).round() / 100.0,
                        "gain": (gain * 10.0).round() / 10.0,
                        "id": id,
                        "vov": (vov * 1000.0).round() / 1000.0,
                        "ft": ft,
                        "vth": (vth * 10000.0).round() / 10000.0,
                        "gds": gds,
                    }));
                    total_points += 1;
                }
            }

            vgs += vgs_step;
        }

        if !points.is_empty() {
            all_data.push(json!({
                "l": l,
                "points": points,
            }));
        }
    }

    // Write output file
    let lookup = json!({
        "process": output.split('/').nth_back(1).unwrap_or("unknown"),
        "device": device_type,
        "w_testbench": 1e-6,
        "w_unit": "meters",
        "id_unit": "amperes (absolute current at w_testbench)",
        "gain_unit": "V/V (gm/gds)",
        "ft_unit": "Hz",
        "sizing_formula": "W_design(um) = Id_needed(A) / id(A) * 1",
        "characterized": chrono::Local::now().format("%Y-%m-%d").to_string(),
        "data": all_data,
    });

    let output_path = format!("{output}/{device_type}_lookup.json");
    std::fs::create_dir_all(output).map_err(|e| {
        VirtuosoError::Io(e)
    })?;
    let json_str = serde_json::to_string_pretty(&lookup).map_err(VirtuosoError::Json)?;
    std::fs::write(&output_path, &json_str)?;

    Ok(json!({
        "status": "success",
        "device": device_type,
        "l_values": l_values.len(),
        "total_points": total_points,
        "output": output_path,
    }))
}
