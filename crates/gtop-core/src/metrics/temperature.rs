use crate::metrics::model::TempSensor;

pub fn collect() -> Vec<TempSensor> {
    let mut sensors = Vec::new();

    let Ok(entries) = std::fs::read_dir("/sys/class/hwmon") else {
        return sensors;
    };

    for entry in entries.flatten() {
        let hwmon_path = entry.path();

        let device_name = read_str(hwmon_path.join("name")).unwrap_or_else(|| "unknown".into());

        // Scan for temp*_input files
        let Ok(files) = std::fs::read_dir(&hwmon_path) else {
            continue;
        };

        for file in files.flatten() {
            let fname = file.file_name();
            let fname_str = fname.to_string_lossy();
            if !fname_str.starts_with("temp") || !fname_str.ends_with("_input") {
                continue;
            }

            // Extract index: temp1_input → "1"
            let idx = fname_str
                .trim_start_matches("temp")
                .trim_end_matches("_input");

            let input_path = hwmon_path.join(fname_str.as_ref());
            let Some(millideg) = read_u64(&input_path) else {
                continue;
            };
            let temp_celsius = millideg as f32 / 1000.0;

            let label_path = hwmon_path.join(format!("temp{}_label", idx));
            let label = read_str(label_path).unwrap_or_else(|| format!("Temp{}", idx));

            let max_celsius = hwmon_path
                .join(format!("temp{}_max", idx))
                .pipe(|p| read_u64(p))
                .map(|v| v as f32 / 1000.0);

            let crit_celsius = hwmon_path
                .join(format!("temp{}_crit", idx))
                .pipe(|p| read_u64(p))
                .map(|v| v as f32 / 1000.0);

            sensors.push(TempSensor {
                label: label.trim().to_string(),
                device: device_name.trim().to_string(),
                temp_celsius,
                max_celsius,
                crit_celsius,
            });
        }
    }

    sensors.sort_by(|a, b| a.device.cmp(&b.device).then(a.label.cmp(&b.label)));
    sensors
}

fn read_str(path: impl AsRef<std::path::Path>) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

fn read_u64(path: impl AsRef<std::path::Path>) -> Option<u64> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

trait Pipe: Sized {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R;
}

impl<T> Pipe for T {
    fn pipe<F, R>(self, f: F) -> R
    where
        F: FnOnce(Self) -> R,
    {
        f(self)
    }
}
