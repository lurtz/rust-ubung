use std::process::Command;

// control pulseaudio to switch outputs

pub const INTERNAL: &str = "alsa_output.pci-0000_00_1b.0.analog-stereo";
pub const CUBIETRUCK: &str = "tunnel.cubietruck-2.local.alsa_output.platform-sound.analog-stereo";

const PACTL: &str = "/usr/bin/pactl";

fn get_sink_inputs() -> Vec<u32> {
    let output = Command::new(PACTL)
        .arg("list")
        .arg("sink-inputs")
        .output()
        .expect("pactl failed");

    let output_stdout = String::from_utf8_lossy(&output.stdout);

    let lines = output_stdout.lines();
    lines
        .filter(|&line| line.starts_with("Sink Input #"))
        .map(|line| line.rsplitn(2, '#'))
        .map(|mut iter| iter.next().unwrap())
        .map(|number| number.parse::<u32>().unwrap())
        .collect()
}

fn move_output_to_default_sink(indexes: &[u32]) {
    for index in indexes {
        let status = Command::new(PACTL)
            .arg("move-sink-input")
            .arg(index.to_string())
            .arg("@DEFAULT_SINK@")
            .status()
            .expect("pactl failed");
        if status.success() {
            println!("move success");
        } else {
            println!("move failed");
        }
    }
}

fn set_default_output(target_output: &str) {
    let _ = Command::new(PACTL)
        .arg("set-default-sink")
        .arg(target_output)
        .status()
        .expect("pactl failed");
}

pub fn switch_ouput(target_output: &str) {
    set_default_output(target_output);
    let indexes = get_sink_inputs();
    move_output_to_default_sink(&indexes);
}
