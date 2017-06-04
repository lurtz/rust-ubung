use std::process::Command;

// control pulseaudio to switch outputs
// sinks:
// alsa_output.pci-0000_00_1b.0.analog-stereo
// tunnel.cubietruck.local.alsa_output.platform-sunxi-sndspdif.0.analog-stereo

pub const INTERNAL : &'static str = "alsa_output.pci-0000_00_1b.0.analog-stereo";
pub const CUBIETRUCK : &'static str = "tunnel.cubietruck.local.alsa_output.platform-sunxi-sndspdif.0.analog-stereo";

const PACTL : &'static str = "/usr/bin/pactl";

fn get_sink_inputs() -> Vec<u32> {
    let output = Command::new(PACTL)
        .arg("list")
        .arg("sink-inputs")
        .output()
        .expect("pactl failed");

    let output_stdout = String::from_utf8_lossy(&output.stdout);

    let lines = output_stdout.lines();
    let result = lines
        .filter(|&line| line.starts_with("Sink Input #"))
        .map(|line| line.rsplitn(2, "#"))
        .map(|mut iter| iter.next().unwrap())
        .map(|number| number.parse::<u32>().unwrap())
        .collect();

    return result;
}

fn move_output_to_default_sink(indexes: &Vec<u32>) {
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

fn set_default_output(target_output : &str) {
    let _ = Command::new(PACTL)
        .arg("set-default-sink")
        .arg(target_output)
        .status()
        .expect("pactl failed");
}

pub fn switch_ouput(target_output : &str) {
    set_default_output(target_output);
    let indexes = get_sink_inputs();
    move_output_to_default_sink(&indexes);
}